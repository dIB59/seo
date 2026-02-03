//! Job processing orchestration for SEO analysis (V2 schema).
//!
//! This module coordinates the analysis pipeline:
//! 1. Resource checking (robots.txt, sitemap, SSL)
//! 2. Page discovery and crawling
//! 3. SEO analysis (light or deep audit)
//! 4. Issue detection and persistence
//! 5. Summary generation (via triggers)

use crate::domain::models::{
    IssueSeverity, Job, JobSettings, JobStatus, LinkType, NewIssue, NewLink, Page,
};
use crate::{
    repository::sqlite::*,
    service::{Auditor, DeepAuditor, LightAuditor, PageDiscovery, ResourceChecker},
};

use anyhow::{Context, Result};
use dashmap::DashMap;
use scraper::{Html, Selector};
use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tauri::Emitter;
use tokio::time::sleep;
use url::Url;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Polling interval when no pending jobs are found
const JOB_POLL_INTERVAL: Duration = Duration::from_secs(15);

/// Delay after job fetch failure before retrying
const JOB_FETCH_RETRY_DELAY: Duration = Duration::from_secs(10);

// ============================================================================
// JOB PROCESSOR V2
// ============================================================================

/// Orchestrates SEO analysis jobs using the normalized schema.
///
/// The processor manages the full lifecycle of analysis jobs:
/// - Polls for pending jobs and processes them sequentially
/// - Coordinates resource checking, crawling, and analysis
/// - Supports job cancellation via atomic flags
/// - Emits progress events to the frontend
pub struct JobProcessor<R: tauri::Runtime = tauri::Wry> {
    // Repositories
    job_db: JobRepository,
    page_db: PageRepository,
    issue_db: IssueRepository,
    link_db: LinkRepository,

    // Services
    discovery: PageDiscovery,
    resource_checker: ResourceChecker,
    light_auditor: Arc<LightAuditor>,
    deep_auditor: Arc<DeepAuditor>,

    // Runtime state
    cancel_map: Arc<DashMap<String, Arc<AtomicBool>>>,
    app_handle: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> JobProcessor<R> {
    // ========================================================================
    // CONSTRUCTION
    // ========================================================================

    pub fn new(pool: SqlitePool, app_handle: tauri::AppHandle<R>) -> Self {
        Self {
            job_db: JobRepository::new(pool.clone()),
            page_db: PageRepository::new(pool.clone()),
            issue_db: IssueRepository::new(pool.clone()),
            link_db: LinkRepository::new(pool.clone()),
            discovery: PageDiscovery::new(),
            resource_checker: ResourceChecker::new(),
            light_auditor: Arc::new(LightAuditor::new()),
            deep_auditor: Arc::new(DeepAuditor::new()),
            cancel_map: Arc::new(DashMap::with_capacity(10)),
            app_handle,
        }
    }

    // ========================================================================
    // JOB POLLING
    // ========================================================================

    /// Main polling loop - fetches and processes pending jobs.
    pub async fn run(&self) -> Result<()> {
        log::info!("JobProcessor: Starting job polling loop");

        loop {
            match self.job_db.get_pending().await {
                Ok(jobs) if !jobs.is_empty() => {
                    for job in jobs {
                        log::info!("Processing job: {} ({})", job.id, job.url);
                        if let Err(e) = self.process_job(job).await {
                            log::error!("Job failed: {}", e);
                        }
                    }
                }
                Ok(_) => {
                    log::trace!("No pending jobs, sleeping...");
                    sleep(JOB_POLL_INTERVAL).await;
                }
                Err(e) => {
                    log::error!("Failed to fetch pending jobs: {}", e);
                    sleep(JOB_FETCH_RETRY_DELAY).await;
                }
            }
        }
    }

    /// Cancels a running job.
    pub async fn cancel(&self, job_id: &str) -> Result<()> {
        log::info!("Cancelling job {}", job_id);
        self.set_cancelled(job_id);
        self.job_db
            .update_status(job_id, JobStatus::Cancelled)
            .await
    }

    // ========================================================================
    // JOB LIFECYCLE
    // ========================================================================

    /// Processes a single analysis job through its full lifecycle.
    pub(crate) async fn process_job(&self, mut job: Job) -> Result<String> {
        let timer = JobTimer::start(&job.id);
        let cancel_flag = self.get_cancel_flag(&job.id);

        // Initialize job
        job.status = JobStatus::Running;
        self.job_db
            .update_status(&job.id, JobStatus::Running)
            .await?;

        // Parse job URL
        let start_url = self.parse_job_url(&job)?;

        // Check site resources (robots.txt, sitemap, SSL)
        let _resources = self.check_site_resources(&start_url).await?;

        // Early exit if cancelled
        if self.is_cancelled(&job.id) {
            log::warn!("Job {} cancelled before crawl", job.id);
            return Ok(job.id.clone());
        }

        // Run discovery and analysis
        let auditor = self.select_auditor(&job.settings);
        let crawl_result = self
            .crawl_and_analyze(&job, &start_url, auditor, &cancel_flag)
            .await?;

        // Persist links
        self.persist_links(&job, &crawl_result.edges).await?;

        // Finalize job (triggers will update summary stats)
        let final_status = self.finalize_job(&job.id).await?;

        log::info!(
            "Job {} completed with status {:?} in {}ms",
            job.id,
            final_status,
            timer.elapsed_ms()
        );

        Ok(job.id.clone())
    }

    // ========================================================================
    // CONFIGURATION & INITIALIZATION
    // ========================================================================

    fn parse_job_url(&self, job: &Job) -> Result<Url> {
        Url::parse(&job.url).with_context(|| format!("Invalid URL: {}", job.url))
    }

    async fn check_site_resources(&self, url: &Url) -> Result<SiteResources> {
        // Just check if resources exist - we don't need the full status
        let robots_txt = self
            .resource_checker
            .check_robots_txt(url.clone())
            .await
            .is_ok();
        let sitemap = self
            .resource_checker
            .check_sitemap_xml(url.clone())
            .await
            .is_ok();

        Ok(SiteResources {
            robots_txt,
            sitemap,
            ssl: url.scheme() == "https",
        })
    }

    fn select_auditor(&self, settings: &JobSettings) -> Arc<dyn Auditor + Send + Sync> {
        if settings.rate_limit_ms >= 2000 {
            if self.deep_auditor.is_available() {
                self.deep_auditor.clone()
            } else {
                log::warn!("[JOB] Deep auditor unavailable, falling back to light auditor");
                self.light_auditor.clone()
            }
        } else {
            self.light_auditor.clone()
        }
    }

    async fn persist_links(&self, job: &Job, edges: &[PageEdge]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }

        let links: Vec<NewLink> = edges
            .iter()
            .map(|e| NewLink {
                job_id: job.id.to_string(),
                source_page_id: e.from_page_id.clone(),
                target_page_id: None,
                target_url: e.to_url.clone(),
                link_text: None,
                link_type: if e.is_internal(&job.url) {
                    LinkType::Internal
                } else {
                    LinkType::External
                },
                is_followed: true,
                status_code: Some(e.status_code as i64),
            })
            .collect();

        self.link_db.insert_batch(&links).await
    }

    async fn finalize_job(&self, job_id: &str) -> Result<JobStatus> {
        let status = JobStatus::Completed;
        self.job_db.update_status(job_id, status.clone()).await?;
        Ok(status)
    }

    // ========================================================================
    // CRAWLING AND ANALYSIS
    // ========================================================================

    async fn crawl_and_analyze(
        &self,
        job: &Job,
        start_url: &Url,
        auditor: Arc<dyn Auditor + Send + Sync>,
        cancel_flag: &Arc<AtomicBool>,
    ) -> Result<CrawlResult> {
        let mut result = CrawlResult::default();
        let max_pages = job.settings.max_pages as usize;

        // Update status to running (analyzing phase)
        self.job_db
            .update_status(&job.id, JobStatus::Running)
            .await?;

        // Discover pages first using the discovery service
        let mut discovered = self
            .discovery
            .discover(
                start_url.clone(),
                job.settings.max_pages,
                job.settings.rate_limit_ms,
                cancel_flag,
                |_| {},
            )
            .await
            .context("Page discovery failed")?;

        if discovered.is_empty() {
            log::warn!("[JOB] Discovery returned no pages, falling back to start URL");
            discovered.push(start_url.clone());
        }

        for (idx, url) in discovered.iter().enumerate() {
            if cancel_flag.load(Ordering::Relaxed) {
                log::info!("Job {} cancelled during analysis", job.id);
                break;
            }

            if idx >= max_pages {
                break;
            }

            // Fetch and analyze page (depth is not tracked by discovery, use 0)
            let analysis = match self
                .analyze_page(url.as_str(), &job.id, 0, &auditor)
                .await
            {
                Ok(result) => Ok(result),
                Err(e) => {
                    if auditor.name() == "Deep (Lighthouse)" {
                        log::warn!(
                            "Deep auditor failed for {} ({}), retrying with light auditor",
                            url,
                            e
                        );
                        let light = self.light_auditor.clone() as Arc<dyn Auditor + Send + Sync>;
                        self.analyze_page(url.as_str(), &job.id, 0, &light).await
                    } else {
                        Err(e)
                    }
                }
            };

            match analysis {
                Ok((page_result, _new_urls)) => {
                    result.pages += 1;
                    result.issues += page_result.issues.len();
                    result.edges.extend(page_result.edges);
                }
                Err(e) => {
                    log::warn!("Failed to analyze {}: {:#}", url, e);
                }
            }

            // Report progress
            let pages_analyzed = (idx + 1).min(max_pages);
            let progress = (pages_analyzed as f64 / max_pages as f64) * 100.0;
            self.report_progress(&job.id, progress, pages_analyzed as i64)
                .await;

            // Apply rate limiting
            self.apply_request_delay(&job.settings).await;
        }

        Ok(result)
    }

    async fn analyze_page(
        &self,
        url: &str,
        job_id: &str,
        depth: i64,
        auditor: &Arc<dyn Auditor + Send + Sync>,
    ) -> Result<(PageResult, Vec<String>)> {
        // Fetch and analyze page using the auditor
        let audit_result = auditor.analyze(url).await?;

        // Parse HTML and extract all data BEFORE any awaits
        let (page, extracted_issues, new_urls, edges) = {
            let html = Html::parse_document(&audit_result.html);

            // Extract basic page data
            let title = extract_title(&html);
            let meta_description = extract_meta_description(&html);
            let canonical_url = extract_canonical(&html);
            let word_count = extract_word_count(&html);

            // Extract links
            let (internal_links, _external_links, all_links) = extract_links(&html, url);

            // Create Page
            let page = Page {
                id: uuid::Uuid::new_v4().to_string(),
                job_id: job_id.to_string(),
                url: url.to_string(),
                depth,
                status_code: Some(audit_result.status_code as i64),
                content_type: None,
                title: title.clone(),
                meta_description: meta_description.clone(),
                canonical_url,
                robots_meta: None,
                word_count: Some(word_count),
                load_time_ms: Some(audit_result.load_time_ms as i64),
                response_size_bytes: Some(audit_result.content_size as i64),
                crawled_at: chrono::Utc::now(),
            };

            // Generate SEO issues
            let mut issues = Vec::new();

            if title.is_none() || title.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                issues.push((
                    "Missing Title".to_string(),
                    "Page has no title tag".to_string(),
                    "Add a descriptive title tag".to_string(),
                    IssueSeverity::Critical,
                ));
            }

            if meta_description
                .is_none()
                || meta_description
                    .as_ref()
                    .map(|d| d.is_empty())
                    .unwrap_or(true)
            {
                issues.push((
                    "Missing Meta Description".to_string(),
                    "Page has no meta description".to_string(),
                    "Add a meta description".to_string(),
                    IssueSeverity::Warning,
                ));
            }

            if audit_result.status_code >= 400 {
                issues.push((
                    "HTTP Error".to_string(),
                    format!("Page returned status code {}", audit_result.status_code),
                    "Fix the HTTP error".to_string(),
                    IssueSeverity::Critical,
                ));
            }

            // Build edges for link tracking
            let edges: Vec<_> = all_links
                .into_iter()
                .map(|(href, is_internal)| (href, if is_internal { 200i32 } else { 0i32 }))
                .collect();

            (page, issues, internal_links, edges)
        };

        // Insert page
        let page_id = self.page_db.insert(&page).await?;

        // Convert and insert issues with the actual page_id
        let issues: Vec<NewIssue> = extracted_issues
            .into_iter()
            .map(|(title, description, recommendation, severity)| NewIssue {
                job_id: job_id.to_string(),
                page_id: Some(page_id.clone()),
                issue_type: title,
                severity,
                message: description,
                details: Some(recommendation),
            })
            .collect();

        if !issues.is_empty() {
            self.issue_db.insert_batch(&issues).await?;
        }

        // Build final edges with page_id
        let final_edges: Vec<PageEdge> = edges
            .into_iter()
            .map(|(href, status_code)| PageEdge::new(&page_id, &href, status_code))
            .collect();

        Ok((
            PageResult {
                issues,
                edges: final_edges,
            },
            new_urls,
        ))
    }

    async fn apply_request_delay(&self, settings: &JobSettings) {
        if settings.rate_limit_ms > 0 {
            sleep(Duration::from_millis(settings.rate_limit_ms as u64)).await;
        }
    }

    async fn report_progress(&self, job_id: &str, progress: f64, pages_analyzed: i64) {
        // Update progress in DB
        if let Err(e) = self.job_db.update_progress(job_id, progress, None).await {
            log::warn!("Failed to update progress: {}", e);
        }

        // Emit event to frontend
        let event = ProgressEvent {
            job_id: job_id.to_string(),
            progress,
            pages_analyzed,
            status: "running".to_string(),
        };

        if let Err(e) = self.app_handle.emit("analysis:progress", &event) {
            log::warn!("Failed to emit progress event: {}", e);
        }
    }

    // ========================================================================
    // CANCELLATION
    // ========================================================================

    fn get_cancel_flag(&self, job_id: &str) -> Arc<AtomicBool> {
        self.cancel_map
            .entry(job_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone()
    }

    fn set_cancelled(&self, job_id: &str) {
        if let Some(flag) = self.cancel_map.get(job_id) {
            flag.store(true, Ordering::Relaxed);
        }
    }

    fn is_cancelled(&self, job_id: &str) -> bool {
        self.cancel_map
            .get(job_id)
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
    }
}

// ============================================================================
// HELPER TYPES
// ============================================================================

/// Represents a link edge between pages during crawling.
#[derive(Debug, Clone)]
struct PageEdge {
    from_page_id: String,
    to_url: String,
    status_code: i32,
}

impl PageEdge {
    fn new(from_page_id: &str, to_url: &str, status_code: i32) -> Self {
        Self {
            from_page_id: from_page_id.to_string(),
            to_url: to_url.to_string(),
            status_code,
        }
    }

    /// Check if this edge points to an internal URL for the given job.
    fn is_internal(&self, base_url: &str) -> bool {
        if let (Ok(edge_url), Ok(base)) = (Url::parse(&self.to_url), Url::parse(base_url)) {
            edge_url.host_str() == base.host_str() && edge_url.port() == base.port()
        } else {
            false
        }
    }
}

#[derive(Default)]
struct CrawlResult {
    pages: usize,
    issues: usize,
    edges: Vec<PageEdge>,
}

struct PageResult {
    issues: Vec<NewIssue>,
    edges: Vec<PageEdge>,
}

/// Site-level resource check results.
#[allow(dead_code)]
struct SiteResources {
    robots_txt: bool,
    sitemap: bool,
    ssl: bool,
}

/// Job timer for measuring total crawl time.
#[allow(dead_code)]
struct JobTimer {
    job_id: String,
    start: std::time::Instant,
}

impl JobTimer {
    fn start(job_id: &str) -> Self {
        Self {
            job_id: job_id.to_string(),
            start: std::time::Instant::now(),
        }
    }

    fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

#[derive(Serialize)]
struct ProgressEvent {
    job_id: String,
    progress: f64,
    pages_analyzed: i64,
    status: String,
}

// ============================================================================
// HTML EXTRACTION HELPERS
// ============================================================================

fn extract_title(html: &Html) -> Option<String> {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = SELECTOR.get_or_init(|| Selector::parse("title").unwrap());
    html.select(selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_meta_description(html: &Html) -> Option<String> {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = SELECTOR.get_or_init(|| Selector::parse("meta[name='description']").unwrap());
    html.select(selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_canonical(html: &Html) -> Option<String> {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = SELECTOR.get_or_init(|| Selector::parse("link[rel='canonical']").unwrap());
    html.select(selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_word_count(html: &Html) -> i64 {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = SELECTOR.get_or_init(|| Selector::parse("body").unwrap());
    html.select(selector)
        .next()
        .map(|body| body.text().collect::<String>().split_whitespace().count() as i64)
        .unwrap_or(0)
}

fn extract_links(html: &Html, base_url: &str) -> (Vec<String>, Vec<String>, Vec<(String, bool)>) {
    static SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = SELECTOR.get_or_init(|| Selector::parse("a[href]").unwrap());

    let base = Url::parse(base_url).ok();
    let base_host = base.as_ref().and_then(|u| u.host_str()).map(|s| s.to_string());
    let base_port = base.as_ref().and_then(|u| u.port());

    let mut internal = Vec::new();
    let mut external = Vec::new();
    let mut all = Vec::new();

    for element in html.select(selector) {
        if let Some(href) = element.value().attr("href") {
            let href = href.trim();

            if href.is_empty()
                || href.starts_with('#')
                || href.starts_with("javascript:")
                || href.starts_with("mailto:")
                || href.starts_with("tel:")
            {
                continue;
            }

            let resolved = if let Some(ref base) = base {
                base.join(href)
                    .map(|u| u.to_string())
                    .unwrap_or_else(|_| href.to_string())
            } else {
                href.to_string()
            };

            let is_internal = if let Ok(link_url) = Url::parse(&resolved) {
                link_url.host_str().map(|h| h.to_string()) == base_host
                    && link_url.port() == base_port
            } else {
                false
            };

            all.push((resolved.clone(), is_internal));

            if is_internal {
                internal.push(resolved);
            } else {
                external.push(resolved);
            }
        }
    }

    (internal, external, all)
}

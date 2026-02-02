//! Job processing orchestration for SEO analysis (V2 Schema).
//!
//! This module coordinates the analysis pipeline using the redesigned schema:
//! 1. Resource checking (robots.txt, sitemap, SSL)
//! 2. Page discovery and crawling
//! 3. SEO analysis (light or deep audit)
//! 4. Issue detection and persistence
//! 5. Summary generation (via triggers)

use crate::domain::models::PageAnalysisData;
use crate::domain::models_v2::{
    IssueSeverity, Job, JobSettings, JobStatus, LinkType, NewIssue, NewLink, Page,
};
use crate::service::job_processor::PageEdge;
use crate::{
    repository::sqlite_v2::*,
    service::{Auditor, DeepAuditor, LightAuditor, PageDiscovery, ResourceChecker},
};

use anyhow::{Context, Result};
use dashmap::DashMap;
use scraper::Html;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::time::sleep;
use url::Url;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Number of URLs to process in parallel during crawling
const CRAWL_BATCH_SIZE: usize = 8;

/// Polling interval when no pending jobs are found
const JOB_POLL_INTERVAL: Duration = Duration::from_secs(15);

/// Delay after job fetch failure before retrying
const JOB_FETCH_RETRY_DELAY: Duration = Duration::from_secs(10);

// ============================================================================
// JOB PROCESSOR V2
// ============================================================================

/// Orchestrates SEO analysis jobs using the V2 schema.
///
/// The processor manages the full lifecycle of analysis jobs:
/// - Polls for pending jobs and processes them sequentially
/// - Coordinates resource checking, crawling, and analysis
/// - Supports job cancellation via atomic flags
/// - Emits progress events to the frontend
pub struct JobProcessorV2<R: tauri::Runtime = tauri::Wry> {
    // Repositories (V2)
    job_db: JobRepositoryV2,
    page_db: PageRepositoryV2,
    issue_db: IssueRepositoryV2,
    link_db: LinkRepositoryV2,

    // Services
    #[allow(dead_code)]
    discovery: PageDiscovery,
    resource_checker: ResourceChecker,
    light_auditor: Arc<LightAuditor>,
    deep_auditor: Arc<DeepAuditor>,

    // Runtime state
    cancel_map: Arc<DashMap<String, Arc<AtomicBool>>>,
    app_handle: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> JobProcessorV2<R> {
    // ========================================================================
    // CONSTRUCTION
    // ========================================================================

    pub fn new(pool: SqlitePool, app_handle: tauri::AppHandle<R>) -> Self {
        Self {
            // V2 Repositories
            job_db: JobRepositoryV2::new(pool.clone()),
            page_db: PageRepositoryV2::new(pool.clone()),
            issue_db: IssueRepositoryV2::new(pool.clone()),
            link_db: LinkRepositoryV2::new(pool.clone()),
            // Services
            discovery: PageDiscovery::new(),
            resource_checker: ResourceChecker::new(),
            light_auditor: Arc::new(LightAuditor::new()),
            deep_auditor: Arc::new(DeepAuditor::new()),
            // Runtime state
            cancel_map: Arc::new(DashMap::with_capacity(10)),
            app_handle,
        }
    }

    // ========================================================================
    // JOB POLLING
    // ========================================================================

    /// Main polling loop - fetches and processes pending jobs.
    pub async fn run(&self) -> Result<()> {
        log::info!("JobProcessorV2: Starting job polling loop");

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
        self.persist_links(&job.id, &crawl_result.edges).await?;

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
        // Use deep auditor if rate limit is high enough
        if settings.rate_limit_ms >= 2000 {
            self.deep_auditor.clone()
        } else {
            self.light_auditor.clone()
        }
    }

    async fn persist_links(&self, job_id: &str, edges: &[PageEdge]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }

        let links: Vec<NewLink> = edges
            .iter()
            .map(|e| NewLink {
                job_id: job_id.to_string(),
                source_page_id: e.from_page_id.clone(),
                target_page_id: None, // Will be resolved later if needed
                target_url: e.to_url.clone(),
                link_text: None,
                link_type: if e.is_internal(job_id) {
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
        // Stage tracking is done via status - no separate stage update needed
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
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: Vec<(String, i64)> = vec![(start_url.to_string(), 0)];
        let max_pages = job.settings.max_pages as usize;
        let max_depth = job.settings.max_depth;

        // Update status to running (analyzing phase)
        self.job_db
            .update_status(&job.id, JobStatus::Running)
            .await?;

        while !queue.is_empty() && visited.len() < max_pages {
            if cancel_flag.load(Ordering::Relaxed) {
                log::info!("Job {} cancelled during crawl", job.id);
                break;
            }

            // Take batch of URLs to process
            let batch: Vec<_> = queue
                .drain(..queue.len().min(CRAWL_BATCH_SIZE))
                .filter(|(url, _)| !visited.contains(url))
                .collect();

            for (url, depth) in batch {
                if visited.len() >= max_pages || depth > max_depth {
                    continue;
                }
                visited.insert(url.clone());

                // Fetch and analyze page
                match self.analyze_page(&url, &job.id, depth, &auditor).await {
                    Ok((page_result, new_urls)) => {
                        result.pages += 1;
                        result.issues += page_result.issues.len();

                        // Enqueue new URLs
                        for new_url in new_urls {
                            if !visited.contains(&new_url) {
                                queue.push((new_url, depth + 1));
                            }
                        }

                        // Collect edges
                        result.edges.extend(page_result.edges);
                    }
                    Err(e) => {
                        log::warn!("Failed to analyze {}: {}", url, e);
                    }
                }

                // Report progress
                let progress = (visited.len() as f64 / max_pages as f64) * 100.0;
                self.report_progress(&job.id, progress, visited.len() as i64)
                    .await;

                // Apply rate limiting
                self.apply_request_delay(&job.settings).await;
            }
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
        // (Html is not Send, so we must complete all HTML operations synchronously)
        let (page, v2_issues, new_urls, edges) = {
            let html = Html::parse_document(&audit_result.html);

            // Extract SEO data using the existing PageAnalysisData builder
            let (page_analysis, issues) = PageAnalysisData::build_from_parsed(
                url.to_string(),
                html,
                audit_result.load_time_ms,
                audit_result.status_code as i64,
                audit_result.content_size as i64,
            );

            // Create V2 Page
            let page = Page {
                id: uuid::Uuid::new_v4().to_string(),
                job_id: job_id.to_string(),
                url: url.to_string(),
                depth,
                status_code: Some(audit_result.status_code as i64),
                content_type: None, // Not available from audit result
                title: page_analysis.title.clone(),
                meta_description: page_analysis.meta_description.clone(),
                canonical_url: page_analysis.canonical_url.clone(),
                robots_meta: None,
                word_count: Some(page_analysis.word_count),
                load_time_ms: Some(audit_result.load_time_ms as i64),
                response_size_bytes: Some(audit_result.content_size as i64),
                crawled_at: chrono::Utc::now(),
            };

            // Prepare issues for insertion (without page_id yet - we'll update after)
            let v2_issues: Vec<_> = issues
                .iter()
                .map(|i| (i.title.clone(), i.description.clone(), i.recommendation.clone(), i.issue_type.clone()))
                .collect();

            // Extract links for further crawling
            let new_urls: Vec<String> = page_analysis
                .detailed_links
                .iter()
                .filter(|l| l.is_internal)
                .map(|l| l.href.clone())
                .collect();

            // Build edges data (without page_id yet)
            let edges: Vec<_> = page_analysis
                .detailed_links
                .iter()
                .map(|l| (l.href.clone(), l.status_code.unwrap_or(200)))
                .collect();

            (page, v2_issues, new_urls, edges)
        }; // Html is dropped here

        // Now we can safely await - Insert page
        let page_id = self.page_db.insert(&page).await?;

        // Convert and insert issues with the actual page_id
        let issues: Vec<NewIssue> = v2_issues
            .into_iter()
            .map(|(title, description, recommendation, issue_type)| NewIssue {
                job_id: job_id.to_string(),
                page_id: Some(page_id.clone()),
                issue_type: title,
                severity: match issue_type {
                    crate::domain::models::IssueType::Critical => IssueSeverity::Critical,
                    crate::domain::models::IssueType::Warning => IssueSeverity::Warning,
                    crate::domain::models::IssueType::Suggestion => IssueSeverity::Info,
                },
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
/// TODO: These will be used for robots.txt parsing and sitemap discovery in future.
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

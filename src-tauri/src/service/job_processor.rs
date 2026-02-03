//! Job processing orchestration for SEO analysis.
//!
//! This module coordinates the analysis pipeline:
//! 1. Resource checking (robots.txt, sitemap, SSL)
//! 2. Page discovery and crawling
//! 3. SEO analysis (light or deep audit)
//! 4. Issue detection and persistence
//! 5. Summary generation

use crate::domain::models::{
    AnalysisJob, AnalysisSettings, IssueType, JobStatus, PageAnalysisData,
    ResourceStatus, SeoIssue,
};
use crate::{
    repository::sqlite::*,
    service::{AuditMode, Auditor, DeepAuditor, LightAuditor, PageDiscovery, ResourceChecker},
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
// JOB PROCESSOR
// ============================================================================

/// Orchestrates SEO analysis jobs from discovery through completion.
/// 
/// The processor manages the full lifecycle of analysis jobs:
/// - Polls for pending jobs and processes them sequentially
/// - Coordinates resource checking, crawling, and analysis
/// - Supports job cancellation via atomic flags
/// - Emits progress events to the frontend
pub struct JobProcessor<R: tauri::Runtime = tauri::Wry> {
    // Repositories
    job_db: JobRepository,
    settings_db: SettingsRepository,
    results_db: ResultsRepository,
    page_db: PageRepository,
    issues_db: IssuesRepository,
    summary_db: SummaryRepository,
    
    // Services
    #[allow(dead_code)] // Reserved for standalone discovery
    discovery: PageDiscovery,
    resource_checker: ResourceChecker,
    light_auditor: Arc<LightAuditor>,
    deep_auditor: Arc<DeepAuditor>,
    
    // Runtime state
    cancel_map: Arc<DashMap<i64, Arc<AtomicBool>>>,
    app_handle: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> JobProcessor<R> {
    // ========================================================================
    // CONSTRUCTION
    // ========================================================================
    
    pub fn new(pool: SqlitePool, app_handle: tauri::AppHandle<R>) -> Self {
        Self {
            // Repositories
            job_db: JobRepository::new(pool.clone()),
            settings_db: SettingsRepository::new(pool.clone()),
            results_db: ResultsRepository::new(pool.clone()),
            page_db: PageRepository::new(pool.clone()),
            issues_db: IssuesRepository::new(pool.clone()),
            summary_db: SummaryRepository::new(pool.clone()),
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
    // PUBLIC API
    // ========================================================================

    /// Runs the job processing loop indefinitely.
    /// 
    /// Polls for pending jobs and processes them one at a time.
    pub async fn run(&self) -> Result<()> {
        log::info!("Job processor started");

        loop {
            match self.job_db.get_pending_jobs().await {
                Ok(jobs) if jobs.is_empty() => {
                    sleep(JOB_POLL_INTERVAL).await;
                }
                Ok(jobs) => {
                    for job in jobs {
                        if let Err(e) = self.process_job(job.clone()).await {
                            log::error!("Job {} failed: {}", job.id, e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to fetch pending jobs: {}", e);
                    sleep(JOB_FETCH_RETRY_DELAY).await;
                }
            }
        }
    }

    /// Cancels a running job.
    pub async fn cancel(&self, job_id: i64) -> Result<()> {
        log::info!("Cancelling job {}", job_id);
        self.set_cancelled(job_id);
        self.job_db.update_status(job_id, JobStatus::Failed).await
    }

    // ========================================================================
    // JOB LIFECYCLE
    // ========================================================================

    /// Processes a single analysis job through its full lifecycle.
    pub(crate) async fn process_job(&self, mut job: AnalysisJob) -> Result<String> {
        let timer = JobTimer::start(job.id);
        let cancel_flag = self.get_cancel_flag(job.id);

        // Initialize job
        job.status = JobStatus::Discovering;
        self.job_db.update_status(job.id, job.status.clone()).await?;

        // Load configuration
        let settings = self.load_settings(&job).await?;
        let start_url = self.parse_job_url(&job)?;

        // Check site resources (robots.txt, sitemap, SSL)
        let resources = self.check_site_resources(&start_url).await?;

        // Create analysis record
        let analysis_id = self.create_analysis_record(&job, &resources).await?;

        // Early exit if cancelled
        if self.is_cancelled(job.id) {
            log::warn!("Job {} cancelled before crawl", job.id);
            return Ok(analysis_id);
        }

        // Run discovery and analysis
        let auditor = self.select_auditor(&settings);
        let crawl_result = self
            .crawl_and_analyze(&start_url, &settings, &analysis_id, &job, &cancel_flag, auditor)
            .await?;

        // Persist results
        self.persist_edges(&crawl_result.edges).await?;
        self.generate_summary(&analysis_id, &crawl_result).await?;

        // Finalize job
        let final_status = self.finalize_job(job.id, &analysis_id).await?;
        timer.finish(final_status);

        Ok(analysis_id)
    }

    // ========================================================================
    // PIPELINE STAGES
    // ========================================================================

    async fn load_settings(&self, job: &AnalysisJob) -> Result<AnalysisSettings> {
        self.settings_db
            .get_by_id(job.settings_id)
            .await
            .context("Failed to load analysis settings")
    }

    fn parse_job_url(&self, job: &AnalysisJob) -> Result<Url> {
        Url::parse(&job.url).context(format!("Invalid job URL: {}", job.url))
    }

    async fn check_site_resources(&self, url: &Url) -> Result<SiteResources> {
        log::info!("Checking site resources for {}", url);
        
        let robots = self.resource_checker.check_robots_txt(url.clone()).await?;
        let sitemap = self.resource_checker.check_sitemap_xml(url.clone()).await?;
        let has_ssl = self.resource_checker.check_ssl_certificate(url);

        log::debug!("Resources: robots={:?}, sitemap={:?}, ssl={}", robots, sitemap, has_ssl);
        Ok(SiteResources { robots, sitemap, has_ssl })
    }

    async fn create_analysis_record(&self, job: &AnalysisJob, resources: &SiteResources) -> Result<String> {
        let analysis_id = self.results_db
            .create(&job.url, resources.sitemap.exists(), resources.robots.exists(), resources.has_ssl)
            .await
            .context("Failed to create analysis record")?;

        self.job_db
            .link_to_result(job.id, &analysis_id)
            .await
            .context("Failed to link job to analysis")?;

        log::debug!("Created analysis record: {}", analysis_id);
        Ok(analysis_id)
    }

    fn select_auditor(&self, settings: &AnalysisSettings) -> &dyn Auditor {
        let mode = AuditMode::from_deep_enabled(settings.lighthouse_analysis);
        match mode {
            AuditMode::Deep => {
                log::info!("Using deep auditor (Lighthouse enabled)");
                self.deep_auditor.as_ref()
            }
            AuditMode::Light => {
                log::info!("Using light auditor");
                self.light_auditor.as_ref()
            }
        }
    }

    async fn persist_edges(&self, edges: &[PageEdge]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }
        log::debug!("Persisting {} page edges", edges.len());
        self.page_db.insert_edges_batch(edges).await
    }

    async fn generate_summary(&self, analysis_id: &str, result: &CrawlResult) -> Result<()> {
        self.summary_db
            .generate_summary(analysis_id, &result.issues, &result.pages)
            .await
            .context("Failed to generate summary")
    }

    async fn finalize_job(&self, job_id: i64, analysis_id: &str) -> Result<JobStatus> {
        let status = if self.is_cancelled(job_id) {
            JobStatus::Failed
        } else {
            JobStatus::Completed
        };

        self.results_db.finalize(analysis_id, status.clone()).await?;
        self.job_db.update_status(job_id, status.clone()).await?;

        Ok(status)
    }

    // ========================================================================
    // CANCELLATION
    // ========================================================================

    fn get_cancel_flag(&self, job_id: i64) -> Arc<AtomicBool> {
        self.cancel_map
            .entry(job_id)
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone()
    }

    fn set_cancelled(&self, job_id: i64) {
        self.get_cancel_flag(job_id).store(true, Ordering::Relaxed);
    }

    fn is_cancelled(&self, job_id: i64) -> bool {
        self.get_cancel_flag(job_id).load(Ordering::Relaxed)
    }

    // ========================================================================
    // EVENTS
    // ========================================================================

    fn emit_progress(&self, job_id: i64, pages_analyzed: usize) {
        #[derive(Clone, Serialize)]
        struct DiscoveryProgress { job_id: i64, count: usize }

        let _ = self.app_handle.emit("discovery-progress", DiscoveryProgress {
            job_id,
            count: pages_analyzed,
        });
    }

    // ========================================================================
    // CRAWLING
    // ========================================================================

    /// Crawls pages starting from the given URL and analyzes each one.
    /// 
    /// Uses breadth-first crawling with batched analysis. Each page is visited
    /// once, analyzed, and its links are queued for further crawling.
    async fn crawl_and_analyze(
        &self,
        start_url: &Url,
        settings: &AnalysisSettings,
        analysis_id: &str,
        job: &AnalysisJob,
        cancel_flag: &AtomicBool,
        auditor: &dyn Auditor,
    ) -> Result<CrawlResult> {
        log::info!("Starting crawl from {} (max_pages: {})", start_url, settings.max_pages);
        
        let mut state = CrawlState::new(start_url);
        let mut result = CrawlResult::default();
        
        self.job_db.update_status(job.id, JobStatus::Processing).await?;
        
        while state.has_work() && result.pages.len() < settings.max_pages as usize {
            if cancel_flag.load(Ordering::Relaxed) {
                log::warn!("Crawl cancelled for job {}", job.id);
                break;
            }
            
            let batch = state.take_batch(settings.max_pages as usize - result.pages.len());
            if batch.is_empty() {
                continue;
            }
            
            self.apply_request_delay(settings).await;
            
            let batch_results = auditor.analyze_urls(&batch).await;
            
            for (url_str, audit_result) in batch.iter().zip(batch_results) {
                match audit_result {
                    Ok(audit) => {
                        let processed = self
                            .process_page_result(url_str, audit, analysis_id, &mut state)
                            .await?;
                        result.merge(processed);
                    }
                    Err(e) => {
                        log::warn!("Failed to analyze {}: {}", url_str, e);
                    }
                }
            }
            
            self.report_progress(job.id, analysis_id, &result, settings).await?;
        }
        
        log::info!(
            "Crawl complete: {} pages, {} issues, {} edges",
            result.pages.len(), result.issues.len(), result.edges.len()
        );
        
        Ok(result)
    }

    /// Processes a successful page audit result.
    async fn process_page_result(
        &self,
        url_str: &str,
        audit: crate::service::AuditResult,
        analysis_id: &str,
        state: &mut CrawlState,
    ) -> Result<CrawlResult> {
        let url = Url::parse(url_str)?;
        let legacy_scores: crate::service::LighthouseScores = audit.scores.into();
        
        // Build page data from HTML
        let document = Html::parse_document(&audit.html);
        let (mut page, mut issues) = PageAnalysisData::build_from_parsed_with_lighthouse(
            url.to_string(),
            document,
            audit.load_time_ms / 1000.0,
            audit.status_code as i64,
            audit.content_size as i64,
            Some(legacy_scores),
        );
        page.analysis_id = analysis_id.to_string();
        
        // Persist page
        let page_id = self.page_db.insert(&page).await.context("Failed to insert page")?;
        
        // Extract and queue new links
        let mut edges = Vec::new();
        let links = PageDiscovery::extract_links(&audit.html, &url);
        
        for link in &links {
            state.maybe_queue_link(link);
            edges.push(PageEdge::new(&page_id, link, page.status_code.unwrap_or(408) as u16));
        }
        
        // Check for HTTP errors
        if let Some(status) = page.status_code {
            if status >= 400 {
                issues.push(self.create_http_error_issue(&page_id, &page.url, status));
            }
        }
        
        // Update page_id on issues and persist
        for issue in &mut issues {
            issue.page_id = page_id.clone();
        }
        if !issues.is_empty() {
            self.issues_db.insert_batch(&issues).await?;
        }
        
        Ok(CrawlResult { pages: vec![page], issues, edges })
    }

    fn create_http_error_issue(&self, page_id: &str, url: &str, status: i64) -> SeoIssue {
        SeoIssue {
            page_id: page_id.to_string(),
            issue_type: IssueType::Critical,
            title: "HTTP Error".to_string(),
            description: format!("Page returned HTTP {}", status),
            page_url: url.to_string(),
            element: None,
            line_number: None,
            recommendation: "Fix the server error or remove links to this page".to_string(),
        }
    }

    async fn apply_request_delay(&self, settings: &AnalysisSettings) {
        if settings.delay_between_requests > 0 {
            sleep(Duration::from_millis(settings.delay_between_requests as u64)).await;
        }
    }

    async fn report_progress(
        &self,
        job_id: i64,
        analysis_id: &str,
        result: &CrawlResult,
        settings: &AnalysisSettings,
    ) -> Result<()> {
        let pages_done = result.pages.len();
        let progress = (pages_done as f64 / settings.max_pages as f64).min(1.0) * 100.0;
        
        log::debug!("Progress: {}/{} pages ({:.1}%)", pages_done, settings.max_pages, progress);
        
        self.results_db
            .update_progress(analysis_id, progress, pages_done as i64, settings.max_pages)
            .await?;
        
        self.emit_progress(job_id, pages_done);
        Ok(())
    }
}

// ============================================================================
// HELPER TYPES
// ============================================================================

/// Site resource check results.
struct SiteResources {
    robots: ResourceStatus,
    sitemap: ResourceStatus,
    has_ssl: bool,
}

/// Accumulated results from crawling.
#[derive(Default)]
struct CrawlResult {
    pages: Vec<PageAnalysisData>,
    issues: Vec<SeoIssue>,
    edges: Vec<PageEdge>,
}

impl CrawlResult {
    fn merge(&mut self, other: CrawlResult) {
        self.pages.extend(other.pages);
        self.issues.extend(other.issues);
        self.edges.extend(other.edges);
    }
}

/// Tracks crawl progress and URL frontier.
struct CrawlState {
    visited: HashSet<String>,
    queue: Vec<Url>,
    base_host: String,
    base_port: Option<u16>,
}

impl CrawlState {
    fn new(start_url: &Url) -> Self {
        Self {
            visited: HashSet::new(),
            queue: vec![start_url.clone()],
            base_host: start_url.host_str().unwrap_or("").to_string(),
            base_port: start_url.port(),
        }
    }
    
    fn has_work(&self) -> bool {
        !self.queue.is_empty()
    }
    
    fn take_batch(&mut self, max_remaining: usize) -> Vec<String> {
        let batch_size = max_remaining.min(CRAWL_BATCH_SIZE).min(self.queue.len());
        
        (0..batch_size)
            .filter_map(|_| self.queue.pop())
            .filter(|url| {
                let url_str = url.to_string();
                // Mark as visited and include in batch only if not already visited
                self.visited.insert(url_str)
            })
            .map(|u| u.to_string())
            .collect()
    }
    
    fn maybe_queue_link(&mut self, link: &str) {
        if let Ok(url) = Url::parse(link) {
            let is_same_origin = url.host_str() == Some(&self.base_host) 
                && url.port() == self.base_port;
            let not_visited = !self.visited.contains(link);
            let not_queued = !self.queue.iter().any(|u| u.as_str() == link);
            
            if is_same_origin && not_visited && not_queued {
                self.queue.push(url);
            }
        }
    }
}

/// Simple timer for job duration logging.
struct JobTimer {
    job_id: i64,
    start: std::time::Instant,
}

impl JobTimer {
    fn start(job_id: i64) -> Self {
        log::info!("Starting job {}", job_id);
        Self { job_id, start: std::time::Instant::now() }
    }
    
    fn finish(self, status: JobStatus) {
        log::info!("Job {} finished: {:?} in {:?}", self.job_id, status, self.start.elapsed());
    }
}

// ============================================================================
// PAGE EDGE
// ============================================================================

/// Represents a link relationship between pages.
/// 
/// Used to build the site graph and detect broken links.
#[derive(Debug, Clone, Serialize)]
pub struct PageEdge {
    /// ID of the source page (foreign key)
    pub from_page_id: String,
    /// Absolute URL of the target
    pub to_url: String,
    /// HTTP status code when the source page was fetched
    pub status_code: u16,
}

impl PageEdge {
    /// Creates a new page edge.
    pub fn new(from_page_id: impl Into<String>, to_url: impl Into<String>, status_code: u16) -> Self {
        Self {
            from_page_id: from_page_id.into(),
            to_url: to_url.into(),
            status_code,
        }
    }

    /// Checks if this edge points to an internal page (same origin).
    pub fn is_internal(&self, base: &str) -> bool {
        let (base_url, target_url) = match (Url::parse(base), Url::parse(&self.to_url)) {
            (Ok(b), Ok(t)) => (b, t),
            _ => return false,
        };
        
        base_url.scheme() == target_url.scheme()
            && base_url.host_str() == target_url.host_str()
            && base_url.port() == target_url.port()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{assertions, fixtures, mocks};

    #[tokio::test]
    #[ignore = "V1 schema no longer exists after migration 0018. Use JobProcessorV2 tests instead."]
    async fn test_end_to_end_job_processing() {
        // 1. Setup Mock Server with HTML that has an image without alt text
        let mut server = mockito::Server::new_async().await;
        let html_body = mocks::html_with_missing_alt();

        let _m1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(&html_body)
            .create_async()
            .await;

        // Mock robots/sitemap to avoid errors
        let _m2 = server
            .mock("GET", "/robots.txt")
            .with_status(404)
            .create_async()
            .await;
        let _m3 = server
            .mock("GET", "/sitemap.xml")
            .with_status(404)
            .create_async()
            .await;

        let server_url = server.url();

        // 2. Setup Processor using shared fixture
        let pool = fixtures::setup_test_db().await;
        let processor = JobProcessor::new(pool.clone(), tauri::test::mock_app().handle().clone());
        let job_repo = JobRepository::new(pool.clone());

        // 3. Create Job with minimal settings
        let settings = fixtures::settings_with_max_pages(1);

        let job_id = job_repo
            .create_with_settings(&server_url, &settings)
            .await
            .unwrap();

        let job = job_repo.get_pending_jobs().await.unwrap().pop().unwrap();

        // 4. Run Processing
        let _result_id = processor
            .process_job(job)
            .await
            .expect("Job processing failed");

        // 5. Verify Results
        let results_repo = ResultsRepository::new(pool.clone());
        let result = results_repo.get_result_by_job_id(job_id).await.unwrap();

        // Check Job Status - verify behavior, not implementation
        assert_eq!(
            result.analysis.status,
            JobStatus::Completed,
            "Job should complete successfully"
        );

        // Check Page Data
        assert!(
            !result.pages.is_empty(),
            "Should have at least one analyzed page"
        );
        let page = &result.pages[0];
        assert!(page.title.is_some(), "Page should have a title");
        assert_eq!(page.h1_count, 1, "Page should have one H1 tag");

        // Check Issues using assertion helper - uses the constant from PageAnalysisData
        assert!(
            assertions::has_issue(&result.issues, PageAnalysisData::ISSUE_IMG_MISSING_ALT),
            "Expected to find '{}' issue",
            PageAnalysisData::ISSUE_IMG_MISSING_ALT
        );
    }

    // ===== Unit tests for extract_links =====

    #[test]
    fn test_extract_links_from_html() {
        let base = Url::parse("https://example.com/page").unwrap();
        let html = r#"
            <html>
                <body>
                    <a href="/about">About</a>
                    <a href="https://external.com/link">External</a>
                    <a href="contact.html">Relative</a>
                </body>
            </html>
        "#;

        let links = PageDiscovery::extract_links(html, &base);

        assert_eq!(links.len(), 3, "Should extract 3 links");
        assert!(
            links.iter().any(|l| l.contains("/about")),
            "Should find /about link"
        );
        assert!(
            links.iter().any(|l| l.contains("external.com")),
            "Should find external link"
        );
        assert!(
            links.iter().any(|l| l.contains("contact.html")),
            "Should find relative link"
        );
    }

    #[test]
    fn test_extract_links_empty_html() {
        let base = Url::parse("https://example.com").unwrap();
        let html = "<html><body><p>No links here</p></body></html>";

        let links = PageDiscovery::extract_links(html, &base);
        assert!(
            links.is_empty(),
            "Should return empty list for HTML without links"
        );
    }

    // ===== Unit tests for PageEdge.is_internal =====

    #[test]
    fn test_page_edge_is_internal_same_domain() {
        let edge = PageEdge {
            from_page_id: "page1".to_string(),
            to_url: "https://example.com/about".to_string(),
            status_code: 200,
        };

        assert!(
            edge.is_internal("https://example.com"),
            "Same domain should be internal"
        );
        assert!(
            edge.is_internal("https://example.com/other"),
            "Same domain with path should be internal"
        );
    }

    #[test]
    fn test_page_edge_is_external_different_domain() {
        let edge = PageEdge {
            from_page_id: "page1".to_string(),
            to_url: "https://other.com/page".to_string(),
            status_code: 200,
        };

        assert!(
            !edge.is_internal("https://example.com"),
            "Different domain should be external"
        );
    }

    #[test]
    fn test_page_edge_different_scheme_is_external() {
        let edge = PageEdge {
            from_page_id: "page1".to_string(),
            to_url: "http://example.com/page".to_string(),
            status_code: 200,
        };

        // HTTP vs HTTPS on same domain should be considered external (different scheme)
        assert!(
            !edge.is_internal("https://example.com"),
            "Different scheme should be external"
        );
    }

    #[test]
    fn test_page_edge_with_port() {
        let edge = PageEdge {
            from_page_id: "page1".to_string(),
            to_url: "https://example.com:8080/page".to_string(),
            status_code: 200,
        };

        assert!(
            !edge.is_internal("https://example.com"),
            "Different port should be external"
        );
        assert!(
            edge.is_internal("https://example.com:8080"),
            "Same port should be internal"
        );
    }

    #[test]
    fn test_extract_links_discord_com() {
        let base = Url::parse("https://discord.com/community/establishing-trust-with-connections-connection-details-and-linked-roles").unwrap();
        let html = crate::test_utils::mocks::discord_html();

        let links = PageDiscovery::extract_links(&html, &base);

        // We expect a significant number of links given the size of the file
        assert!(!links.is_empty(), "Should extract links from Discord HTML");

        // Check for specific links we know exist/should exist
        assert!(
            links.iter().any(|l| l.contains("/download")),
            "Should find /download link"
        );
        assert!(
            links.iter().any(|l| l.contains("/nitro")),
            "Should find /nitro link"
        );
        assert!(
            links.iter().any(|l| l.contains("/safety")),
            "Should find /safety link"
        );
        assert!(
            links.iter().any(|l| l.contains("support.discord.com")),
            "Should find support subdomain link"
        );
        assert!(
            links.iter().any(|l| l.contains("/developers")),
            "Should find /developers link"
        );

        // Check that relative links were resolved correctly
        assert!(
            links.iter().any(|l| l.starts_with("https://discord.com/")),
            "Links should be absolute"
        );
    }
}
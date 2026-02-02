//! Application layer - coordinates services
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

pub struct JobProcessor<R: tauri::Runtime = tauri::Wry> {
    job_db: JobRepository,
    settings_db: SettingsRepository,
    results_db: ResultsRepository,
    page_db: PageRepository,
    issues_db: IssuesRepository,
    summary_db: SummaryRepository,
    #[allow(dead_code)] // May be used for standalone discovery in future
    discovery: PageDiscovery,
    resource_checker: ResourceChecker,
    light_auditor: Arc<LightAuditor>,
    deep_auditor: Arc<DeepAuditor>,
    cancel_map: Arc<DashMap<i64, Arc<AtomicBool>>>,
    app_handle: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> JobProcessor<R> {
    pub fn new(pool: SqlitePool, app_handle: tauri::AppHandle<R>) -> Self {
        Self {
            job_db: JobRepository::new(pool.clone()),
            settings_db: SettingsRepository::new(pool.clone()),
            results_db: ResultsRepository::new(pool.clone()),
            page_db: PageRepository::new(pool.clone()),
            issues_db: IssuesRepository::new(pool.clone()),
            summary_db: SummaryRepository::new(pool.clone()),
            discovery: PageDiscovery::new(),
            resource_checker: ResourceChecker::new(),
            light_auditor: Arc::new(LightAuditor::new()),
            deep_auditor: Arc::new(DeepAuditor::new()),
            cancel_map: Arc::new(DashMap::with_capacity(10)),
            app_handle,
        }
    }

    fn emit_discovery_progress(&self, job_id: i64, count: usize) {
        #[derive(Clone, Serialize)]
        struct DiscoveryProgress {
            job_id: i64,
            count: usize,
        }

        let _ = self
            .app_handle
            .emit("discovery-progress", DiscoveryProgress { job_id, count });
    }

    fn cancel_flag(&self, job_id: i64) -> Arc<AtomicBool> {
        self.cancel_map
            .entry(job_id)
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone()
    }

    pub async fn cancel(&self, job_id: i64) -> Result<()> {
        self.cancel_flag(job_id).store(true, Ordering::Relaxed);
        self.job_db.update_status(job_id, JobStatus::Failed).await
    }

    fn is_cancelled(&self, job_id: i64) -> bool {
        self.cancel_flag(job_id).load(Ordering::Relaxed)
    }

    pub async fn run(&self) -> Result<()> {
        log::info!("Starting SEO analysis job processor");

        loop {
            match self.job_db.get_pending_jobs().await {
                Ok(jobs) => {
                    if jobs.is_empty() {
                        sleep(Duration::from_secs(15)).await;
                        continue;
                    }

                    for job in jobs {
                        if let Err(e) = self.process_job(job.clone()).await {
                            log::error!("Failed to process job {}: {}", &job.id, e);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to fetch jobs: {}", e);
                    sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }

    pub(crate) async fn process_job(&self, mut job: AnalysisJob) -> Result<String> {
        log::info!("========================================");
        log::info!("[STAGE 1/7] Starting job {} for URL: {}", job.id, job.url);
        log::info!("========================================");
        let job_start_time = std::time::Instant::now();
        let cancel_flag = self.cancel_flag(job.id);

        // 1. Update status
        log::debug!("[JOB {}] Setting initial status to Discovering", job.id);
        job.status = JobStatus::Discovering;
        self.job_db
            .update_status(job.id, job.status.clone())
            .await?;

        // 2. Fetch settings
        log::debug!("[JOB {}] [STAGE 2/7] Fetching analysis settings (settings_id: {})", job.id, job.settings_id);
        let settings = self
            .settings_db
            .get_by_id(job.settings_id)
            .await
            .context("Failed to fetch analysis settings")?;
        log::debug!(
            "[JOB {}] Settings loaded: max_pages={}, delay={}ms, lighthouse={}",
            job.id, settings.max_pages, settings.delay_between_requests, settings.lighthouse_analysis
        );

        let start_url = Url::parse(&job.url).context(format!("Unable to Parse URL {}", job.url))?;
        log::debug!("[JOB {}] Parsed URL: scheme={}, host={:?}", job.id, start_url.scheme(), start_url.host_str());

        // 3. Check resources in parallel
        log::info!("[JOB {}] [STAGE 3/7] Checking site resources (robots.txt, sitemap.xml, SSL)", job.id);
        let resource_check_start = std::time::Instant::now();
        let robots_status: ResourceStatus = self
            .resource_checker
            .check_robots_txt(start_url.clone())
            .await?;
        log::debug!("[JOB {}] robots.txt status: {:?}", job.id, robots_status);
        let sitemap_status: ResourceStatus = self
            .resource_checker
            .check_sitemap_xml(start_url.clone())
            .await?;
        log::debug!("[JOB {}] sitemap.xml status: {:?}", job.id, sitemap_status);
        let has_ssl = self.resource_checker.check_ssl_certificate(&start_url);
        log::debug!("[JOB {}] SSL certificate: {}", job.id, has_ssl);
        log::info!("[JOB {}] Resource check completed in {:?}", job.id, resource_check_start.elapsed());

        // 4. Create analysis record
        log::info!("[JOB {}] [STAGE 4/7] Creating analysis record in database", job.id);
        let analysis_result_id = self
            .results_db
            .create(
                &job.url,
                sitemap_status.exists(),
                robots_status.exists(),
                has_ssl,
            )
            .await
            .context("Unable to create Result index")?;
        log::debug!("[JOB {}] Analysis record created with ID: {}", job.id, analysis_result_id);

        self.job_db
            .link_to_result(job.id, &analysis_result_id)
            .await
            .context("Unable to link job to result")?;
        log::debug!("[JOB {}] Job linked to analysis result", job.id);

        if self.is_cancelled(job.id) {
            log::warn!("[JOB {}] Job cancelled before discovery phase", job.id);
            return Ok(analysis_result_id);
        }

        // 5. Discovery + Analysis
        // When Deep audit is enabled, we do discovery and analysis in ONE pass
        // to avoid fetching each page twice and to capture JS-rendered content
        log::info!("[JOB {}] [STAGE 5/7] Starting discovery and analysis phase", job.id);
        
        let audit_mode = AuditMode::from_deep_enabled(settings.lighthouse_analysis);
        let auditor: &dyn Auditor = match audit_mode {
            AuditMode::Deep => self.deep_auditor.as_ref(),
            AuditMode::Light => self.light_auditor.as_ref(),
        };
        
        log::info!(
            "[JOB {}] Mode: {} ({})",
            job.id,
            auditor.name(),
            if settings.lighthouse_analysis { "deep" } else { "light" }
        );
        let discovery_start = std::time::Instant::now();
        let (all_issues, analyzed_page_data, edges) = self.discover_and_analyze(
            &start_url,
            &settings,
            &analysis_result_id,
            &job,
            cancel_flag.as_ref(),
            auditor,
        )
        .await?;
        log::info!(
            "[JOB {}] Discovery+Analysis completed in {:?} - Pages: {}, Issues: {}, Edges: {}",
            job.id, discovery_start.elapsed(), analyzed_page_data.len(), all_issues.len(), edges.len()
        );

        // ---- persist edges ----
        log::info!("[JOB {}] [STAGE 6/7] Persisting {} page edges to database", job.id, edges.len());
        if !edges.is_empty() {
            let edges_start = std::time::Instant::now();
            self.page_db.insert_edges_batch(&edges).await?;
            log::debug!("[JOB {}] Edges persisted in {:?}", job.id, edges_start.elapsed());
        } else {
            log::debug!("[JOB {}] No edges to persist", job.id);
        }

        log::info!("[JOB {}] Page analysis completed", job.id);

        // 6. Generate summary
        log::info!("[JOB {}] [STAGE 7/7] Generating analysis summary", job.id);
        let summary_start = std::time::Instant::now();
        self.summary_db
            .generate_summary(&analysis_result_id, &all_issues, &analyzed_page_data)
            .await
            .context("Unable to update issues for analysis")?;
        log::debug!("[JOB {}] Summary generated in {:?}", job.id, summary_start.elapsed());

        // 7. Finalise
        log::debug!("[JOB {}] Finalizing job status", job.id);
        let final_status = if self.is_cancelled(job.id) {
            log::warn!("[JOB {}] Job was cancelled, marking as Failed", job.id);
            self.results_db
                .finalize(&analysis_result_id, JobStatus::Failed)
                .await?;
            self.job_db.update_status(job.id, JobStatus::Failed).await?;
            JobStatus::Failed
        } else {
            self.results_db
                .finalize(&analysis_result_id, JobStatus::Completed)
                .await?;
            self.job_db
                .update_status(job.id, JobStatus::Completed)
                .await?;
            JobStatus::Completed
        };

        log::info!("========================================");
        log::info!(
            "[JOB {}] COMPLETED - Status: {:?}, Total time: {:?}",
            job.id, final_status, job_start_time.elapsed()
        );
        log::info!("========================================");
        Ok(analysis_result_id)
    }

    /// Unified discovery + analysis using the provided auditor.
    /// 
    /// Each page is visited ONCE, extracted for links, and analyzed.
    /// The auditor determines the analysis depth (Light vs Deep).
    async fn discover_and_analyze(
        &self,
        start_url: &Url,
        settings: &AnalysisSettings,
        analysis_result_id: &str,
        job: &AnalysisJob,
        cancel_flag: &AtomicBool,
        auditor: &dyn Auditor,
    ) -> Result<(Vec<SeoIssue>, Vec<PageAnalysisData>, Vec<PageEdge>)> {
        const BATCH_SIZE: usize = 8;
        
        log::info!(
            "[JOB {}] Starting {} analysis (batch_size: {})", 
            job.id, auditor.name(), BATCH_SIZE
        );
        
        let base_host = start_url.host_str().unwrap_or("").to_string();
        let base_port = start_url.port();
        
        let mut visited: HashSet<String> = HashSet::new();
        let mut to_visit: Vec<Url> = vec![start_url.clone()];
        
        let mut all_issues: Vec<SeoIssue> = Vec::new();
        let mut analyzed_page_data: Vec<PageAnalysisData> = Vec::new();
        let mut edges: Vec<PageEdge> = Vec::new();
        
        // Update status to Processing
        self.job_db
            .update_status(job.id, JobStatus::Processing)
            .await?;
        
        // Process pages until we hit the limit or exhaust all URLs
        while !to_visit.is_empty() && analyzed_page_data.len() < settings.max_pages as usize {
            if cancel_flag.load(Ordering::Relaxed) {
                log::warn!("[JOB {}] Crawl cancelled by user", job.id);
                break;
            }
            
            // Build batch of URLs to analyze
            let remaining = settings.max_pages as usize - analyzed_page_data.len();
            let batch_size = remaining.min(BATCH_SIZE).min(to_visit.len());
            
            let batch_urls: Vec<String> = (0..batch_size)
                .filter_map(|_| to_visit.pop())
                .filter(|url| {
                    let url_str = url.to_string();
                    if visited.contains(&url_str) {
                        false
                    } else {
                        visited.insert(url_str);
                        true
                    }
                })
                .map(|u| u.to_string())
                .collect();
            
            if batch_urls.is_empty() {
                continue;
            }
            
            log::info!(
                "[JOB {}] Analyzing batch of {} URLs (total: {}/{})",
                job.id, batch_urls.len(), analyzed_page_data.len(), settings.max_pages
            );
            
            // Apply delay if configured
            if settings.delay_between_requests > 0 {
                sleep(Duration::from_millis(settings.delay_between_requests as u64)).await;
            }
            
            // Analyze URLs using the auditor
            let batch_results = auditor.analyze_urls(&batch_urls).await;
            
            // Process results
            for (i, result) in batch_results.into_iter().enumerate() {
                let url_str = batch_urls.get(i).cloned().unwrap_or_default();
                let url = match Url::parse(&url_str) {
                    Ok(u) => u,
                    Err(_) => continue,
                };
                
                match result {
                    Ok(audit_result) => {
                        log::debug!("[JOB {}] Analysis successful for: {}", job.id, url);
                        
                        // Convert audit scores to legacy format for PageAnalysisData
                        let legacy_scores: crate::service::LighthouseScores = audit_result.scores.into();
                        
                        // Parse and build page data
                        let document = Html::parse_document(&audit_result.html);
                        let (mut page, mut issues) = PageAnalysisData::build_from_parsed_with_lighthouse(
                            url.to_string(),
                            document,
                            audit_result.load_time_ms / 1000.0,
                            audit_result.status_code as i64,
                            audit_result.content_size as i64,
                            Some(legacy_scores),
                        );
                        
                        page.analysis_id = analysis_result_id.to_string();
                        
                        // Insert page into database
                        let page_id = self.page_db
                            .insert(&page)
                            .await
                            .context("Unable to insert page analysis data")?;
                        
                        // Extract links and queue new ones
                        let targets = PageDiscovery::extract_links(&audit_result.html, &url);
                        log::debug!("[JOB {}] Found {} links on {}", job.id, targets.len(), url);
                        
                        for tgt in &targets {
                            if let Ok(target_url) = Url::parse(tgt) {
                                if target_url.host_str() == Some(&base_host)
                                    && target_url.port() == base_port
                                    && !visited.contains(tgt)
                                    && !to_visit.iter().any(|u| u.as_str() == tgt)
                                {
                                    to_visit.push(target_url);
                                }
                            }
                            
                            edges.push(PageEdge {
                                from_page_id: page_id.clone(),
                                to_url: tgt.clone(),
                                status_code: page.status_code.unwrap_or(408) as u16,
                            });
                        }
                        
                        // Check for HTTP errors
                        if let Some(status) = page.status_code {
                            if status >= 400 {
                                issues.push(SeoIssue {
                                    page_id: page_id.clone(),
                                    issue_type: IssueType::Critical,
                                    description: format!("Page returned HTTP {}", status),
                                    title: "HTTP Error".to_string(),
                                    page_url: page.url.clone(),
                                    element: None,
                                    line_number: None,
                                    recommendation: "Fix the server error or remove links to this page".to_string(),
                                });
                            }
                        }
                        
                        // Update page_id on all issues and persist
                        for issue in &mut issues {
                            issue.page_id = page_id.clone();
                        }
                        if !issues.is_empty() {
                            self.issues_db.insert_batch(&issues).await?;
                        }
                        
                        all_issues.extend(issues);
                        analyzed_page_data.push(page);
                    }
                    Err(e) => {
                        log::warn!("[JOB {}] Error analysing {}: {}", job.id, url, e);
                    }
                }
            }
            
            // Update progress
            let progress = (analyzed_page_data.len() as f64 / settings.max_pages as f64).min(1.0) * 100.0;
            log::info!(
                "[JOB {}] Progress: {}/{} ({:.1}%)",
                job.id, analyzed_page_data.len(), settings.max_pages, progress
            );
            
            self.results_db
                .update_progress(
                    analysis_result_id,
                    progress,
                    analyzed_page_data.len() as i64,
                    settings.max_pages,
                )
                .await?;
            
            self.emit_discovery_progress(job.id, analyzed_page_data.len());
        }
        
        log::info!(
            "[JOB {}] Crawl complete - Pages: {}, Issues: {}, Edges: {}",
            job.id, analyzed_page_data.len(), all_issues.len(), edges.len()
        );
        
        Ok((all_issues, analyzed_page_data, edges))
    }
}

impl JobProcessor {
}

/// A light-weight edge we can persist.
#[derive(Debug, Clone, Serialize)]
pub struct PageEdge {
    pub from_page_id: String, // FK to the row you already insert in `page_db`
    pub to_url: String,       // absolute URL
    pub status_code: u16,     // what we saw when we hit that URL
}

impl PageEdge {
    pub fn is_internal(&self, base: &str) -> bool {
        let base_url = match Url::parse(base) {
            Ok(u) => u,
            Err(_) => return false,
        };
        let target_url = match Url::parse(&self.to_url) {
            Ok(u) => u,
            Err(_) => return false,
        };
        base_url.scheme() == target_url.scheme()
            && base_url.host_str() == target_url.host_str()
            && base_url.port() == target_url.port()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{assertions, fixtures, mocks};

    #[tokio::test]
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

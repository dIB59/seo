//! Application layer - coordinates services
use crate::domain::models::{
    AnalysisJob, AnalysisSettings, AnalysisStatus, IssueType, JobStatus, PageAnalysisData,
    ResourceStatus, SeoIssue,
};

use crate::{
    repository::sqlite::*,
    service::{LighthouseService, PageDiscovery, ResourceChecker},
};
use anyhow::{Context, Result};
use dashmap::DashMap;
use scraper::{Html, Selector};
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::Mutex;
use tokio::time::sleep;
use url::Url;

pub struct JobProcessor<R: tauri::Runtime = tauri::Wry> {
    job_db: JobRepository,
    settings_db: SettingsRepository,
    results_db: ResultsRepository,
    page_db: PageRepository,
    issues_db: IssuesRepository,
    summary_db: SummaryRepository,
    discovery: PageDiscovery,
    resource_checker: ResourceChecker,
    lighthouse: Arc<Mutex<LighthouseService>>,
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
            lighthouse: Arc::new(Mutex::new(LighthouseService::new())),
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
        // When Lighthouse is enabled, we do discovery and analysis in ONE pass
        // to avoid fetching each page twice and to capture JS-rendered content
        log::info!("[JOB {}] [STAGE 5/7] Starting discovery and analysis phase", job.id);
        log::info!(
            "[JOB {}] Mode: {}",
            job.id,
            if settings.lighthouse_analysis { "Lighthouse (unified discovery+analysis)" } else { "Basic HTTP (two-phase)" }
        );
        let discovery_start = std::time::Instant::now();
        let (all_issues, analyzed_page_data, edges) = if settings.lighthouse_analysis {
            self.discover_and_analyze_with_lighthouse(
                &start_url,
                &settings,
                &analysis_result_id,
                &job,
                cancel_flag.as_ref(),
            )
            .await?
        } else {
            self.discover_then_analyze_basic(
                &start_url,
                &settings,
                &analysis_result_id,
                &job,
                cancel_flag.as_ref(),
            )
            .await?
        };
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
            log::warn!("[JOB {}] Job was cancelled, marking as Error", job.id);
            self.results_db
                .finalize(&analysis_result_id, AnalysisStatus::Error)
                .await?;
            self.job_db.update_status(job.id, JobStatus::Failed).await?;
            AnalysisStatus::Error
        } else {
            self.results_db
                .finalize(&analysis_result_id, AnalysisStatus::Completed)
                .await?;
            self.job_db
                .update_status(job.id, JobStatus::Completed)
                .await?;
            AnalysisStatus::Completed
        };

        log::info!("========================================");
        log::info!(
            "[JOB {}] COMPLETED - Status: {:?}, Total time: {:?}",
            job.id, final_status, job_start_time.elapsed()
        );
        log::info!("========================================");
        Ok(analysis_result_id)
    }

    /// Analyze a page using the Lighthouse service (headless Chrome)
    async fn analyze_page_with_lighthouse(
        &self,
        url: &Url,
    ) -> Result<(PageAnalysisData, Vec<SeoIssue>, String)> {
        log::debug!("[LIGHTHOUSE] Starting analysis for: {}", url);
        let start = std::time::Instant::now();

        log::trace!("[LIGHTHOUSE] Acquiring Lighthouse service lock...");
        let lighthouse = self.lighthouse.lock().await;
        log::trace!("[LIGHTHOUSE] Lock acquired, running analysis...");
        
        let result = lighthouse
            .analyze(url.as_str())
            .await
            .context("Lighthouse analysis failed")?;
        log::debug!(
            "[LIGHTHOUSE] Analysis returned - status: {}, content_size: {}, load_time: {:.2}ms",
            result.status_code, result.content_size, result.load_time_ms
        );
        log::trace!(
            "[LIGHTHOUSE] Scores - perf: {:?}, access: {:?}, seo: {:?}",
            result.scores.performance, result.scores.accessibility, result.scores.seo
        );

        log::trace!("[LIGHTHOUSE] Parsing HTML document ({} bytes)", result.html.len());
        let document = Html::parse_document(&result.html);

        let (page, issues) = PageAnalysisData::build_from_parsed_with_lighthouse(
            url.to_string(),
            document,
            result.load_time_ms / 1000.0, // Convert to seconds
            result.status_code as i64,
            result.content_size as i64,
            Some(result.scores),
        );
        log::debug!(
            "[LIGHTHOUSE] Page analysis complete for {} - {} issues found, took {:?}",
            url, issues.len(), start.elapsed()
        );

        Ok((page, issues, result.html))
    }

    /// Analyze a page using the HTTP client (faster, no Lighthouse)
    async fn analyze_page_basic(&self, url: &Url) -> Result<(PageAnalysisData, Vec<SeoIssue>, String)> {
        log::debug!("[BASIC] Starting HTTP analysis for: {}", url);
        let client =
            crate::service::http::create_client(crate::service::http::ClientType::HeavyEmulation)?;
        let start = std::time::Instant::now();

        // 1.  Fetch the page
        log::trace!("[BASIC] Sending HTTP GET request to {}", url);
        let response = client.get(url.as_str()).send().await?;
        let status_code = response.status().as_u16() as i64;
        let content_size = response.content_length().unwrap_or(0) as i64;
        log::debug!("[BASIC] Response received - status: {}, content_size: {}", status_code, content_size);
        let html = response.text().await?;
        let load_time = start.elapsed().as_secs_f64();
        log::debug!("[BASIC] HTML fetched ({} bytes) in {:.2}s", html.len(), load_time);

        // 2.  Parse once
        let document = Html::parse_document(&html);
        let base_url = url.clone();
        let selector = Selector::parse("a[href]").unwrap();
        for anchor in document.select(&selector) {
            let href = match anchor.value().attr("href") {
                Some(h) => h,
                None => continue,
            };
            let target = match base_url.join(href) {
                Ok(u) => u,
                Err(_) => continue,
            };
            let _text = anchor.text().collect::<String>();
            // check for nofollow
            let _nofollow = anchor
                .value()
                .attr("rel")
                .map(|r| r.contains("nofollow"))
                .unwrap_or(false);

            let _is_internal = target.domain() == base_url.domain();
            // TODO: check status code
            let _status_code = 0;
        }

        // 5.  Build the data object and issues
        let (page, issues) = PageAnalysisData::build_from_parsed(
            base_url.to_string(),
            document.clone(),
            load_time,
            status_code,
            content_size,
        );

        Ok((page, issues, html))
    }

    /// Unified discovery + analysis using Lighthouse
    /// Each page is visited ONCE via Lighthouse, which:
    /// 1. Renders JavaScript to get the full DOM
    /// 2. Extracts links from the rendered page for discovery
    /// 3. Runs Lighthouse audits for scores
    /// 4. Returns the rendered HTML for SEO analysis
    async fn discover_and_analyze_with_lighthouse(
        &self,
        start_url: &Url,
        settings: &AnalysisSettings,
        analysis_result_id: &str,
        job: &AnalysisJob,
        cancel_flag: &AtomicBool,
    ) -> Result<(Vec<SeoIssue>, Vec<PageAnalysisData>, Vec<PageEdge>)> {
        log::info!("[JOB {}] [LIGHTHOUSE-MODE] Starting unified discovery+analysis", job.id);
        log::debug!("[JOB {}] [LIGHTHOUSE-MODE] Start URL: {}, Max pages: {}", job.id, start_url, settings.max_pages);
        
        let mut visited: HashSet<String> = HashSet::new();
        let mut to_visit = vec![start_url.clone()];
        
        let base_host = start_url.host_str().unwrap_or("");
        let base_port = start_url.port();
        log::debug!("[JOB {}] [LIGHTHOUSE-MODE] Base host: {}, port: {:?}", job.id, base_host, base_port);
        
        let mut all_issues = Vec::new();
        let mut analyzed_page_data = Vec::new();
        let mut edges: Vec<PageEdge> = Vec::new();
        let mut url_to_id: HashMap<String, String> = HashMap::new();
        
        // Update status to Processing (discovery happens as part of analysis)
        log::debug!("[JOB {}] [LIGHTHOUSE-MODE] Updating job status to Processing", job.id);
        self.job_db
            .update_status(job.id, JobStatus::Processing)
            .await?;
        
        log::info!("[JOB {}] [LIGHTHOUSE-MODE] Beginning page crawl loop", job.id);
        while let Some(url) = to_visit.pop() {
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                log::warn!("[JOB {}] [LIGHTHOUSE-MODE] Crawl cancelled by user", job.id);
                break;
            }
            
            let url_str = url.to_string();
            if visited.contains(&url_str) {
                log::trace!("[JOB {}] [LIGHTHOUSE-MODE] Skipping already visited: {}", job.id, url_str);
                continue;
            }
            
            if visited.len() >= settings.max_pages as usize {
                log::info!("[JOB {}] [LIGHTHOUSE-MODE] Reached max pages limit: {}", job.id, settings.max_pages);
                break;
            }
            
            visited.insert(url_str.clone());
            log::info!(
                "[JOB {}] [LIGHTHOUSE-MODE] Processing page {}/{}: {}",
                job.id, visited.len(), settings.max_pages, url_str
            );
            self.emit_discovery_progress(job.id, visited.len());
            
            // Delay between requests
            if settings.delay_between_requests > 0 {
                log::trace!("[JOB {}] [LIGHTHOUSE-MODE] Waiting {}ms before next request", job.id, settings.delay_between_requests);
                sleep(Duration::from_millis(settings.delay_between_requests as u64)).await;
            }
            
            // Run Lighthouse analysis (includes fetching + rendering + scoring)
            log::debug!("[JOB {}] [LIGHTHOUSE-MODE] Running Lighthouse analysis for: {}", job.id, url);
            match self.analyze_page_with_lighthouse(&url).await {
                Ok((mut page, mut issues, html_str)) => {
                    log::debug!("[JOB {}] [LIGHTHOUSE-MODE] Analysis successful for: {}", job.id, url);
                    page.analysis_id = analysis_result_id.to_string();
                    
                    log::trace!("[JOB {}] [LIGHTHOUSE-MODE] Inserting page data into database", job.id);
                    let page_id = self
                        .page_db
                        .insert(&page)
                        .await
                        .context("Unable to insert page analysis data")?;
                    log::trace!("[JOB {}] [LIGHTHOUSE-MODE] Page inserted with ID: {}", job.id, page_id);
                    
                    url_to_id.insert(url_str.clone(), page_id.clone());
                    
                    // Extract links from the RENDERED HTML (includes JS-generated links)
                    log::trace!("[JOB {}] [LIGHTHOUSE-MODE] Extracting links from rendered HTML", job.id);
                    let targets = JobProcessor::extract_links(&html_str, &url);
                    log::debug!("[JOB {}] [LIGHTHOUSE-MODE] Found {} links on page", job.id, targets.len());
                    
                    let mut new_links_queued = 0;
                    for tgt in &targets {
                        // Add to discovery queue if internal and not visited
                        if let Ok(target_url) = Url::parse(tgt) {
                            if target_url.host_str() == Some(base_host)
                                && target_url.port() == base_port
                                && !visited.contains(tgt)
                                && !to_visit.iter().any(|u| u.as_str() == tgt)
                            {
                                to_visit.push(target_url);
                                new_links_queued += 1;
                            }
                        }
                        
                        let edge = PageEdge {
                            from_page_id: page_id.clone(),
                            to_url: tgt.clone(),
                            status_code: page.status_code.unwrap_or(408) as u16,
                        };
                        edges.push(edge);
                    }
                    log::debug!(
                        "[JOB {}] [LIGHTHOUSE-MODE] Queued {} new internal links (queue size: {})",
                        job.id, new_links_queued, to_visit.len()
                    );
                    
                    // Check for broken links (status code errors)
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
                    
                    for issue in &mut issues {
                        issue.page_id = page_id.clone();
                    }
                    if !issues.is_empty() {
                        log::debug!("[JOB {}] [LIGHTHOUSE-MODE] Persisting {} issues for page", job.id, issues.len());
                    }
                    self.issues_db.insert_batch(&issues).await?;
                    all_issues.extend(issues);
                    analyzed_page_data.push(page);
                    
                    // Update progress
                    let progress = (visited.len() as f64 / settings.max_pages as f64).min(1.0) * 100.0;
                    log::trace!("[JOB {}] [LIGHTHOUSE-MODE] Progress: {:.1}%", job.id, progress);
                    self.results_db
                        .update_progress(
                            analysis_result_id,
                            progress,
                            visited.len() as i64,
                            settings.max_pages,
                        )
                        .await?;
                }
                Err(e) => {
                    log::warn!("[JOB {}] [LIGHTHOUSE-MODE] Error analysing {}: {}", job.id, url, e);
                    continue;
                }
            }
        }
        
        log::info!(
            "[JOB {}] [LIGHTHOUSE-MODE] Crawl complete - Pages: {}, Issues: {}, Edges: {}",
            job.id, analyzed_page_data.len(), all_issues.len(), edges.len()
        );
        
        Ok((all_issues, analyzed_page_data, edges))
    }

    /// Traditional two-phase approach: discover first, then analyze
    /// Used when Lighthouse is disabled (faster, but no JS rendering)
    async fn discover_then_analyze_basic(
        &self,
        start_url: &Url,
        settings: &AnalysisSettings,
        analysis_result_id: &str,
        job: &AnalysisJob,
        cancel_flag: &AtomicBool,
    ) -> Result<(Vec<SeoIssue>, Vec<PageAnalysisData>, Vec<PageEdge>)> {
        // Phase 1: Discover pages (using basic HTTP)
        log::info!("[JOB {}] [BASIC-MODE] Phase 1: Starting page discovery", job.id);
        log::debug!("[JOB {}] [BASIC-MODE] Start URL: {}, Max pages: {}", job.id, start_url, settings.max_pages);
        let discovery_start = std::time::Instant::now();
        let pages = self
            .discovery
            .discover(
                start_url.clone(),
                settings.max_pages,
                settings.delay_between_requests,
                cancel_flag,
                |count| {
                    self.emit_discovery_progress(job.id, count);
                },
            )
            .await
            .context("Unable to discover pages")?;
        
        let total_pages = pages.len() as i64;
        log::info!(
            "[JOB {}] [BASIC-MODE] Phase 1 complete: Discovered {} pages in {:?}",
            job.id, total_pages, discovery_start.elapsed()
        );
        
        // Update status to Processing after discovery
        log::debug!("[JOB {}] [BASIC-MODE] Updating job status to Processing", job.id);
        self.job_db
            .update_status(job.id, JobStatus::Processing)
            .await?;
        
        self.results_db
            .update_progress(analysis_result_id, 8.0, 0, total_pages)
            .await
            .context("Unable to update Analysis Results progress")?;
        
        // Phase 2: Analyze pages
        log::info!("[JOB {}] [BASIC-MODE] Phase 2: Starting page analysis ({} pages)", job.id, total_pages);
        let analysis_start = std::time::Instant::now();
        let mut all_issues = Vec::new();
        let mut analyzed_page_data = Vec::new();
        let mut analyzed_count = 0;
        let mut url_to_id: HashMap<String, String> = HashMap::with_capacity(pages.len());
        let mut edges: Vec<PageEdge> = Vec::new();
        
        for page_url in pages {
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                log::warn!("[JOB {}] [BASIC-MODE] Analysis cancelled by user", job.id);
                break;
            }
            
            analyzed_count += 1;
            log::info!(
                "[JOB {}] [BASIC-MODE] Analyzing page {}/{}: {}",
                job.id, analyzed_count, total_pages, page_url
            );
            
            match self.analyze_page_basic(&page_url).await {
                Ok((mut page, mut issues, html_str)) => {
                    log::debug!("[JOB {}] [BASIC-MODE] Analysis successful for: {}", job.id, page_url);
                    page.analysis_id = analysis_result_id.to_string();
                    
                    log::trace!("[JOB {}] [BASIC-MODE] Inserting page data into database", job.id);
                    let page_id = self
                        .page_db
                        .insert(&page)
                        .await
                        .context("Unable to insert page analysis data")?;
                    log::trace!("[JOB {}] [BASIC-MODE] Page inserted with ID: {}", job.id, page_id);
                    
                    url_to_id.insert(page_url.to_string(), page_id.clone());
                    
                    log::trace!("[JOB {}] [BASIC-MODE] Extracting links from HTML", job.id);
                    let targets = JobProcessor::extract_links(&html_str, &page_url);
                    log::debug!("[JOB {}] [BASIC-MODE] Found {} links on page", job.id, targets.len());
                    for tgt in targets {
                        let edge = PageEdge {
                            from_page_id: page_id.clone(),
                            to_url: tgt.clone(),
                            status_code: page.status_code.unwrap_or(408) as u16,
                        };
                        edges.push(edge.clone());
                        
                        if edge.status_code >= 400 {
                            issues.push(SeoIssue {
                                page_id: page_id.to_string(),
                                issue_type: IssueType::Critical,
                                description: format!(
                                    "Broken link: {} returned {}",
                                    tgt, edge.status_code
                                ),
                                title: "Broken Link".to_string(),
                                page_url: page.url.clone(),
                                element: None,
                                line_number: None,
                                recommendation: "Remove broken Link from the page".to_string(),
                            });
                        }
                    }
                    
                    for issue in &mut issues {
                        issue.page_id = page_id.clone();
                    }
                    if !issues.is_empty() {
                        log::debug!("[JOB {}] [BASIC-MODE] Persisting {} issues for page", job.id, issues.len());
                    }
                    self.issues_db.insert_batch(&issues).await?;
                    all_issues.extend(issues);
                    analyzed_page_data.push(page);
                    
                    let progress = (analyzed_count as f64 / total_pages as f64) * 100.0;
                    log::trace!("[JOB {}] [BASIC-MODE] Progress: {:.1}%", job.id, progress);
                    self.results_db
                        .update_progress(
                            analysis_result_id,
                            progress,
                            analyzed_count,
                            total_pages,
                        )
                        .await?;
                }
                Err(e) => {
                    log::warn!("[JOB {}] [BASIC-MODE] Error analysing {}: {}", job.id, page_url, e);
                    continue;
                }
            }
        }
        
        log::info!(
            "[JOB {}] [BASIC-MODE] Phase 2 complete - Analyzed {} pages in {:?}",
            job.id, analyzed_page_data.len(), analysis_start.elapsed()
        );
        
        Ok((all_issues, analyzed_page_data, edges))
    }
}

impl JobProcessor {
    /// Extract every absolute link (`<a href="…">`) from the given HTML.
    /// The returned strings are already resolved against `base`.
    pub(crate) fn extract_links(html: &str, base: &Url) -> Vec<String> {
        // `Selector::parse` is expensive (allocations + regex compilation), so we
        // keep the parsed selector in a `std::sync::OnceLock` to pay that cost
        // exactly once per process and reuse it on every call.
        // TODO:
        // TEST IF THIS MAKES A DIFFERENCE OR NOT
        static A: OnceLock<Selector> = OnceLock::new();
        let selector = A.get_or_init(|| Selector::parse("a[href]").unwrap());
        Html::parse_document(html)
            .select(&selector)
            .filter_map(|a| a.value().attr("href"))
            .filter_map(|raw| base.join(raw).ok())
            .map(|u| u.into())
            .collect()
    }
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

        let links = JobProcessor::extract_links(html, &base);

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

        let links = JobProcessor::extract_links(html, &base);
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

        let links = JobProcessor::extract_links(&html, &base);

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

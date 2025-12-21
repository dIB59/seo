//! Application layer - coordinates services
use crate::domain::models::{
    AnalysisJob, AnalysisStatus, IssueType, JobStatus, PageAnalysisData, ResourceStatus, SeoIssue,
};

use crate::{
    repository::sqlite::*,
    service::{PageDiscovery, ResourceChecker},
};
use anyhow::{Context, Result};
use dashmap::DashMap;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

fn assert_send_static<F>(f: F) -> F
where
    F: std::future::Future + Send + 'static,
{
    f
}

pub struct JobProcessor {
    job_db: JobRepository,
    settings_db: SettingsRepository,
    results_db: ResultsRepository,
    page_db: PageRepository,
    issues_db: IssuesRepository,
    summary_db: SummaryRepository,
    discovery: PageDiscovery,
    resource_checker: ResourceChecker,
    cancel_map: Arc<DashMap<i64, Arc<AtomicBool>>>,
}

impl JobProcessor {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            job_db: JobRepository::new(pool.clone()),
            settings_db: SettingsRepository::new(pool.clone()),
            results_db: ResultsRepository::new(pool.clone()),
            page_db: PageRepository::new(pool.clone()),
            issues_db: IssuesRepository::new(pool.clone()),
            summary_db: SummaryRepository::new(pool.clone()),
            discovery: PageDiscovery::new(),
            resource_checker: ResourceChecker::new(),
            cancel_map: Arc::new(DashMap::with_capacity(10)),
        }
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
        log::info!("Processing job {} for URL: {}", job.id, job.url);
        let cancel_flag = self.cancel_flag(job.id);

        // 1. Update status
        job.status = JobStatus::Processing;
        self.job_db.update_status(job.id, job.status).await?;

        // 2. Fetch settings
        let settings = self
            .settings_db
            .get_by_id(job.settings_id)
            .await
            .context("Failed to fetch analysis settings")?;

        let start_url = Url::parse(&job.url).context(format!("Unable to Parse URL {}", job.url))?;

        // 3. Check resources in parallel
        let robots_status: ResourceStatus = self
            .resource_checker
            .check_robots_txt(start_url.clone())
            .await?;
        let sitemap_status: ResourceStatus = self
            .resource_checker
            .check_sitemap_xml(start_url.clone())
            .await?;
        let has_ssl = self.resource_checker.check_ssl_certificate(&start_url);

        // 4. Create analysis record
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

        self.job_db
            .link_to_result(job.id, &analysis_result_id)
            .await
            .context("Unable to link job to result")?;

        if self.is_cancelled(job.id) {
            return Ok(analysis_result_id);
        }

        // 5. Discover pages
        let pages = self
            .discovery
            .discover(
                start_url.clone(),
                settings.max_pages,
                settings.delay_between_requests,
                cancel_flag.as_ref(),
            )
            .await
            .context("Unable to discover pages")?;
        let total_pages = pages.len() as i32;
        log::info!("Discovered {} pages", total_pages);

        self.results_db
            .update_progress(&analysis_result_id, 8.0, 0, pages.len() as i64)
            .await
            .context("Unable to update Analysis Results progress")?;

        // 6. Analyse pages + build link graph
        let mut all_issues = Vec::new();
        let mut analyzed_page_data = Vec::new();
        let mut analyzed_count = 0;

        // url  -> page_id   (so we can resolve edges later)
        let mut url_to_id: HashMap<String, String> = HashMap::with_capacity(pages.len());

        // all edges we collect
        let mut edges: Vec<PageEdge> = Vec::new();

        log::info!("Starting page analysis for job {}", job.id);

        for page_url in pages {
            match self.analyze_page(&page_url).await {
                Ok((mut page, mut issues, html_str)) => {
                    page.analysis_id = analysis_result_id.clone();

                    let page_id = self
                        .page_db
                        .insert(&page)
                        .await
                        .context("Unable to insert page analysis data")?;

                    // remember mapping
                    url_to_id.insert(page_url.to_string(), page_id.clone());

                    // we already have page.html and page.status_code from analyze_page
                    let targets = Self::extract_links(&html_str, &page_url);
                    for tgt in targets {
                        let edge = PageEdge {
                            from_page_id: page_id.clone(),
                            to_url: tgt.clone(),
                            status_code: page.status_code.unwrap_or(408) as u16, // status of *this* page
                        };

                        edges.push(edge.clone());
                        log::info!("{:?}", edge);

                        if edge.status_code >= 400 {
                            dbg!(&edge);
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
                    self.issues_db.insert_batch(&issues).await?;
                    all_issues.extend(issues);
                    analyzed_page_data.push(page);
                    analyzed_count += 1;

                    // progress
                    let progress = (analyzed_count as f64 / total_pages as f64) * 100.0;
                    self.results_db
                        .update_progress(
                            &analysis_result_id,
                            progress,
                            analyzed_count,
                            total_pages as i64,
                        )
                        .await?;
                }
                Err(e) => {
                    log::warn!("Error analysing {}: {}", page_url, e);
                    continue;
                }
            }
        }

        // ---- persist edges ----
        if !edges.is_empty() {
            self.page_db.insert_edges_batch(&edges).await?;
        }

        log::info!("Completed page analysis for job {}", job.id);

        // 7. Generate summary
        self.summary_db
            .generate_summary(&analysis_result_id, &all_issues, &analyzed_page_data)
            .await
            .context("Unable to update issues for analysis")?;

        // 8. Finalise
        let final_status = if self.is_cancelled(job.id) {
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

        log::info!("Job {} completed with status {:?}", job.id, final_status);
        Ok(analysis_result_id)
    }

    async fn analyze_page(&self, url: &Url) -> Result<(PageAnalysisData, Vec<SeoIssue>, String)> {
        let client = Client::new();
        let start = std::time::Instant::now();

        // 1.  Fetch the page
        let response = client.get(url.as_str()).send().await?;
        let status_code = response.status().as_u16() as i64;
        let content_size = response.content_length().unwrap_or(0) as i64;
        let html = response.text().await?;
        let load_time = start.elapsed().as_secs_f64();

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
        let (mut page, mut issues) = PageAnalysisData::build_from_parsed(
            base_url.to_string(),
            document.clone(),
            load_time,
            status_code,
            content_size,
        );

        Ok((page, issues, html))
    }

    /// Extract every absolute link (`<a href="…">`) from the given HTML.
    /// The returned strings are already resolved against `base`.
    fn extract_links(html: &str, base: &Url) -> Vec<String> {
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
        let processor = JobProcessor::new(pool.clone());
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

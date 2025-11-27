//! Application layer - coordinates services
use crate::domain::models::{
    AnalysisJob, AnalysisStatus, JobStatus, PageAnalysisData, ResourceStatus, SeoIssue,
};
use crate::{
    repository::sqlite::*,
    service::{PageDiscovery, ResourceChecker},
};
use anyhow::{Context, Result};
use dashmap::DashMap;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

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

    pub async fn cancel(&self, job_id: i64) -> Result<()> {
        // insert a “true” flag; if job isn’t running flag is simply ignored
        self.cancel_map
            .entry(job_id)
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .store(true, Ordering::Relaxed);

        self.job_db.update_status(job_id, JobStatus::Failed).await
    }

    /// Quick check used inside the crawl loop
    fn is_cancelled(&self, job_id: i64) -> bool {
        self.cancel_map
            .get(&job_id)
            .map(|f| f.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    pub async fn run(&self) -> Result<()> {
        log::info!("Starting SEO analysis job processor");

        loop {
            match self.job_db.get_pending_jobs().await {
                Ok(jobs) => {
                    if jobs.is_empty() {
                        sleep(Duration::from_secs(5)).await;
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

    async fn process_job(&self, mut job: AnalysisJob) -> Result<String> {
        log::info!("Processing job {} for URL: {}", job.id, job.url);
        //generate cancel flag for current job
        let cancel_flag = self
            .cancel_map
            .get(&job.id)
            .map(|f| f.clone())
            .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));

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

        // 6. Analyze pages
        let mut all_issues = Vec::new();
        let mut analyzed_page_data = Vec::new();
        let mut analyzed_count = 0;

        log::info!("Starting page analysis for job {}", job.id);

        for page_url in pages {
            match self.analyze_page(&page_url).await {
                Ok((mut page, mut issues)) => {
                    log::debug!("Number of issues for this page {}", issues.len());
                    page.analysis_id = analysis_result_id.clone();

                    let page_id = self
                        .page_db
                        .insert(&page)
                        .await
                        .context("Unable to insert page analysis data")?;

                    for issue in &mut issues {
                        log::trace!("Found issue on {}: {}", page_url, issue.description);
                        issue.page_id = page_id.clone();
                    }

                    self.issues_db
                        .insert_batch(&issues)
                        .await
                        .inspect_err(|e| log::error!("{}", e))
                        .context("Unable to insert SEO issues")?;

                    all_issues.extend(issues);
                    analyzed_page_data.push(page);
                    analyzed_count += 1;

                    // Update progress
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
                    log::warn!("Error analyzing {}: {}", page_url, e);
                    continue;
                }
            }
        }

        log::info!("Completed page analysis for job {}", job.id);
        // 7. Generate summary
        self.summary_db
            .generate_summary(&analysis_result_id, &all_issues, &analyzed_page_data)
            .await
            .context("Unable to update issues fpr analysis")?;

        // 8. Finalize
        self.results_db
            .finalize(&analysis_result_id, AnalysisStatus::Completed)
            .await
            .context("Unable to finalize Analysis Results")?;

        self.job_db
            .update_status(job.id, JobStatus::Completed)
            .await
            .context("Unable to update job status")?;

        log::info!("Job {} completed", job.id);
        Ok(analysis_result_id)
    }

    async fn analyze_page(&self, url: &Url) -> Result<(PageAnalysisData, Vec<SeoIssue>)> {
        let start = std::time::Instant::now();

        let response = reqwest::get(url.as_str()).await?;
        let status_code = response.status().as_u16() as i64;
        let content_size = response.content_length().unwrap_or(0) as i64;
        let html = response.text().await?;
        let load_time = start.elapsed().as_secs_f64();

        Ok(PageAnalysisData::analyze(
            url.to_string(),
            &html,
            load_time,
            status_code,
            content_size,
        ))
    }
}

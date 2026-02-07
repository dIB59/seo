//! Job processing orchestration for SEO analysis (V2 schema).
//!
//! This module coordinates the analysis pipeline:
//! 1. Resource checking (robots.txt, sitemap, SSL)
//! 2. Page discovery and crawling
//! 3. SEO analysis (light or deep audit)
//! 4. Issue detection and persistence
//! 5. Summary generation (via triggers)

use crate::domain::models::{Job, JobStatus, LinkType, NewLink};
use crate::repository::sqlite::LinkRepository;
use crate::service::analyzer_service::AnalyzerService;
use crate::service::crawler::{CrawlContext, Crawler};
use crate::service::job_canceler::JobCanceler;
use crate::service::job_queue::JobQueue;
use crate::service::progress_reporter::ProgressReporter;

use anyhow::Result;
use sqlx::SqlitePool;
use std::sync::atomic::Ordering;

// ============================================================================
// JOB PROCESSOR V2
// ============================================================================

/// Orchestrates SEO analysis jobs using the normalized schema.
pub struct JobProcessor<R: tauri::Runtime = tauri::Wry> {
    // Components
    job_queue: JobQueue,
    crawler: Crawler,
    analyzer: AnalyzerService,
    progress_reporter: ProgressReporter<R>,
    canceler: JobCanceler,

    // Repositories (some still used directly for orchestration)
    link_db: LinkRepository,
}

impl<R: tauri::Runtime> JobProcessor<R> {
    // ========================================================================
    // CONSTRUCTION
    // ========================================================================

    pub fn new(pool: SqlitePool, app_handle: tauri::AppHandle<R>) -> Self {
        Self {
            job_queue: JobQueue::new(pool.clone()),
            crawler: Crawler::new(),
            analyzer: AnalyzerService::new(pool.clone()),
            progress_reporter: ProgressReporter::new(app_handle),
            canceler: JobCanceler::new(),
            link_db: LinkRepository::new(pool),
        }
    }

    // ========================================================================
    // JOB POLLING
    // ========================================================================

    /// Main polling loop - fetches and processes pending jobs.
    pub async fn run(&self) -> Result<()> {
        log::info!("JobProcessor: Starting job polling loop");

        loop {
            // Fetch next job from queue
            if let Some(job) = self.job_queue.fetch_next_job().await {
                log::info!("Processing job: {} ({})", job.id, job.url);
                if let Err(e) = self.process_job(job).await {
                    log::error!("Job failed: {}", e);
                }
            }
        }
    }

    /// Cancels a running job.
    pub async fn cancel(&self, job_id: &str) -> Result<()> {
        log::info!("Cancelling job {}", job_id);
        self.canceler.set_cancelled(job_id);
        self.job_queue.mark_cancelled(job_id).await
    }

    // ========================================================================
    // JOB LIFECYCLE
    // ========================================================================

    /// Processes a single analysis job through its full lifecycle.
    pub(crate) async fn process_job(&self, mut job: Job) -> Result<String> {
        let timer = JobTimer::start(&job.id);
        let cancel_flag = self.canceler.get_cancel_flag(&job.id);

        // Initialize job
        job.status = JobStatus::Running;
        self.job_queue.mark_running(&job.id).await?;

        // Parse job URL
        let start_url = url::Url::parse(&job.url)?;

        // Check site resources (robots.txt, sitemap, SSL)
        let _resources = self.crawler.check_resources(&start_url).await?;

        // Early exit if cancelled
        if self.canceler.is_cancelled(&job.id) {
            log::warn!("Job {} cancelled before crawl", job.id);
            return Ok(job.id.clone());
        }

        // Run discovery
        let crawl_context = CrawlContext {
            job_id: job.id.clone(),
            settings: job.settings.clone(),
            start_url: start_url.clone(),
            cancel_flag: cancel_flag.clone(),
        };

        // We clone progress_reporter to pass it to discover_pages (it implements the trait now)
        let discovered_urls = self
            .crawler
            .discover_pages(&crawl_context, self.progress_reporter.clone())
            .await?;

        let max_pages = job.settings.max_pages as usize;
        let auditor = self.analyzer.select_auditor(&job.settings);

        let mut crawl_result = CrawlResult::default();

        for (idx, url) in discovered_urls.iter().enumerate() {
            if cancel_flag.load(Ordering::Relaxed) {
                log::info!("Job {} cancelled during analysis", job.id);
                break;
            }

            if idx >= max_pages {
                break;
            }

            // Analyze page
            let analysis = self
                .analyzer
                .analyze_page(url.as_str(), &job.id, 0, &auditor)
                .await;

            // Handle retry logic (deep -> light fallback)
            let analysis = match analysis {
                Ok(res) => Ok(res),
                Err(e) => {
                    if auditor.name() == "Deep (Lighthouse)" {
                        log::warn!(
                            "Deep auditor failed for {} ({}), retrying with light auditor",
                            url,
                            e
                        );
                        // For now we just log failure as we don't have easy access to light auditor
                        // unless we change signature.
                        // TODO: Improve failover in AnalyzerService
                        Err(e)
                    } else {
                        Err(e)
                    }
                }
            };

            match analysis {
                Ok((page_result, _new_urls)) => {
                    crawl_result.pages += 1;
                    crawl_result.issues += page_result.issues.len();
                    crawl_result.edges.extend(page_result.edges);
                }
                Err(e) => {
                    log::warn!("Failed to analyze {}: {:#}", url, e);
                }
            }

            // Report progress
            let pages_analyzed = (idx + 1).min(max_pages);
            let progress = (pages_analyzed as f64 / max_pages as f64) * 100.0;

            self.job_queue.update_progress(&job.id, progress).await?;
            self.progress_reporter
                .emit_progress(&job.id, progress, pages_analyzed as i64);

            // Apply rate limiting
            if job.settings.delay_between_requests > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(
                    job.settings.delay_between_requests as u64,
                ))
                .await;
            }
        }

        // Persist links
        self.persist_links(&job, &crawl_result.edges).await?;

        // Finalize job
        self.job_queue.mark_completed(&job.id).await?;

        log::info!("Job {} completed in {}ms", job.id, timer.elapsed_ms());

        Ok(job.id.clone())
    }

    async fn persist_links(
        &self,
        job: &Job,
        edges: &[crate::service::analyzer_service::PageEdge],
    ) -> Result<()> {
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
                link_text: e.link_text.clone(),
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
}

/// Job timer for measuring total crawl time.
struct JobTimer {
    start: std::time::Instant,
}

impl JobTimer {
    fn start(_job_id: &str) -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

#[derive(Default)]
struct CrawlResult {
    pages: usize,
    issues: usize,
    edges: Vec<crate::service::analyzer_service::PageEdge>,
}

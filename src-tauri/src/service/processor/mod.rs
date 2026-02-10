//! Job processing orchestration for SEO analysis (V2 schema).

mod analyzer;
mod canceler;
mod crawler;
mod queue;
pub mod reporter;

pub use analyzer::{AnalyzerService, PageEdge, PageResult};
pub use canceler::JobCanceler;
pub use crawler::{CrawlContext, Crawler, SiteResources};
pub use queue::JobQueue;
pub use reporter::ProgressReporter;

use crate::domain::models::{Job, JobStatus, NewLink};
use crate::service::processor::reporter::{ProgressEmitter, ProgressEvent};
use anyhow::Result;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Orchestrates SEO analysis jobs using the normalized schema.
pub struct JobProcessor {
    // Components
    job_queue: JobQueue,
    crawler: Crawler,
    analyzer: AnalyzerService,
    progress_emitter: Arc<dyn ProgressEmitter>, // ← No generics!
    canceler: JobCanceler,

    // Repositories (some still used directly for orchestration)
    link_db: Arc<dyn crate::repository::LinkRepository>,
}

impl JobProcessor {
    /// Construct a JobProcessor with explicit repository and service dependencies (DI-only).
    pub fn new(
        job_repo: Arc<dyn crate::repository::JobRepository>,
        link_repo: Arc<dyn crate::repository::LinkRepository>,
        analyzer: AnalyzerService,
        progress_emitter: Arc<dyn ProgressEmitter>, // ← Trait object
    ) -> Self {
        Self {
            job_queue: JobQueue::new(job_repo),
            crawler: Crawler::new(),
            analyzer,
            progress_emitter,
            canceler: JobCanceler::new(),
            link_db: link_repo,
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        // Get current cancel flags and set them
        self.canceler.cancel_all();
        match self.job_queue.cancel_all_running_jobs().await {
            Ok(_) => {
                tracing::info!("All running jobs cancelled successfully");
                return Ok(());
            },
            Err(e) => {
                tracing::error!("Failed to cancel running jobs during shutdown: {}", e);
                return Err(anyhow::anyhow!(
                    "Failed to cancel running jobs during shutdown: {}",
                    e
                ));
            }
            
        };
    }

    /// Main polling loop - fetches and processes pending jobs.
    pub async fn run(&self) -> Result<()> {
        tracing::info!("JobProcessor: Starting job polling loop");

        loop {
            // Fetch next job from queue
            if let Some(job) = self.job_queue.fetch_next_job().await {
                tracing::info!("Processing job: {} ({})", job.id, job.url);
                if let Err(e) = self.process_job(job).await {
                    tracing::error!("Job failed: {}", e);
                }
            }
        }
    }

    /// Cancels a running job.
    pub async fn cancel(&self, job_id: &str) -> Result<()> {
        tracing::info!("Cancelling job {}", job_id);
        self.canceler.set_cancelled(job_id);
        self.job_queue.mark_cancelled(job_id).await
    }

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
            tracing::warn!("Job {} cancelled before crawl", job.id);
            return Ok(job.id.clone());
        }

        // Run discovery
        let crawl_context = CrawlContext {
            job_id: job.id.clone(),
            settings: job.settings.clone(),
            start_url: start_url.clone(),
            cancel_flag: cancel_flag.clone(),
        };

        // Pass progress emitter to crawler (implements DiscoveryProgressEmitter trait)
        let discovered_urls = self
            .crawler
            .discover_pages(&crawl_context, self.progress_emitter.clone())
            .await?;

        // Early exit if cancelled
        if self.canceler.is_cancelled(&job.id) {
            tracing::warn!("Job {} cancelled before analysis", job.id);
            return Ok(job.id.clone());
        }

        let max_pages = job.settings.max_pages as usize;
        let auditor = self.analyzer.select_auditor(&job.settings);

        let mut crawl_result = CrawlResult::default();

        for (idx, url) in discovered_urls.iter().enumerate() {
            if cancel_flag.load(Ordering::Relaxed) {
                tracing::info!("Job {} cancelled during analysis", job.id);
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

            match analysis {
                Ok((page_result, _new_urls)) => {
                    crawl_result.pages += 1;
                    crawl_result.issues += page_result.issues.len();
                    crawl_result.edges.extend(page_result.edges);
                }
                Err(e) => tracing::warn!("Failed to analyze {}: {:#}", url, e),
            }

            // Report progress via unified event emission
            let pages_analyzed = (idx + 1).min(max_pages);
            let progress = (pages_analyzed as f64 / max_pages as f64) * 100.0;

            self.job_queue.update_progress(&job.id, progress).await?;
            self.progress_emitter.emit(ProgressEvent::Analysis {
                job_id: job.id.clone(),
                progress,
                pages_analyzed,
                total_pages: discovered_urls.len(),
            });

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

        tracing::info!("Job {} completed in {}ms", job.id, timer.elapsed_ms());

        Ok(job.id.clone())
    }

    async fn persist_links(&self, job: &Job, edges: &[PageEdge]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }

        let links: Vec<NewLink> = edges
            .iter()
            .map(|e| {
                NewLink::create(
                    &job.id,
                    &e.from_page_id,
                    &e.to_url,
                    e.link_text.clone(),
                    Some(e.status_code as i64),
                    &job.url,
                )
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
    edges: Vec<PageEdge>,
}
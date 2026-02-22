mod analyzer;
mod canceler;
mod crawler;
mod queue;
pub mod reporter;

pub use crate::service::discovery::SiteResources;
pub use analyzer::{AnalyzerService, PageResult};
pub use canceler::JobCanceler;
pub use crawler::{CrawlContext, Crawler};
pub use queue::JobQueue;
pub use reporter::ProgressReporter;

use crate::domain::{Job, JobStatus, NewLink};
use crate::service::processor::reporter::{ProgressEmitter, ProgressEvent};
use anyhow::Result;
use std::sync::atomic::Ordering;
use std::sync::Arc;

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
    pub fn new(
        job_repo: Arc<dyn crate::repository::JobRepository>,
        link_repo: Arc<dyn crate::repository::LinkRepository>,
        analyzer: AnalyzerService,
        crawler: Crawler,
        progress_emitter: Arc<dyn ProgressEmitter>,
    ) -> Self {
        Self {
            job_queue: JobQueue::new(job_repo),
            crawler,
            analyzer,
            progress_emitter,
            canceler: JobCanceler::new(),
            link_db: link_repo,
        }
    }

    pub fn analyzer(&self) -> &AnalyzerService {
        &self.analyzer
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.canceler.cancel_all();
        match self.job_queue.cancel_all_running_jobs().await {
            Ok(_) => {
                tracing::info!("All running jobs cancelled successfully");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to cancel running jobs during shutdown: {}", e);
                Err(anyhow::anyhow!(
                    "Failed to cancel running jobs during shutdown: {}",
                    e
                ))
            }
        }
    }

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

    pub async fn cancel(&self, job_id: &str) -> Result<()> {
        tracing::info!("Cancelling job {}", job_id);
        self.canceler.set_cancelled(job_id);
        self.job_queue.mark_cancelled(job_id).await
    }

    pub(crate) async fn process_job(&self, mut job: Job) -> Result<String> {
        let timer = JobTimer::start(&job.id);
        let cancel_flag = self.canceler.get_cancel_flag(&job.id);

        // Initialize job
        job.status = JobStatus::Discovery;
        self.job_queue.mark_discovery(&job.id).await?;

        // Check site resources (robots.txt, sitemap, SSL)
        let resources = self.crawler.check_resources(&job.url).await?;
        self.job_queue
            .update_resources(&job.id, resources.sitemap, resources.robots_txt)
            .await?;

        // Early exit if cancelled
        if self.canceler.is_cancelled(&job.id) {
            tracing::warn!("Job {} cancelled before crawl", job.id);
            return Ok(job.id.clone());
        }

        // Run discovery
        let crawl_context = CrawlContext {
            job_id: job.id.clone(),
            settings: job.settings.clone(),
            start_url: job.url.clone(),
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

        // Update status to Processing
        job.status = JobStatus::Processing;
        self.job_queue.mark_processing(&job.id).await?;

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
            let analysis = self.analyzer.analyze_page(url, &job.id, 0, &auditor).await;

            match analysis {
                Ok((page_result, _new_urls)) => {
                    crawl_result.pages += 1;
                    crawl_result.issues += page_result.issues.len();
                    crawl_result.links.extend(page_result.links);
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
        self.persist_links(&crawl_result.links).await?;

        // Finalize job
        self.job_queue.mark_completed(&job.id).await?;

        tracing::info!("Job {} completed in {}ms", job.id, timer.elapsed_ms());

        Ok(job.id.clone())
    }

    async fn persist_links(&self, links: &[NewLink]) -> Result<()> {
        if links.is_empty() {
            return Ok(());
        }

        self.link_db.insert_batch(links).await
    }
}

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
    links: Vec<NewLink>,
}

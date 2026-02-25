mod analyzer;
mod canceler;
mod channel;
mod crawler;
mod domain_semaphore;
mod page_queue;
mod queue;
pub mod reporter;

pub use crate::service::discovery::SiteResources;
pub use analyzer::{AnalyzerService, PageResult};
pub use canceler::JobCanceler;
pub use channel::{JobChannel, JobChannelConfig, JobDispatcher, JobNotifier};
pub use crawler::{CrawlContext, Crawler};
pub use domain_semaphore::DomainSemaphore;
pub use page_queue::PageQueueManager;
pub use queue::{JobQueue, JobQueueConfig};
pub use reporter::ProgressReporter;

use crate::domain::{Job, JobStatus, NewLink};
use crate::service::processor::reporter::{ProgressEmitter, ProgressEvent};
use anyhow::Result;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    pub max_workers: usize,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        let cpu_count = num_cpus::get();
        Self {
            max_workers: cpu_count.min(8),
        }
    }
}

pub struct JobProcessor {
    job_queue: Arc<JobQueue>,
    crawler: Arc<Crawler>,
    analyzer: Arc<AnalyzerService>,
    progress_emitter: Arc<dyn ProgressEmitter>,
    canceler: Arc<JobCanceler>,
    domain_semaphore: Arc<DomainSemaphore>,
    page_queue_manager: Arc<PageQueueManager>,
    link_db: Arc<dyn crate::repository::LinkRepository>,
    worker_config: WorkerPoolConfig,
}

impl JobProcessor {
    /// Create a new job processor with default configuration.
    pub fn new(
        job_repo: Arc<dyn crate::repository::JobRepository>,
        link_repo: Arc<dyn crate::repository::LinkRepository>,
        page_queue_repo: Arc<dyn crate::repository::PageQueueRepository>,
        analyzer: AnalyzerService,
        crawler: Crawler,
        progress_emitter: Arc<dyn ProgressEmitter>,
    ) -> Self {
        Self::with_config(
            job_repo,
            link_repo,
            page_queue_repo,
            analyzer,
            crawler,
            progress_emitter,
            WorkerPoolConfig::default(),
        )
    }

    /// Create a new job processor with the specified configuration.
    pub fn with_config(
        job_repo: Arc<dyn crate::repository::JobRepository>,
        link_repo: Arc<dyn crate::repository::LinkRepository>,
        page_queue_repo: Arc<dyn crate::repository::PageQueueRepository>,
        analyzer: AnalyzerService,
        crawler: Crawler,
        progress_emitter: Arc<dyn ProgressEmitter>,
        worker_config: WorkerPoolConfig,
    ) -> Self {
        Self {
            job_queue: Arc::new(JobQueue::new(job_repo)),
            crawler: Arc::new(crawler),
            analyzer: Arc::new(analyzer),
            progress_emitter,
            canceler: Arc::new(JobCanceler::new()),
            domain_semaphore: Arc::new(DomainSemaphore::new()),
            page_queue_manager: Arc::new(PageQueueManager::new(page_queue_repo)),
            link_db: link_repo,
            worker_config,
        }
    }

    /// Get the job queue notifier for signaling new jobs.
    pub fn notifier(&self) -> &JobNotifier {
        self.job_queue.notifier()
    }

    /// Notify the job processor that a new job is available.
    /// This dispatches pending jobs to the channel and wakes up workers.
    pub async fn notify_new_job(&self) {
        self.job_queue.notify_new_job().await;
    }

    /// Get the job queue (for testing purposes).
    #[cfg(test)]
    pub fn job_queue(&self) -> &Arc<JobQueue> {
        &self.job_queue
    }

    /// Get the analyzer service.
    pub fn analyzer(&self) -> &AnalyzerService {
        &self.analyzer
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.canceler.cancel_all();
        self.job_queue
            .cancel_all_running_jobs()
            .await
            .inspect(|_| tracing::info!("All running jobs cancelled successfully"))
            .inspect_err(|e| tracing::error!("Failed to cancel running jobs during shutdown: {}", e))
            .map_err(|e| anyhow::anyhow!("Failed to cancel running jobs during shutdown: {}", e))
    }

    pub async fn run(&self) -> Result<()> {
        tracing::info!(
            "JobProcessor: Starting worker pool with {} workers",
            self.worker_config.max_workers
        );

        let dispatched = self.job_queue.dispatch_pending_jobs().await?;
        if dispatched > 0 {
            tracing::info!("Dispatched {} pending jobs from previous session", dispatched);
        }

        let worker_handles: Vec<_> = (0..self.worker_config.max_workers)
            .map(|worker_id| self.spawn_worker(worker_id))
            .collect();

        tracing::info!("Spawned {} workers", worker_handles.len());

        futures::future::join_all(worker_handles).await;

        tracing::info!("All workers have shut down");
        Ok(())
    }

    fn worker_context(&self) -> WorkerContext {
        WorkerContext {
            job_queue: self.job_queue.clone(),
            crawler: self.crawler.clone(),
            analyzer: self.analyzer.clone(),
            progress_emitter: self.progress_emitter.clone(),
            canceler: self.canceler.clone(),
            link_db: self.link_db.clone(),
            domain_semaphore: self.domain_semaphore.clone(),
            page_queue_manager: self.page_queue_manager.clone(),
        }
    }

    fn spawn_worker(&self, worker_id: usize) -> tokio::task::JoinHandle<()> {
        let ctx = self.worker_context();

        tokio::spawn(async move {
            tracing::info!("Worker {} started", worker_id);

            loop {
                match ctx.job_queue.receive_job().await {
                    Some(job) => {
                        tracing::info!(
                            "Worker {}: Processing job {} ({})",
                            worker_id,
                            job.id,
                            job.url
                        );

                        let cancel_token = ctx.canceler.get_token(&job.id);

                        let domain_permit = ctx
                            .domain_semaphore
                            .acquire_with_cancel(&job.url, Some(&cancel_token))
                            .await;

                        if domain_permit.is_none() {
                            tracing::error!(
                                "Worker {}: Failed to acquire domain lock for {}",
                                worker_id,
                                job.url
                            );
                            continue;
                        }

                        if let Err(e) = ctx.process_job(job).await {
                            tracing::error!("Worker {}: Job failed: {}", worker_id, e);
                        }
                    }
                    None => {
                        tracing::info!("Worker {}: Channel closed, shutting down", worker_id);
                        break;
                    }
                }
            }

            tracing::info!("Worker {} stopped", worker_id);
        })
    }

    pub async fn cancel(&self, job_id: &str) -> Result<()> {
        tracing::info!("Cancelling job {}", job_id);
        self.canceler.cancel(job_id);
        self.job_queue.mark_cancelled(job_id).await
    }

    pub async fn process_job(&self, job: Job) -> Result<String> {
        let cancel_token = self.canceler.get_token(&job.id);

        let _domain_permit = self
            .domain_semaphore
            .acquire_with_cancel(&job.url, Some(&cancel_token))
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to acquire domain lock for {}", job.url))?;

        let ctx = self.worker_context();
        ctx.process_job(job).await
    }
}

struct WorkerContext {
    job_queue: Arc<JobQueue>,
    crawler: Arc<Crawler>,
    analyzer: Arc<AnalyzerService>,
    progress_emitter: Arc<dyn ProgressEmitter>,
    canceler: Arc<JobCanceler>,
    link_db: Arc<dyn crate::repository::LinkRepository>,
    domain_semaphore: Arc<DomainSemaphore>,
    page_queue_manager: Arc<PageQueueManager>,
}

impl WorkerContext {
    async fn process_job(&self, mut job: Job) -> Result<String> {
        let timer = JobTimer::start(&job.id);
        let cancel_token = self.canceler.get_token(&job.id);

        if self.canceler.is_cancelled(&job.id) {
            tracing::warn!("Job {} cancelled before crawl", job.id);
            self.canceler.cleanup(&job.id);
            return Ok(job.id.clone());
        }

        job.status = JobStatus::Discovery;
        self.job_queue.mark_discovery(&job.id).await?;

        let resources = self.crawler.check_resources(&job.url).await?;
        self.job_queue
            .update_resources(&job.id, resources.sitemap, resources.robots_txt)
            .await?;

        let crawl_context = CrawlContext {
            job_id: job.id.clone(),
            settings: job.settings.clone(),
            start_url: job.url.clone(),
            cancel_token: cancel_token.clone(),
        };

        let discovered_urls = self
            .crawler
            .discover_pages(&crawl_context, self.progress_emitter.clone())
            .await?;

        if self.canceler.is_cancelled(&job.id) {
            tracing::warn!("Job {} cancelled before analysis", job.id);
            self.canceler.cleanup(&job.id);
            return Ok(job.id.clone());
        }

        let max_pages = job.settings.max_pages as usize;
        let urls_to_queue: Vec<String> = discovered_urls.iter().take(max_pages).cloned().collect();
        self.page_queue_manager
            .insert_discovered_urls(&job.id, &urls_to_queue, 0)
            .await?;

        tracing::info!(
            "Job {}: Inserted {} URLs into page queue",
            job.id,
            urls_to_queue.len()
        );

        job.status = JobStatus::Processing;
        self.job_queue.mark_processing(&job.id).await?;

        let auditor = self.analyzer.select_auditor(&job.settings);
        let total_pages = urls_to_queue.len();

        let mut crawl_result = CrawlResult::default();
        let mut pages_analyzed = 0;
        let mut was_cancelled = false;

        while let Some(page_item) = self.page_queue_manager.claim_next_page(&job.id).await? {
            if cancel_token.is_cancelled() {
                tracing::info!("Job {} cancelled during analysis", job.id);
                self.page_queue_manager
                    .mark_failed(&page_item.id, "Job cancelled")
                    .await?;
                was_cancelled = true;
                break;
            }

            let analysis = self.analyzer.analyze_page(&page_item.url, &job.id, page_item.depth, &auditor).await;

            match analysis {
                Ok((page_result, _new_urls)) => {
                    crawl_result.pages += 1;
                    crawl_result.issues += page_result.issues.len();
                    crawl_result.links.extend(page_result.links);
                    self.page_queue_manager.mark_completed(&page_item.id).await?;
                }
                Err(e) => {
                    tracing::warn!("Failed to analyze {}: {:#}", page_item.url, e);
                    self.page_queue_manager
                        .mark_failed(&page_item.id, &e.to_string())
                        .await?;
                }
            }

            pages_analyzed += 1;

            let progress = (pages_analyzed as f64 / total_pages as f64) * 100.0;

            self.job_queue.update_progress(&job.id, progress).await?;
            self.progress_emitter.emit(ProgressEvent::Analysis {
                job_id: job.id.clone(),
                progress,
                pages_analyzed,
                total_pages,
            });

            if job.settings.delay_between_requests > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(
                    job.settings.delay_between_requests as u64,
                ))
                .await;
            }
        }

        self.persist_links(&crawl_result.links).await?;

        if was_cancelled {
            self.job_queue.mark_cancelled(&job.id).await?;
            tracing::info!("Job {} cancelled after {}ms", job.id, timer.elapsed_ms());
        } else {
            self.job_queue.mark_completed(&job.id).await?;
            tracing::info!("Job {} completed in {}ms", job.id, timer.elapsed_ms());
        }

        self.canceler.cleanup(&job.id);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_config_default() {
        let config = WorkerPoolConfig::default();
        assert!(config.max_workers >= 1);
        assert!(config.max_workers <= 8);
    }
}

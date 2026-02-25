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
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Configuration for the worker pool.
#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    /// Maximum number of concurrent workers.
    pub max_workers: usize,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        // Default to number of CPU cores, capped at 8
        let cpu_count = num_cpus::get();
        Self {
            max_workers: cpu_count.min(8),
        }
    }
}

pub struct JobProcessor {
    // Components
    job_queue: Arc<JobQueue>,
    crawler: Arc<Crawler>,
    analyzer: Arc<AnalyzerService>,
    progress_emitter: Arc<dyn ProgressEmitter>,
    canceler: Arc<JobCanceler>,
    domain_semaphore: Arc<DomainSemaphore>,
    page_queue_manager: Arc<PageQueueManager>,

    // Repositories
    link_db: Arc<dyn crate::repository::LinkRepository>,

    // Configuration
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

    /// Shutdown the job processor gracefully.
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

    /// Run the job processor with a worker pool.
    /// This is the new event-driven mode that spawns multiple workers.
    pub async fn run(&self) -> Result<()> {
        tracing::info!(
            "JobProcessor: Starting worker pool with {} workers",
            self.worker_config.max_workers
        );

        // Dispatch any pending jobs from previous sessions
        let dispatched = self.job_queue.dispatch_pending_jobs().await?;
        if dispatched > 0 {
            tracing::info!("Dispatched {} pending jobs from previous session", dispatched);
        }

        // Spawn worker tasks
        let mut worker_handles = Vec::new();
        for worker_id in 0..self.worker_config.max_workers {
            let handle = self.spawn_worker(worker_id);
            worker_handles.push(handle);
        }

        tracing::info!("Spawned {} workers", worker_handles.len());

        // Wait for all workers (they run indefinitely until channel closes)
        futures::future::join_all(worker_handles).await;

        tracing::info!("All workers have shut down");
        Ok(())
    }

    /// Spawn a single worker task.
    fn spawn_worker(&self, worker_id: usize) -> tokio::task::JoinHandle<()> {
        let job_queue = self.job_queue.clone();
        let crawler = self.crawler.clone();
        let analyzer = self.analyzer.clone();
        let progress_emitter = self.progress_emitter.clone();
        let canceler = self.canceler.clone();
        let link_db = self.link_db.clone();
        let domain_semaphore = self.domain_semaphore.clone();
        let page_queue_manager = self.page_queue_manager.clone();

        tokio::spawn(async move {
            tracing::info!("Worker {} started", worker_id);

            loop {
                // Receive next job from the queue
                match job_queue.receive_job().await {
                    Some(job) => {
                        tracing::info!(
                            "Worker {}: Processing job {} ({})",
                            worker_id,
                            job.id,
                            job.url
                        );

                        // Get the cancel flag early so we can check cancellation
                        // while waiting for the domain lock
                        let cancel_flag = canceler.get_cancel_flag(&job.id);

                        // Acquire domain lock before processing
                        // This ensures jobs for the same domain don't run concurrently
                        // Pass the cancel flag so we can detect cancellation while waiting
                        let domain_permit = domain_semaphore
                            .acquire_with_cancel(&job.url, Some(&cancel_flag))
                            .await;
                        
                        if domain_permit.is_none() {
                            tracing::error!(
                                "Worker {}: Failed to acquire domain lock for {}",
                                worker_id,
                                job.url
                            );
                            continue;
                        }

                        let processor = WorkerProcessor {
                            job_queue: &job_queue,
                            crawler: &crawler,
                            analyzer: &analyzer,
                            progress_emitter: &progress_emitter,
                            canceler: &canceler,
                            link_db: &link_db,
                            page_queue_manager: &page_queue_manager,
                        };

                        if let Err(e) = processor.process_job(job).await {
                            tracing::error!("Worker {}: Job failed: {}", worker_id, e);
                        }
                        
                        // Domain lock is released when domain_permit is dropped
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

    /// Cancel a specific job.
    pub async fn cancel(&self, job_id: &str) -> Result<()> {
        tracing::info!("Cancelling job {}", job_id);
        self.canceler.set_cancelled(job_id);
        self.job_queue.mark_cancelled(job_id).await
    }

    /// Process a single job (for backward compatibility and direct invocation).
    /// Note: This method acquires the domain lock to respect rate limiting.
    pub async fn process_job(&self, job: Job) -> Result<String> {
        // Get the cancel flag early so we can check cancellation
        // while waiting for the domain lock
        let cancel_flag = self.canceler.get_cancel_flag(&job.id);
        
        // Acquire domain lock before processing
        // Pass the cancel flag so we can detect cancellation while waiting
        let _domain_permit = self.domain_semaphore
            .acquire_with_cancel(&job.url, Some(&cancel_flag))
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to acquire domain lock for {}", job.url))?;
        
        let processor = WorkerProcessor {
            job_queue: &self.job_queue,
            crawler: &self.crawler,
            analyzer: &self.analyzer,
            progress_emitter: &self.progress_emitter,
            canceler: &self.canceler,
            link_db: &self.link_db,
            page_queue_manager: &self.page_queue_manager,
        };
        processor.process_job(job).await
    }
}

/// Processor for a single worker to handle jobs.
/// This is separated to allow clean borrowing within async tasks.
struct WorkerProcessor<'a> {
    job_queue: &'a JobQueue,
    crawler: &'a Crawler,
    analyzer: &'a AnalyzerService,
    progress_emitter: &'a Arc<dyn ProgressEmitter>,
    canceler: &'a JobCanceler,
    link_db: &'a Arc<dyn crate::repository::LinkRepository>,
    page_queue_manager: &'a Arc<PageQueueManager>,
}

impl<'a> WorkerProcessor<'a> {
    async fn process_job(&self, mut job: Job) -> Result<String> {
        let timer = JobTimer::start(&job.id);
        let cancel_flag = self.canceler.get_cancel_flag(&job.id);

        // Early exit if cancelled
        if self.canceler.is_cancelled(&job.id) {
            tracing::warn!("Job {} cancelled before crawl", job.id);
            self.canceler.cleanup(&job.id);
            return Ok(job.id.clone());
        }

        // Initialize job
        job.status = JobStatus::Discovery;
        self.job_queue.mark_discovery(&job.id).await?;

        // Check site resources (robots.txt, sitemap, SSL)
        let resources = self.crawler.check_resources(&job.url).await?;
        self.job_queue
            .update_resources(&job.id, resources.sitemap, resources.robots_txt)
            .await?;

        

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
            self.canceler.cleanup(&job.id);
            return Ok(job.id.clone());
        }

        // Insert discovered URLs into page_queue for resumable processing
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

        // Update status to Processing
        job.status = JobStatus::Processing;
        self.job_queue.mark_processing(&job.id).await?;

        let auditor = self.analyzer.select_auditor(&job.settings);
        let total_pages = urls_to_queue.len();

        let mut crawl_result = CrawlResult::default();
        let mut pages_analyzed = 0;
        let mut was_cancelled = false;

        // Process pages from the page queue
        while let Some(page_item) = self.page_queue_manager.claim_next_page(&job.id).await? {
            if cancel_flag.load(Ordering::Relaxed) {
                tracing::info!("Job {} cancelled during analysis", job.id);
                // Mark the page as failed due to cancellation
                self.page_queue_manager
                    .mark_failed(&page_item.id, "Job cancelled")
                    .await?;
                was_cancelled = true;
                break;
            }

            // Analyze page
            let analysis = self.analyzer.analyze_page(&page_item.url, &job.id, page_item.depth, &auditor).await;

            match analysis {
                Ok((page_result, _new_urls)) => {
                    crawl_result.pages += 1;
                    crawl_result.issues += page_result.issues.len();
                    crawl_result.links.extend(page_result.links);

                    // Mark page as completed
                    self.page_queue_manager.mark_completed(&page_item.id).await?;
                }
                Err(e) => {
                    tracing::warn!("Failed to analyze {}: {:#}", page_item.url, e);
                    // Mark page as failed with error message
                    self.page_queue_manager
                        .mark_failed(&page_item.id, &e.to_string())
                        .await?;
                }
            }

            pages_analyzed += 1;

            // Report progress via unified event emission
            let progress = (pages_analyzed as f64 / total_pages as f64) * 100.0;

            self.job_queue.update_progress(&job.id, progress).await?;
            self.progress_emitter.emit(ProgressEvent::Analysis {
                job_id: job.id.clone(),
                progress,
                pages_analyzed,
                total_pages,
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

        // Finalize job - mark as cancelled or completed based on cancellation status
        if was_cancelled {
            self.job_queue.mark_cancelled(&job.id).await?;
            tracing::info!("Job {} cancelled after {}ms", job.id, timer.elapsed_ms());
        } else {
            self.job_queue.mark_completed(&job.id).await?;
            tracing::info!("Job {} completed in {}ms", job.id, timer.elapsed_ms());
        }

        // Clean up cancel flag to prevent memory leaks
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

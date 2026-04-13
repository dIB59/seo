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

use crate::contexts::{Job, NewLink};
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

    /// Hand out the shared `DeepAuditor` so the lifecycle can spawn its
    /// persistent sidecar task at startup.
    ///
    /// Replaces the previous `pub fn analyzer()` getter that leaked the
    /// entire `AnalyzerService` to callers — now the only thing they can
    /// reach is the one piece of orchestration state they actually need.
    pub fn deep_auditor(&self) -> std::sync::Arc<crate::service::auditor::DeepAuditor> {
        self.analyzer.deep_auditor()
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
    /// Shared cancellation bail-out: log a warn at the given stage,
    /// drop the cancellation token from the registry, and return the
    /// job id as the successful exit value. Centralizes the two
    /// near-identical early-exit blocks (before crawl, before
    /// analysis) so a future stage can be added by one line.
    fn bail_if_cancelled(&self, job: &Job, stage: &'static str) -> Option<String> {
        if self.canceler.is_cancelled(&job.id) {
            tracing::warn!("Job {} cancelled {}", job.id, stage);
            self.canceler.cleanup(&job.id);
            return Some(job.id.as_str().to_string());
        }
        None
    }

    async fn process_job(&self, job: Job) -> Result<String> {
        // Enforce the lifecycle invariant at the entry: `process_job`
        // must only be handed jobs that are actually in `Pending`. The
        // queue loader technically filters on `status = 'pending'`, but
        // this check catches bugs where a caller forgets to requeue or
        // re-uses a terminal job by mistake — and makes the "pending"
        // precondition visible in code rather than only in SQL.
        use crate::contexts::analysis::{AnyJob, JobStatus};
        let job = match AnyJob::from(job) {
            AnyJob::Pending(s) => s.into_inner(),
            any => {
                let status: JobStatus = any.job().status.clone();
                anyhow::bail!(
                    "process_job called with non-pending job {} (status: {})",
                    any.job().id,
                    status.as_str()
                );
            }
        };

        // `job` is owned and dropped at function exit. The lifecycle is
        // tracked in the DB via `mark_*` (which call the JobRepository
        // typed-result methods); the local `job.status` field is only
        // read for branching here, never re-persisted, so we no longer
        // mutate it. The previous `job.status = JobStatus::X` writes
        // were dead code that misled readers into thinking they
        // mattered.
        let timer = JobTimer::start(&job.id);
        let cancel_token = self.canceler.get_token(&job.id);

        if let Some(id) = self.bail_if_cancelled(&job, "before crawl") {
            return Ok(id);
        }

        let job_id_str = job.id.as_str().to_string();

        self.job_queue.mark_discovery(&job.id).await?;

        let resources = self.crawler.check_resources(&job.url).await?;
        self.job_queue
            .update_resources(&job.id, resources.sitemap(), resources.robots_txt())
            .await?;

        let crawl_context = CrawlContext {
            job_id: job_id_str.clone(),
            settings: job.settings.clone(),
            start_url: job.url.clone(),
            cancel_token: cancel_token.clone(),
        };

        let discovered_pages = self
            .crawler
            .discover_pages(&crawl_context, self.progress_emitter.clone())
            .await?;

        if let Some(id) = self.bail_if_cancelled(&job, "before analysis") {
            return Ok(id);
        }

        let max_pages = job.settings.max_pages as usize;
        let pages_to_queue: Vec<_> = discovered_pages.into_iter().take(max_pages).collect();
        self.page_queue_manager
            .insert_discovered_pages(
                &job.id,
                &pages_to_queue,
                crate::contexts::analysis::Depth::root(),
            )
            .await?;

        tracing::info!(
            "Job {}: Discovery returned {} pages, queued {} (max_pages={})",
            job.id,
            pages_to_queue.len(),
            pages_to_queue.len(),
            max_pages,
        );
        if pages_to_queue.is_empty() {
            tracing::error!("Job {}: No pages to analyze — discovery returned nothing!", job.id);
        }

        self.job_queue.mark_processing(&job.id).await?;

        let auditor = self.analyzer.select_auditor(&job.settings);
        let total_pages = pages_to_queue.len();

        // ── Parallel analysis ────────────────────────────────────────
        //
        // With cached HTML from the discovery phase, analysis is
        // CPU-bound (HTML parsing + checks + DB writes) — no HTTP
        // contention, no per-domain rate limiting needed. We claim
        // pages from the queue in batches and analyze them
        // concurrently using a bounded semaphore.
        //
        // Concurrency is capped at the number of available CPU cores
        // (min 2, max 8) to avoid overwhelming SQLite with concurrent
        // writes.
        let concurrency = num_cpus::get().clamp(2, 8);
        tracing::info!(
            "Job {}: Analyzing {} pages with concurrency {}",
            job.id, total_pages, concurrency,
        );

        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
        let pages_analyzed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let crawl_links = Arc::new(tokio::sync::Mutex::new(Vec::<NewLink>::new()));
        let was_cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::with_capacity(total_pages);

        // Extract Arc-wrapped shared state outside the loop to avoid
        // re-cloning per iteration for fields that don't change.
        let job_id_arc: Arc<str> = Arc::from(job.id.as_str());

        while let Some(mut page_item) = self.page_queue_manager.claim_next_page(&job.id).await? {
            if cancel_token.is_cancelled() {
                tracing::info!("Job {} cancelled during analysis", job.id);
                self.page_queue_manager
                    .mark_failed(&page_item.id, "Job cancelled")
                    .await?;
                was_cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
                break;
            }

            let permit = semaphore.clone().acquire_owned().await
                .map_err(|e| anyhow::anyhow!("semaphore closed: {e}"))?;

            let analyzer = self.analyzer.clone();
            let auditor = auditor.clone();
            let page_queue_manager = self.page_queue_manager.clone();
            let job_queue = self.job_queue.clone();
            let progress_emitter = self.progress_emitter.clone();
            let job_id = Arc::clone(&job_id_arc);
            let cancel = cancel_token.clone();
            let pages_analyzed = pages_analyzed.clone();
            let crawl_links = crawl_links.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit; // held until this task completes

                if cancel.is_cancelled() {
                    let _ = page_queue_manager
                        .mark_failed(&page_item.id, "Job cancelled")
                        .await;
                    return;
                }

                // Use cached HTML from discovery if available
                let analysis = if let (Some(html), Some(status), Some(load_time)) = (
                    page_item.cached_html.take(),
                    page_item.http_status,
                    page_item.cached_load_time_ms,
                ) {
                    let final_url = page_item.final_url.take()
                        .unwrap_or_else(|| page_item.url.clone());
                    let cached = crate::service::auditor::CachedHtml {
                        html,
                        final_url,
                        status_code: status,
                        load_time_ms: load_time,
                    };
                    analyzer
                        .analyze_page_cached(&page_item.url, &job_id, page_item.depth, &auditor, cached)
                        .await
                } else {
                    analyzer
                        .analyze_page(&page_item.url, &job_id, page_item.depth, &auditor)
                        .await
                };

                match analysis {
                    Ok((page_result, _new_urls)) => {
                        let n_issues = page_result.issues.len();
                        let n_links = page_result.links.len();
                        crawl_links.lock().await.extend(page_result.links);
                        let _ = page_queue_manager.mark_completed(&page_item.id).await;
                        tracing::info!(
                            "[ANALYSIS] OK: {} — {} issues, {} links",
                            page_item.url, n_issues, n_links,
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "[ANALYSIS] FAILED: {} — {:#}",
                            page_item.url, e,
                        );
                        let _ = page_queue_manager
                            .mark_failed(&page_item.id, &e.to_string())
                            .await;
                    }
                }

                let done = pages_analyzed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                let progress = (done as f64 / total_pages as f64) * 100.0;

                let _ = job_queue.update_progress(&job_id, progress).await;
                progress_emitter.emit(ProgressEvent::Analysis {
                    job_id: job_id.to_string(),
                    progress,
                    pages_analyzed: done,
                    total_pages,
                });
            }));
        }

        // Wait for all in-flight analysis tasks to complete
        for handle in handles {
            if let Err(e) = handle.await {
                tracing::error!("Analysis task panicked: {e}");
            }
        }

        let collected_links: Vec<NewLink> = crawl_links.lock().await.drain(..).collect();
        self.persist_links(&collected_links).await?;

        if was_cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            self.job_queue.mark_cancelled(&job.id).await?;
            tracing::info!("Job {} cancelled after {}ms", job.id, timer.elapsed_ms());
        } else {
            self.job_queue.mark_completed(&job.id).await?;
            tracing::info!("Job {} completed in {}ms", job.id, timer.elapsed_ms());
        }

        // Emit a final progress event AFTER the job status is persisted
        // so the frontend refreshes the job list and sees the new status
        // (completed/cancelled) immediately — not on the next poll.
        self.progress_emitter.emit(ProgressEvent::Analysis {
            job_id: job.id.as_str().to_string(),
            progress: 100.0,
            pages_analyzed: total_pages,
            total_pages,
        });

        self.canceler.cleanup(&job.id);

        Ok(job.id.as_str().to_string())
    }

    async fn persist_links(&self, links: &[NewLink]) -> Result<()> {
        if links.is_empty() {
            return Ok(());
        }
        self.link_db.insert_batch(links).await?;
        Ok(())
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

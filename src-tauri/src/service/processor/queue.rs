use crate::contexts::{Job, JobStatus};
use crate::repository::JobRepository as JobRepositoryTrait;
use crate::service::processor::channel::{JobChannel, JobChannelConfig, JobNotifier};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

const JOB_POLL_INTERVAL: Duration = Duration::from_secs(15);
const JOB_FETCH_RETRY_DELAY: Duration = Duration::from_secs(10);

/// Configuration for the job queue.
#[derive(Debug, Clone)]
pub struct JobQueueConfig {
    /// Whether to use channel-based job distribution.
    pub use_channel: bool,
    /// Channel buffer size (if using channel mode).
    pub channel_buffer_size: usize,
}

impl Default for JobQueueConfig {
    fn default() -> Self {
        Self {
            use_channel: true,
            channel_buffer_size: 100,
        }
    }
}

pub struct JobQueue {
    repo: Arc<dyn JobRepositoryTrait>,
    /// Optional channel for event-driven job distribution.
    channel: Option<JobChannel>,
    /// Notifier for waking workers when new jobs arrive.
    notifier: JobNotifier,
    /// Notification receiver for the polling loop.
    notification_rx: Option<mpsc::Receiver<()>>,
}

impl JobQueue {
    /// Create a new job queue with the default configuration (channel-based).
    pub fn new(repo: Arc<dyn JobRepositoryTrait>) -> Self {
        Self::with_config(repo, JobQueueConfig::default())
    }

    /// Create a new job queue with the specified configuration.
    pub fn with_config(repo: Arc<dyn JobRepositoryTrait>, config: JobQueueConfig) -> Self {
        let (notifier, notification_rx) = JobNotifier::new();
        let channel = if config.use_channel {
            Some(JobChannel::new(JobChannelConfig {
                buffer_size: config.channel_buffer_size,
            }))
        } else {
            None
        };

        Self {
            repo,
            channel,
            notifier,
            notification_rx: Some(notification_rx),
        }
    }

    /// Get the job notifier for signaling new jobs.
    pub fn notifier(&self) -> &JobNotifier {
        &self.notifier
    }

    /// Get the job channel (if using channel mode).
    pub fn channel(&self) -> Option<&JobChannel> {
        self.channel.as_ref()
    }

    /// Notify that a new job has been created.
    /// This dispatches pending jobs to the channel and wakes up any sleeping workers.
    pub async fn notify_new_job(&self) {
        // Dispatch pending jobs to the channel so workers can receive them
        if let Err(e) = self.dispatch_pending_jobs().await {
            tracing::error!("Failed to dispatch pending jobs on notify: {}", e);
        }
        // Also send notification signal
        self.notifier.notify().await;
    }

    /// Fetch the next job using the channel (event-driven).
    /// Returns None if the channel is closed.
    pub async fn receive_job(&self) -> Option<Job> {
        if let Some(channel) = &self.channel {
            channel.recv().await
        } else {
            // Fallback to polling if channel is not available
            self.fetch_next_job().await
        }
    }

    /// Fetch the next job using polling (legacy mode).
    /// This is kept for backward compatibility and fallback.
    pub async fn fetch_next_job(&self) -> Option<Job> {
        loop {
            match self.repo.get_pending().await {
                Ok(jobs) if !jobs.is_empty() => {
                    // Return the first pending job
                    // In a real queue, we might want to lock it or mark it as processing immediately
                    // But for now we just return the first one found
                    return Some(jobs[0].clone());
                }
                Ok(_) => {
                    tracing::trace!("No pending jobs, sleeping...");
                    sleep(JOB_POLL_INTERVAL).await;
                }
                Err(e) => {
                    tracing::error!("Failed to fetch pending jobs: {}", e);
                    sleep(JOB_FETCH_RETRY_DELAY).await;
                }
            }
        }
    }

    /// Fetch pending jobs and dispatch them to the channel.
    /// This is used to transition from polling to channel mode.
    pub async fn dispatch_pending_jobs(&self) -> Result<usize> {
        let channel = match &self.channel {
            Some(c) => c,
            None => return Ok(0),
        };

        let jobs = self.repo.get_pending().await?;
        let count = jobs.len();

        for job in jobs {
            if let Err(e) = channel.sender().send(job).await {
                tracing::error!("Failed to dispatch job to channel: {}", e);
            }
        }

        Ok(count)
    }

    /// Hybrid mode: wait for notification or poll periodically.
    /// This combines the best of both worlds - immediate response when
    /// jobs are created, with fallback polling for reliability.
    pub async fn fetch_next_job_hybrid(&mut self) -> Option<Job> {
        let notification_rx = self.notification_rx.as_mut()?;

        loop {
            // First, check if there are pending jobs
            match self.repo.get_pending().await {
                Ok(jobs) if !jobs.is_empty() => {
                    return Some(jobs[0].clone());
                }
                Ok(_) => {
                    // No jobs, wait for notification or timeout
                    tracing::trace!("No pending jobs, waiting for notification...");
                }
                Err(e) => {
                    tracing::error!("Failed to fetch pending jobs: {}", e);
                }
            }

            // Wait for notification with timeout
            tokio::select! {
                _ = notification_rx.recv() => {
                    tracing::trace!("Received job notification");
                    // Loop back to check for jobs
                }
                _ = sleep(JOB_POLL_INTERVAL) => {
                    tracing::trace!("Poll timeout, checking for jobs");
                    // Loop back to check for jobs
                }
            }
        }
    }

    pub async fn mark_discovery(&self, job_id: &str) -> Result<()> {
        self.repo.update_status(job_id, JobStatus::Discovery).await
    }

    pub async fn mark_processing(&self, job_id: &str) -> Result<()> {
        self.repo.update_status(job_id, JobStatus::Processing).await
    }

    pub async fn mark_completed(&self, job_id: &str) -> Result<()> {
        self.repo.update_status(job_id, JobStatus::Completed).await
    }

    pub async fn mark_cancelled(&self, job_id: &str) -> Result<()> {
        self.repo.update_status(job_id, JobStatus::Cancelled).await
    }

    pub async fn cancel_all_running_jobs(&self) -> Result<()> {
        let jobs = self.repo.get_running_jobs_id().await?;
        tracing::info!("Cancelling {} running jobs", jobs.len());
        for job in jobs {
            self.mark_failed(&job, "Application Exit").await?;
        }
        Ok(())
    }

    pub async fn mark_failed(&self, job_id: &str, error: &str) -> Result<()> {
        self.repo.set_error(job_id, error).await?;
        tracing::info!("Job {} failed: {}", job_id, error);
        Ok(())
    }

    pub async fn update_progress(&self, job_id: &str, progress: f64) -> Result<()> {
        self.repo.update_progress(job_id, progress).await
    }

    pub async fn update_resources(
        &self,
        job_id: &str,
        sitemap_found: bool,
        robots_txt_found: bool,
    ) -> Result<()> {
        self.repo
            .update_resources(job_id, sitemap_found, robots_txt_found)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full tests would require a mock JobRepository
    // This is a basic structure test

    #[test]
    fn test_job_queue_config_default() {
        let config = JobQueueConfig::default();
        assert!(config.use_channel);
        assert_eq!(config.channel_buffer_size, 100);
    }
}

use crate::domain::models::{Job, JobStatus};
use crate::repository::JobRepository as JobRepositoryTrait;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Polling interval when no pending jobs are found
const JOB_POLL_INTERVAL: Duration = Duration::from_secs(15);

/// Delay after job fetch failure before retrying
const JOB_FETCH_RETRY_DELAY: Duration = Duration::from_secs(10);

pub struct JobQueue {
    repo: Arc<dyn JobRepositoryTrait>,
}

impl JobQueue {
    /// Create a new JobQueue from an existing repository implementation (DI-friendly).
    pub fn new(repo: Arc<dyn JobRepositoryTrait>) -> Self {
        Self { repo }
    }

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

    pub async fn mark_running(&self, job_id: &str) -> Result<()> {
        self.repo.update_status(job_id, JobStatus::Running).await
    }

    pub async fn mark_completed(&self, job_id: &str) -> Result<()> {
        self.repo.update_status(job_id, JobStatus::Completed).await
    }

    pub async fn mark_cancelled(&self, job_id: &str) -> Result<()> {
        self.repo.update_status(job_id, JobStatus::Cancelled).await
    }

    pub async fn cancel_all_running_jobs(&self)  -> Result<()> {
        let jobs = self.repo.get_running_jobs_id().await?;
        tracing::info!("Cancelling {} running jobs", jobs.len());
        for job in jobs {
            self.mark_failed(&job, "Application Exit").await?;
        }
        Ok(())
    }

    pub async fn mark_failed(&self, job_id: &str, error: &str) -> Result<()> {
        self.repo.update_status(job_id, JobStatus::Failed).await?;
        // Ideally we would save the error message too, but Job model might not have an error field yet
        // If it does, we should update it. Checking domain models...
        // Assuming for now we just mark failed.
        tracing::error!("Job {} failed: {}", job_id, error);
        Ok(())
    }

    pub async fn update_progress(&self, job_id: &str, progress: f64) -> Result<()> {
        self.repo.update_progress(job_id, progress, None).await
    }
}

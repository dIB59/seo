use crate::domain::models::{Job, JobStatus};
use crate::repository::sqlite::JobRepository;
use anyhow::Result;
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::time::sleep;

/// Polling interval when no pending jobs are found
const JOB_POLL_INTERVAL: Duration = Duration::from_secs(15);

/// Delay after job fetch failure before retrying
const JOB_FETCH_RETRY_DELAY: Duration = Duration::from_secs(10);

pub struct JobQueue {
    repo: JobRepository,
}

impl JobQueue {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repo: JobRepository::new(pool),
        }
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
                    log::trace!("No pending jobs, sleeping...");
                    sleep(JOB_POLL_INTERVAL).await;
                }
                Err(e) => {
                    log::error!("Failed to fetch pending jobs: {}", e);
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

    pub async fn mark_failed(&self, job_id: &str, error: &str) -> Result<()> {
        self.repo.update_status(job_id, JobStatus::Failed).await?;
        // Ideally we would save the error message too, but Job model might not have an error field yet
        // If it does, we should update it. Checking domain models...
        // Assuming for now we just mark failed.
        log::error!("Job {} failed: {}", job_id, error);
        Ok(())
    }

    pub async fn update_progress(&self, job_id: &str, progress: f64) -> Result<()> {
        self.repo.update_progress(job_id, progress, None).await
    }
}

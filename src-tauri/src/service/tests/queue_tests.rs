use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::domain::{Job, JobInfo, JobSettings, JobStatus};
use crate::repository::JobRepository;
use crate::service::processor::JobQueue;
use anyhow::Result;

struct MockJobRepo {
    next: Option<Job>,
}

impl MockJobRepo {
    fn new(job: Job) -> Self {
        Self { next: Some(job) }
    }
}

#[async_trait]
impl JobRepository for MockJobRepo {
    async fn create(&self, _url: &str, _settings: &JobSettings) -> Result<String> {
        Err(anyhow::anyhow!("not implemented"))
    }

    async fn get_by_id(&self, _id: &str) -> Result<Job> {
        Err(anyhow::anyhow!("not implemented"))
    }

    async fn get_all(&self) -> Result<Vec<JobInfo>> {
        Ok(vec![])
    }

    async fn get_paginated(&self, _limit: i64, _offset: i64) -> Result<Vec<JobInfo>> {
        Ok(vec![])
    }

    async fn get_paginated_with_total(
        &self,
        _limit: i64,
        _offset: i64,
        _url_filter: Option<String>,
        _status_filter: Option<String>,
    ) -> Result<(Vec<JobInfo>, i64)> {
        Ok((vec![], 0))
    }

    async fn get_pending(&self) -> Result<Vec<Job>> {
        Ok(self.next.clone().into_iter().collect())
    }

    async fn get_running_jobs_id(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn update_status(&self, _job_id: &str, _status: JobStatus) -> Result<()> {
        Ok(())
    }

    async fn update_progress(&self, _id: &str, _progress: f64) -> Result<()> {
        Ok(())
    }

    async fn update_resources(&self, _id: &str, _sitemap: bool, _robots: bool) -> Result<()> {
        Ok(())
    }

    async fn set_error(&self, _job_id: &str, _error: &str) -> Result<()> {
        Ok(())
    }

    async fn count(&self) -> Result<i64> {
        Ok(0)
    }

    async fn delete(&self, _job_id: &str) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn fetch_next_job_returns_mocked_job() {
    let job = Job {
        id: "job-123".to_string(),
        url: "https://example.com".to_string(),
        status: JobStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        completed_at: None,
        settings: JobSettings::default(),
        summary: Default::default(),
        progress: 0.0,
        error_message: None,
        sitemap_found: false,
        robots_txt_found: false,
    };

    let repo = Arc::new(MockJobRepo::new(job.clone()));
    let queue = JobQueue::new(repo);

    let res = queue.fetch_next_job().await;
    assert!(res.is_some());
    let found = res.unwrap();
    assert_eq!(found.id, job.id);
}

/// Test that notify_new_job dispatches pending jobs to the channel
/// so workers can receive them via receive_job()
#[tokio::test]
async fn notify_new_job_dispatches_pending_jobs_to_channel() {
    let job = Job {
        id: "job-456".to_string(),
        url: "https://test.example.com".to_string(),
        status: JobStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        completed_at: None,
        settings: JobSettings::default(),
        summary: Default::default(),
        progress: 0.0,
        error_message: None,
        sitemap_found: false,
        robots_txt_found: false,
    };

    let repo = Arc::new(MockJobRepo::new(job.clone()));
    let queue = JobQueue::new(repo);

    // Simulate what happens when a job is created:
    // 1. Job is already in DB (mocked above)
    // 2. notify_new_job is called
    queue.notify_new_job().await;

    // Now a worker should be able to receive the job via the channel
    // Use a timeout to avoid hanging if the test fails
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        queue.receive_job()
    ).await;

    // This should succeed - the job should be dispatched to the channel
    assert!(result.is_ok(), "Job should be dispatched to channel on notify");
    let job_opt = result.unwrap();
    assert!(job_opt.is_some(), "Job should be received from channel");
    let received = job_opt.unwrap();
    assert_eq!(received.id, job.id);
}

/// Test that dispatch_pending_jobs sends all pending jobs to the channel
#[tokio::test]
async fn dispatch_pending_jobs_sends_to_channel() {
    let job = Job {
        id: "job-789".to_string(),
        url: "https://dispatch.example.com".to_string(),
        status: JobStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        completed_at: None,
        settings: JobSettings::default(),
        summary: Default::default(),
        progress: 0.0,
        error_message: None,
        sitemap_found: false,
        robots_txt_found: false,
    };

    let repo = Arc::new(MockJobRepo::new(job.clone()));
    let queue = JobQueue::new(repo);

    // Dispatch pending jobs
    let count = queue.dispatch_pending_jobs().await.unwrap();
    assert_eq!(count, 1, "One job should be dispatched");

    // Now receive it from the channel
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        queue.receive_job()
    ).await;

    assert!(result.is_ok(), "Job should be in channel after dispatch");
    let job_opt = result.unwrap();
    assert!(job_opt.is_some());
    assert_eq!(job_opt.unwrap().id, job.id);
}

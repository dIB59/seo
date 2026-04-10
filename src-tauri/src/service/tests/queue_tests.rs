use async_trait::async_trait;
use std::sync::Arc;

use crate::contexts::{Job, JobId, JobInfo, JobSettings, JobStatus};
use crate::repository::JobRepository;
use crate::service::processor::JobQueue;

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
    async fn create(
        &self,
        _url: &str,
        _settings: &JobSettings,
    ) -> crate::repository::RepositoryResult<String> {
        Err(crate::repository::RepositoryError::not_found("job", "mock"))
    }

    async fn get_by_id(&self, _id: &str) -> crate::repository::RepositoryResult<Job> {
        Err(crate::repository::RepositoryError::not_found("job", "mock"))
    }

    async fn get_all(&self) -> crate::repository::RepositoryResult<Vec<JobInfo>> {
        Ok(vec![])
    }

    async fn get_paginated(
        &self,
        _limit: i64,
        _offset: i64,
    ) -> crate::repository::RepositoryResult<Vec<JobInfo>> {
        Ok(vec![])
    }

    async fn get_paginated_with_total(
        &self,
        _query: crate::contexts::analysis::JobPageQuery,
    ) -> crate::repository::RepositoryResult<(Vec<JobInfo>, i64)> {
        Ok((vec![], 0))
    }

    async fn get_pending(&self) -> crate::repository::RepositoryResult<Vec<Job>> {
        Ok(self.next.clone().into_iter().collect())
    }

    async fn get_running_jobs_id(&self) -> crate::repository::RepositoryResult<Vec<String>> {
        Ok(vec![])
    }

    async fn update_status(
        &self,
        _job_id: &str,
        _status: JobStatus,
    ) -> crate::repository::RepositoryResult<()> {
        Ok(())
    }

    async fn update_progress(
        &self,
        _id: &str,
        _progress: f64,
    ) -> crate::repository::RepositoryResult<()> {
        Ok(())
    }

    async fn update_resources(
        &self,
        _id: &str,
        _sitemap: bool,
        _robots: bool,
    ) -> crate::repository::RepositoryResult<()> {
        Ok(())
    }

    async fn set_error(
        &self,
        _job_id: &str,
        _error: &str,
    ) -> crate::repository::RepositoryResult<()> {
        Ok(())
    }

    async fn count(&self) -> crate::repository::RepositoryResult<i64> {
        Ok(0)
    }

    async fn delete(&self, _job_id: &str) -> crate::repository::RepositoryResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn fetch_next_job_returns_mocked_job() {
    let job = Job {
        id: JobId::from("job-123"),
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
        id: JobId::from("job-456"),
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
        id: JobId::from("job-789"),
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

/// Regression test for: "jobs don't start when created, have to restart app"
/// 
/// This test verifies that when notify_new_job() is called on JobQueue,
/// the job is dispatched to the channel so workers can receive it immediately.
/// 
/// The bug was that the command called notifier().notify() which only sent
/// a signal but didn't dispatch jobs to the channel. The fix was to:
/// 1. Add notify_new_job() to JobQueue which calls dispatch_pending_jobs()
/// 2. Add notify_new_job() to JobProcessor which delegates to the queue
/// 3. Update the command to call processor.notify_new_job() instead of notifier().notify()
///
/// This test verifies step 1 - that the queue properly dispatches jobs.
#[tokio::test]
async fn regression_notify_new_job_dispatches_to_channel() {
    let job = Job {
        id: JobId::from("regression-job-002"),
        url: "https://regression.example.com".to_string(),
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

    // This is what the command should call (via JobProcessor.notify_new_job())
    queue.notify_new_job().await;

    // The job should be immediately available in the channel
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        queue.receive_job()
    ).await;

    assert!(result.is_ok(), "REGRESSION: Job should be dispatched without restart");
    let job_opt = result.unwrap();
    assert!(job_opt.is_some(), "REGRESSION: Job should be received from channel");
    assert_eq!(job_opt.unwrap().id, job.id, "REGRESSION: Correct job should be received");
}

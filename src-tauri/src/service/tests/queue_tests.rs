use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::models::{Job, JobSettings, JobStatus};
use crate::service::processor::JobQueue;
use crate::repository::JobRepository;
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

    async fn get_all(&self) -> Result<Vec<crate::domain::models::JobInfo>> {
        Ok(vec![])
    }

    async fn get_pending(&self) -> Result<Vec<Job>> {
        Ok(self.next.clone().into_iter().collect())
    }

    async fn update_status(&self, _job_id: &str, _status: JobStatus) -> Result<()> {
        Ok(())
    }

    async fn update_progress(&self, _id: &str, _progress: f64, _current_stage: Option<&str>) -> Result<()> {
        Ok(())
    }

    async fn set_error(&self, _job_id: &str, _error: &str) -> Result<()> {
        Ok(())
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
        current_stage: None,
        error_message: None,
    };

    let repo = Arc::new(MockJobRepo::new(job.clone()));
    let queue = JobQueue::new(repo);

    let res = queue.fetch_next_job().await;
    assert!(res.is_some());
    let found = res.unwrap();
    assert_eq!(found.id, job.id);
}

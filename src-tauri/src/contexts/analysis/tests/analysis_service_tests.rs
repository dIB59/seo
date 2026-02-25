// TDD Tests for AnalysisService
// These tests define the expected interface and behavior of the AnalysisService

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::contexts::analysis::{
    AnalysisService, JobStatus, JobFilter,
};
use crate::repository::JobRepository;

// ============================================================================
// Mock Repositories for Testing
// ============================================================================

/// Mock JobRepository for testing
struct MockJobRepository {
    jobs: RwLock<HashMap<String, crate::domain::Job>>,
}

impl MockJobRepository {
    fn new() -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl JobRepository for MockJobRepository {
    async fn create(&self, url: &str, settings: &crate::domain::JobSettings) -> anyhow::Result<String> {
        let now = chrono::Utc::now();
        let job = crate::domain::Job {
            id: uuid::Uuid::new_v4().to_string(),
            url: url.to_string(),
            status: crate::domain::JobStatus::Pending,
            progress: 0.0,
            settings: settings.clone(),
            summary: crate::domain::JobSummary::default(),
            sitemap_found: false,
            robots_txt_found: false,
            created_at: now,
            updated_at: now,
            completed_at: None,
            error_message: None,
        };
        let id = job.id.clone();
        self.jobs.write().await.insert(id.clone(), job);
        Ok(id)
    }

    async fn get_by_id(&self, id: &str) -> anyhow::Result<crate::domain::Job> {
        self.jobs.read().await.get(id).cloned()
            .ok_or_else(|| anyhow::anyhow!("Job not found"))
    }

    async fn get_all(&self) -> anyhow::Result<Vec<crate::domain::JobInfo>> {
        Ok(self.jobs.read().await.values().map(|j| crate::domain::JobInfo {
            id: j.id.clone(),
            url: j.url.clone(),
            status: j.status.clone(),
            progress: j.progress,
            total_pages: j.summary.total_pages,
            total_issues: j.summary.total_issues,
            created_at: j.created_at,
            max_pages: j.settings.max_pages,
            lighthouse_analysis: j.settings.lighthouse_analysis,
        }).collect())
    }

    async fn get_paginated(&self, limit: i64, offset: i64) -> anyhow::Result<Vec<crate::domain::JobInfo>> {
        let all = self.get_all().await?;
        Ok(all.into_iter().skip(offset as usize).take(limit as usize).collect())
    }

    async fn get_paginated_with_total(
        &self,
        limit: i64,
        offset: i64,
        _url_filter: Option<String>,
        _status_filter: Option<String>,
    ) -> anyhow::Result<(Vec<crate::domain::JobInfo>, i64)> {
        let all = self.get_all().await?;
        let total = all.len() as i64;
        Ok((all.into_iter().skip(offset as usize).take(limit as usize).collect(), total))
    }

    async fn get_pending(&self) -> anyhow::Result<Vec<crate::domain::Job>> {
        Ok(self.jobs.read().await.values().filter(|j| j.status == crate::domain::JobStatus::Pending).cloned().collect())
    }

    async fn get_running_jobs_id(&self) -> anyhow::Result<Vec<String>> {
        Ok(self.jobs.read().await.values()
            .filter(|j| j.status == crate::domain::JobStatus::Processing || j.status == crate::domain::JobStatus::Discovery)
            .map(|j| j.id.clone())
            .collect())
    }

    async fn update_status(&self, job_id: &str, status: crate::domain::JobStatus) -> anyhow::Result<()> {
        if let Some(job) = self.jobs.write().await.get_mut(job_id) {
            job.status = status;
        }
        Ok(())
    }

    async fn update_progress(&self, id: &str, progress: f64) -> anyhow::Result<()> {
        if let Some(job) = self.jobs.write().await.get_mut(id) {
            job.progress = progress;
        }
        Ok(())
    }

    async fn update_resources(
        &self,
        id: &str,
        sitemap_found: bool,
        robots_txt_found: bool,
    ) -> anyhow::Result<()> {
        if let Some(job) = self.jobs.write().await.get_mut(id) {
            job.sitemap_found = sitemap_found;
            job.robots_txt_found = robots_txt_found;
        }
        Ok(())
    }

    async fn set_error(&self, job_id: &str, error: &str) -> anyhow::Result<()> {
        if let Some(job) = self.jobs.write().await.get_mut(job_id) {
            job.status = crate::domain::JobStatus::Failed;
            job.error_message = Some(error.to_string());
        }
        Ok(())
    }

    async fn count(&self) -> anyhow::Result<i64> {
        Ok(self.jobs.read().await.len() as i64)
    }

    async fn delete(&self, job_id: &str) -> anyhow::Result<()> {
        self.jobs.write().await.remove(job_id);
        Ok(())
    }
}

// ============================================================================
// Tests for AnalysisService Interface
// ============================================================================

/// Test: AnalysisService can be created with repositories
#[tokio::test]
async fn test_analysis_service_can_be_created() {
    // Arrange
    let job_repo = Arc::new(MockJobRepository::new());
    
    // Act - Create service (this will fail until we implement AnalysisService)
    let _service = AnalysisService::new(job_repo);
    
    // Assert - Service was created successfully
    // If we get here without panic, the test passes
}

/// Test: AnalysisService can create a job
#[tokio::test]
async fn test_analysis_service_create_job() {
    // Arrange
    let job_repo = Arc::new(MockJobRepository::new());
    let service = AnalysisService::new(job_repo.clone());
    let settings = crate::domain::JobSettings::default();
    
    // Act
    let job_id = service.create_job("https://example.com", &settings).await
        .expect("Failed to create job");
    
    // Assert
    assert!(!job_id.is_empty(), "Job ID should not be empty");
    
    // Verify job was stored
    let job = job_repo.get_by_id(&job_id).await
        .expect("Job should exist in repository");
    assert_eq!(job.url, "https://example.com");
    assert_eq!(job.status, crate::domain::JobStatus::Pending);
}

/// Test: AnalysisService can get a job by ID
#[tokio::test]
async fn test_analysis_service_get_job() {
    // Arrange
    let job_repo = Arc::new(MockJobRepository::new());
    let service = AnalysisService::new(job_repo.clone());
    let settings = crate::domain::JobSettings::default();
    
    // Create a job first
    let job_id = service.create_job("https://example.com", &settings).await
        .expect("Failed to create job");
    
    // Act
    let job = service.get_job(&job_id).await
        .expect("Failed to get job");
    
    // Assert
    assert_eq!(job.id, job_id);
    assert_eq!(job.url, "https://example.com");
}

/// Test: AnalysisService can list jobs with filter
#[tokio::test]
async fn test_analysis_service_list_jobs() {
    // Arrange
    let job_repo = Arc::new(MockJobRepository::new());
    let service = AnalysisService::new(job_repo);
    let settings = crate::domain::JobSettings::default();
    
    // Create multiple jobs
    service.create_job("https://example1.com", &settings).await
        .expect("Failed to create job 1");
    service.create_job("https://example2.com", &settings).await
        .expect("Failed to create job 2");
    
    // Act
    let filter = JobFilter::default();
    let jobs = service.list_jobs(filter).await
        .expect("Failed to list jobs");
    
    // Assert
    assert_eq!(jobs.len(), 2);
}

/// Test: AnalysisService can cancel a job
#[tokio::test]
async fn test_analysis_service_cancel_job() {
    // Arrange
    let job_repo = Arc::new(MockJobRepository::new());
    let service = AnalysisService::new(job_repo.clone());
    let settings = crate::domain::JobSettings::default();
    
    let job_id = service.create_job("https://example.com", &settings).await
        .expect("Failed to create job");
    
    // Act
    service.cancel_job(&job_id).await
        .expect("Failed to cancel job");
    
    // Assert
    let job = job_repo.get_by_id(&job_id).await
        .expect("Job should exist");
    assert_eq!(job.status, crate::domain::JobStatus::Cancelled);
}

/// Test: AnalysisService returns error for non-existent job
#[tokio::test]
async fn test_analysis_service_get_nonexistent_job() {
    // Arrange
    let job_repo = Arc::new(MockJobRepository::new());
    let service = AnalysisService::new(job_repo);
    
    // Act
    let result = service.get_job("nonexistent-id").await;
    
    // Assert
    assert!(result.is_err(), "Should return error for non-existent job");
}

/// Test: AnalysisService can get analysis progress
#[tokio::test]
async fn test_analysis_service_get_progress() {
    // Arrange
    let job_repo = Arc::new(MockJobRepository::new());
    let service = AnalysisService::new(job_repo.clone());
    let settings = crate::domain::JobSettings::default();
    
    let job_id = service.create_job("https://example.com", &settings).await
        .expect("Failed to create job");
    
    // Act
    let progress = service.get_progress(&job_id).await
        .expect("Failed to get progress");
    
    // Assert
    assert_eq!(progress.job_id, job_id);
    assert_eq!(progress.progress, Some(0.0));
}

// ============================================================================
// Tests for JobSettings Validation
// ============================================================================

/// Test: JobSettings default values are sensible
#[test]
fn test_job_settings_defaults() {
    let settings = crate::domain::JobSettings::default();
    
    assert_eq!(settings.max_pages, 100);
    assert!(settings.include_subdomains);
    assert!(settings.check_images);
    assert!(!settings.mobile_analysis);
    assert!(!settings.lighthouse_analysis);
    assert_eq!(settings.delay_between_requests, 500);
}

/// Test: JobSettings can be customized
#[test]
fn test_job_settings_custom() {
    let settings = crate::domain::JobSettings {
        max_pages: 50,
        include_subdomains: false,
        check_images: false,
        mobile_analysis: true,
        lighthouse_analysis: true,
        delay_between_requests: 1000,
    };
    
    assert_eq!(settings.max_pages, 50);
    assert!(!settings.include_subdomains);
    assert!(!settings.check_images);
    assert!(settings.mobile_analysis);
    assert!(settings.lighthouse_analysis);
    assert_eq!(settings.delay_between_requests, 1000);
}

// ============================================================================
// Tests for JobFilter
// ============================================================================

/// Test: JobFilter default allows all jobs
#[test]
fn test_job_filter_defaults() {
    let filter = JobFilter::default();
    
    assert!(filter.status.is_none());
    assert!(filter.url_contains.is_none());
    assert!(filter.limit.is_none());
    assert!(filter.offset.is_none());
}

/// Test: JobFilter can be configured
#[test]
fn test_job_filter_custom() {
    let filter = JobFilter {
        status: Some(JobStatus::Completed),
        url_contains: Some("example".to_string()),
        limit: Some(10),
        offset: Some(0),
    };
    
    assert_eq!(filter.status, Some(JobStatus::Completed));
    assert_eq!(filter.url_contains, Some("example".to_string()));
    assert_eq!(filter.limit, Some(10));
    assert_eq!(filter.offset, Some(0));
}

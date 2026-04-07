// AnalysisService - Service Layer for Analysis Bounded Context
// Single point of coupling for external modules
// Uses Strangler Fig pattern - delegates to existing implementation

use std::sync::Arc;
use anyhow::Result;

use super::super::domain::*;
use crate::repository::{JobRepository, ResultsRepository};
use crate::service::JobProcessor;

/// Service layer for the Analysis bounded context.
/// Single point of coupling - external modules interact only through this service.
pub struct AnalysisService {
    job_repo: Arc<dyn JobRepository>,
    results_repo: Option<Arc<dyn ResultsRepository>>,
    job_processor: Option<Arc<JobProcessor>>,
}

impl AnalysisService {
    /// Create a new AnalysisService with the given repositories
    pub fn new(job_repo: Arc<dyn JobRepository>) -> Self {
        Self {
            job_repo,
            results_repo: None,
            job_processor: None,
        }
    }

    /// Create a new AnalysisService with all dependencies
    pub fn with_repositories(
        job_repo: Arc<dyn JobRepository>,
        results_repo: Arc<dyn ResultsRepository>,
    ) -> Self {
        Self {
            job_repo,
            results_repo: Some(results_repo),
            job_processor: None,
        }
    }

    /// Create a new AnalysisService with full dependencies including job processor
    pub fn with_processor(
        job_repo: Arc<dyn JobRepository>,
        results_repo: Arc<dyn ResultsRepository>,
        job_processor: Arc<JobProcessor>,
    ) -> Self {
        Self {
            job_repo,
            results_repo: Some(results_repo),
            job_processor: Some(job_processor),
        }
    }

    // === Job Lifecycle ===

    /// Create a new analysis job
    pub async fn create_job(&self, url: &str, settings: &JobSettings) -> Result<JobId> {
        let id = self.job_repo.create(url, settings).await?;
        Ok(id)
    }

    /// Get a job by its ID
    pub async fn get_job(&self, id: &str) -> Result<Job> {
        self.job_repo.get_by_id(id).await
    }

    /// Cancel a running job
    pub async fn cancel_job(&self, id: &str) -> Result<()> {
        if let Some(processor) = &self.job_processor {
            processor.cancel(id).await?;
        } else {
            self.job_repo.update_status(id, JobStatus::Cancelled).await?;
        }
        Ok(())
    }

    /// List jobs with optional filtering
    pub async fn list_jobs(&self, filter: JobFilter) -> Result<Vec<JobInfo>> {
        let limit = filter.limit.unwrap_or(100);
        let offset = filter.offset.unwrap_or(0);
        
        let (jobs, _total) = self.job_repo
            .get_paginated_with_total(
                limit,
                offset,
                filter.url_contains.clone(),
                filter.status.map(|s| s.as_str().to_string()),
            )
            .await?;
        
        Ok(jobs)
    }

    /// Get all jobs
    pub async fn get_all_jobs(&self) -> Result<Vec<JobInfo>> {
        self.job_repo.get_all().await
    }

    /// Get paginated jobs
    pub async fn get_paginated_jobs(&self, limit: i64, offset: i64) -> Result<Vec<JobInfo>> {
        self.job_repo.get_paginated(limit, offset).await
    }

    /// Get paginated jobs with total count and filters
    pub async fn get_paginated_jobs_with_total(
        &self,
        limit: i64,
        offset: i64,
        url_filter: Option<String>,
        status_filter: Option<String>,
    ) -> Result<(Vec<JobInfo>, i64)> {
        self.job_repo
            .get_paginated_with_total(limit, offset, url_filter, status_filter)
            .await
    }

    // === Analysis Execution ===

    /// Notify the job processor that a new job is available
    pub async fn notify_new_job(&self) {
        if let Some(processor) = &self.job_processor {
            processor.notify_new_job().await;
        }
    }

    /// Get the current progress of an analysis
    pub async fn get_progress(&self, job_id: &str) -> Result<AnalysisProgress> {
        let job = self.job_repo.get_by_id(job_id).await?;
        Ok(AnalysisProgress::from(job))
    }

    /// Get the complete results of an analysis
    pub async fn get_results(&self, job_id: &str) -> Result<AnalysisResult> {
        let job = self.job_repo.get_by_id(job_id).await?;
        let results_repo = self.results_repo.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ResultsRepository not configured"))?;
        let pages = results_repo.get_pages(job_id).await?;
        let issues = results_repo.get_issues(job_id).await?;
        let links = results_repo.get_links(job_id).await?;
        let lighthouse = results_repo.get_lighthouse(job_id).await?;
        
        Ok(AnalysisResult {
            job,
            pages,
            issues,
            links,
            lighthouse,
        })
    }

    /// Get the complete job result with all related data
    pub async fn get_complete_result(&self, job_id: &str) -> Result<CompleteJobResult> {
        let results_repo = self.results_repo.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ResultsRepository not configured"))?;
        results_repo.get_complete_result(job_id).await
    }

    // === Page Access ===

    /// Get all pages for a job
    pub async fn get_pages(&self, job_id: &str) -> Result<Vec<Page>> {
        let results_repo = self.results_repo.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ResultsRepository not configured"))?;
        results_repo.get_pages(job_id).await
    }

    // === Issue Access ===

    /// Get all issues for a job
    pub async fn get_issues(&self, job_id: &str) -> Result<Vec<Issue>> {
        let results_repo = self.results_repo.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ResultsRepository not configured"))?;
        results_repo.get_issues(job_id).await
    }

}

#[cfg(test)]
mod tests {
    // Tests are in tests/analysis_service_tests.rs
}

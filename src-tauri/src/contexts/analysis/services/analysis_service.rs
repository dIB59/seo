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
        Ok(JobId::from(id))
    }

    /// Get a job by its ID.
    pub async fn get_job(&self, id: &str) -> Result<Job> {
        Ok(self.job_repo.get_by_id(id).await?)
    }

    /// Get a job by its ID, wrapped in the typestate dispatch enum.
    ///
    /// Prefer this in new code: callers can match on the specific
    /// lifecycle stage and the compiler enforces exhaustive handling.
    /// Existing call sites that just need a `Job` continue to use
    /// [`get_job`](Self::get_job).
    pub async fn get_job_state(&self, id: &str) -> Result<crate::contexts::analysis::AnyJob> {
        let job = self.job_repo.get_by_id(id).await?;
        Ok(crate::contexts::analysis::AnyJob::from(job))
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

    /// Typestate-aware cancellation. Loads the job, dispatches on its
    /// runtime state via [`AnyJob`], and refuses to cancel if the job
    /// is already in a terminal state (`Completed`, `Failed`, or
    /// previously `Cancelled`). The original [`cancel_job`] is a silent
    /// no-op in those cases — this version makes the precondition
    /// explicit at the boundary so the caller can surface a clear
    /// error.
    pub async fn cancel_job_typed(&self, id: &str) -> Result<()> {
        let any = self.get_job_state(id).await?;
        if any.is_terminal() {
            return Err(anyhow::anyhow!(
                "cannot cancel job {id}: already {}",
                any.stage_name()
            ));
        }
        // Pending/Discovery/Processing → cancel via the existing path.
        self.cancel_job(id).await
    }

    /// List jobs with optional filtering
    pub async fn list_jobs(&self, filter: JobFilter) -> Result<Vec<JobInfo>> {
        let limit = filter.limit.unwrap_or(100);
        let offset = filter.offset.unwrap_or(0);
        let pagination = crate::contexts::analysis::Pagination::new(limit, offset)?;

        let mut query = crate::contexts::analysis::JobPageQuery::new(pagination);
        if let Some(url) = filter.url_contains.clone() {
            query = query.with_url_filter(url);
        }
        if let Some(status) = filter.status {
            query = query.with_status(status.as_str());
        }

        let (jobs, _total) = self.job_repo.get_paginated_with_total(query).await?;
        Ok(jobs)
    }

    /// Get all jobs
    pub async fn get_all_jobs(&self) -> Result<Vec<JobInfo>> {
        Ok(self.job_repo.get_all().await?)
    }

    /// Get paginated jobs
    pub async fn get_paginated_jobs(&self, limit: i64, offset: i64) -> Result<Vec<JobInfo>> {
        Ok(self.job_repo.get_paginated(limit, offset).await?)
    }

    /// Get paginated jobs with total count and filters.
    ///
    /// The (limit, offset, url_filter, status_filter) tuple is bundled into
    /// a validated `JobPageQuery` before hitting the repository layer.
    pub async fn get_paginated_jobs_with_total(
        &self,
        limit: i64,
        offset: i64,
        url_filter: Option<String>,
        status_filter: Option<String>,
    ) -> Result<(Vec<JobInfo>, i64)> {
        let pagination = crate::contexts::analysis::Pagination::new(limit, offset)?;
        let mut query = crate::contexts::analysis::JobPageQuery::new(pagination);
        if let Some(url) = url_filter {
            query = query.with_url_filter(url);
        }
        if let Some(status) = status_filter {
            query = query.with_status(status);
        }
        Ok(self.job_repo.get_paginated_with_total(query).await?)
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
        Ok(results_repo.get_complete_result(job_id).await?)
    }

    // === Page Access ===

    /// Get all pages for a job
    pub async fn get_pages(&self, job_id: &str) -> Result<Vec<Page>> {
        let results_repo = self.results_repo.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ResultsRepository not configured"))?;
        Ok(results_repo.get_pages(job_id).await?)
    }

    // === Issue Access ===

    /// Get all issues for a job
    pub async fn get_issues(&self, job_id: &str) -> Result<Vec<Issue>> {
        let results_repo = self.results_repo.as_ref()
            .ok_or_else(|| anyhow::anyhow!("ResultsRepository not configured"))?;
        Ok(results_repo.get_issues(job_id).await?)
    }

}

#[cfg(test)]
mod tests {
    // Tests are in tests/analysis_service_tests.rs
}

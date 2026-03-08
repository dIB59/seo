// AnalysisService Factory
// Creates AnalysisService instances with proper dependency injection

use std::sync::Arc;
use super::services::AnalysisService;
use crate::repository::{JobRepository, ResultsRepository};
use crate::service::JobProcessor;

/// Factory for creating AnalysisService instances
pub struct AnalysisServiceFactory;

impl AnalysisServiceFactory {
    /// Create an AnalysisService from an existing JobRepository
    pub fn from_repository(job_repo: Arc<dyn JobRepository>) -> AnalysisService {
        AnalysisService::new(job_repo)
    }

    /// Create an AnalysisService with both JobRepository and ResultsRepository
    pub fn with_repositories(
        job_repo: Arc<dyn JobRepository>,
        results_repo: Arc<dyn ResultsRepository>,
    ) -> AnalysisService {
        AnalysisService::with_repositories(job_repo, results_repo)
    }

    /// Create an AnalysisService with full dependencies including JobProcessor
    pub fn with_processor(
        job_repo: Arc<dyn JobRepository>,
        results_repo: Arc<dyn ResultsRepository>,
        job_processor: Arc<JobProcessor>,
    ) -> AnalysisService {
        AnalysisService::with_processor(job_repo, results_repo, job_processor)
    }
}

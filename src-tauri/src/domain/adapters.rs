//! Adapter layer for converting domain models to API response types.
//!
//! This module provides conversion functions from domain models to the
//! API response types used by the frontend.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::{Job, JobInfo};

/// Analysis progress for frontend updates
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalysisProgress {
    pub job_id: String,
    pub url: String,
    pub job_status: String,
    pub result_id: Option<String>,
    pub progress: Option<f64>,
    pub analyzed_pages: Option<i64>,
    pub total_pages: Option<i64>,
}

// ============================================================================
// JOB TO ANALYSIS PROGRESS
// ============================================================================

impl From<Job> for AnalysisProgress {
    fn from(job: Job) -> Self {
        Self {
            job_id: job.id.clone(),
            url: job.url,
            job_status: job.status.as_str().to_string(),
            result_id: Some(job.id.clone()),
            progress: Some(job.progress),
            analyzed_pages: Some(job.summary.pages_crawled),
            total_pages: Some(job.summary.total_pages),
        }
    }
}

impl From<JobInfo> for AnalysisProgress {
    fn from(info: JobInfo) -> Self {
        Self {
            job_id: info.id.clone(),
            url: info.url,
            job_status: info.status.as_str().to_string(),
            result_id: Some(info.id.clone()),
            progress: Some(info.progress),
            analyzed_pages: Some(info.total_pages),
            total_pages: Some(info.total_pages),
        }
    }
}

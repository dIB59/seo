//! Analysis progress formatting for frontend updates.
//!
//! This module provides the `AnalysisProgress` DTO and conversion logic
//! from the internal `Job` domain model.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::{Job, JobInfo, JobStatus};

/// Analysis progress for frontend updates
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalysisProgress {
    pub job_id: String,
    pub url: String,
    pub job_status: JobStatus,
    pub result_id: Option<String>,
    pub progress: Option<f64>,
    pub max_pages: Option<i64>,
    pub is_deep_audit: Option<bool>,
    pub total_issues: Option<i64>,
}

impl From<Job> for AnalysisProgress {
    fn from(job: Job) -> Self {
        Self {
            job_id: job.id.clone(),
            url: job.url,
            job_status: job.status,
            result_id: Some(job.id.clone()),
            progress: Some(job.progress),
            max_pages: Some(job.settings.max_pages),
            is_deep_audit: Some(job.settings.lighthouse_analysis),
            total_issues: Some(job.summary.total_issues),
        }
    }
}

impl From<JobInfo> for AnalysisProgress {
    fn from(info: JobInfo) -> Self {
        Self {
            job_id: info.id.clone(),
            url: info.url,
            job_status: info.status,
            result_id: Some(info.id.clone()),
            progress: Some(info.progress),
            max_pages: Some(info.max_pages),
            is_deep_audit: Some(info.lighthouse_analysis),
            total_issues: Some(info.total_issues),
        }
    }
}

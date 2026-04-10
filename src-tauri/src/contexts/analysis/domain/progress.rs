//! Analysis progress formatting for frontend updates.
//!
//! This module provides the `AnalysisProgress` DTO and conversion logic
//! from the internal `Job` domain model.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{Job, JobInfo, JobStatus};

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

impl From<JobInfo> for AnalysisProgress {
    fn from(info: JobInfo) -> Self {
        let id_str = info.id().as_str().to_string();
        Self {
            job_id: id_str.clone(),
            url: info.url().to_string(),
            job_status: info.status().clone(),
            result_id: Some(id_str),
            progress: Some(info.progress()),
            max_pages: Some(info.max_pages()),
            is_deep_audit: Some(info.lighthouse_analysis()),
            total_issues: Some(info.total_issues()),
        }
    }
}

/// Project a full [`Job`] down to the progress DTO by routing through
/// [`JobInfo`] — reuses the single [`From<JobInfo>`] implementation so
/// the field-mapping logic lives in exactly one place. Adding a new
/// `AnalysisProgress` field now means updating one From impl, not two.
impl From<Job> for AnalysisProgress {
    fn from(job: Job) -> Self {
        AnalysisProgress::from(JobInfo::from(&job))
    }
}

#[cfg(test)]
mod tests {
    //! Characterization tests for the `AnalysisProgress` DTO and its
    //! `From<Job>` / `From<JobInfo>` conversions. The DTO is the
    //! frontend-visible payload of the analysis progress event, so the
    //! field shapes here are pinned by the Tauri bindings.
    //!
    //! Both From impls always populate the `Option<_>` fields with
    //! `Some(_)`. A future Phase 2 cleanup may drop the `Option<>`
    //! wrappers; until then, these tests pin the current behavior so
    //! the migration can land under green.

    use super::*;
    use crate::contexts::analysis::{JobId, JobSettings, JobSummary};
    use chrono::Utc;

    fn make_job() -> Job {
        Job {
            id: JobId::from("job-123"),
            url: "https://example.com".to_string(),
            status: JobStatus::Processing,
            settings: JobSettings {
                max_pages: 250,
                lighthouse_analysis: true,
                ..JobSettings::default()
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            summary: JobSummary::new(0, 0, 17, 0, 0, 0),
            progress: 42.5,
            error_message: None,
            sitemap_found: false,
            robots_txt_found: false,
        }
    }

    #[test]
    fn from_job_copies_id_into_both_job_id_and_result_id() {
        let progress = AnalysisProgress::from(make_job());
        assert_eq!(progress.job_id, "job-123");
        assert_eq!(progress.result_id.as_deref(), Some("job-123"));
    }

    #[test]
    fn from_job_propagates_url_and_status() {
        let progress = AnalysisProgress::from(make_job());
        assert_eq!(progress.url, "https://example.com");
        assert_eq!(progress.job_status, JobStatus::Processing);
    }

    #[test]
    fn from_job_extracts_progress_and_settings() {
        let progress = AnalysisProgress::from(make_job());
        assert_eq!(progress.progress, Some(42.5));
        assert_eq!(progress.max_pages, Some(250));
        assert_eq!(progress.is_deep_audit, Some(true));
        assert_eq!(progress.total_issues, Some(17));
    }

    #[test]
    fn from_job_always_populates_optional_fields_with_some() {
        // Pinning the "always Some" invariant — a Phase 2 cleanup may
        // drop these Option<_> wrappers, but until then this test
        // catches accidental None.
        let progress = AnalysisProgress::from(make_job());
        assert!(progress.progress.is_some());
        assert!(progress.max_pages.is_some());
        assert!(progress.is_deep_audit.is_some());
        assert!(progress.total_issues.is_some());
        assert!(progress.result_id.is_some());
    }

    #[test]
    fn from_job_info_routes_through_same_optional_pattern() {
        let info = JobInfo::new(
            JobId::from("job-info-1"),
            "https://other.test".to_string(),
            JobStatus::Completed,
            100.0,
            50,
            3,
            Utc::now(),
            100,
            false,
        );
        let progress = AnalysisProgress::from(info);
        assert_eq!(progress.job_id, "job-info-1");
        assert_eq!(progress.url, "https://other.test");
        assert_eq!(progress.job_status, JobStatus::Completed);
        assert_eq!(progress.progress, Some(100.0));
        assert_eq!(progress.max_pages, Some(100));
        assert_eq!(progress.is_deep_audit, Some(false));
        assert_eq!(progress.total_issues, Some(3));
        assert_eq!(progress.result_id.as_deref(), Some("job-info-1"));
    }

    #[test]
    fn analysis_progress_serde_round_trip() {
        let progress = AnalysisProgress::from(make_job());
        let json = serde_json::to_value(&progress).unwrap();
        // Frontend payload pinned: job_id is a flat string (not the
        // newtype object form), is_deep_audit is a bool, etc.
        assert_eq!(json["job_id"], "job-123");
        assert_eq!(json["is_deep_audit"], true);
        assert_eq!(json["progress"], 42.5);
        let parsed: AnalysisProgress = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.job_id, "job-123");
    }
}
//! Adapter layer for converting domain models to API response types.
//!
//! This module provides conversion functions from domain models to the
//! API response types used by the frontend.

use crate::domain::models::{
    AnalysisProgress, Issue, Job, JobInfo, Page, PageAnalysisData, SeoIssue,
};

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

// ============================================================================
// PAGE CONVERSION
// ============================================================================

impl From<Page> for PageAnalysisData {
    fn from(page: Page) -> Self {
        Self {
            analysis_id: page.job_id,
            url: page.url,
            title: page.title,
            meta_description: page.meta_description,
            meta_keywords: None,
            canonical_url: page.canonical_url,
            h1_count: 0,
            h2_count: 0,
            h3_count: 0,
            word_count: page.word_count.unwrap_or(0),
            image_count: 0,
            images_without_alt: 0,
            internal_links: 0,
            external_links: 0,
            load_time: page.load_time_ms.unwrap_or(0) as f64 / 1000.0,
            status_code: page.status_code,
            content_size: page.response_size_bytes.unwrap_or(0),
            mobile_friendly: true,
            has_structured_data: false,
            lighthouse_performance: None,
            lighthouse_accessibility: None,
            lighthouse_best_practices: None,
            lighthouse_seo: None,
            lighthouse_seo_audits: None,
            lighthouse_performance_metrics: None,
            links: vec![],
            headings: vec![],
            images: vec![],
            detailed_links: vec![],
        }
    }
}

// ============================================================================
// ISSUE CONVERSION
// ============================================================================

impl From<Issue> for SeoIssue {
    fn from(issue: Issue) -> Self {
        Self {
            page_id: issue.page_id.unwrap_or_default(),
            severity: issue.severity,
            title: issue.issue_type.clone(),
            description: issue.message,
            page_url: String::new(), // Will be populated from page data if needed
            element: issue.details.clone(),
            recommendation: issue.details.unwrap_or_default(),
            line_number: None,
        }
    }
}



// ============================================================================
// RESPONSE TYPES FOR NEW API
// ============================================================================

/// Job creation response (uses string ID for V2 schema)
#[derive(Debug, serde::Serialize)]
pub struct JobCreatedResponse {
    pub job_id: String,
    pub url: String,
    pub status: String,
}

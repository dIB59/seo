//! Adapter layer for converting domain models to API response types.
//!
//! This module provides conversion functions from domain models to the
//! API response types used by the frontend.

use crate::domain::models::{
    AnalysisProgress, AnalysisSummary, CompleteAnalysisResult, AnalysisResults,
    PageAnalysisData, SeoIssue, IssueType,
    CompleteJobResult, Issue, IssueSeverity, Job, JobInfo, Page,
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

impl From<IssueSeverity> for IssueType {
    fn from(severity: IssueSeverity) -> Self {
        match severity {
            IssueSeverity::Critical => IssueType::Critical,
            IssueSeverity::Warning => IssueType::Warning,
            IssueSeverity::Info => IssueType::Suggestion,
        }
    }
}

impl From<Issue> for SeoIssue {
    fn from(issue: Issue) -> Self {
        Self {
            page_id: issue.page_id.unwrap_or_default(),
            issue_type: issue.severity.into(),
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
// COMPLETE RESULT CONVERSION
// ============================================================================

impl From<CompleteJobResult> for CompleteAnalysisResult {
    fn from(result: CompleteJobResult) -> Self {
        let job = &result.job;
        
        // Build AnalysisResults from Job
        let analysis = AnalysisResults {
            id: job.id.clone(),
            url: job.url.clone(),
            status: job.status.clone(),
            progress: job.progress,
            total_pages: job.summary.total_pages,
            analyzed_pages: job.summary.pages_crawled,
            started_at: Some(job.created_at),
            completed_at: job.completed_at,
            sitemap_found: false, // TODO: store in job metadata
            robots_txt_found: false,
            ssl_certificate: job.url.starts_with("https"),
            created_at: job.created_at,
        };

        // Convert pages
        let pages: Vec<PageAnalysisData> = result.pages.into_iter().map(|p| p.into()).collect();

        // Convert issues with page URLs
        let issues: Vec<SeoIssue> = result.issues.into_iter().map(|i| i.into()).collect();

        // Build summary from job stats
        let summary = AnalysisSummary {
            analysis_id: job.id.clone(),
            seo_score: calculate_seo_score(job),
            avg_load_time: 0.0, // TODO: calculate from pages
            total_words: pages.iter().map(|p| p.word_count).sum(),
            total_issues: job.summary.total_issues,
        };

        Self {
            analysis,
            pages,
            issues,
            summary,
        }
    }
}

/// Calculate a simple SEO score based on issue counts.
fn calculate_seo_score(job: &Job) -> i64 {
    let total = job.summary.total_issues;
    let critical = job.summary.critical_issues;
    let warning = job.summary.warning_issues;
    
    if total == 0 {
        return 100;
    }
    
    // Deduct points for issues: 10 for critical, 5 for warning, 1 for info
    let deductions = (critical * 10) + (warning * 5) + (total - critical - warning);
    let score = 100 - deductions;
    
    score.clamp(0, 100)
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

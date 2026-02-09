use std::{str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use specta::Type;
use tauri::State;
use url::Url;


use crate::{
    domain::models::{AnalysisProgress, AnalysisResults, AnalysisSummary, CompleteAnalysisResult, ImageElement, JobSettings, JobStatus, PageAnalysisData, SeoIssue}, error::CommandError, lifecycle::app_state::AppState, service::JobProcessor
};

#[derive(Debug, serde::Serialize, Type)]
pub struct AnalysisJobResponse {
    pub job_id: String,
    pub url: String,
    pub status: JobStatus,
}

// Specta-friendly DTOs (map date/time to strings and avoid types Specta cannot
// easily represent)
#[derive(Debug, serde::Serialize, Type)]
pub struct AnalysisResultsResponse {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub progress: f64,
    pub total_pages: i64,
    pub analyzed_pages: i64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub sitemap_found: bool,
    pub robots_txt_found: bool,
    pub ssl_certificate: bool,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, Type)]
pub struct PageAnalysisDataResponse {
    pub analysis_id: String,
    pub url: String,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub canonical_url: Option<String>,
    pub h1_count: i64,
    pub h2_count: i64,
    pub h3_count: i64,
    pub word_count: i64,
    pub image_count: i64,
    pub images_without_alt: i64,
    pub internal_links: i64,
    pub external_links: i64,
    pub load_time: f64,
    pub status_code: Option<i64>,
    pub content_size: i64,
    pub mobile_friendly: bool,
    pub has_structured_data: bool,
    pub lighthouse_performance: Option<f64>,
    pub lighthouse_accessibility: Option<f64>,
    pub lighthouse_best_practices: Option<f64>,
    pub lighthouse_seo: Option<f64>,
    pub lighthouse_seo_audits: Option<serde_json::Value>,
    pub lighthouse_performance_metrics: Option<serde_json::Value>,
    pub images: Vec<ImageElement>,
    pub detailed_links: Vec<crate::domain::models::LinkDetail>,
}

#[derive(Debug, serde::Serialize, Type)]
pub struct SeoIssueResponse {
    pub page_id: String,
    pub severity: crate::domain::models::IssueSeverity,
    pub title: String,
    pub description: String,
    pub page_url: String,
    pub element: Option<String>,
    pub recommendation: String,
    pub line_number: Option<i32>,
}

#[derive(Debug, serde::Serialize, Type)]
pub struct CompleteAnalysisResponse {
    pub analysis: AnalysisResultsResponse,
    pub pages: Vec<PageAnalysisDataResponse>,
    pub issues: Vec<SeoIssueResponse>,
    pub summary: AnalysisSummary,
}

impl From<AnalysisResults> for AnalysisResultsResponse {
    fn from(a: AnalysisResults) -> Self {
        Self {
            id: a.id,
            url: a.url,
            status: a.status,
            progress: a.progress,
            total_pages: a.total_pages,
            analyzed_pages: a.analyzed_pages,
            started_at: a.started_at.map(|d| d.to_rfc3339()),
            completed_at: a.completed_at.map(|d| d.to_rfc3339()),
            sitemap_found: a.sitemap_found,
            robots_txt_found: a.robots_txt_found,
            ssl_certificate: a.ssl_certificate,
            created_at: a.created_at.to_rfc3339(),
        }
    }
}

impl From<PageAnalysisData> for PageAnalysisDataResponse {
    fn from(p: PageAnalysisData) -> Self {
        Self {
            analysis_id: p.analysis_id,
            url: p.url,
            title: p.title,
            meta_description: p.meta_description,
            meta_keywords: p.meta_keywords,
            canonical_url: p.canonical_url,
            h1_count: p.h1_count,
            h2_count: p.h2_count,
            h3_count: p.h3_count,
            word_count: p.word_count,
            image_count: p.image_count,
            images_without_alt: p.images_without_alt,
            internal_links: p.internal_links,
            external_links: p.external_links,
            load_time: p.load_time,
            status_code: p.status_code,
            content_size: p.content_size,
            mobile_friendly: p.mobile_friendly,
            has_structured_data: p.has_structured_data,
            lighthouse_performance: p.lighthouse_performance,
            lighthouse_accessibility: p.lighthouse_accessibility,
            lighthouse_best_practices: p.lighthouse_best_practices,
            lighthouse_seo: p.lighthouse_seo,
            lighthouse_seo_audits: p.lighthouse_seo_audits,
            lighthouse_performance_metrics: p.lighthouse_performance_metrics,
            images: p.images,
            detailed_links: p.detailed_links,
        }
    }
}

impl From<SeoIssue> for SeoIssueResponse {
    fn from(i: SeoIssue) -> Self {
        Self {
            page_id: i.page_id,
            severity: i.severity,
            title: i.title,
            description: i.description,
            page_url: i.page_url,
            element: i.element,
            recommendation: i.recommendation,
            line_number: i.line_number,
        }
    }
}

impl From<CompleteAnalysisResult> for CompleteAnalysisResponse {
    fn from(c: CompleteAnalysisResult) -> Self {
        Self {
            analysis: c.analysis.into(),
            pages: c.pages.into_iter().map(|p| p.into()).collect(),
            issues: c.issues.into_iter().map(|i| i.into()).collect(),
            summary: c.summary,
        }
    }
}

#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct AnalysisSettingsRequest {
    pub max_pages: i64,
    pub include_external_links: bool,
    pub check_images: bool,
    pub mobile_analysis: bool,
    pub lighthouse_analysis: bool,
    pub delay_between_requests: i64,
}

impl Default for AnalysisSettingsRequest {
    fn default() -> Self {
        Self {
            max_pages: 100,
            include_external_links: false,
            check_images: true,
            mobile_analysis: false,
            lighthouse_analysis: false,
            delay_between_requests: 500,
        }
    }
}

impl From<AnalysisSettingsRequest> for JobSettings {
    fn from(req: AnalysisSettingsRequest) -> Self {
        Self {
            max_pages: req.max_pages,
            include_external_links: req.include_external_links,
            check_images: req.check_images,
            mobile_analysis: req.mobile_analysis,
            lighthouse_analysis: req.lighthouse_analysis,
            delay_between_requests: req.delay_between_requests,
        }
    }
}

fn validate_url(url: &str) -> Result<Url> {
    Url::from_str(url).with_context(|| format!("Invalid URL: {}", url))
}

#[tauri::command]
#[specta::specta] // < You must annotate your commands
pub async fn start_analysis(
    url: String,
    settings: Option<AnalysisSettingsRequest>,
    app_state: State<'_, AppState>,
) -> Result<AnalysisJobResponse, CommandError> {
    log::info!("Starting analysis: {}", url);
    log::info!("Settings: {:?}", settings);
    let parsed_url = validate_url(&url).context("Bad URL")?;

    let analysis_settings: JobSettings = settings.unwrap_or_default().into();

    let repository = app_state.job_repo.clone();
    let job_id = repository
        .create(parsed_url.as_str(), &analysis_settings)
        .await
        .map_err(CommandError::from)?;

    Ok(AnalysisJobResponse {
        job_id,
        url,
        status: JobStatus::Pending,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_analysis_progress(
    job_id: String,
    app_state: State<'_, AppState>,
) -> Result<AnalysisProgress, CommandError> {
    log::info!("Getting analysis progress for job: {}", job_id);

    let repository = app_state.job_repo.clone();

    let job = repository
        .get_by_id(&job_id)
        .await
        .map_err(CommandError::from)?;

    // Convert V2 Job to V1 AnalysisProgress
    Ok(job.into())
}

//TODO:
//Implement pagination
#[tauri::command]
#[specta::specta]
pub async fn get_all_jobs(app_state: State<'_, AppState>) -> Result<Vec<AnalysisProgress>, CommandError> {
    log::info!("Fetching all analysis jobs");

    let repository = app_state.job_repo.clone();

    let jobs = repository.get_all().await.map_err(CommandError::from)?;

    // Convert V2 Jobs to V1 AnalysisProgress
    let progress: Vec<AnalysisProgress> = jobs.into_iter().map(|j| j.into()).collect();
    log::trace!("{:?}", progress.first());

    Ok(progress)
}

#[tauri::command]
#[specta::specta]
pub async fn cancel_analysis(
    job_id: String,
    job_processor: State<'_, Arc<JobProcessor>>,
) -> Result<(), CommandError> {
    log::trace!("Cancelling analysis job: {}", job_id);
    job_processor.cancel(&job_id).await.map_err(CommandError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn get_result(
    job_id: String,
    app_state: State<'_, AppState>,
) -> Result<CompleteAnalysisResponse, CommandError> {
    log::trace!("Getting result ID for job: {}", job_id);

    let repo = app_state.results_repo.clone();
    let assembler = crate::service::AnalysisAssembler::new(repo);

    let result = assembler
        .assemble(&job_id)
        .await
        .map_err(CommandError::from)?;

    Ok(CompleteAnalysisResponse::from(result))
}

use std::str::FromStr;

use tauri::State;
use url::Url;
use anyhow::{Context, Result};

use crate::{db::DbState, domain::models::{AnalysisProgress, CompleteAnalysisResult, JobStatus}, error::CommandError, repository::sqlite::{JobRepository, ResultsRepository}};

#[derive(Debug, serde::Serialize)]
pub struct AnalysisJobResponse {
    pub job_id: i64,
    pub url: String,
    pub status: String,
}

#[derive(Debug, serde::Deserialize)]
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

fn validate_url(url: &str) -> Result<Url> {
    Url::from_str(url).with_context(|| format!("Invalid URL: {}", url))
}

#[tauri::command]
pub async fn start_analysis(
    url: String,
    settings: Option<AnalysisSettingsRequest>,
    db: State<'_, DbState>,
) -> Result<AnalysisJobResponse, CommandError> {
    log::info!("Starting analysis: {}", url);
    log::info!("Settings: {:?}", settings);
    let parsed_url = validate_url(&url).context("Bad URL")?;

    let analysis_settings = settings.unwrap_or_default();
    let pool = &db.0;

    let repository = JobRepository::new(pool.clone());
    let job_id = repository.create_with_settings(parsed_url.as_str(), &analysis_settings)
        .await
        .map_err(CommandError::from)?;

    Ok(AnalysisJobResponse {
        job_id,
        url,
        status: "queued".to_string(),
    })
}

#[tauri::command]
pub async fn get_analysis_progress(
    job_id: i64,
    db: State<'_, DbState>,
) -> Result<AnalysisProgress, CommandError> {
    log::info!("Getting analysis progress for job: {}", job_id);

    let pool = &db.0;
    let repository = JobRepository::new(pool.clone());

    let progress = repository.get_progress(job_id)
        .await
        .map_err(CommandError::from)?;

    Ok(progress)
}

#[tauri::command]
pub async fn get_all_jobs(db: State<'_, DbState>) -> Result<Vec<AnalysisProgress>, CommandError> {
    log::info!("Fetching all analysis jobs");

    let pool = &db.0;
    let repository = JobRepository::new(pool.clone());

    let jobs = repository.get_all()
        .await
        .map_err(CommandError::from)?;

    Ok(jobs)
}

#[tauri::command]
pub async fn cancel_analysis(job_id: i64, db: State<'_, DbState>) -> Result<(), CommandError> {
    log::info!("Cancelling analysis job: {}", job_id);

    let pool = &db.0;
    let repository = JobRepository::new(pool.clone());

    repository.update_status(job_id, JobStatus::Failed)
        .await
        .map_err(CommandError::from)
}


struct GetResultResponse {
}

#[tauri::command]
pub async fn get_result(
    job_id: i64,
    db: State<'_, DbState>,
) -> Result<CompleteAnalysisResult, CommandError> {
    log::info!("Getting result ID for job: {}", job_id);

    let pool = &db.0;
    let repository = ResultsRepository::new(pool.clone());

    repository.get_result_by_job_id(job_id)
        .await
        .map_err(CommandError::from)
}
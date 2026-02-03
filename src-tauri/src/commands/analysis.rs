use std::{str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use tauri::State;
use url::Url;

use crate::{
    db::DbState,
    domain::models::{AnalysisProgress, CompleteAnalysisResult, JobStatus},
    domain::models::JobSettings,
    error::CommandError,
    repository::sqlite::{JobRepository, ResultsRepository},
    service::JobProcessor,
};

#[derive(Debug, serde::Serialize)]
pub struct AnalysisJobResponse {
    pub job_id: String,
    pub url: String,
    pub status: JobStatus,
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
pub async fn start_analysis(
    url: String,
    settings: Option<AnalysisSettingsRequest>,
    db: State<'_, DbState>,
) -> Result<AnalysisJobResponse, CommandError> {
    log::info!("Starting analysis: {}", url);
    log::info!("Settings: {:?}", settings);
    let parsed_url = validate_url(&url).context("Bad URL")?;

    let analysis_settings: JobSettings = settings.unwrap_or_default().into();
    let pool = &db.0;

    let repository = JobRepository::new(pool.clone());
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
pub async fn get_analysis_progress(
    job_id: String,
    db: State<'_, DbState>,
) -> Result<AnalysisProgress, CommandError> {
    log::info!("Getting analysis progress for job: {}", job_id);

    let pool = &db.0;
    let repository = JobRepository::new(pool.clone());

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
pub async fn get_all_jobs(db: State<'_, DbState>) -> Result<Vec<AnalysisProgress>, CommandError> {
    log::info!("Fetching all analysis jobs");

    let pool = &db.0;
    let repository = JobRepository::new(pool.clone());

    let jobs = repository.get_all().await.map_err(CommandError::from)?;
    
    // Convert V2 Jobs to V1 AnalysisProgress
    let progress: Vec<AnalysisProgress> = jobs.into_iter().map(|j| j.into()).collect();
    log::trace!("{:?}", progress.first());

    Ok(progress)
}

#[tauri::command]
pub async fn cancel_analysis(
    job_id: String,
    job_processor: State<'_, Arc<JobProcessor>>,
) -> Result<(), CommandError> {
    log::trace!("Cancelling analysis job: {}", job_id);
    job_processor.cancel(&job_id).await.map_err(CommandError)
}

#[tauri::command]
pub async fn get_result(
    job_id: String,
    db: State<'_, DbState>,
) -> Result<CompleteAnalysisResult, CommandError> {
    log::trace!("Getting result ID for job: {}", job_id);

    let pool = &db.0;
    let repository = ResultsRepository::new(pool.clone());

    let result = repository
        .get_complete_result(&job_id)
        .await
        .map_err(CommandError::from)?;

    // Convert V2 CompleteJobResult to V1 CompleteAnalysisResult
    Ok(result.into())
}

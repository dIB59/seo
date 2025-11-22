use crate::error::CommandError;
use crate::DbState;
use anyhow::{Context, Result};
use serde::Deserialize;
use sqlx::SqliteConnection;
use std::str::FromStr;
use tauri::State;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct AnalysisSettings {
    max_pages: u32,
    include_external_links: bool,
    check_images: bool,
    mobile_analysis: bool,
    lighthouse_analysis: bool,
    delay_between_requests: u64,
}

impl Default for AnalysisSettings {
    fn default() -> Self {
        Self {
            max_pages: 10,
            include_external_links: false,
            check_images: true,
            mobile_analysis: false,
            lighthouse_analysis: false,
            delay_between_requests: 500,
        }
    }
}

impl AnalysisSettings {
    /// Convert to database values (SQLite uses integers for booleans)
    fn as_db_values(&self) -> (i64, i64, i64, i64, i64, i64) {
        (
            self.max_pages as i64,
            self.include_external_links as i64,
            self.check_images as i64,
            self.mobile_analysis as i64,
            self.lighthouse_analysis as i64,
            self.delay_between_requests as i64,
        )
    }
}

/// Response structure for created analysis job
#[derive(Debug, serde::Serialize)]
pub struct AnalysisJobResponse {
    pub job_id: i64,
    pub url: String,
    pub status: String,
}

/// Progress information for an analysis
#[derive(Debug, serde::Serialize)]
pub struct AnalysisProgress {
    pub job_id: i64,
    pub url: String,
    pub job_status: String,
    pub result_id: Option<String>,
    pub analysis_status: Option<String>,
    pub progress: Option<f64>,
    pub analyzed_pages: Option<i64>,
    pub total_pages: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct AnalysisProgressRow {
    job_id: i64,
    url: String,
    job_status: String,
    result_id: Option<String>,
    analysis_status: Option<String>,
    progress: Option<f64>,
    analyzed_pages: Option<i64>,
    total_pages: Option<i64>,
}

impl From<AnalysisProgressRow> for AnalysisProgress {
    fn from(row: AnalysisProgressRow) -> Self {
        Self {
            job_id: row.job_id,
            url: row.url,
            job_status: row.job_status,
            result_id: row.result_id,
            analysis_status: row.analysis_status,
            progress: row.progress,
            analyzed_pages: row.analyzed_pages,
            total_pages: row.total_pages,
        }
    }
}

/// Validate URL before any database operations
fn validate_url(url: &str) -> Result<Url> {
    Url::from_str(url).with_context(|| format!("Invalid URL: {}", url))
}

/// Insert settings and return its ID within an active transaction
async fn insert_settings(tx: &mut SqliteConnection, settings: &AnalysisSettings) -> Result<i64> {
    let values = settings.as_db_values();
    log::debug!("Inserting analysis settings");

    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO analysis_settings (
            max_pages, 
            include_external_links, 
            check_images, 
            mobile_analysis, 
            lighthouse_analysis, 
            delay_between_requests
        )
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
        values.0,
        values.1,
        values.2,
        values.3,
        values.4,
        values.5
    )
    .fetch_one(tx)
    .await
    .context("Failed to insert settings")?;

    log::debug!("Settings created with ID: {}", id);
    Ok(id)
}

/// Create analysis job record within transaction
async fn insert_analysis_job(
    tx: &mut SqliteConnection,
    url: &str,
    settings_id: i64,
) -> Result<i64> {
    log::debug!("Inserting analysis job record");

    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO analysis_jobs (url, settings_id, status) 
        VALUES (?, ?, 'queued') 
        RETURNING id
        "#,
        url,
        settings_id
    )
    .fetch_one(tx)
    .await
    .context("Failed to insert analysis job")?;

    log::debug!("Analysis job created with ID: {}", id);
    Ok(id)
}

/// Create analysis job with settings in a transaction
async fn create_analysis_job(
    pool: &sqlx::SqlitePool,
    url: &Url,
    settings: &AnalysisSettings,
) -> Result<i64> {
    let mut tx = pool.begin().await.context("Failed to start transaction")?;

    let settings_id = insert_settings(tx.as_mut(), settings).await?;
    let job_id = insert_analysis_job(tx.as_mut(), url.as_str(), settings_id).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    log::info!("Analysis job {} created successfully", job_id);
    Ok(job_id)
}

/// Update analysis job status
pub async fn update_job_status(pool: &sqlx::SqlitePool, job_id: i64, status: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE analysis_jobs SET status = ? WHERE id = ?",
        status,
        job_id
    )
    .execute(pool)
    .await
    .context("Failed to update job status")?;

    log::debug!("Updated job {} status to: {}", job_id, status);
    Ok(())
}

/// Link analysis job to completed result
pub async fn link_job_to_result(
    pool: &sqlx::SqlitePool,
    job_id: i64,
    result_id: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE analysis_jobs 
        SET result_id = ?, status = 'completed' 
        WHERE id = ?
        "#,
        result_id,
        job_id
    )
    .execute(pool)
    .await
    .context("Failed to link job to result")?;

    log::info!("Linked job {} to result {}", job_id, result_id);
    Ok(())
}

#[tauri::command]
pub async fn start_analysis(
    url: String,
    settings: Option<AnalysisSettings>,
    db: State<'_, DbState>,
) -> Result<AnalysisJobResponse, CommandError> {
    log::info!("Starting analysis: {}", url);

    let parsed_url = validate_url(&url).context("Bad URL")?;
    let analysis_settings = settings.unwrap_or_default();
    let pool = &db.0;

    let job_id = create_analysis_job(pool, &parsed_url, &analysis_settings)
        .await
        .map_err(CommandError::from)?;

    Ok(AnalysisJobResponse {
        job_id,
        url: url,
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

    let progress = sqlx::query_as!(
        AnalysisProgress,
        r#"
        SELECT 
            aj.id as job_id,
            aj.url as url,
            aj.status as job_status,
            aj.result_id,
            ar.status as analysis_status,
            ar.progress,
            ar.analyzed_pages,
            ar.total_pages
        FROM analysis_jobs aj
        LEFT JOIN analysis_results ar ON aj.result_id = ar.id
        WHERE aj.id = ?
        "#,
        job_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch analysis progress")
    .map_err(CommandError::from)?;

    Ok(progress)
}

#[tauri::command]
pub async fn get_all_jobs(db: State<'_, DbState>) -> Result<Vec<AnalysisProgress>, CommandError> {
    log::info!("Fetching all analysis jobs");

    let pool = &db.0;

    let rows = sqlx::query_as::<_, AnalysisProgressRow>(
        r#"
        SELECT 
            aj.id as job_id,
            aj.status as job_status,
            aj.result_id,
            aj.url as url
            ar.status as analysis_status,
            ar.progress,
            ar.analyzed_pages,
            ar.total_pages
        FROM analysis_jobs aj
        LEFT JOIN analysis_results ar ON aj.result_id = ar.id
        ORDER BY aj.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch analysis jobs")
    .map_err(CommandError::from)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn cancel_analysis(job_id: i64, db: State<'_, DbState>) -> Result<(), CommandError> {
    log::info!("Cancelling analysis job: {}", job_id);

    let pool = &db.0;

    update_job_status(pool, job_id, "failed")
        .await
        .map_err(CommandError::from)
}

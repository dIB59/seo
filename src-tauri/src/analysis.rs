use anyhow::{Context, Result};
use serde::Deserialize;
use std::str::FromStr;
use tauri::State;
use url::Url;

use crate::{db::DbState, error::CommandError}; // Alias to avoid confusion

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

/// Validate URL before any database operations
fn validate_url(url: &str) -> Result<Url> {
    Url::from_str(url).with_context(|| format!("Invalid URL: {}", url))
}

/// Insert settings and return its ID within an active transaction
async fn insert_settings(
    tx: &mut sqlx::SqliteConnection,
    settings: &AnalysisSettings,
) -> Result<i64> {
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

/// Create analysis record within transaction
async fn insert_analysis(
    tx: &mut sqlx::SqliteConnection,
    url: &str,
    settings_id: i64,
) -> Result<i64> {
    log::debug!("Inserting analysis record");

    let id = sqlx::query_scalar!(
        "INSERT INTO analyses (url, settings_id) VALUES (?, ?) RETURNING id",
        url,
        settings_id
    )
    .fetch_one(tx)
    .await
    .context("Failed to insert analysis")?;

    log::debug!("Analysis created with ID: {}", id);
    Ok(id)
}

async fn create_analysis(
    pool: &sqlx::SqlitePool,
    url: &Url,
    settings: &AnalysisSettings,
) -> Result<i64> {
    let mut tx = pool.begin().await.context("Failed to start transaction")?;

    let settings_id = insert_settings(tx.as_mut(), settings).await?;
    let analysis_id = insert_analysis(tx.as_mut(), url.as_str(), settings_id).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    log::info!("Analysis {} created successfully", analysis_id);
    Ok(analysis_id)
}

#[tauri::command]
pub async fn start_analysis(
    url: String,
    settings: Option<AnalysisSettings>,
    db: State<'_, DbState>,
) -> Result<i64, CommandError> {
    log::info!("Starting analysis: {}", url);

    let parsed_url = validate_url(&url)?;

    let analysis_settings = settings.unwrap_or_default();

    let pool = &db.0;
    create_analysis(pool, &parsed_url, &analysis_settings)
        .await
        .map_err(CommandError::from)
}

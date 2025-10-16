use serde::Deserialize;
use sqlx::Executor;
use tauri::State;

use crate::{db::DbState, error::CommandError};
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct AnalysisSettings {
    max_pages: u32,
    include_external_links: bool,
    check_images: bool,
    mobile_analysis: bool,
    lighthouse_analysis: bool,
    delay_between_requests: u64, // ms
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

#[tauri::command]
pub async fn start_analysis(
    url: String,
    settings: Option<AnalysisSettings>,
    db: State<'_, DbState>,
) -> Result<i64, CommandError> {
    log::info!("Starting analysis for URL: {}", url);
    let settings = settings.unwrap_or_default();
    let pool: SqlitePool = db.inner().0.clone();

    // Start a transaction
    let mut tx = pool.begin().await.expect("");
    log::debug!("Database transaction started");
    let max_pages = settings.max_pages as i64;
    let include_external = if settings.include_external_links {
        1i64
    } else {
        0i64
    };
    let check_images = if settings.check_images { 1i64 } else { 0i64 };
    let mobile_analysis = if settings.mobile_analysis { 1i64 } else { 0i64 };
    let lighthouse_analysis = if settings.lighthouse_analysis {
        1i64
    } else {
        0i64
    };
    let delay = settings.delay_between_requests as i64;

    // Insert analysis settings and get the ID
    let settings_id: i64 = sqlx::query_scalar!(
        r#"
        INSERT INTO analysis_settings
            (max_pages, include_external_links, check_images, mobile_analysis, lighthouse_analysis, delay_between_requests)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
        max_pages,
        include_external,
        check_images,
        mobile_analysis,
        lighthouse_analysis,
        delay
    )
    .fetch_one(tx.as_mut())
    .await.expect("");

    // Insert analysis and get the ID
    let analysis_id: i64 = sqlx::query_scalar!(
        r#"
        INSERT INTO analyses (url, settings_id)
        VALUES (?, ?)
        RETURNING id
        "#,
        url,
        settings_id
    )
    .fetch_one(tx.as_mut())
    .await
    .expect("");

    // Commit the transaction
    tx.commit().await.expect("");

    Ok(analysis_id)
}

use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::domain::models::AnalysisSettings;

pub struct SettingsRepository {
    pool: SqlitePool,
}

impl SettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: i64) -> Result<AnalysisSettings> {
        let row = sqlx::query!("SELECT * FROM analysis_settings WHERE id = ?", id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to fetch analysis settings")?;

        Ok(AnalysisSettings {
            id: row.id,
            max_pages: row.max_pages,
            include_external_links: row.include_external_links != 0,
            check_images: row.check_images != 0,
            mobile_analysis: row.mobile_analysis != 0,
            lighthouse_analysis: row.lighthouse_analysis != 0,
            delay_between_requests: row.delay_between_requests,
            created_at: row.created_at.expect("Must exist").and_utc(),
        })
    }
}

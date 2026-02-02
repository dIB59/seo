//! # DEPRECATED - V1 Repository
//!
//! This module contains the V1 SQLite repository implementation.
//! It has been superseded by the V2 schema (migration 0018+).
//!
//! **Warning:** This code uses runtime SQL queries because the V1 table
//! (analysis_settings) no longer exists in the schema after migration 0018.
//!
//! This repository will only work at runtime if V1 tables still exist
//! in the database (e.g., for legacy data migration purposes).

use anyhow::{Context, Result};
use sqlx::{Row, SqlitePool};

use crate::domain::models::AnalysisSettings;

pub struct SettingsRepository {
    pool: SqlitePool,
}

impl SettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: i64) -> Result<AnalysisSettings> {
        let row = sqlx::query("SELECT * FROM analysis_settings WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to fetch analysis settings")?;

        Ok(AnalysisSettings {
            id: row.get("id"),
            max_pages: row.get("max_pages"),
            include_external_links: row.get::<i32, _>("include_external_links") != 0,
            check_images: row.get::<i32, _>("check_images") != 0,
            mobile_analysis: row.get::<i32, _>("mobile_analysis") != 0,
            lighthouse_analysis: row.get::<i32, _>("lighthouse_analysis") != 0,
            delay_between_requests: row.get("delay_between_requests"),
            created_at: row.get::<Option<chrono::NaiveDateTime>, _>("created_at").expect("Must exist").and_utc(),
        })
    }
}

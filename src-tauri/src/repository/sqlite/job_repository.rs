use anyhow::{Context, Result};
use sqlx::{Row, SqlitePool};

use crate::{
    commands::analysis::AnalysisSettingsRequest,
    domain::models::{AnalysisJob, AnalysisProgress, JobStatus},
    repository::sqlite::map_job_status,
};

/// V1 Job Repository - DEPRECATED
/// 
/// This repository uses the old V1 schema (analysis_jobs, analysis_settings, etc.)
/// which has been replaced by the V2 schema (jobs table).
/// 
/// Uses runtime SQL queries to compile without requiring V1 tables to exist.
/// Will fail at runtime if V1 tables don't exist.
pub struct JobRepository {
    pool: SqlitePool,
}

impl JobRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_pending_jobs(&self) -> Result<Vec<AnalysisJob>> {
        let rows = sqlx::query(
            "SELECT id, url, settings_id, created_at, status, result_id \
             FROM analysis_jobs \
             WHERE status IN ('queued', 'processing') \
             ORDER BY created_at ASC"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pending jobs")?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let id: i64 = row.get("id");
                let created_at: chrono::NaiveDateTime = row.get("created_at");
                AnalysisJob {
                    id,
                    url: row.get("url"),
                    settings_id: row.get("settings_id"),
                    created_at: created_at.and_utc(),
                    status: map_job_status(row.get::<&str, _>("status")),
                    result_id: row.get("result_id"),
                }
            })
            .collect())
    }

    pub async fn update_status(&self, job_id: i64, status: JobStatus) -> Result<()> {
        log::info!("Job {} being updated to {:?}", job_id, status);
        sqlx::query("UPDATE analysis_jobs SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(job_id)
            .execute(&self.pool)
            .await
            .context("Failed to update job status")?;
        Ok(())
    }

    pub async fn create_with_settings(
        &self,
        url: &str,
        settings: &AnalysisSettingsRequest,
    ) -> Result<i64> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to start transaction")?;
        
        // Insert settings
        let settings_id: i64 = sqlx::query_scalar(
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
            "#
        )
        .bind(settings.max_pages)
        .bind(settings.include_external_links as i64)
        .bind(settings.check_images as i64)
        .bind(settings.mobile_analysis as i64)
        .bind(settings.lighthouse_analysis as i64)
        .bind(settings.delay_between_requests)
        .fetch_one(tx.as_mut())
        .await
        .context("Failed to insert settings")?;

        // Insert job
        let job_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO analysis_jobs (url, settings_id, status) 
            VALUES (?, ?, 'queued') 
            RETURNING id
            "#
        )
        .bind(url)
        .bind(settings_id)
        .fetch_one(tx.as_mut())
        .await
        .context("Failed to insert analysis job")?;

        tx.commit().await.context("Failed to commit transaction")?;

        log::info!("Analysis job {} created successfully", job_id);
        Ok(job_id)
    }

    pub async fn get_progress(&self, job_id: i64) -> Result<AnalysisProgress> {
        let row = sqlx::query(
            r#"
            SELECT 
                aj.id as job_id,
                aj.url,
                aj.status as job_status,
                aj.result_id,
                ar.progress,
                ar.analyzed_pages,
                ar.total_pages
            FROM analysis_jobs aj
            LEFT JOIN analysis_results ar ON aj.result_id = ar.id
            WHERE aj.id = ?
            "#
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch analysis progress")?;

        let job_id_val: i64 = row.get("job_id");
        Ok(AnalysisProgress {
            job_id: job_id_val.to_string(),
            url: row.get("url"),
            job_status: row.get("job_status"),
            result_id: row.get("result_id"),
            progress: row.get("progress"),
            analyzed_pages: row.get("analyzed_pages"),
            total_pages: row.get("total_pages"),
        })
    }

    pub async fn get_all(&self) -> Result<Vec<AnalysisProgress>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                aj.id as job_id,
                aj.url,
                aj.status as job_status,
                aj.result_id,
                ar.status as analysis_status,
                ar.progress,
                ar.analyzed_pages,
                ar.total_pages
            FROM analysis_jobs aj
            LEFT JOIN analysis_results ar ON aj.result_id = ar.id
            ORDER BY aj.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch analysis jobs")?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let job_id: i64 = row.get("job_id");
                AnalysisProgress {
                    job_id: job_id.to_string(),
                    url: row.get("url"),
                    job_status: row.get("job_status"),
                    result_id: row.get("result_id"),
                    progress: row.get("progress"),
                    analyzed_pages: row.get("analyzed_pages"),
                    total_pages: row.get("total_pages"),
                }
            })
            .collect())
    }

    pub async fn link_to_result(&self, job_id: i64, result_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE analysis_jobs 
            SET result_id = ?
            WHERE id = ?
            "#
        )
        .bind(result_id)
        .bind(job_id)
        .execute(&self.pool)
        .await
        .context("Failed to link job to result")?;

        log::info!("Linked job {} to result {}", job_id, result_id);
        Ok(())
    }
}

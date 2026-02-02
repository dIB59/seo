use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::{
    commands::analysis::AnalysisSettingsRequest,
    domain::models::{AnalysisJob, AnalysisProgress, JobStatus},
    repository::sqlite::map_job_status,
};

pub struct JobRepository {
    pool: SqlitePool,
}

impl JobRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_pending_jobs(&self) -> Result<Vec<AnalysisJob>> {
        let rows = sqlx::query!(
            "SELECT id, url, settings_id, created_at, status, result_id \
             FROM analysis_jobs \
             WHERE status IN ('queued', 'processing') \
             ORDER BY created_at ASC
             "
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pending jobs")?;

        Ok(rows
            .into_iter()
            .map(|row| AnalysisJob {
                id: row.id.expect("ID must not be null"),
                url: row.url,
                settings_id: row.settings_id,
                created_at: row
                    .created_at
                    .expect("Created at must not be null")
                    .and_utc(),
                status: map_job_status(&row.status),
                result_id: row.result_id,
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
        let max_pages = settings.max_pages;
        let include_external_links = settings.include_external_links as i64;
        let check_images = settings.check_images as i64;
        let mobile_analysis = settings.mobile_analysis as i64;
        let lighthouse_analysis = settings.lighthouse_analysis as i64;
        let delay_between_requests = settings.delay_between_requests;

        let settings_id = sqlx::query_scalar!(
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
            max_pages,
            include_external_links,
            check_images,
            mobile_analysis,
            lighthouse_analysis,
            delay_between_requests
        )
        .fetch_one(tx.as_mut())
        .await
        .context("Failed to insert settings")?;

        // Insert job
        let job_id = sqlx::query_scalar!(
            r#"
            INSERT INTO analysis_jobs (url, settings_id, status) 
            VALUES (?, ?, 'queued') 
            RETURNING id
            "#,
            url,
            settings_id
        )
        .fetch_one(tx.as_mut())
        .await
        .context("Failed to insert analysis job")?;

        tx.commit().await.context("Failed to commit transaction")?;

        log::info!("Analysis job {} created successfully", job_id);
        Ok(job_id)
    }

    pub async fn get_progress(&self, job_id: i64) -> Result<AnalysisProgress> {
        let row = sqlx::query!(
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
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch analysis progress")?;

        Ok(AnalysisProgress {
            job_id: row.job_id.to_string(),
            url: row.url,
            job_status: row.job_status,
            result_id: row.result_id,
            progress: row.progress,
            analyzed_pages: row.analyzed_pages,
            total_pages: row.total_pages,
        })
    }

    pub async fn get_all(&self) -> Result<Vec<AnalysisProgress>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                aj.id as "job_id!",
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
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch analysis jobs")?;

        Ok(rows
            .into_iter()
            .map(|row| AnalysisProgress {
                job_id: row.job_id.to_string(),
                url: row.url,
                job_status: row.job_status,
                result_id: row.result_id,
                progress: row.progress,
                analyzed_pages: row.analyzed_pages,
                total_pages: row.total_pages,
            })
            .collect())
    }

    pub async fn link_to_result(&self, job_id: i64, result_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE analysis_jobs 
            SET result_id = ?
            WHERE id = ?
            "#,
            result_id,
            job_id
        )
        .execute(&self.pool)
        .await
        .context("Failed to link job to result")?;

        log::info!("Linked job {} to result {}", job_id, result_id);
        Ok(())
    }
}

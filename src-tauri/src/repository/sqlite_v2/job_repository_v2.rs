//! Job repository for the redesigned schema.
//!
//! The `jobs` table consolidates:
//! - Job metadata (id, url, status, timestamps)
//! - Settings (max_pages, max_depth, rate_limit, etc.)
//! - Summary stats (total_pages, total_issues, etc.)
//!
//! Note: Uses runtime SQL (not compile-time checked) to work before migration.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::domain::models_v2::{Job, JobInfo, JobSettings, JobStatus, JobSummary};
use super::map_job_status_v2;

pub struct JobRepositoryV2 {
    pool: SqlitePool,
}

impl JobRepositoryV2 {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new job with settings.
    /// Returns the job ID (UUID string).
    pub async fn create(&self, url: &str, settings: &JobSettings) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO jobs (
                id, url, status, created_at, updated_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent
            )
            VALUES (?, ?, 'pending', ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(url)
        .bind(&now)
        .bind(&now)
        .bind(settings.max_pages)
        .bind(settings.max_depth)
        .bind(settings.respect_robots_txt)
        .bind(settings.include_subdomains)
        .bind(settings.rate_limit_ms)
        .bind(&settings.user_agent)
        .execute(&self.pool)
        .await
        .context("Failed to create job")?;

        log::info!("Created job {} for URL: {}", id, url);
        Ok(id)
    }

    /// Get a job by ID with full details.
    pub async fn get_by_id(&self, job_id: &str) -> Result<Job> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, url, status, created_at, updated_at, completed_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent,
                total_pages, pages_crawled, total_issues, 
                critical_issues, warning_issues, info_issues,
                progress, current_stage, error_message
            FROM jobs
            WHERE id = ?
            "#,
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch job")?;

        Ok(row_to_job(&row))
    }

    /// Get all jobs (lightweight info for listing).
    pub async fn get_all(&self) -> Result<Vec<JobInfo>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, url, status, progress, 
                total_pages, total_issues, created_at
            FROM jobs
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch jobs")?;

        Ok(rows.into_iter().map(|row| row_to_job_info(&row)).collect())
    }

    /// Get pending/running jobs (for job processor).
    pub async fn get_pending(&self) -> Result<Vec<Job>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id, url, status, created_at, updated_at, completed_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent,
                total_pages, pages_crawled, total_issues, 
                critical_issues, warning_issues, info_issues,
                progress, current_stage, error_message
            FROM jobs
            WHERE status IN ('pending', 'running')
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pending jobs")?;

        Ok(rows.into_iter().map(|row| row_to_job(&row)).collect())
    }

    /// Update job status.
    pub async fn update_status(&self, job_id: &str, status: JobStatus) -> Result<()> {
        let completed_at = if status.is_terminal() {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE jobs 
            SET status = ?, completed_at = COALESCE(?, completed_at)
            WHERE id = ?
            "#,
        )
        .bind(status.as_str())
        .bind(completed_at)
        .bind(job_id)
        .execute(&self.pool)
        .await
        .context("Failed to update job status")?;

        log::info!("Updated job {} to status: {}", job_id, status);
        Ok(())
    }

    /// Update job progress.
    pub async fn update_progress(
        &self,
        job_id: &str,
        progress: f64,
        current_stage: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE jobs 
            SET progress = ?, current_stage = ?
            WHERE id = ?
            "#,
        )
        .bind(progress)
        .bind(current_stage)
        .bind(job_id)
        .execute(&self.pool)
        .await
        .context("Failed to update job progress")?;

        Ok(())
    }

    /// Update job with error.
    pub async fn set_error(&self, job_id: &str, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE jobs 
            SET status = 'failed', error_message = ?, completed_at = ?
            WHERE id = ?
            "#,
        )
        .bind(error)
        .bind(Utc::now().to_rfc3339())
        .bind(job_id)
        .execute(&self.pool)
        .await
        .context("Failed to set job error")?;

        log::error!("Job {} failed: {}", job_id, error);
        Ok(())
    }

    /// Delete a job and all related data (CASCADE).
    pub async fn delete(&self, job_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM jobs WHERE id = ?")
            .bind(job_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete job")?;

        log::info!("Deleted job {}", job_id);
        Ok(())
    }
}

// Helper functions to convert rows to domain types
fn row_to_job(row: &sqlx::sqlite::SqliteRow) -> Job {
    Job {
        id: row.get("id"),
        url: row.get("url"),
        status: map_job_status_v2(row.get::<&str, _>("status")),
        created_at: parse_datetime(row.get("created_at")),
        updated_at: parse_datetime(row.get("updated_at")),
        completed_at: row.get::<Option<&str>, _>("completed_at").map(parse_datetime),
        settings: JobSettings {
            max_pages: row.get("max_pages"),
            max_depth: row.get("max_depth"),
            respect_robots_txt: row.get::<i64, _>("respect_robots_txt") != 0,
            include_subdomains: row.get::<i64, _>("include_subdomains") != 0,
            rate_limit_ms: row.get("rate_limit_ms"),
            user_agent: row.get("user_agent"),
        },
        summary: JobSummary {
            total_pages: row.get("total_pages"),
            pages_crawled: row.get("pages_crawled"),
            total_issues: row.get("total_issues"),
            critical_issues: row.get("critical_issues"),
            warning_issues: row.get("warning_issues"),
            info_issues: row.get("info_issues"),
        },
        progress: row.get("progress"),
        current_stage: row.get("current_stage"),
        error_message: row.get("error_message"),
    }
}

fn row_to_job_info(row: &sqlx::sqlite::SqliteRow) -> JobInfo {
    JobInfo {
        id: row.get("id"),
        url: row.get("url"),
        status: map_job_status_v2(row.get::<&str, _>("status")),
        progress: row.get("progress"),
        total_pages: row.get("total_pages"),
        total_issues: row.get("total_issues"),
        created_at: parse_datetime(row.get("created_at")),
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

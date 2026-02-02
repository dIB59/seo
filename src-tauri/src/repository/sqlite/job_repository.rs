//! Job repository for the redesigned schema.
//!
//! The `jobs` table consolidates:
//! - Job metadata (id, url, status, timestamps)
//! - Settings (max_pages, max_depth, rate_limit, etc.)
//! - Summary stats (total_pages, total_issues, etc.)
//!
//! Uses compile-time checked queries with sqlx::query!() macro.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;

use crate::domain::models::{Job, JobInfo, JobSettings, JobStatus, JobSummary};
use super::map_job_status;

pub struct JobRepository {
    pool: SqlitePool,
}

impl JobRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new job with settings.
    /// Returns the job ID (UUID string).
    pub async fn create(&self, url: &str, settings: &JobSettings) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let respect_robots = if settings.respect_robots_txt { 1i32 } else { 0 };
        let include_subs = if settings.include_subdomains { 1i32 } else { 0 };

        sqlx::query!(
            r#"
            INSERT INTO jobs (
                id, url, status, created_at, updated_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent
            )
            VALUES (?1, ?2, 'pending', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            id,
            url,
            now,
            now,
            settings.max_pages,
            settings.max_depth,
            respect_robots,
            include_subs,
            settings.rate_limit_ms,
            settings.user_agent,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create job")?;

        log::info!("Created job {} for URL: {}", id, url);
        Ok(id)
    }

    /// Get a job by ID with full details.
    pub async fn get_by_id(&self, job_id: &str) -> Result<Job> {
        let row = sqlx::query!(
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
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to fetch job")?;

        Ok(Job {
            id: row.id,
            url: row.url,
            status: map_job_status(&row.status),
            created_at: parse_datetime(&row.created_at),
            updated_at: parse_datetime(&row.updated_at),
            completed_at: row.completed_at.as_deref().map(parse_datetime),
            settings: JobSettings {
                max_pages: row.max_pages,
                max_depth: row.max_depth,
                respect_robots_txt: row.respect_robots_txt != 0,
                include_subdomains: row.include_subdomains != 0,
                rate_limit_ms: row.rate_limit_ms,
                user_agent: row.user_agent,
            },
            summary: JobSummary {
                total_pages: row.total_pages,
                pages_crawled: row.pages_crawled,
                total_issues: row.total_issues,
                critical_issues: row.critical_issues,
                warning_issues: row.warning_issues,
                info_issues: row.info_issues,
            },
            progress: row.progress,
            current_stage: row.current_stage,
            error_message: row.error_message,
        })
    }

    /// Get all jobs (lightweight info for listing).
    pub async fn get_all(&self) -> Result<Vec<JobInfo>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, url, status, progress, 
                total_pages, total_issues, created_at
            FROM jobs
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch jobs")?;

        Ok(rows
            .into_iter()
            .map(|row| JobInfo {
                id: row.id,
                url: row.url,
                status: map_job_status(&row.status),
                progress: row.progress,
                total_pages: row.total_pages,
                total_issues: row.total_issues,
                created_at: parse_datetime(&row.created_at),
            })
            .collect())
    }

    /// Get pending/running jobs (for job processor).
    pub async fn get_pending(&self) -> Result<Vec<Job>> {
        let rows = sqlx::query!(
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
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch pending jobs")?;

        Ok(rows
            .into_iter()
            .map(|row| Job {
                id: row.id,
                url: row.url,
                status: map_job_status(&row.status),
                created_at: parse_datetime(&row.created_at),
                updated_at: parse_datetime(&row.updated_at),
                completed_at: row.completed_at.as_deref().map(parse_datetime),
                settings: JobSettings {
                    max_pages: row.max_pages,
                    max_depth: row.max_depth,
                    respect_robots_txt: row.respect_robots_txt != 0,
                    include_subdomains: row.include_subdomains != 0,
                    rate_limit_ms: row.rate_limit_ms,
                    user_agent: row.user_agent,
                },
                summary: JobSummary {
                    total_pages: row.total_pages,
                    pages_crawled: row.pages_crawled,
                    total_issues: row.total_issues,
                    critical_issues: row.critical_issues,
                    warning_issues: row.warning_issues,
                    info_issues: row.info_issues,
                },
                progress: row.progress,
                current_stage: row.current_stage,
                error_message: row.error_message,
            })
            .collect())
    }

    /// Update job status.
    pub async fn update_status(&self, job_id: &str, status: JobStatus) -> Result<()> {
        let status_str = status.as_str();
        let completed_at = if status.is_terminal() {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        sqlx::query!(
            r#"
            UPDATE jobs 
            SET status = ?1, completed_at = COALESCE(?2, completed_at)
            WHERE id = ?3
            "#,
            status_str,
            completed_at,
            job_id,
        )
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
        sqlx::query!(
            r#"
            UPDATE jobs 
            SET progress = ?1, current_stage = ?2
            WHERE id = ?3
            "#,
            progress,
            current_stage,
            job_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to update job progress")?;

        Ok(())
    }

    /// Update job with error.
    pub async fn set_error(&self, job_id: &str, error: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        
        sqlx::query!(
            r#"
            UPDATE jobs 
            SET status = 'failed', error_message = ?1, completed_at = ?2
            WHERE id = ?3
            "#,
            error,
            now,
            job_id,
        )
        .execute(&self.pool)
        .await
        .context("Failed to set job error")?;

        log::error!("Job {} failed: {}", job_id, error);
        Ok(())
    }

    /// Delete a job and all related data (CASCADE).
    pub async fn delete(&self, job_id: &str) -> Result<()> {
        sqlx::query!("DELETE FROM jobs WHERE id = ?", job_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete job")?;

        log::info!("Deleted job {}", job_id);
        Ok(())
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

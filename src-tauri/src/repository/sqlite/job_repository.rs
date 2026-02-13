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

use super::map_job_status;
use crate::domain::models::{Job, JobInfo, JobSettings, JobStatus, JobSummary};

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
        let respect_robots = 1i32;
        let include_subs = 0i32;
        let max_depth = 3i64;
        let user_agent: Option<String> = None;

        let lighthouse_analysis = if settings.lighthouse_analysis {
            1i32
        } else {
            0
        };

        sqlx::query!(
            r#"
            INSERT INTO jobs (
                id, url, status, created_at, updated_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent, lighthouse_analysis
            )
            VALUES (?1, ?2, 'pending', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            id,
            url,
            now,
            now,
            settings.max_pages,
            max_depth,
            respect_robots,
            include_subs,
            settings.delay_between_requests,
            user_agent,
            lighthouse_analysis,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create job")?;

        tracing::info!("Created job {} for URL: {}", id, url);
        Ok(id)
    }

    /// Get a job by ID with full details.
    pub async fn get_by_id(&self, job_id: &str) -> Result<Job> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, url, status, created_at, updated_at, completed_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent, lighthouse_analysis,
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
                include_external_links: false,
                check_images: true,
                mobile_analysis: false,
                lighthouse_analysis: row.lighthouse_analysis != 0,
                delay_between_requests: row.rate_limit_ms,
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

    /// Get paginated jobs for history listing.
    pub async fn get_paginated(&self, limit: i64, offset: i64) -> Result<Vec<JobInfo>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, url, status, progress, 
                total_pages, total_issues, created_at
            FROM jobs
            ORDER BY created_at DESC
            LIMIT ?1 OFFSET ?2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch paginated jobs")?;

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

    /// Optimized count and fetch in one query using window functions
    pub async fn get_paginated_with_total(
        &self,
        limit: i64,
        offset: i64,
        url_filter: Option<String>,
        status_filter: Option<String>,
    ) -> Result<(Vec<JobInfo>, i64)> {
        let url_pattern = url_filter
            .map(|f| format!("%{}%", f))
            .unwrap_or_else(|| "%".to_string());
        let status_pattern = status_filter.unwrap_or_else(|| "%".to_string());

        let rows = sqlx::query!(
            r#"
            SELECT 
                id, url, status, progress, 
                total_pages, total_issues, created_at,
                COUNT(*) OVER() as "total_count!"
            FROM jobs
            WHERE url LIKE ?1 AND status LIKE ?2
            ORDER BY created_at DESC
            LIMIT ?3 OFFSET ?4
            "#,
            url_pattern,
            status_pattern,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch paginated jobs with total and filters")?;

        let total = rows.first().map(|r| r.total_count).unwrap_or(0);

        let items = rows
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
            .collect();

        Ok((items, total))
    }

    pub async fn count(&self) -> Result<i64> {
        let row = sqlx::query!("SELECT COUNT(*) as count FROM jobs")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count jobs")?;

        Ok(row.count as i64)
    }

    /// Get pending/running jobs (for job processor).
    pub async fn get_pending(&self) -> Result<Vec<Job>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, url, status, created_at, updated_at, completed_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent, lighthouse_analysis,
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
                    include_external_links: false,
                    check_images: true,
                    mobile_analysis: false,
                    lighthouse_analysis: row.lighthouse_analysis != 0,
                    delay_between_requests: row.rate_limit_ms,
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

        tracing::info!("Updated job {} to status: {}", job_id, status);
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

        Ok(())
    }

    /// Delete a job and all related data (CASCADE).
    pub async fn delete(&self, job_id: &str) -> Result<()> {
        sqlx::query!("DELETE FROM jobs WHERE id = ?", job_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete job")?;

        tracing::info!("Deleted job {}", job_id);
        Ok(())
    }

    async fn get_running_jobs_id(&self) -> Result<Vec<String>> {
        let rows = sqlx::query!(
            r#"
            SELECT id FROM jobs WHERE status = 'running'
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch running jobs")?;
        Ok(rows.into_iter().map(|row| row.id).collect())
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

// Implement the abstract repository trait using the concrete sqlite repository
use crate::repository::JobRepository as JobRepositoryTrait;
use async_trait::async_trait;

#[async_trait]
impl JobRepositoryTrait for JobRepository {
    async fn create(&self, url: &str, settings: &JobSettings) -> Result<String> {
        // Call the inherent method
        JobRepository::create(self, url, settings).await
    }

    async fn get_by_id(&self, id: &str) -> Result<Job> {
        JobRepository::get_by_id(self, id).await
    }

    async fn get_all(&self) -> Result<Vec<JobInfo>> {
        JobRepository::get_all(self).await
    }

    async fn get_paginated(&self, limit: i64, offset: i64) -> Result<Vec<JobInfo>> {
        JobRepository::get_paginated(self, limit, offset).await
    }

    async fn get_paginated_with_total(
        &self,
        limit: i64,
        offset: i64,
        url_filter: Option<String>,
        status_filter: Option<String>,
    ) -> Result<(Vec<JobInfo>, i64)> {
        JobRepository::get_paginated_with_total(self, limit, offset, url_filter, status_filter)
            .await
    }

    async fn get_pending(&self) -> Result<Vec<Job>> {
        JobRepository::get_pending(self).await
    }

    async fn get_running_jobs_id(&self) -> Result<Vec<String>> {
        JobRepository::get_running_jobs_id(self).await
    }

    async fn update_status(&self, job_id: &str, status: JobStatus) -> Result<()> {
        JobRepository::update_status(self, job_id, status).await
    }

    async fn update_progress(
        &self,
        id: &str,
        progress: f64,
        current_stage: Option<&str>,
    ) -> Result<()> {
        JobRepository::update_progress(self, id, progress, current_stage).await
    }

    async fn set_error(&self, job_id: &str, error: &str) -> Result<()> {
        JobRepository::set_error(self, job_id, error).await
    }

    async fn count(&self) -> Result<i64> {
        JobRepository::count(self).await
    }

    async fn delete(&self, job_id: &str) -> Result<()> {
        JobRepository::delete(self, job_id).await
    }
}

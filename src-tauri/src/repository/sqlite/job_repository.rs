use chrono::Utc;
use sqlx::SqlitePool;

use super::map_job_status;
use crate::contexts::{Job, JobId, JobInfo, JobPageQuery, JobSettings, JobStatus, JobSummary};
use crate::repository::JobRepository as JobRepositoryTrait;
use async_trait::async_trait;

pub struct JobRepository {
    pool: SqlitePool,
}

impl JobRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobRepositoryTrait for JobRepository {
    async fn create(
        &self,
        url: &str,
        settings: &JobSettings,
    ) -> crate::repository::RepositoryResult<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let lighthouse_analysis = i32::from(settings.lighthouse_analysis);

        sqlx::query!(
            r#"
            INSERT INTO jobs (
                id, url, status, created_at, updated_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent, lighthouse_analysis,
                sitemap_found, robots_txt_found
            )
            VALUES (?1, ?2, 'pending', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, 0)
            "#,
            id,
            url,
            now,
            now,
            settings.max_pages,
            5, // max_depth
            1, // respect_robots_txt
            settings.include_subdomains,
            settings.delay_between_requests,
            "SEO-Insikt-Crawler/0.1", // user_agent
            lighthouse_analysis,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Created job {} for URL: {}", id, url);
        Ok(id)
    }

    async fn get_by_id(&self, job_id: &str) -> crate::repository::RepositoryResult<Job> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, url, status, created_at, updated_at, completed_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent, lighthouse_analysis,
                total_pages, pages_crawled, total_issues, 
                critical_issues, warning_issues, info_issues,
                progress, error_message, sitemap_found, robots_txt_found
            FROM jobs
            WHERE id = ?
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(super::job_from_row!(row))
    }

    async fn get_all(&self) -> crate::repository::RepositoryResult<Vec<JobInfo>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, url, status, progress, 
                total_pages, total_issues, created_at,
                max_pages, lighthouse_analysis
            FROM jobs
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                make_job_info(
                    row.id,
                    row.url,
                    &row.status,
                    row.progress,
                    row.total_pages,
                    row.total_issues,
                    &row.created_at,
                    row.max_pages,
                    row.lighthouse_analysis,
                )
            })
            .collect())
    }

    async fn get_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> crate::repository::RepositoryResult<Vec<JobInfo>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, url, status, progress, 
                total_pages, total_issues, created_at,
                max_pages, lighthouse_analysis
            FROM jobs
            ORDER BY created_at DESC
            LIMIT ?1 OFFSET ?2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                make_job_info(
                    row.id,
                    row.url,
                    &row.status,
                    row.progress,
                    row.total_pages,
                    row.total_issues,
                    &row.created_at,
                    row.max_pages,
                    row.lighthouse_analysis,
                )
            })
            .collect())
    }

    async fn get_paginated_with_total(
        &self,
        query: JobPageQuery,
    ) -> crate::repository::RepositoryResult<(Vec<JobInfo>, i64)> {
        let (pagination, url_contains, status) = query.into_parts();
        let url_pattern = url_contains
            .map_or_else(|| "%".to_string(), |f| format!("%{}%", f));
        let status_pattern = status.unwrap_or_else(|| "%".to_string());
        let limit = pagination.limit();
        let offset = pagination.offset();

        let rows = sqlx::query!(
            r#"
            SELECT
                id, url, status, progress,
                total_pages, total_issues, created_at,
                max_pages, lighthouse_analysis,
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
        .await?;

        let total = rows.first().map(|r| r.total_count).unwrap_or(0);

        let items = rows
            .into_iter()
            .map(|row| {
                make_job_info(
                    row.id,
                    row.url,
                    &row.status,
                    row.progress,
                    row.total_pages,
                    row.total_issues,
                    &row.created_at,
                    row.max_pages,
                    row.lighthouse_analysis,
                )
            })
            .collect();

        Ok((items, total))
    }

    async fn count(&self) -> crate::repository::RepositoryResult<i64> {
        let row = sqlx::query!("SELECT COUNT(*) as count FROM jobs")
            .fetch_one(&self.pool)
            .await?; // sqlx::Error → RepositoryError via #[from]
        Ok(row.count as i64)
    }

    async fn get_pending(&self) -> crate::repository::RepositoryResult<Vec<Job>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, url, status, created_at, updated_at, completed_at,
                max_pages, max_depth, respect_robots_txt, include_subdomains, 
                rate_limit_ms, user_agent, lighthouse_analysis,
                total_pages, pages_crawled, total_issues, 
                critical_issues, warning_issues, info_issues,
                progress, error_message, sitemap_found, robots_txt_found
            FROM jobs
            WHERE status IN ('pending', 'discovery', 'processing')
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| super::job_from_row!(row)).collect())
    }

    async fn update_resources(
        &self,
        job_id: &str,
        sitemap_found: bool,
        robots_txt_found: bool,
    ) -> crate::repository::RepositoryResult<()> {
        let sitemap = i32::from(sitemap_found);
        let robots = i32::from(robots_txt_found);

        sqlx::query!(
            r#"
            UPDATE jobs
            SET sitemap_found = ?1, robots_txt_found = ?2
            WHERE id = ?3
            "#,
            sitemap,
            robots,
            job_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_status(
        &self,
        job_id: &str,
        status: JobStatus,
    ) -> crate::repository::RepositoryResult<()> {
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
        .await?;

        tracing::info!("Updated job {} to status: {}", job_id, status);
        Ok(())
    }

    async fn update_progress(
        &self,
        job_id: &str,
        progress: f64,
    ) -> crate::repository::RepositoryResult<()> {
        sqlx::query!(
            r#"
            UPDATE jobs
            SET progress = ?1
            WHERE id = ?2
            "#,
            progress,
            job_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn set_error(
        &self,
        job_id: &str,
        error: &str,
    ) -> crate::repository::RepositoryResult<()> {
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
        .await?;

        Ok(())
    }

    async fn delete(&self, job_id: &str) -> crate::repository::RepositoryResult<()> {
        sqlx::query!("DELETE FROM jobs WHERE id = ?", job_id)
            .execute(&self.pool)
            .await?; // sqlx::Error → RepositoryError via #[from]

        tracing::info!("Deleted job {}", job_id);
        Ok(())
    }

    async fn get_running_jobs_id(&self) -> crate::repository::RepositoryResult<Vec<String>> {
        let rows = sqlx::query!(
            r#"
            SELECT id FROM jobs WHERE status IN ('discovery', 'processing')
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|row| row.id).collect())
    }
}

use super::parse_datetime;

#[allow(clippy::too_many_arguments)]
fn make_job_info(
    id: String,
    url: String,
    status: &str,
    progress: f64,
    total_pages: i64,
    total_issues: i64,
    created_at: &str,
    max_pages: i64,
    lighthouse_analysis: i64,
) -> JobInfo {
    JobInfo::new(
        JobId::from(id),
        url,
        map_job_status(status),
        progress,
        total_pages,
        total_issues,
        parse_datetime(created_at),
        max_pages,
        lighthouse_analysis != 0,
    )
}

/// Build a [`JobSummary`] from the row's six count columns. Centralized
/// so every Job decoder argues with the same constructor — adding a new
/// counter (e.g. `noindex_issues`) becomes a one-place change.
pub(super) fn decode_job_summary(
    total_pages: i64,
    pages_crawled: i64,
    total_issues: i64,
    critical_issues: i64,
    warning_issues: i64,
    info_issues: i64,
) -> JobSummary {
    JobSummary::new(
        total_pages,
        pages_crawled,
        total_issues,
        critical_issues,
        warning_issues,
        info_issues,
    )
}

/// Build a [`JobSettings`] from the row's flat columns. Centralized so the
/// `get_by_id`, `get_pending`, and `results_repository::get_complete_result`
/// decoders agree on which booleans are hard-coded vs sourced from the row.
#[allow(clippy::too_many_arguments)]
pub(super) fn decode_job_settings(
    max_pages: i64,
    include_subdomains: i64,
    lighthouse_analysis: i64,
    rate_limit_ms: i64,
) -> JobSettings {
    JobSettings {
        max_pages,
        include_subdomains: include_subdomains != 0,
        check_images: true,
        mobile_analysis: false,
        lighthouse_analysis: lighthouse_analysis != 0,
        delay_between_requests: rate_limit_ms,
    }
}


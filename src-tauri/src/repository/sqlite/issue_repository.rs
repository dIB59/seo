//! Issue repository for the redesigned schema.
//!
//! Issues have a direct `job_id` foreign key, eliminating expensive JOINs
//! through analysis_results → page_analysis.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;

use super::map_severity;
use crate::domain::models::{Issue, IssueSeverity, NewIssue};

pub struct IssueRepository {
    pool: SqlitePool,
}

impl IssueRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Insert multiple issues in a batch (FAST: single transaction).
    pub async fn insert_batch(&self, issues: &[NewIssue]) -> Result<()> {
        if issues.is_empty() {
            return Ok(());
        }

        const CHUNK_SIZE: usize = 100;
        let mut tx = self.pool.begin().await?;

        for chunk in issues.chunks(CHUNK_SIZE) {
            let mut qb = sqlx::QueryBuilder::new(
                r#"
                INSERT INTO issues (
                    job_id, page_id, type, severity, message, details, created_at
                ) "#,
            );

            qb.push_values(chunk, |mut b, issue| {
                b.push_bind(&issue.job_id)
                    .push_bind(&issue.page_id)
                    .push_bind(&issue.issue_type)
                    .push_bind(issue.severity.as_str())
                    .push_bind(&issue.message)
                    .push_bind(&issue.details)
                    .push_bind(Utc::now().to_rfc3339());
            });

            qb.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;
        log::debug!("Inserted {} issues", issues.len());
        Ok(())
    }

    /// Get all issues for a job (FAST: direct FK lookup, no JOINs!).
    pub async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<Issue>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, job_id, page_id, type as issue_type, severity, message, details, created_at
            FROM issues
            WHERE job_id = ?
            ORDER BY 
                CASE severity 
                    WHEN 'critical' THEN 1 
                    WHEN 'warning' THEN 2 
                    ELSE 3 
                END,
                type ASC
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch issues for job")?;

        Ok(rows
            .into_iter()
            .map(|row| Issue {
                id: row.id.expect("Must exist"),
                job_id: row.job_id,
                page_id: row.page_id,
                issue_type: row.issue_type,
                severity: map_severity(row.severity.as_str()),
                message: row.message,
                details: row.details,
                created_at: parse_datetime(row.created_at.as_str()),
            })
            .collect())
    }

    /// Get issues for a specific page.
    pub async fn get_by_page_id(&self, page_id: &str) -> Result<Vec<Issue>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, job_id, page_id, type as issue_type, severity, message, details, created_at
            FROM issues
            WHERE page_id = ?
            ORDER BY 
                CASE severity 
                    WHEN 'critical' THEN 1 
                    WHEN 'warning' THEN 2 
                    ELSE 3 
                END
            "#,
            page_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch issues for page")?;

        Ok(rows
            .into_iter()
            .map(|row| Issue {
                id: row.id.expect("Must exist"),
                job_id: row.job_id,
                page_id: row.page_id,
                issue_type: row.issue_type,
                severity: map_severity(row.severity.as_str()),
                message: row.message,
                details: row.details,
                created_at: parse_datetime(row.created_at.as_str()),
            })
            .collect())
    }

    /// Get issues by severity for a job.
    pub async fn get_by_job_and_severity(
        &self,
        job_id: &str,
        severity: IssueSeverity,
    ) -> Result<Vec<Issue>> {
        let severity_str = severity.as_str();
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, job_id, page_id, type as issue_type, severity, message, details, created_at
            FROM issues
            WHERE job_id = ? AND severity = ?
            ORDER BY type ASC
            "#,
            job_id,
            severity_str
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch issues by severity")?;

        Ok(rows
            .into_iter()
            .map(|row| Issue {
                id: row.id.expect("Must exist"),
                job_id: row.job_id,
                page_id: row.page_id,
                issue_type: row.issue_type,
                severity: map_severity(row.severity.as_str()),
                message: row.message,
                details: row.details,
                created_at: parse_datetime(row.created_at.as_str()),
            })
            .collect())
    }

    /// Get issue counts by severity for a job (FAST: uses index).
    pub async fn count_by_severity(&self, job_id: &str) -> Result<IssueCounts> {
        let row = sqlx::query!(
            r#"
            SELECT 
                SUM(CASE WHEN severity = 'critical' THEN 1 ELSE 0 END) as critical,
                SUM(CASE WHEN severity = 'warning' THEN 1 ELSE 0 END) as warning,
                SUM(CASE WHEN severity = 'info' THEN 1 ELSE 0 END) as info
            FROM issues
            WHERE job_id = ?
            "#,
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to count issues")?;

        Ok(IssueCounts {
            critical: row.critical.unwrap_or(0) as i64,
            warning: row.warning.unwrap_or(0) as i64,
            info: row.info.unwrap_or(0) as i64,
        })
    }

    /// Get total issue count for a job.
    pub async fn count_by_job_id(&self, job_id: &str) -> Result<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM issues WHERE job_id = ?",
            job_id
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to count issues")?;

        Ok(row.count as i64)
    }

    /// Get grouped issues by type for dashboard display.
    pub async fn get_grouped_by_type(&self, job_id: &str) -> Result<Vec<IssueGroup>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                type as issue_type, 
                severity,
                COUNT(*) as count,
                GROUP_CONCAT(DISTINCT message) as messages
            FROM issues
            WHERE job_id = ?
            GROUP BY type, severity
            ORDER BY 
                CASE severity 
                    WHEN 'critical' THEN 1 
                    WHEN 'warning' THEN 2 
                    ELSE 3 
                END,
                count DESC
            "#,
            job_id
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to get grouped issues")?;

        Ok(rows
            .into_iter()
            .map(|row| IssueGroup {
                issue_type: row.issue_type,
                severity: map_severity(row.severity.as_str()),
                count: row.count,
                sample_messages: row
                    .messages
                    .map(|m| m.split(',').take(3).map(String::from).collect())
                    .unwrap_or_default(),
            })
            .collect())
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// Issue counts by severity.
#[derive(Debug, Clone, Default)]
pub struct IssueCounts {
    pub critical: i64,
    pub warning: i64,
    pub info: i64,
}

impl IssueCounts {
    pub fn total(&self) -> i64 {
        self.critical + self.warning + self.info
    }
}

/// Grouped issue summary.
#[derive(Debug, Clone)]
pub struct IssueGroup {
    pub issue_type: String,
    pub severity: IssueSeverity,
    pub count: i64,
    pub sample_messages: Vec<String>,
}

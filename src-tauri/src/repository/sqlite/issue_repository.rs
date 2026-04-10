use chrono::Utc;
use sqlx::SqlitePool;

use super::map_severity;
use crate::contexts::analysis::{Issue, IssueSeverity, NewIssue};
use crate::repository::{IssueRepository as IssueRepositoryTrait, RepositoryResult};
use async_trait::async_trait;

/// Project a sqlx anonymous issue row through [`make_issue`]. Three call
/// sites in this module (`get_by_job_id`, `get_by_page_id`,
/// `get_by_job_and_severity`) all built the same 8-positional projection
/// — same rationale as `job_from_row!` / `page_from_row!`.
macro_rules! issue_from_row {
    ($row:expr) => {{
        let row = $row;
        $crate::repository::sqlite::issue_repository::make_issue(
            row.id,
            row.job_id,
            row.page_id,
            row.issue_type,
            row.severity.as_str(),
            row.message,
            row.details,
            row.created_at.as_str(),
        )
    }};
}
#[allow(clippy::too_many_arguments)]
pub(super) fn make_issue(
    id: i64,
    job_id: String,
    page_id: Option<String>,
    issue_type: String,
    severity: &str,
    message: String,
    details: Option<String>,
    created_at: &str,
) -> Issue {
    Issue {
        id,
        job_id,
        page_id,
        issue_type,
        severity: map_severity(severity),
        message,
        details,
        created_at: parse_datetime(created_at),
    }
}

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

#[derive(Debug, Clone)]
pub struct IssueGroup {
    pub issue_type: String,
    pub severity: IssueSeverity,
    pub count: i64,
    pub sample_messages: Vec<String>,
}

pub struct IssueRepository {
    pool: SqlitePool,
}

impl IssueRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IssueRepositoryTrait for IssueRepository {
    async fn insert_batch(&self, issues: &[NewIssue]) -> RepositoryResult<()> {
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
        tracing::debug!("Inserted {} issues", issues.len());
        Ok(())
    }

    async fn get_by_job_id(&self, job_id: &str) -> RepositoryResult<Vec<Issue>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id as "id!", job_id, page_id, type as issue_type, severity, message, details, created_at
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
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| issue_from_row!(row))
            .collect())
    }

    async fn get_by_page_id(&self, page_id: &str) -> RepositoryResult<Vec<Issue>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id as "id!", job_id, page_id, type as issue_type, severity, message, details, created_at
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
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| issue_from_row!(row))
            .collect())
    }

    async fn get_by_job_and_severity(
        &self,
        job_id: &str,
        severity: IssueSeverity,
    ) -> RepositoryResult<Vec<Issue>> {
        let severity_str = severity.as_str();
        let rows = sqlx::query!(
            r#"
            SELECT 
                id as "id!", job_id, page_id, type as issue_type, severity, message, details, created_at
            FROM issues
            WHERE job_id = ? AND severity = ?
            ORDER BY type ASC
            "#,
            job_id,
            severity_str
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| issue_from_row!(row))
            .collect())
    }

    async fn count_by_severity(&self, job_id: &str) -> RepositoryResult<IssueCounts> {
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
        .await?;

        Ok(IssueCounts {
            critical: row.critical.unwrap_or(0) as i64,
            warning: row.warning.unwrap_or(0) as i64,
            info: row.info.unwrap_or(0) as i64,
        })
    }

    async fn count_by_job_id(&self, job_id: &str) -> RepositoryResult<i64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM issues WHERE job_id = ?",
            job_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count as i64)
    }

    async fn get_grouped_by_type(&self, job_id: &str) -> RepositoryResult<Vec<IssueGroup>> {
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
        .await?;

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

use super::parse_datetime;


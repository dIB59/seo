use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::contexts::extension::Operator;
use crate::contexts::report::{
    BusinessImpact, FixEffort, PatternCategory, PatternSeverity, ReportPattern,
    ReportPatternParams,
};
use crate::repository::{RepositoryError, RepositoryResult, ReportPatternRepository};

pub struct SqliteReportPatternRepository {
    pool: SqlitePool,
}

impl SqliteReportPatternRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReportPatternRepository for SqliteReportPatternRepository {
    async fn list_patterns(&self) -> RepositoryResult<Vec<ReportPattern>> {
        let rows = sqlx::query(
            "SELECT id, name, description, category, severity, field, operator, threshold,
                    min_prevalence, business_impact, fix_effort, recommendation, is_builtin, enabled
             FROM report_patterns
             ORDER BY is_builtin DESC, name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_pattern).collect()
    }

    async fn list_enabled_patterns(&self) -> RepositoryResult<Vec<ReportPattern>> {
        let rows = sqlx::query(
            "SELECT id, name, description, category, severity, field, operator, threshold,
                    min_prevalence, business_impact, fix_effort, recommendation, is_builtin, enabled
             FROM report_patterns
             WHERE enabled = 1
             ORDER BY is_builtin DESC, name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_pattern).collect()
    }

    async fn get_pattern(&self, id: &str) -> RepositoryResult<ReportPattern> {
        let row = sqlx::query(
            "SELECT id, name, description, category, severity, field, operator, threshold,
                    min_prevalence, business_impact, fix_effort, recommendation, is_builtin, enabled
             FROM report_patterns WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::not_found("report_pattern", id),
            other => RepositoryError::from(other),
        })?;

        row_to_pattern(&row)
    }

    async fn create_pattern(
        &self,
        params: &ReportPatternParams,
    ) -> RepositoryResult<ReportPattern> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO report_patterns
                (id, name, description, category, severity, field, operator, threshold,
                 min_prevalence, business_impact, fix_effort, recommendation, is_builtin, enabled)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?)",
        )
        .bind(&id)
        .bind(&params.name)
        .bind(&params.description)
        .bind(params.category.as_str())
        .bind(params.severity.as_str())
        .bind(&params.field)
        .bind(params.operator.to_string())
        .bind(&params.threshold)
        .bind(params.min_prevalence)
        .bind(params.business_impact.as_str())
        .bind(params.fix_effort.as_str())
        .bind(&params.recommendation)
        .bind(params.enabled)
        .execute(&self.pool)
        .await?;

        self.get_pattern(&id).await
    }

    async fn update_pattern(
        &self,
        id: &str,
        params: &ReportPatternParams,
    ) -> RepositoryResult<ReportPattern> {
        sqlx::query(
            "UPDATE report_patterns SET
                name = ?, description = ?, category = ?, severity = ?, field = ?,
                operator = ?, threshold = ?, min_prevalence = ?, business_impact = ?,
                fix_effort = ?, recommendation = ?, enabled = ?,
                updated_at = datetime('now')
             WHERE id = ? AND is_builtin = 0",
        )
        .bind(&params.name)
        .bind(&params.description)
        .bind(params.category.as_str())
        .bind(params.severity.as_str())
        .bind(&params.field)
        .bind(params.operator.to_string())
        .bind(&params.threshold)
        .bind(params.min_prevalence)
        .bind(params.business_impact.as_str())
        .bind(params.fix_effort.as_str())
        .bind(&params.recommendation)
        .bind(params.enabled)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_pattern(id).await
    }

    async fn toggle_pattern(&self, id: &str, enabled: bool) -> RepositoryResult<()> {
        sqlx::query(
            "UPDATE report_patterns SET enabled = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(enabled)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_pattern(&self, id: &str) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM report_patterns WHERE id = ? AND is_builtin = 0")
            .bind(id)
            .execute(&self.pool)
            .await?;
        super::require_affected(result.rows_affected(), "report_pattern", id)
    }
}

fn row_to_pattern(row: &sqlx::sqlite::SqliteRow) -> RepositoryResult<ReportPattern> {
    use sqlx::Row;

    let category: String = row.try_get("category")?;
    let severity: String = row.try_get("severity")?;
    let operator: String = row.try_get("operator")?;
    let business_impact: String = row.try_get("business_impact")?;
    let fix_effort: String = row.try_get("fix_effort")?;
    let is_builtin: i64 = row.try_get("is_builtin")?;
    let enabled: i64 = row.try_get("enabled")?;

    // Generic over the typed Parse*Error structs in pattern.rs / extension.
    // Each carries Display via thiserror, so a single closure handles them all
    // — replaces the previous `anyhow::Error` plumbing now that every parser
    // returns a real typed error.
    let decode_err = |what: &str, e: &dyn std::fmt::Display| {
        RepositoryError::decode("report_pattern", format!("invalid {what}: {e}"))
    };

    Ok(ReportPattern {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        category: category
            .parse::<PatternCategory>()
            .map_err(|e| decode_err("category", &e))?,
        severity: severity
            .parse::<PatternSeverity>()
            .map_err(|e| decode_err("severity", &e))?,
        field: row.try_get("field")?,
        operator: operator
            .parse::<Operator>()
            .map_err(|e| decode_err("operator", &e))?,
        threshold: row.try_get("threshold")?,
        min_prevalence: row.try_get("min_prevalence")?,
        business_impact: business_impact
            .parse::<BusinessImpact>()
            .map_err(|e| decode_err("business_impact", &e))?,
        fix_effort: fix_effort
            .parse::<FixEffort>()
            .map_err(|e| decode_err("fix_effort", &e))?,
        recommendation: row.try_get("recommendation")?,
        is_builtin: is_builtin != 0,
        enabled: enabled != 0,
    })
}

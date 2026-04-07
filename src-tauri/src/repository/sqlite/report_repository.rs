use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::contexts::extension::Operator;
use crate::contexts::report::{
    BusinessImpact, FixEffort, PatternCategory, PatternSeverity, ReportPattern,
    ReportPatternParams,
};
use crate::repository::ReportPatternRepository;

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
    async fn list_patterns(&self) -> Result<Vec<ReportPattern>> {
        let rows = sqlx::query(
            "SELECT id, name, description, category, severity, field, operator, threshold,
                    min_prevalence, business_impact, fix_effort, recommendation, is_builtin, enabled
             FROM report_patterns
             ORDER BY is_builtin DESC, name ASC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list report patterns")?;

        rows.iter().map(row_to_pattern).collect()
    }

    async fn list_enabled_patterns(&self) -> Result<Vec<ReportPattern>> {
        let rows = sqlx::query(
            "SELECT id, name, description, category, severity, field, operator, threshold,
                    min_prevalence, business_impact, fix_effort, recommendation, is_builtin, enabled
             FROM report_patterns
             WHERE enabled = 1
             ORDER BY is_builtin DESC, name ASC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list enabled report patterns")?;

        rows.iter().map(row_to_pattern).collect()
    }

    async fn get_pattern(&self, id: &str) -> Result<ReportPattern> {
        let row = sqlx::query(
            "SELECT id, name, description, category, severity, field, operator, threshold,
                    min_prevalence, business_impact, fix_effort, recommendation, is_builtin, enabled
             FROM report_patterns WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Report pattern not found")?;

        row_to_pattern(&row)
    }

    async fn create_pattern(&self, params: &ReportPatternParams) -> Result<ReportPattern> {
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
        .await
        .context("Failed to create report pattern")?;

        self.get_pattern(&id).await
    }

    async fn update_pattern(&self, id: &str, params: &ReportPatternParams) -> Result<ReportPattern> {
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
        .await
        .context("Failed to update report pattern")?;

        self.get_pattern(id).await
    }

    async fn toggle_pattern(&self, id: &str, enabled: bool) -> Result<()> {
        sqlx::query(
            "UPDATE report_patterns SET enabled = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(enabled)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to toggle report pattern")?;
        Ok(())
    }

    async fn delete_pattern(&self, id: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM report_patterns WHERE id = ? AND is_builtin = 0",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to delete report pattern")?;

        if result.rows_affected() == 0 {
            anyhow::bail!("Pattern '{}' not found or is a built-in pattern (cannot be deleted)", id);
        }
        Ok(())
    }
}

fn row_to_pattern(row: &sqlx::sqlite::SqliteRow) -> Result<ReportPattern> {
    use sqlx::Row;

    let category: String = row.try_get("category")?;
    let severity: String = row.try_get("severity")?;
    let operator: String = row.try_get("operator")?;
    let business_impact: String = row.try_get("business_impact")?;
    let fix_effort: String = row.try_get("fix_effort")?;
    let is_builtin: i64 = row.try_get("is_builtin")?;
    let enabled: i64 = row.try_get("enabled")?;

    Ok(ReportPattern {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        category: category.parse::<PatternCategory>()?,
        severity: severity.parse::<PatternSeverity>()?,
        field: row.try_get("field")?,
        operator: operator.parse::<Operator>()?,
        threshold: row.try_get("threshold")?,
        min_prevalence: row.try_get("min_prevalence")?,
        business_impact: business_impact.parse::<BusinessImpact>()?,
        fix_effort: fix_effort.parse::<FixEffort>()?,
        recommendation: row.try_get("recommendation")?,
        is_builtin: is_builtin != 0,
        enabled: enabled != 0,
    })
}

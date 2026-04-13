//! SQLite implementation of `ReportTemplateRepository`.

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::contexts::report::{ReportTemplate, TemplateSection};
use crate::repository::{
    ReportTemplateRepository as ReportTemplateRepositoryTrait, RepositoryError, RepositoryResult,
};

pub struct ReportTemplateRepository {
    pool: SqlitePool,
}

impl ReportTemplateRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Ensure the default template row exists and its sections_json
    /// matches the current code-defined default. Called at every app
    /// startup so schema upgrades that change the default template
    /// (e.g. adding rich data variables) propagate automatically.
    ///
    /// Only overwrites if `is_builtin = 1` — user-customized templates
    /// are never touched.
    pub async fn ensure_default_sections(&self) -> RepositoryResult<()> {
        use crate::contexts::report::template::defaults::{default_template, DEFAULT_TEMPLATE_ID};

        let template = default_template();
        let expected_json = serde_json::to_string(&template.sections)
            .map_err(|e| RepositoryError::decode("report_template", e.to_string()))?;

        // Check if the row exists and if its content differs from the
        // code-defined default.
        let current: Option<String> = sqlx::query_scalar(
            "SELECT sections_json FROM report_templates WHERE id = ? AND is_builtin = 1",
        )
        .bind(DEFAULT_TEMPLATE_ID)
        .fetch_optional(&self.pool)
        .await?;

        match current {
            Some(ref json) if json == &expected_json => {
                // Already up to date — nothing to do.
            }
            Some(_) => {
                // Exists but stale — update to match the new code default.
                sqlx::query(
                    "UPDATE report_templates SET sections_json = ?, updated_at = datetime('now') WHERE id = ?",
                )
                .bind(&expected_json)
                .bind(DEFAULT_TEMPLATE_ID)
                .execute(&self.pool)
                .await?;
                tracing::info!("[INIT] Updated default report template sections to latest");
            }
            None => {
                // Row doesn't exist (fresh install) — insert it.
                sqlx::query(
                    "INSERT OR IGNORE INTO report_templates (id, name, is_builtin, sections_json, is_active)
                     VALUES (?, ?, 1, ?, 1)",
                )
                .bind(DEFAULT_TEMPLATE_ID)
                .bind(&template.name)
                .bind(&expected_json)
                .execute(&self.pool)
                .await?;
                tracing::info!("[INIT] Seeded default report template");
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ReportTemplateRepositoryTrait for ReportTemplateRepository {
    async fn list_templates(&self) -> RepositoryResult<Vec<ReportTemplate>> {
        let rows = sqlx::query_as::<_, TemplateRow>(
            "SELECT id, name, is_builtin, sections_json, selected_tags_json, is_active
             FROM report_templates ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TemplateRow::into_domain).collect()
    }

    async fn get_template(&self, id: &str) -> RepositoryResult<ReportTemplate> {
        let row = sqlx::query_as::<_, TemplateRow>(
            "SELECT id, name, is_builtin, sections_json, selected_tags_json, is_active
             FROM report_templates WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::not_found("report_template", id),
            other => RepositoryError::from(other),
        })?;

        row.into_domain()
    }

    async fn get_active_template(&self) -> RepositoryResult<Option<ReportTemplate>> {
        let row = sqlx::query_as::<_, TemplateRow>(
            "SELECT id, name, is_builtin, sections_json, selected_tags_json, is_active
             FROM report_templates WHERE is_active = 1 LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_domain()?)),
            None => Ok(None),
        }
    }

    async fn create_template(&self, template: &ReportTemplate) -> RepositoryResult<()> {
        let json = serde_json::to_string(&template.sections)
            .map_err(|e| RepositoryError::decode("report_template", e.to_string()))?;

        let tags_json = serde_json::to_string(&template.selected_tags)
            .map_err(|e| RepositoryError::decode("report_template", e.to_string()))?;

        sqlx::query(
            "INSERT INTO report_templates (id, name, is_builtin, sections_json, selected_tags_json, is_active)
             VALUES (?, ?, ?, ?, ?, 0)",
        )
        .bind(&template.id)
        .bind(&template.name)
        .bind(i64::from(template.is_builtin))
        .bind(&json)
        .bind(&tags_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_template(&self, template: &ReportTemplate) -> RepositoryResult<()> {
        let json = serde_json::to_string(&template.sections)
            .map_err(|e| RepositoryError::decode("report_template", e.to_string()))?;

        let tags_json = serde_json::to_string(&template.selected_tags)
            .map_err(|e| RepositoryError::decode("report_template", e.to_string()))?;

        let result = sqlx::query(
            "UPDATE report_templates
             SET name = ?, sections_json = ?, selected_tags_json = ?, updated_at = datetime('now')
             WHERE id = ?",
        )
        .bind(&template.name)
        .bind(&json)
        .bind(&tags_json)
        .bind(&template.id)
        .execute(&self.pool)
        .await?;

        super::require_affected(result.rows_affected(), "report_template", &template.id)
    }

    async fn set_active_template(&self, id: &str) -> RepositoryResult<()> {
        // Deactivate all, then activate the target
        sqlx::query("UPDATE report_templates SET is_active = 0")
            .execute(&self.pool)
            .await?;
        let result = sqlx::query(
            "UPDATE report_templates SET is_active = 1, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        super::require_affected(result.rows_affected(), "report_template", id)
    }

    async fn delete_template(&self, id: &str) -> RepositoryResult<()> {
        let result =
            sqlx::query("DELETE FROM report_templates WHERE id = ? AND is_builtin = 0")
                .bind(id)
                .execute(&self.pool)
                .await?;

        super::require_affected(result.rows_affected(), "report_template", id)
    }
}

// ── Row type ─────────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct TemplateRow {
    id: String,
    name: String,
    is_builtin: i64,
    sections_json: String,
    selected_tags_json: String,
    #[allow(dead_code)]
    is_active: i64,
}

impl TemplateRow {
    fn into_domain(self) -> RepositoryResult<ReportTemplate> {
        let sections: Vec<TemplateSection> =
            serde_json::from_str(&self.sections_json).map_err(|e| {
                RepositoryError::decode(
                    "report_template",
                    format!("invalid sections JSON: {e}"),
                )
            })?;

        let selected_tags: Vec<String> =
            serde_json::from_str(&self.selected_tags_json).unwrap_or_default();

        Ok(ReportTemplate {
            id: self.id,
            name: self.name,
            is_builtin: self.is_builtin != 0,
            sections,
            selected_tags,
        })
    }
}

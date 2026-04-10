use async_trait::async_trait;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::contexts::analysis::IssueSeverity;
use crate::contexts::extension::{
    CustomCheck, CustomCheckParams, CustomExtractor, CustomExtractorParams, Operator,
};
use crate::repository::{ExtensionRepository, RepositoryError, RepositoryResult};

pub struct SqliteExtensionRepository {
    pool: SqlitePool,
}

impl SqliteExtensionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

use super::require_affected;

fn check_from_params(id: String, p: &CustomCheckParams) -> CustomCheck {
    CustomCheck {
        id,
        name: p.name.clone(),
        severity: p.severity,
        field: p.field.clone(),
        operator: p.operator.clone(),
        threshold: p.threshold.clone(),
        message_template: p.message_template.clone(),
        enabled: p.enabled,
    }
}

fn extractor_from_params(id: String, p: &CustomExtractorParams) -> CustomExtractor {
    CustomExtractor {
        id,
        name: p.name.clone(),
        tag: p.tag.clone(),
        selector: p.selector.clone(),
        attribute: p.attribute.clone(),
        multiple: p.multiple,
        enabled: p.enabled,
    }
}

#[async_trait]
impl ExtensionRepository for SqliteExtensionRepository {
    // --- CustomCheck CRUD ---

    async fn create_check(&self, params: &CustomCheckParams) -> RepositoryResult<CustomCheck> {
        let id = uuid::Uuid::new_v4().to_string();
        let severity = params.severity.as_str();
        let operator = params.operator.to_string();
        let enabled = i64::from(params.enabled);

        sqlx::query(
            "INSERT INTO custom_checks (id, name, severity, field, operator, threshold, message_template, enabled)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&params.name)
        .bind(severity)
        .bind(&params.field)
        .bind(&operator)
        .bind(&params.threshold)
        .bind(&params.message_template)
        .bind(enabled)
        .execute(&self.pool)
        .await?;

        Ok(check_from_params(id, params))
    }

    async fn list_checks(&self) -> RepositoryResult<Vec<CustomCheck>> {
        let rows = sqlx::query_as::<_, CheckRow>(
            "SELECT id, name, severity, field, operator, threshold, message_template, enabled
             FROM custom_checks ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(CheckRow::into_domain).collect()
    }

    async fn get_check(&self, id: &str) -> RepositoryResult<CustomCheck> {
        let row = sqlx::query_as::<_, CheckRow>(
            "SELECT id, name, severity, field, operator, threshold, message_template, enabled
             FROM custom_checks WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::not_found("custom_check", id),
            other => RepositoryError::from(other),
        })?;

        row.into_domain()
    }

    async fn update_check(
        &self,
        id: &str,
        params: &CustomCheckParams,
    ) -> RepositoryResult<CustomCheck> {
        let severity = params.severity.as_str();
        let operator = params.operator.to_string();
        let enabled = i64::from(params.enabled);

        let rows_affected = sqlx::query(
            "UPDATE custom_checks
             SET name = ?, severity = ?, field = ?, operator = ?, threshold = ?,
                 message_template = ?, enabled = ?, updated_at = datetime('now')
             WHERE id = ?",
        )
        .bind(&params.name)
        .bind(severity)
        .bind(&params.field)
        .bind(&operator)
        .bind(&params.threshold)
        .bind(&params.message_template)
        .bind(enabled)
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        require_affected(rows_affected, "custom_check", id)?;
        Ok(check_from_params(id.to_string(), params))
    }

    async fn delete_check(&self, id: &str) -> RepositoryResult<()> {
        let rows_affected = sqlx::query("DELETE FROM custom_checks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        require_affected(rows_affected, "custom_check", id)
    }

    async fn list_enabled_checks(&self) -> RepositoryResult<Vec<CustomCheck>> {
        let rows = sqlx::query_as::<_, CheckRow>(
            "SELECT id, name, severity, field, operator, threshold, message_template, enabled
             FROM custom_checks WHERE enabled = 1 ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(CheckRow::into_domain).collect()
    }

    // --- CustomExtractor CRUD ---

    async fn create_extractor(
        &self,
        params: &CustomExtractorParams,
    ) -> RepositoryResult<CustomExtractor> {
        let id = uuid::Uuid::new_v4().to_string();
        let multiple = i64::from(params.multiple);
        let enabled = i64::from(params.enabled);

        sqlx::query(
            "INSERT INTO custom_extractors (id, name, tag, selector, attribute, multiple, enabled)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&params.name)
        .bind(&params.tag)
        .bind(&params.selector)
        .bind(&params.attribute)
        .bind(multiple)
        .bind(enabled)
        .execute(&self.pool)
        .await?;

        Ok(extractor_from_params(id, params))
    }

    async fn list_extractors(&self) -> RepositoryResult<Vec<CustomExtractor>> {
        let rows = sqlx::query_as::<_, ExtractorRow>(
            "SELECT id, name, tag, selector, attribute, multiple, enabled
             FROM custom_extractors ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ExtractorRow::into_domain).collect())
    }

    async fn get_extractor(&self, id: &str) -> RepositoryResult<CustomExtractor> {
        let row = sqlx::query_as::<_, ExtractorRow>(
            "SELECT id, name, tag, selector, attribute, multiple, enabled
             FROM custom_extractors WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::not_found("custom_extractor", id),
            other => RepositoryError::from(other),
        })?;

        Ok(row.into_domain())
    }

    async fn update_extractor(
        &self,
        id: &str,
        params: &CustomExtractorParams,
    ) -> RepositoryResult<CustomExtractor> {
        let multiple = i64::from(params.multiple);
        let enabled = i64::from(params.enabled);

        let rows_affected = sqlx::query(
            "UPDATE custom_extractors
             SET name = ?, tag = ?, selector = ?, attribute = ?, multiple = ?, enabled = ?,
                 updated_at = datetime('now')
             WHERE id = ?",
        )
        .bind(&params.name)
        .bind(&params.tag)
        .bind(&params.selector)
        .bind(&params.attribute)
        .bind(multiple)
        .bind(enabled)
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        require_affected(rows_affected, "custom_extractor", id)?;
        Ok(extractor_from_params(id.to_string(), params))
    }

    async fn delete_extractor(&self, id: &str) -> RepositoryResult<()> {
        let rows_affected = sqlx::query("DELETE FROM custom_extractors WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        require_affected(rows_affected, "custom_extractor", id)
    }

    async fn list_enabled_extractors(&self) -> RepositoryResult<Vec<CustomExtractor>> {
        let rows = sqlx::query_as::<_, ExtractorRow>(
            "SELECT id, name, tag, selector, attribute, multiple, enabled
             FROM custom_extractors WHERE enabled = 1 ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ExtractorRow::into_domain).collect())
    }
}

// --- Row types for sqlx ---

#[derive(sqlx::FromRow)]
struct CheckRow {
    id: String,
    name: String,
    severity: String,
    field: String,
    operator: String,
    threshold: Option<String>,
    message_template: String,
    enabled: i64,
}

impl CheckRow {
    fn into_domain(self) -> RepositoryResult<CustomCheck> {
        Ok(CustomCheck {
            id: self.id,
            name: self.name,
            severity: IssueSeverity::from_str(&self.severity).map_err(|e| {
                // `e` carries the offending string via the typed
                // ParseIssueSeverityError; surface it instead of
                // re-formatting from `self.severity`.
                RepositoryError::decode("custom_check", format!("invalid severity: {e}"))
            })?,
            field: self.field,
            operator: Operator::from_str(&self.operator).map_err(|e| {
                RepositoryError::decode("custom_check", format!("invalid operator: {e}"))
            })?,
            threshold: self.threshold,
            message_template: self.message_template,
            enabled: self.enabled != 0,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ExtractorRow {
    id: String,
    name: String,
    tag: String,
    selector: String,
    attribute: Option<String>,
    multiple: i64,
    enabled: i64,
}

impl ExtractorRow {
    fn into_domain(self) -> CustomExtractor {
        CustomExtractor {
            id: self.id,
            name: self.name,
            tag: self.tag,
            selector: self.selector,
            attribute: self.attribute,
            multiple: self.multiple != 0,
            enabled: self.enabled != 0,
        }
    }
}

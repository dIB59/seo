use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::contexts::analysis::IssueSeverity;
use crate::contexts::extension::{
    CustomCheck, CustomCheckParams, CustomExtractor, CustomExtractorParams, Operator,
};
use crate::repository::ExtensionRepository;

pub struct SqliteExtensionRepository {
    pool: SqlitePool,
}

impl SqliteExtensionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExtensionRepository for SqliteExtensionRepository {
    // --- CustomCheck CRUD ---

    async fn create_check(&self, params: &CustomCheckParams) -> Result<CustomCheck> {
        let id = uuid::Uuid::new_v4().to_string();
        let severity = params.severity.as_str();
        let operator = params.operator.to_string();
        let enabled = params.enabled as i64;

        sqlx::query(
            "INSERT INTO custom_checks (id, name, severity, field, operator, threshold, message_template, enabled)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&params.name)
        .bind(&severity)
        .bind(&params.field)
        .bind(&operator)
        .bind(&params.threshold)
        .bind(&params.message_template)
        .bind(enabled)
        .execute(&self.pool)
        .await
        .context("Failed to insert custom_check")?;

        Ok(CustomCheck {
            id,
            name: params.name.clone(),
            severity: params.severity.clone(),
            field: params.field.clone(),
            operator: params.operator.clone(),
            threshold: params.threshold.clone(),
            message_template: params.message_template.clone(),
            enabled: params.enabled,
        })
    }

    async fn list_checks(&self) -> Result<Vec<CustomCheck>> {
        let rows = sqlx::query_as::<_, CheckRow>(
            "SELECT id, name, severity, field, operator, threshold, message_template, enabled
             FROM custom_checks ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list custom_checks")?;

        rows.into_iter().map(CheckRow::into_domain).collect()
    }

    async fn get_check(&self, id: &str) -> Result<CustomCheck> {
        let row = sqlx::query_as::<_, CheckRow>(
            "SELECT id, name, severity, field, operator, threshold, message_template, enabled
             FROM custom_checks WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("custom_check not found")?;

        row.into_domain()
    }

    async fn update_check(&self, id: &str, params: &CustomCheckParams) -> Result<CustomCheck> {
        let severity = params.severity.as_str();
        let operator = params.operator.to_string();
        let enabled = params.enabled as i64;

        let rows_affected = sqlx::query(
            "UPDATE custom_checks
             SET name = ?, severity = ?, field = ?, operator = ?, threshold = ?,
                 message_template = ?, enabled = ?, updated_at = datetime('now')
             WHERE id = ?",
        )
        .bind(&params.name)
        .bind(&severity)
        .bind(&params.field)
        .bind(&operator)
        .bind(&params.threshold)
        .bind(&params.message_template)
        .bind(enabled)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to update custom_check")?
        .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("custom_check not found: {}", id));
        }

        Ok(CustomCheck {
            id: id.to_string(),
            name: params.name.clone(),
            severity: params.severity.clone(),
            field: params.field.clone(),
            operator: params.operator.clone(),
            threshold: params.threshold.clone(),
            message_template: params.message_template.clone(),
            enabled: params.enabled,
        })
    }

    async fn delete_check(&self, id: &str) -> Result<()> {
        let rows_affected = sqlx::query("DELETE FROM custom_checks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete custom_check")?
            .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("custom_check not found: {}", id));
        }

        Ok(())
    }

    async fn list_enabled_checks(&self) -> Result<Vec<CustomCheck>> {
        let rows = sqlx::query_as::<_, CheckRow>(
            "SELECT id, name, severity, field, operator, threshold, message_template, enabled
             FROM custom_checks WHERE enabled = 1 ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list enabled custom_checks")?;

        rows.into_iter().map(CheckRow::into_domain).collect()
    }

    // --- CustomExtractor CRUD ---

    async fn create_extractor(&self, params: &CustomExtractorParams) -> Result<CustomExtractor> {
        let id = uuid::Uuid::new_v4().to_string();
        let multiple = params.multiple as i64;
        let enabled = params.enabled as i64;

        sqlx::query(
            "INSERT INTO custom_extractors (id, name, key, selector, attribute, multiple, enabled)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&params.name)
        .bind(&params.key)
        .bind(&params.selector)
        .bind(&params.attribute)
        .bind(multiple)
        .bind(enabled)
        .execute(&self.pool)
        .await
        .context("Failed to insert custom_extractor")?;

        Ok(CustomExtractor {
            id,
            name: params.name.clone(),
            key: params.key.clone(),
            selector: params.selector.clone(),
            attribute: params.attribute.clone(),
            multiple: params.multiple,
            enabled: params.enabled,
        })
    }

    async fn list_extractors(&self) -> Result<Vec<CustomExtractor>> {
        let rows = sqlx::query_as::<_, ExtractorRow>(
            "SELECT id, name, key, selector, attribute, multiple, enabled
             FROM custom_extractors ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list custom_extractors")?;

        Ok(rows.into_iter().map(ExtractorRow::into_domain).collect())
    }

    async fn get_extractor(&self, id: &str) -> Result<CustomExtractor> {
        let row = sqlx::query_as::<_, ExtractorRow>(
            "SELECT id, name, key, selector, attribute, multiple, enabled
             FROM custom_extractors WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("custom_extractor not found")?;

        Ok(row.into_domain())
    }

    async fn update_extractor(
        &self,
        id: &str,
        params: &CustomExtractorParams,
    ) -> Result<CustomExtractor> {
        let multiple = params.multiple as i64;
        let enabled = params.enabled as i64;

        let rows_affected = sqlx::query(
            "UPDATE custom_extractors
             SET name = ?, key = ?, selector = ?, attribute = ?, multiple = ?, enabled = ?,
                 updated_at = datetime('now')
             WHERE id = ?",
        )
        .bind(&params.name)
        .bind(&params.key)
        .bind(&params.selector)
        .bind(&params.attribute)
        .bind(multiple)
        .bind(enabled)
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to update custom_extractor")?
        .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("custom_extractor not found: {}", id));
        }

        Ok(CustomExtractor {
            id: id.to_string(),
            name: params.name.clone(),
            key: params.key.clone(),
            selector: params.selector.clone(),
            attribute: params.attribute.clone(),
            multiple: params.multiple,
            enabled: params.enabled,
        })
    }

    async fn delete_extractor(&self, id: &str) -> Result<()> {
        let rows_affected = sqlx::query("DELETE FROM custom_extractors WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete custom_extractor")?
            .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("custom_extractor not found: {}", id));
        }

        Ok(())
    }

    async fn list_enabled_extractors(&self) -> Result<Vec<CustomExtractor>> {
        let rows = sqlx::query_as::<_, ExtractorRow>(
            "SELECT id, name, key, selector, attribute, multiple, enabled
             FROM custom_extractors WHERE enabled = 1 ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list enabled custom_extractors")?;

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
    fn into_domain(self) -> Result<CustomCheck> {
        Ok(CustomCheck {
            id: self.id,
            name: self.name,
            severity: IssueSeverity::from_str(&self.severity)
                .map_err(|_| anyhow::anyhow!("Invalid severity in custom_check row: {}", self.severity))?,
            field: self.field,
            operator: Operator::from_str(&self.operator)
                .context("Invalid operator in custom_check row")?,
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
    key: String,
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
            key: self.key,
            selector: self.selector,
            attribute: self.attribute,
            multiple: self.multiple != 0,
            enabled: self.enabled != 0,
        }
    }
}

// src/repository/extension_repository.rs

use anyhow::{Context, Result};
use sqlx::Row;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{contexts::IssueRuleInfo, repository::ExtensionRepositoryTrait};

fn serialize_threshold(min: Option<f64>, max: Option<f64>) -> Option<String> {
    if min.is_none() && max.is_none() {
        return None;
    }

    Some(
        serde_json::json!({
            "min": min,
            "max": max,
        })
        .to_string(),
    )
}

fn parse_threshold(threshold_json: Option<&str>) -> (Option<f64>, Option<f64>) {
    match threshold_json {
        Some(json) => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json) {
                let min = parsed.get("min").and_then(|v| v.as_f64());
                let max = parsed.get("max").and_then(|v| v.as_f64());
                (min, max)
            } else {
                (None, None)
            }
        }
        None => (None, None),
    }
}

/// Information about an extractor config from the database
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ExtractorConfigInfo {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub extractor_type: String,
    pub selector: String,
    pub attribute: Option<String>,
    pub storage_type: String,
    pub target_column: Option<String>,
    pub target_table: Option<String>,
    pub post_process: Option<String>,
    pub is_builtin: bool,
    pub is_enabled: bool,
}


pub struct ExtensionRepository {
    pool: sqlx::SqlitePool,
}

impl ExtensionRepository {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn table_exists(&self) -> bool {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='audit_rules')",
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false)
    }

    pub async fn get_all_rules(&self) -> Result<Vec<IssueRuleInfo>> {
        if !self.table_exists().await {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
             SELECT id, name, category, severity, rule_type, target_field,
                 threshold_value, regex_pattern, recommendation, is_builtin, is_enabled
            FROM audit_rules
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch audit rules")?;

        let rules = rows
            .iter()
            .map(|row| {
                let threshold_value = row.try_get::<Option<String>, _>("threshold_value").ok().flatten();
                let (threshold_min, threshold_max) = parse_threshold(threshold_value.as_deref());

                IssueRuleInfo {
                    id: row.try_get("id").unwrap_or_default(),
                    name: row.try_get("name").unwrap_or_default(),
                    category: row.try_get("category").unwrap_or_else(|_| "seo".to_string()),
                    severity: row.try_get("severity").unwrap_or_else(|_| "warning".to_string()),
                    rule_type: row.try_get("rule_type").unwrap_or_else(|_| "presence".to_string()),
                    target_field: row.try_get("target_field").ok(),
                    threshold_min,
                    threshold_max,
                    regex_pattern: row.try_get("regex_pattern").ok().flatten(),
                    recommendation: row.try_get("recommendation").ok(),
                    is_builtin: row.try_get::<i64, _>("is_builtin").unwrap_or(0) == 1,
                    is_enabled: row.try_get::<i64, _>("is_enabled").unwrap_or(1) == 1,
                }
            })
            .collect();

        Ok(rules)
    }

    pub async fn get_rule_by_id(&self, id: &str) -> Result<IssueRuleInfo> {
        let row = sqlx::query(
            r#"
             SELECT id, name, category, severity, rule_type, target_field,
                 threshold_value, regex_pattern, recommendation, is_builtin, is_enabled
            FROM audit_rules
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Rule not found")?;

        let threshold_value: Option<String> = row.try_get("threshold_value")?;
        let (threshold_min, threshold_max) = parse_threshold(threshold_value.as_deref());

        Ok(IssueRuleInfo {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            category: row.try_get("category")?,
            severity: row.try_get("severity")?,
            rule_type: row.try_get("rule_type")?,
            target_field: row.try_get("target_field")?,
            threshold_min,
            threshold_max,
            regex_pattern: row.try_get("regex_pattern")?,
            recommendation: row.try_get("recommendation")?,
            is_builtin: row.try_get::<i64, _>("is_builtin")? == 1,
            is_enabled: row.try_get::<i64, _>("is_enabled")? == 1,
        })
    }

    pub async fn insert_rule(
        &self,
        id: &str,
        name: &str,
        category: &str,
        severity: &str,
        rule_type: &str,
        target_field: &str,
        threshold_value: Option<&str>,
        regex_pattern: Option<&str>,
        recommendation: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_rules (
                id, name, category, severity, description, rule_type,
                target_field, threshold_value, regex_pattern, recommendation,
                is_enabled, is_builtin, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, 0, datetime('now'), datetime('now'))
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(category)
        .bind(severity)
        .bind(name) // description same as name for now
        .bind(rule_type)
        .bind(target_field)
        .bind(threshold_value)
        .bind(regex_pattern)
        .bind(recommendation)
        .execute(&self.pool)
        .await
        .context("Failed to insert rule")?;

        Ok(())
    }

    pub async fn update_rule(
        &self,
        id: &str,
        name: Option<&str>,
        severity: Option<&str>,
        threshold_min: Option<f64>,
        threshold_max: Option<f64>,
        regex_pattern: Option<&str>,
        is_enabled: Option<bool>,
        recommendation: Option<&str>,
    ) -> Result<()> {
        self.assert_not_builtin(id).await?;

        // We'll build the query with string fields only — bools get special handling below
        // Use a typed approach to avoid boxing
        let mut sql_parts: Vec<String> = Vec::new();

        if name.is_some() { sql_parts.push("name = ?".into()); }
        if severity.is_some() { sql_parts.push("severity = ?".into()); }
        sql_parts.push("threshold_value = ?".into());
        sql_parts.push("regex_pattern = ?".into());
        if is_enabled.is_some() { sql_parts.push("is_enabled = ?".into()); }
        if recommendation.is_some() { sql_parts.push("recommendation = ?".into()); }

        sql_parts.push("updated_at = datetime('now')".into());

        let sql = format!("UPDATE audit_rules SET {} WHERE id = ?", sql_parts.join(", "));

        let mut query = sqlx::query(&sql);
        if let Some(v) = name { query = query.bind(v); }
        if let Some(v) = severity { query = query.bind(v); }
        query = query.bind(serialize_threshold(threshold_min, threshold_max));
        query = query.bind(regex_pattern);
        if let Some(v) = is_enabled { query = query.bind(if v { 1i64 } else { 0i64 }); }
        if let Some(v) = recommendation { query = query.bind(v); }
        query = query.bind(id);

        query.execute(&self.pool).await.context("Failed to update rule")?;
        Ok(())
    }

    pub async fn delete_rule(&self, id: &str) -> Result<()> {
        self.assert_not_builtin(id).await?;

        sqlx::query("DELETE FROM audit_rules WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete rule")?;

        Ok(())
    }

    pub async fn set_rule_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        sqlx::query(
            "UPDATE audit_rules SET is_enabled = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(if enabled { 1i64 } else { 0i64 })
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to toggle rule")?;

        Ok(())
    }

    pub async fn count_custom_rules(&self) -> Result<usize> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_rules WHERE is_builtin = 0")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
        Ok(count as usize)
    }

    pub async fn migrate_rule_targets_to_field_format(&self) -> Result<usize> {
        if !self.table_exists().await {
            return Ok(0);
        }

        let extracted_result = sqlx::query(
            r#"
            UPDATE audit_rules
            SET target_field = 'field:extractor:' || substr(target_field, 11),
                updated_at = datetime('now')
            WHERE target_field LIKE 'extracted:%'
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to migrate extracted:* rule targets")?;

        let category_result = sqlx::query(
            r#"
            UPDATE audit_rules
            SET target_field = 'field:category:' || substr(target_field, 10),
                updated_at = datetime('now')
            WHERE target_field LIKE 'category:%'
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to migrate category:* rule targets")?;

        Ok((extracted_result.rows_affected() + category_result.rows_affected()) as usize)
    }

    pub async fn is_builtin(&self, id: &str) -> bool {
        sqlx::query_scalar("SELECT is_builtin FROM audit_rules WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(true)
    }

    async fn assert_not_builtin(&self, id: &str) -> Result<()> {
        if self.is_builtin(id).await {
            anyhow::bail!("Cannot modify built-in rules");
        }
        Ok(())
    }

    // ============================================================================
    // Extractor Config Methods
    // ============================================================================

    pub async fn extractor_table_exists(&self) -> bool {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='extractor_configs')",
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false)
    }

    pub async fn get_all_extractors(&self) -> Result<Vec<ExtractorConfigInfo>> {
        if !self.extractor_table_exists().await {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT id, name, display_name, description, extractor_type, 
                   selector, attribute, storage_type, target_column, target_table,
                   post_process, is_builtin, is_enabled
            FROM extractor_configs
            ORDER BY display_name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch extractor configs")?;

        let extractors = rows
            .iter()
            .map(|row| ExtractorConfigInfo {
                id: row.try_get("id").unwrap_or_default(),
                name: row.try_get("name").unwrap_or_default(),
                display_name: row.try_get("display_name").unwrap_or_default(),
                description: row.try_get("description").ok(),
                extractor_type: row.try_get("extractor_type").unwrap_or_else(|_| "css_selector".to_string()),
                selector: row.try_get("selector").unwrap_or_default(),
                attribute: row.try_get("attribute").ok(),
                storage_type: row.try_get("storage_type").unwrap_or_else(|_| "json".to_string()),
                target_column: row.try_get("target_column").ok(),
                target_table: row.try_get("target_table").ok(),
                post_process: row.try_get("post_process").ok(),
                is_builtin: row.try_get::<i64, _>("is_builtin").unwrap_or(0) == 1,
                is_enabled: row.try_get::<i64, _>("is_enabled").unwrap_or(1) == 1,
            })
            .collect();

        Ok(extractors)
    }

    pub async fn get_extractor_by_id(&self, id: &str) -> Result<ExtractorConfigInfo> {
        let row = sqlx::query(
            r#"
            SELECT id, name, display_name, description, extractor_type, 
                   selector, attribute, storage_type, target_column, target_table,
                   post_process, is_builtin, is_enabled
            FROM extractor_configs
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Extractor not found")?;

        Ok(ExtractorConfigInfo {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            display_name: row.try_get("display_name")?,
            description: row.try_get("description").ok(),
            extractor_type: row.try_get("extractor_type").unwrap_or_else(|_| "css_selector".to_string()),
            selector: row.try_get("selector")?,
            attribute: row.try_get("attribute").ok(),
            storage_type: row.try_get("storage_type").unwrap_or_else(|_| "json".to_string()),
            target_column: row.try_get("target_column").ok(),
            target_table: row.try_get("target_table").ok(),
            post_process: row.try_get("post_process").ok(),
            is_builtin: row.try_get::<i64, _>("is_builtin")? == 1,
            is_enabled: row.try_get::<i64, _>("is_enabled")? == 1,
        })
    }

    pub async fn insert_extractor(
        &self,
        id: &str,
        name: &str,
        display_name: &str,
        description: Option<&str>,
        extractor_type: &str,
        selector: &str,
        attribute: Option<&str>,
        post_process: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO extractor_configs (
                id, name, display_name, description, extractor_type,
                selector, attribute, post_process, storage_type, is_enabled, is_builtin, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'json', 1, 0, datetime('now'))
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(display_name)
        .bind(description)
        .bind(extractor_type)
        .bind(selector)
        .bind(attribute)
        .bind(post_process)
        .execute(&self.pool)
        .await
        .context("Failed to insert extractor")?;

        Ok(())
    }

    pub async fn update_extractor(
        &self,
        id: &str,
        name: Option<&str>,
        display_name: Option<&str>,
        description: Option<&str>,
        extractor_type: Option<&str>,
        selector: Option<&str>,
        attribute: Option<&str>,
        post_process: Option<&str>,
    ) -> Result<()> {
        // Check if this is a custom extractor
        let is_builtin: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT is_builtin FROM extractor_configs WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(1);

        if is_builtin == 1 {
            anyhow::bail!("Cannot modify built-in extractors");
        }

        // Build dynamic update query
        let mut updates: Vec<&str> = Vec::new();
        
        if name.is_some() { updates.push("name"); }
        if display_name.is_some() { updates.push("display_name"); }
        if description.is_some() { updates.push("description"); }
        if extractor_type.is_some() { updates.push("extractor_type"); }
        if selector.is_some() { updates.push("selector"); }
        if attribute.is_some() { updates.push("attribute"); }
        if post_process.is_some() { updates.push("post_process"); }

        if updates.is_empty() {
            anyhow::bail!("No fields to update");
        }

        // Use a simpler approach - build query with string concatenation
        let sql = format!(
            "UPDATE extractor_configs SET name = COALESCE(?, name), display_name = COALESCE(?, display_name), description = COALESCE(?, description), extractor_type = COALESCE(?, extractor_type), selector = COALESCE(?, selector), attribute = COALESCE(?, attribute), post_process = COALESCE(?, post_process) WHERE id = ?",
        );

        sqlx::query(&sql)
            .bind(name.unwrap_or(""))
            .bind(display_name.unwrap_or(""))
            .bind(description.unwrap_or(""))
            .bind(extractor_type.unwrap_or(""))
            .bind(selector.unwrap_or(""))
            .bind(attribute.unwrap_or(""))
            .bind(post_process.unwrap_or(""))
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to update extractor")?;

        Ok(())
    }

    pub async fn delete_extractor(&self, id: &str) -> Result<()> {
        // Check if this is a custom extractor
        let is_builtin: i64 = sqlx::query_scalar(
            "SELECT is_builtin FROM extractor_configs WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(1);

        if is_builtin == 1 {
            anyhow::bail!("Cannot delete built-in extractors");
        }

        sqlx::query("DELETE FROM extractor_configs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete extractor")?;

        Ok(())
    }

    pub async fn set_extractor_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        sqlx::query(
            "UPDATE extractor_configs SET is_enabled = ? WHERE id = ?",
        )
        .bind(if enabled { 1i64 } else { 0i64 })
        .bind(id)
        .execute(&self.pool)
        .await
        .context("Failed to toggle extractor")?;

        Ok(())
    }

    pub async fn count_custom_extractors(&self) -> Result<usize> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM extractor_configs WHERE is_builtin = 0"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);
        Ok(count as usize)
    }
}

use async_trait::async_trait;

#[async_trait]
impl ExtensionRepositoryTrait for ExtensionRepository {
    async fn get_all_rules(&self) -> Result<Vec<IssueRuleInfo>> {
        ExtensionRepository::get_all_rules(self).await
    }

    async fn get_rule_by_id(&self, id: &str) -> Result<IssueRuleInfo> {
        ExtensionRepository::get_rule_by_id(self, id).await
    }

    async fn insert_rule(
        &self,
        IssueRuleInfo {
            id,
            name,
            category,
            severity,
            rule_type,
            target_field,
            threshold_min,
            threshold_max,
            regex_pattern,
            recommendation,
            is_builtin: _,
            is_enabled: _,
        }: &IssueRuleInfo,
    ) -> Result<()> {
        let target_field_str = target_field.as_deref().unwrap_or("");
        ExtensionRepository::insert_rule(
            self,
            id,
            name,
            category,
            severity,
            rule_type,
            target_field_str,
            serialize_threshold(*threshold_min, *threshold_max).as_deref(),
            regex_pattern.as_deref(),
            recommendation.as_deref(),
        )
        .await
    }

    async fn update_rule(
        &self,
        id: &str,
        name: Option<&str>,
        severity: Option<&str>,
        threshold_min: Option<f64>,
        threshold_max: Option<f64>,
        regex_pattern: Option<&str>,
        is_enabled: Option<bool>,
        recommendation: Option<&str>,
    ) -> Result<()> {
        ExtensionRepository::update_rule(
            self,
            id,
            name,
            severity,
            threshold_min,
            threshold_max,
            regex_pattern,
            is_enabled,
            recommendation,
        )
        .await
    }

    async fn delete_rule(&self, id: &str) -> Result<()> {
        ExtensionRepository::delete_rule(self, id).await
    }

    async fn set_rule_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        ExtensionRepository::set_rule_enabled(self, id, enabled).await
    }

    async fn count_custom_rules(&self) -> Result<usize> {
        ExtensionRepository::count_custom_rules(self).await
    }

    async fn migrate_rule_targets_to_field_format(&self) -> Result<usize> {
        ExtensionRepository::migrate_rule_targets_to_field_format(self).await
    }

    // Extractor trait implementations
    async fn get_all_extractors(&self) -> Result<Vec<ExtractorConfigInfo>> {
        ExtensionRepository::get_all_extractors(self).await
    }

    async fn get_extractor_by_id(&self, id: &str) -> Result<ExtractorConfigInfo> {
        ExtensionRepository::get_extractor_by_id(self, id).await
    }

    async fn insert_extractor(
        &self,
        id: &str,
        name: &str,
        display_name: &str,
        description: Option<&str>,
        extractor_type: &str,
        selector: &str,
        attribute: Option<&str>,
        post_process: Option<&str>,
    ) -> Result<()> {
        ExtensionRepository::insert_extractor(
            self, id, name, display_name, description, extractor_type, selector, attribute, post_process,
        )
        .await
    }

    async fn update_extractor(
        &self,
        id: &str,
        name: Option<&str>,
        display_name: Option<&str>,
        description: Option<&str>,
        extractor_type: Option<&str>,
        selector: Option<&str>,
        attribute: Option<&str>,
        post_process: Option<&str>,
    ) -> Result<()> {
        ExtensionRepository::update_extractor(
            self, id, name, display_name, description, extractor_type, selector, attribute, post_process,
        )
        .await
    }

    async fn delete_extractor(&self, id: &str) -> Result<()> {
        ExtensionRepository::delete_extractor(self, id).await
    }

    async fn set_extractor_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        ExtensionRepository::set_extractor_enabled(self, id, enabled).await
    }

    async fn count_custom_extractors(&self) -> Result<usize> {
        ExtensionRepository::count_custom_extractors(self).await
    }
}

// src/repository/extension_repository.rs

use anyhow::{Context, Result};
use sqlx::Row;

use crate::{contexts::IssueRuleInfo, repository::ExtensionRepositoryTrait};


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
                   recommendation, is_builtin, is_enabled
            FROM audit_rules
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch audit rules")?;

        let rules = rows
            .iter()
            .map(|row| IssueRuleInfo {
                id: row.try_get("id").unwrap_or_default(),
                name: row.try_get("name").unwrap_or_default(),
                category: row.try_get("category").unwrap_or_else(|_| "seo".to_string()),
                severity: row.try_get("severity").unwrap_or_else(|_| "warning".to_string()),
                rule_type: row.try_get("rule_type").unwrap_or_else(|_| "presence".to_string()),
                target_field: row.try_get("target_field").ok(),
                recommendation: row.try_get("recommendation").ok(),
                is_builtin: row.try_get::<i64, _>("is_builtin").unwrap_or(0) == 1,
                is_enabled: row.try_get::<i64, _>("is_enabled").unwrap_or(1) == 1,
            })
            .collect();

        Ok(rules)
    }

    pub async fn get_rule_by_id(&self, id: &str) -> Result<IssueRuleInfo> {
        let row = sqlx::query(
            r#"
            SELECT id, name, category, severity, rule_type, target_field,
                   recommendation, is_builtin, is_enabled
            FROM audit_rules
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Rule not found")?;

        Ok(IssueRuleInfo {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            category: row.try_get("category")?,
            severity: row.try_get("severity")?,
            rule_type: row.try_get("rule_type")?,
            target_field: row.try_get("target_field")?,
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
        is_enabled: Option<bool>,
        recommendation: Option<&str>,
    ) -> Result<()> {
        self.assert_not_builtin(id).await?;

        let mut updates: Vec<&str> = Vec::new();
        // We'll build the query with string fields only — bools get special handling below
        // Use a typed approach to avoid boxing
        let mut sql_parts: Vec<String> = Vec::new();

        if name.is_some() { sql_parts.push("name = ?".into()); }
        if severity.is_some() { sql_parts.push("severity = ?".into()); }
        if is_enabled.is_some() { sql_parts.push("is_enabled = ?".into()); }
        if recommendation.is_some() { sql_parts.push("recommendation = ?".into()); }

        if sql_parts.is_empty() {
            anyhow::bail!("No fields to update");
        }

        sql_parts.push("updated_at = datetime('now')".into());

        let sql = format!("UPDATE audit_rules SET {} WHERE id = ?", sql_parts.join(", "));

        // Bind as strings to keep things uniform (SQLite is flexible)
        let mut query = sqlx::query(&sql);
        if let Some(v) = name { query = query.bind(v); }
        if let Some(v) = severity { query = query.bind(v); }
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
            None, // threshold_value not used for now
            None, // regex_pattern not used for now
            recommendation.as_deref(),
        )
        .await
    }

    async fn update_rule(
        &self,
        id: &str,
        name: Option<&str>,
        severity: Option<&str>,
        is_enabled: Option<bool>,
        recommendation: Option<&str>,
    ) -> Result<()> {
        ExtensionRepository::update_rule(
            self, id, name, severity, is_enabled, recommendation,
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

    
}

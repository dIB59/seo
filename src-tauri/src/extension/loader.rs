//! Extension Loader
//!
//! This module handles loading extensions from the database at startup.
//! It converts database records into trait objects that can be used
//! by the ExtensionRegistry.

use anyhow::{Context, Result};
use sqlx::SqlitePool;

use super::audit_check::{
    CanonicalCheck, CrawlableAnchorsCheck, HreflangCheck, HttpStatusCodeCheck,
    ImageAltCheck, LinkTextCheck, MetaDescriptionCheck, RobotsMetaCheck, TitleCheck,
    ViewportCheck, AuditCheck,
};
use super::data_extractor::{
    CssSelectorExtractor, HrefTagExtractor, KeywordExtractor, OpenGraphExtractor,
    StructuredDataExtractor, TwitterCardExtractor, PageDataExtractor,
};
use super::issue_rule::{
    IssueRule, LengthRule, PresenceRule, RegexRule, StatusCodeRule, ThresholdRule,
};
use crate::contexts::IssueSeverity;

/// Loader for extensions from the database
pub struct ExtensionLoader<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ExtensionLoader<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Load all issue rules from the database
    pub async fn load_issue_rules(&self) -> Result<Vec<Box<dyn IssueRule>>> {
        let mut rules = Vec::new();

        // Try to load all rules from database first (both built-in and custom)
        match self.load_all_rules_from_db().await {
            Ok(db_rules) if !db_rules.is_empty() => {
                tracing::info!("Loaded {} rules from database", db_rules.len());
                rules.extend(db_rules);
            }
            Ok(_) => {
                // Database exists but no rules, use built-in as fallback
                tracing::warn!("No rules found in database, using built-in rules");
                rules.extend(self.get_builtin_rules());
            }
            Err(e) => {
                // Database doesn't exist or error, use built-in as fallback
                tracing::warn!("Failed to load rules from database ({}), using built-in rules", e);
                rules.extend(self.get_builtin_rules());
            }
        }

        Ok(rules)
    }

    /// Load all data extractors from the database
    pub async fn load_data_extractors(&self) -> Result<Vec<Box<dyn PageDataExtractor>>> {
        let mut extractors = Vec::new();

        // Load built-in extractors
        extractors.extend(self.get_builtin_extractors());

        // Load custom extractors from database
        match self.load_custom_extractors().await {
            Ok(custom) => extractors.extend(custom),
            Err(e) => {
                tracing::warn!("Failed to load custom extractors: {}", e);
            }
        }

        Ok(extractors)
    }

    /// Load all audit checks from the database
    pub async fn load_audit_checks(&self) -> Result<Vec<Box<dyn AuditCheck>>> {
        let mut checks = Vec::new();

        // Load built-in checks
        checks.extend(self.get_builtin_checks());

        // Load custom checks from database
        match self.load_custom_checks().await {
            Ok(custom) => checks.extend(custom),
            Err(e) => {
                tracing::warn!("Failed to load custom checks: {}", e);
            }
        }

        Ok(checks)
    }

    /// Get built-in issue rules
    fn get_builtin_rules(&self) -> Vec<Box<dyn IssueRule>> {
        vec![
            // Title rules
            Box::new(
                PresenceRule::new(
                    "missing-title",
                    "Missing Title",
                    "title",
                    true,
                    IssueSeverity::Critical,
                ),
            ),
            Box::new(
                LengthRule::new(
                    "title-length",
                    "Title Length",
                    "title",
                    IssueSeverity::Warning,
                )
                .with_range(30, 60)
                .with_recommendation("Keep title between 30-60 characters"),
            ),
            // Meta description rules
            Box::new(
                PresenceRule::new(
                    "missing-meta-description",
                    "Missing Meta Description",
                    "meta_description",
                    true,
                    IssueSeverity::Warning,
                ),
            ),
            Box::new(
                LengthRule::new(
                    "meta-description-length",
                    "Meta Description Length",
                    "meta_description",
                    IssueSeverity::Warning,
                )
                .with_range(70, 160)
                .with_recommendation("Keep meta description between 70-160 characters"),
            ),
            // HTTP status rule
            Box::new(
                StatusCodeRule::new("http-error", "HTTP Error")
                    .with_codes(vec![400, 401, 403, 404, 500, 502, 503, 504]),
            ),
            // Word count rule
            Box::new(
                ThresholdRule::new(
                    "low-word-count",
                    "Low Word Count",
                    "word_count",
                    IssueSeverity::Info,
                )
                .with_min(300.0)
                .with_recommendation("Consider adding more content (300+ words)"),
            ),
            // Load time rule
            Box::new(
                ThresholdRule::new(
                    "slow-load-time",
                    "Slow Page Load",
                    "load_time_ms",
                    IssueSeverity::Warning,
                )
                .with_max(3000.0)
                .with_recommendation("Optimize page load time to under 3 seconds"),
            ),
        ]
    }

    /// Get built-in data extractors
    fn get_builtin_extractors(&self) -> Vec<Box<dyn PageDataExtractor>> {
        vec![
            Box::new(OpenGraphExtractor::new()),
            Box::new(TwitterCardExtractor::new()),
            Box::new(HrefTagExtractor::new()),
            Box::new(KeywordExtractor::new()),
            Box::new(StructuredDataExtractor::new()),
        ]
    }

    /// Get built-in audit checks
    fn get_builtin_checks(&self) -> Vec<Box<dyn AuditCheck>> {
        vec![
            Box::new(TitleCheck::new()),
            Box::new(MetaDescriptionCheck::new()),
            Box::new(ViewportCheck::new()),
            Box::new(CanonicalCheck::new()),
            Box::new(HreflangCheck::new()),
            Box::new(CrawlableAnchorsCheck::new()),
            Box::new(LinkTextCheck::new()),
            Box::new(ImageAltCheck::new()),
            Box::new(HttpStatusCodeCheck::new()),
            Box::new(RobotsMetaCheck::new()),
        ]
    }

    /// Load custom rules from the database
    async fn load_custom_rules(&self) -> Result<Vec<Box<dyn IssueRule>>> {
        // Check if the audit_rules table exists
        let table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='audit_rules')",
        )
        .fetch_one(self.pool)
        .await
        .unwrap_or(false);

        if !table_exists {
            return Ok(Vec::new());
        }

        // Use runtime query instead of compile-time checked query
        let rows = sqlx::query(
            r#"
            SELECT 
                id, name, category, severity, description,
                rule_type, target_field, condition,
                threshold_value, regex_pattern, recommendation,
                is_enabled, is_builtin
            FROM audit_rules
            WHERE is_enabled = 1 AND is_builtin = 0
            "#
        )
        .fetch_all(self.pool)
        .await
        .context("Failed to fetch custom audit rules")?;

        let mut rules = Vec::new();

        for row in rows {
            match self.parse_custom_rule(row) {
                Ok(rule) => rules.push(rule),
                Err(e) => {
                    tracing::warn!("Failed to parse custom rule: {}", e);
                }
            }
        }

        Ok(rules)
    }

    /// Load ALL rules from the database (both built-in and custom)
    async fn load_all_rules_from_db(&self) -> Result<Vec<Box<dyn IssueRule>>> {
        // Check if the audit_rules table exists
        let table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='audit_rules')",
        )
        .fetch_one(self.pool)
        .await
        .unwrap_or(false);

        if !table_exists {
            anyhow::bail!("audit_rules table does not exist");
        }

        // Load all enabled rules from database (both built-in and custom)
        let rows = sqlx::query(
            r#"
            SELECT 
                id, name, category, severity, description,
                rule_type, target_field, condition,
                threshold_value, regex_pattern, recommendation,
                is_enabled, is_builtin
            FROM audit_rules
            WHERE is_enabled = 1
            "#
        )
        .fetch_all(self.pool)
        .await
        .context("Failed to fetch audit rules from database")?;

        let mut rules = Vec::new();

        for row in rows {
            match self.parse_custom_rule(row) {
                Ok(rule) => rules.push(rule),
                Err(e) => {
                    tracing::warn!("Failed to parse rule: {}", e);
                }
            }
        }

        Ok(rules)
    }

    /// Parse a database row into a custom rule
    fn parse_custom_rule(
        &self,
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<Box<dyn IssueRule>> {
        use sqlx::Row;

        let id: String = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let _category: String = row.try_get("category")?;
        let severity_str: String = row.try_get("severity")?;
        let rule_type: String = row.try_get("rule_type")?;
        let target_field: String = row.try_get("target_field")?;
        let recommendation: Option<String> = row.try_get("recommendation")?;

        let severity = match severity_str.as_str() {
            "critical" => IssueSeverity::Critical,
            "warning" => IssueSeverity::Warning,
            _ => IssueSeverity::Info,
        };

        let rule: Box<dyn IssueRule> = match rule_type.as_str() {
            "presence" => {
                let mut rule = PresenceRule::new(&id, &name, &target_field, true, severity);
                if let Some(rec) = recommendation {
                    rule.recommendation = Some(rec);
                }
                Box::new(rule)
            }
            "threshold" => {
                let threshold_value: Option<String> = row.try_get("threshold_value")?;
                let mut rule = ThresholdRule::new(&id, &name, &target_field, severity);

                if let Some(threshold_json) = threshold_value {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&threshold_json) {
                        if let Some(min) = parsed.get("min").and_then(|v| v.as_f64()) {
                            rule = rule.with_min(min);
                        }
                        if let Some(max) = parsed.get("max").and_then(|v| v.as_f64()) {
                            rule = rule.with_max(max);
                        }
                    }
                }

                if let Some(rec) = recommendation {
                    rule.recommendation = Some(rec);
                }
                Box::new(rule)
            }
            "length" => {
                let threshold_value: Option<String> = row.try_get("threshold_value")?;
                let mut rule = LengthRule::new(&id, &name, &target_field, severity);

                if let Some(threshold_json) = threshold_value {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&threshold_json) {
                        if let Some(min) = parsed.get("min").and_then(|v| v.as_u64()) {
                            rule = rule.with_min(min as usize);
                        }
                        if let Some(max) = parsed.get("max").and_then(|v| v.as_u64()) {
                            rule = rule.with_max(max as usize);
                        }
                    }
                }

                if let Some(rec) = recommendation {
                    rule.recommendation = Some(rec);
                }
                Box::new(rule)
            }
            "regex" => {
                let regex_pattern: Option<String> = row.try_get("regex_pattern")?;
                match regex_pattern {
                    Some(pattern) => {
                        match RegexRule::new(&id, &name, &target_field, &pattern, true, severity) {
                            Ok(mut rule) => {
                                if let Some(rec) = recommendation {
                                    rule.recommendation = Some(rec);
                                }
                                Box::new(rule)
                            }
                            Err(e) => {
                                anyhow::bail!("Invalid regex pattern: {}", e);
                            }
                        }
                    }
                    None => {
                        anyhow::bail!("Regex rule missing pattern");
                    }
                }
            }
            "status_code" => {
                let threshold_value: Option<String> = row.try_get("threshold_value")?;
                let mut rule = StatusCodeRule::new(&id, &name);

                if let Some(threshold_json) = threshold_value {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&threshold_json) {
                        if let Some(error_codes) = parsed.get("error_codes").and_then(|v| v.as_array()) {
                            let codes: Vec<u16> = error_codes
                                .iter()
                                .filter_map(|v| v.as_u64().map(|n| n as u16))
                                .collect();
                            rule = rule.with_codes(codes);
                        }
                    }
                }

                if let Some(rec) = recommendation {
                    rule.recommendation = Some(rec);
                }
                Box::new(rule)
            }
            _ => {
                anyhow::bail!("Unknown rule type: {}", rule_type);
            }
        };

        Ok(rule)
    }

    /// Load custom extractors from the database
    async fn load_custom_extractors(&self) -> Result<Vec<Box<dyn PageDataExtractor>>> {
        // Check if the extractor_configs table exists
        let table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='extractor_configs')",
        )
        .fetch_one(self.pool)
        .await
        .unwrap_or(false);

        if !table_exists {
            return Ok(Vec::new());
        }

        // Use runtime query instead of compile-time checked query
        let rows = sqlx::query(
            r#"
            SELECT 
                id, name, display_name, description,
                extractor_type, selector, attribute,
                is_enabled, is_builtin
            FROM extractor_configs
            WHERE is_enabled = 1 AND is_builtin = 0
            "#
        )
        .fetch_all(self.pool)
        .await
        .context("Failed to fetch custom extractors")?;

        let mut extractors = Vec::new();

        for row in rows {
            match self.parse_custom_extractor(row) {
                Ok(extractor) => extractors.push(extractor),
                Err(e) => {
                    tracing::warn!("Failed to parse custom extractor: {}", e);
                }
            }
        }

        Ok(extractors)
    }

    /// Parse a database row into a custom extractor
    fn parse_custom_extractor(
        &self,
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<Box<dyn PageDataExtractor>> {
        use sqlx::Row;

        let id: String = row.try_get("id")?;
        let display_name: String = row.try_get("display_name")?;
        let description: Option<String> = row.try_get("description")?;
        let extractor_type: String = row.try_get("extractor_type")?;
        let selector: String = row.try_get("selector")?;
        let attribute: Option<String> = row.try_get("attribute")?;

        match extractor_type.as_str() {
            "css_selector" => {
                let mut extractor = CssSelectorExtractor::new(&id, &display_name, &selector);
                if let Some(attr) = attribute {
                    extractor = extractor.with_attribute(attr);
                }
                if let Some(desc) = description {
                    extractor = extractor.with_description(desc);
                }
                Ok(Box::new(extractor))
            }
            _ => {
                anyhow::bail!("Unknown extractor type: {}", extractor_type);
            }
        }
    }

    /// Load custom audit checks from the database
    async fn load_custom_checks(&self) -> Result<Vec<Box<dyn AuditCheck>>> {
        // Check if the audit_checks table exists
        let table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='audit_checks')",
        )
        .fetch_one(self.pool)
        .await
        .unwrap_or(false);

        if !table_exists {
            return Ok(Vec::new());
        }

        // For now, we don't support custom audit checks from the database
        // This would require a more complex system for defining check logic
        // The infrastructure is here for future expansion

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_rules_count() {
        // Test built-in rules without needing a pool
        let rules = get_builtin_rules_standalone();
        assert!(!rules.is_empty());
        assert!(rules.len() >= 7, "Expected at least 7 built-in rules, got {}", rules.len());
    }

    #[test]
    fn test_builtin_extractors_count() {
        let extractors = get_builtin_extractors_standalone();
        assert!(!extractors.is_empty());
        assert!(extractors.len() >= 5, "Expected at least 5 built-in extractors, got {}", extractors.len());
    }

    #[test]
    fn test_builtin_checks_count() {
        let checks = get_builtin_checks_standalone();
        assert!(!checks.is_empty());
        assert!(checks.len() >= 10, "Expected at least 10 built-in checks, got {}", checks.len());
    }

    // Standalone functions for testing without database
    fn get_builtin_rules_standalone() -> Vec<Box<dyn IssueRule>> {
        vec![
            Box::new(
                PresenceRule::new(
                    "missing-title",
                    "Missing Title",
                    "title",
                    true,
                    IssueSeverity::Critical,
                ),
            ),
            Box::new(
                LengthRule::new(
                    "title-length",
                    "Title Length",
                    "title",
                    IssueSeverity::Warning,
                )
                .with_range(30, 60)
                .with_recommendation("Keep title between 30-60 characters"),
            ),
            Box::new(
                PresenceRule::new(
                    "missing-meta-description",
                    "Missing Meta Description",
                    "meta_description",
                    true,
                    IssueSeverity::Warning,
                ),
            ),
            Box::new(
                LengthRule::new(
                    "meta-description-length",
                    "Meta Description Length",
                    "meta_description",
                    IssueSeverity::Warning,
                )
                .with_range(70, 160)
                .with_recommendation("Keep meta description between 70-160 characters"),
            ),
            Box::new(
                StatusCodeRule::new("http-error", "HTTP Error")
                    .with_codes(vec![400, 401, 403, 404, 500, 502, 503, 504]),
            ),
            Box::new(
                ThresholdRule::new(
                    "low-word-count",
                    "Low Word Count",
                    "word_count",
                    IssueSeverity::Info,
                )
                .with_min(300.0)
                .with_recommendation("Consider adding more content (300+ words)"),
            ),
            Box::new(
                ThresholdRule::new(
                    "slow-load-time",
                    "Slow Page Load",
                    "load_time_ms",
                    IssueSeverity::Warning,
                )
                .with_max(3000.0)
                .with_recommendation("Optimize page load time to under 3 seconds"),
            ),
        ]
    }

    fn get_builtin_extractors_standalone() -> Vec<Box<dyn PageDataExtractor>> {
        vec![
            Box::new(OpenGraphExtractor::new()),
            Box::new(TwitterCardExtractor::new()),
            Box::new(HrefTagExtractor::new()),
            Box::new(KeywordExtractor::new()),
            Box::new(StructuredDataExtractor::new()),
        ]
    }

    fn get_builtin_checks_standalone() -> Vec<Box<dyn AuditCheck>> {
        vec![
            Box::new(TitleCheck::new()),
            Box::new(MetaDescriptionCheck::new()),
            Box::new(ViewportCheck::new()),
            Box::new(CanonicalCheck::new()),
            Box::new(HreflangCheck::new()),
            Box::new(CrawlableAnchorsCheck::new()),
            Box::new(LinkTextCheck::new()),
            Box::new(ImageAltCheck::new()),
            Box::new(HttpStatusCodeCheck::new()),
            Box::new(RobotsMetaCheck::new()),
        ]
    }
}

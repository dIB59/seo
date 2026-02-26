//! Extension Loader
//!
//! This module handles loading extensions from the database at startup.
//! It converts database records into trait objects that can be used
//! by the ExtensionRegistry.
//!
//! This loader provides backward compatibility with the legacy extension
//! format while supporting the new trait-based system.

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::sync::Arc;

use super::traits::{DataExtractor, ExtensionConfig, IssueGenerator};
use super::capabilities::ExtensionCapability;
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
    pub async fn load_issue_rules(&self) -> Result<Vec<Arc<dyn IssueGenerator>>> {
        let mut rules = Vec::new();

        // Try to load custom rules from database
        match self.load_custom_rules_from_db().await {
            Ok(db_rules) if !db_rules.is_empty() => {
                tracing::info!("Loaded {} custom rules from database", db_rules.len());
                rules.extend(db_rules);
            }
            Ok(_) => {
                tracing::debug!("No custom rules found in database");
            }
            Err(e) => {
                tracing::warn!("Failed to load rules from database: {}", e);
            }
        }

        Ok(rules)
    }

    /// Load custom rules from the database
    async fn load_custom_rules_from_db(&self) -> Result<Vec<Arc<dyn IssueGenerator>>> {
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

        // Load all enabled custom rules from database
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

    /// Parse a database row into a custom rule
    fn parse_custom_rule(
        &self,
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<Arc<dyn IssueGenerator>> {
        use sqlx::Row;

        let id: String = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let rule_type: String = row.try_get("rule_type")?;
        let target_field: String = row.try_get("target_field")?;
        let recommendation: Option<String> = row.try_get("recommendation")?;
        let severity_str: String = row.try_get("severity")?;

        let severity = match severity_str.as_str() {
            "critical" => IssueSeverity::Critical,
            "warning" => IssueSeverity::Warning,
            _ => IssueSeverity::Info,
        };

        // Create a dynamic rule based on type
        let rule: Arc<dyn IssueGenerator> = match rule_type.as_str() {
            "presence" => Arc::new(DynamicPresenceRule {
                config: ExtensionConfig::new(&id, &name)
                    .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                    .with_description(recommendation.clone().unwrap_or_default()),
                target_field,
                severity,
                recommendation,
            }),
            "threshold" => {
                let threshold_value: Option<String> = row.try_get("threshold_value")?;
                let (min, max) = parse_threshold(&threshold_value);
                
                Arc::new(DynamicThresholdRule {
                    config: ExtensionConfig::new(&id, &name)
                        .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                        .with_description(recommendation.clone().unwrap_or_default()),
                    target_field,
                    min,
                    max,
                    severity,
                    recommendation,
                })
            }
            "length" => {
                let threshold_value: Option<String> = row.try_get("threshold_value")?;
                let (min, max) = parse_threshold(&threshold_value);
                
                Arc::new(DynamicLengthRule {
                    config: ExtensionConfig::new(&id, &name)
                        .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                        .with_description(recommendation.clone().unwrap_or_default()),
                    target_field,
                    min: min.map(|n| n as usize),
                    max: max.map(|n| n as usize),
                    severity,
                    recommendation,
                })
            }
            _ => {
                anyhow::bail!("Unknown rule type: {}", rule_type);
            }
        };

        Ok(rule)
    }

    /// Load custom data extractors from the database
    pub async fn load_custom_extractors(&self) -> Result<Vec<Arc<dyn DataExtractor>>> {
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
    ) -> Result<Arc<dyn DataExtractor>> {
        use sqlx::Row;

        let id: String = row.try_get("id")?;
        let display_name: String = row.try_get("display_name")?;
        let description: Option<String> = row.try_get("description")?;
        let extractor_type: String = row.try_get("extractor_type")?;
        let selector: String = row.try_get("selector")?;
        let attribute: Option<String> = row.try_get("attribute")?;

        match extractor_type.as_str() {
            "css_selector" => {
                Ok(Arc::new(DynamicCssExtractor {
                    config: ExtensionConfig::new(&id, &display_name)
                        .with_capabilities(vec![ExtensionCapability::DataExtraction])
                        .with_description(description.unwrap_or_default()),
                    selector,
                    attribute,
                    multiple: false,
                }))
            }
            _ => {
                anyhow::bail!("Unknown extractor type: {}", extractor_type);
            }
        }
    }
}

/// Parse threshold value from JSON
fn parse_threshold(threshold_json: &Option<String>) -> (Option<f64>, Option<f64>) {
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

// ============================================================================
// Dynamic Rule Implementations
// ============================================================================

use crate::extension::context::ValidationContext;
use crate::extension::result::ValidationResult;
use crate::extension::traits::Extension;
use crate::contexts::{NewIssue, Page};
use std::collections::HashMap;

/// Dynamic presence rule loaded from database
struct DynamicPresenceRule {
    config: ExtensionConfig,
    target_field: String,
    severity: IssueSeverity,
    recommendation: Option<String>,
}

impl Extension for DynamicPresenceRule {
    fn id(&self) -> &str {
        &self.config.id
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }
    
    fn capabilities(&self) -> Vec<ExtensionCapability> {
        vec![ExtensionCapability::IssueGeneration]
    }
}

impl IssueGenerator for DynamicPresenceRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let value = get_page_field(&context.page, &self.target_field);
        let exists = value.is_some() && !value.as_ref().is_some_and(|v| v.is_empty());
        
        let result = if !exists {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: self.name().to_string(),
                    severity: self.severity,
                    message: format!("{} is missing", self.target_field),
                    details: self.recommendation.clone(),
                })
        } else {
            ValidationResult::new(self.id().to_string())
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        self.severity
    }
    
    fn recommendation(&self) -> Option<&str> {
        self.recommendation.as_deref()
    }
}

/// Dynamic threshold rule loaded from database
struct DynamicThresholdRule {
    config: ExtensionConfig,
    target_field: String,
    min: Option<f64>,
    max: Option<f64>,
    severity: IssueSeverity,
    recommendation: Option<String>,
}

impl Extension for DynamicThresholdRule {
    fn id(&self) -> &str {
        &self.config.id
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }
    
    fn capabilities(&self) -> Vec<ExtensionCapability> {
        vec![ExtensionCapability::IssueGeneration]
    }
}

impl IssueGenerator for DynamicThresholdRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let value = match get_numeric_field(&context.page, &self.target_field) {
            Some(v) => v,
            None => return Ok(ValidationResult::new(self.id().to_string())),
        };
        
        let below_min = self.min.is_some_and(|min| value < min);
        let above_max = self.max.is_some_and(|max| value > max);
        
        let result = if below_min || above_max {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: self.name().to_string(),
                    severity: self.severity,
                    message: format!("{} ({}) is outside acceptable range", self.target_field, value),
                    details: self.recommendation.clone(),
                })
        } else {
            ValidationResult::new(self.id().to_string())
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        self.severity
    }
}

/// Dynamic length rule loaded from database
struct DynamicLengthRule {
    config: ExtensionConfig,
    target_field: String,
    min: Option<usize>,
    max: Option<usize>,
    severity: IssueSeverity,
    recommendation: Option<String>,
}

impl Extension for DynamicLengthRule {
    fn id(&self) -> &str {
        &self.config.id
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }
    
    fn capabilities(&self) -> Vec<ExtensionCapability> {
        vec![ExtensionCapability::IssueGeneration]
    }
}

impl IssueGenerator for DynamicLengthRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let value = match get_page_field(&context.page, &self.target_field) {
            Some(v) => v,
            None => return Ok(ValidationResult::new(self.id().to_string())),
        };
        
        let len = value.len();
        let below_min = self.min.is_some_and(|min| len < min);
        let above_max = self.max.is_some_and(|max| len > max);
        
        let result = if below_min || above_max {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: self.name().to_string(),
                    severity: self.severity,
                    message: format!("{} length ({}) is outside recommended range", self.target_field, len),
                    details: self.recommendation.clone(),
                })
        } else {
            ValidationResult::new(self.id().to_string())
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        self.severity
    }
}

// ============================================================================
// Dynamic Extractor Implementations
// ============================================================================

use crate::extension::context::ExtractionContext;
use crate::extension::result::{ExtractionMetadata, ExtractionResult, ExtractedValue};
use anyhow::anyhow;
use scraper::{Html, Selector};

/// Dynamic CSS selector extractor loaded from database
struct DynamicCssExtractor {
    config: ExtensionConfig,
    selector: String,
    attribute: Option<String>,
    multiple: bool,
}

impl Extension for DynamicCssExtractor {
    fn id(&self) -> &str {
        &self.config.id
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn description(&self) -> Option<&str> {
        self.config.description.as_deref()
    }
    
    fn capabilities(&self) -> Vec<ExtensionCapability> {
        vec![ExtensionCapability::DataExtraction]
    }
}

impl DataExtractor for DynamicCssExtractor {
    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult> {
        let document = Html::parse_document(&context.html);
        let selector = Selector::parse(&self.selector)
            .map_err(|e| anyhow!("Invalid selector '{}': {:?}", self.selector, e))?;
        
        let mut data = HashMap::new();
        
        if self.multiple {
            let values: Vec<String> = document
                .select(&selector)
                .filter_map(|el| {
                    if let Some(attr) = &self.attribute {
                        el.value().attr(attr).map(|s| s.to_string())
                    } else {
                        let text = el.text().collect::<String>();
                        let trimmed = text.trim().to_string();
                        if trimmed.is_empty() { None } else { Some(trimmed) }
                    }
                })
                .collect();
            data.insert("values".to_string(), ExtractedValue::List(values));
        } else {
            let value = document.select(&selector).next().and_then(|el| {
                if let Some(attr) = &self.attribute {
                    el.value().attr(attr).map(|s| s.to_string())
                } else {
                    let text = el.text().collect::<String>();
                    let trimmed = text.trim().to_string();
                    if trimmed.is_empty() { None } else { Some(trimmed) }
                }
            });
            
            data.insert("value".to_string(), ExtractedValue::Text(value.unwrap_or_default()));
        }
        
        Ok(ExtractionResult {
            extension_id: self.id().to_string(),
            data,
            metadata: ExtractionMetadata::default(),
        })
    }
    
    fn column_type(&self) -> &str {
        "TEXT"
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get a string field from a page
fn get_page_field(page: &Page, field: &str) -> Option<String> {
    match field {
        "title" => page.title.clone(),
        "meta_description" => page.meta_description.clone(),
        "canonical_url" => page.canonical_url.clone(),
        "robots_meta" => page.robots_meta.clone(),
        "content_type" => page.content_type.clone(),
        "url" => Some(page.url.clone()),
        _ => None,
    }
}

/// Get a numeric field from a page
fn get_numeric_field(page: &Page, field: &str) -> Option<f64> {
    match field {
        "word_count" => page.word_count.map(|v| v as f64),
        "load_time_ms" => page.load_time_ms.map(|v| v as f64),
        "response_size_bytes" => page.response_size_bytes.map(|v| v as f64),
        "status_code" => page.status_code.map(|v| v as f64),
        "depth" => Some(page.depth as f64),
        _ => None,
    }
}

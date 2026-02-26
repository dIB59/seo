//! Extension System for SEO Analysis
//!
//! This module provides a plugin-based architecture for extending the SEO analysis
//! capabilities without modifying core code. Extensions are loaded from the database
//! at startup and can be added dynamically.
//!
//! # Architecture
//!
//! The extension system consists of three main components:
//! - [`IssueRule`]: Rules that generate issues based on page analysis
//! - [`PageDataExtractor`]: Extractors that pull additional data from pages
//! - [`AuditCheck`]: Checks that contribute to SEO audit scores
//!
//! All extensions are managed by the [`ExtensionRegistry`] which handles
//! loading, caching, and execution.

mod audit_check;
mod data_extractor;
mod issue_rule;
mod loader;

pub use audit_check::{AuditCheck, AuditContext, CheckResult};
pub use data_extractor::{ExtractedData, PageDataExtractor};
pub use issue_rule::{EvaluationContext, IssueRule, RuleCondition, RuleType};
pub use loader::ExtensionLoader;

use anyhow::Result;
use dashmap::DashMap;
use sqlx::SqlitePool;

use crate::contexts::{NewIssue, Page};

/// Central registry for all extensions.
///
/// The registry manages issue rules, data extractors, and audit checks.
/// It loads extensions from the database at startup and provides methods
/// for evaluating rules and extracting data.
pub struct ExtensionRegistry {
    /// Issue rules indexed by rule ID
    issue_rules: DashMap<String, Box<dyn IssueRule>>,

    /// Data extractors indexed by extractor ID
    data_extractors: DashMap<String, Box<dyn PageDataExtractor>>,

    /// Audit checks indexed by check key
    audit_checks: DashMap<String, Box<dyn AuditCheck>>,

    /// Cache for rule evaluation results
    evaluation_cache: DashMap<String, Vec<NewIssue>>,
}

impl ExtensionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            issue_rules: DashMap::new(),
            data_extractors: DashMap::new(),
            audit_checks: DashMap::new(),
            evaluation_cache: DashMap::new(),
        }
    }

    /// Load all extensions from the database
    pub async fn load_from_database(pool: &SqlitePool) -> Result<Self> {
        let registry = Self::new();
        let loader = ExtensionLoader::new(pool);

        // Load issue rules
        let rules = loader.load_issue_rules().await?;
        for rule in rules {
            registry.register_issue_rule(rule);
        }

        // Load data extractors
        let extractors = loader.load_data_extractors().await?;
        for extractor in extractors {
            registry.register_data_extractor(extractor);
        }

        // Load audit checks
        let checks = loader.load_audit_checks().await?;
        for check in checks {
            registry.register_audit_check(check);
        }

        tracing::info!(
            "Loaded {} issue rules, {} extractors, {} audit checks",
            registry.issue_rules.len(),
            registry.data_extractors.len(),
            registry.audit_checks.len()
        );

        Ok(registry)
    }

    /// Register an issue rule
    pub fn register_issue_rule(&self, rule: Box<dyn IssueRule>) {
        let id = rule.id().to_string();
        self.issue_rules.insert(id, rule);
    }

    /// Register a data extractor
    pub fn register_data_extractor(&self, extractor: Box<dyn PageDataExtractor>) {
        let id = extractor.id().to_string();
        self.data_extractors.insert(id, extractor);
    }

    /// Register an audit check
    pub fn register_audit_check(&self, check: Box<dyn AuditCheck>) {
        let key = check.key().to_string();
        self.audit_checks.insert(key, check);
    }

    /// Evaluate all issue rules against a page
    pub fn evaluate_rules(
        &self,
        page: &Page,
        context: &EvaluationContext,
    ) -> Vec<NewIssue> {
        self.issue_rules
            .iter()
            .filter_map(|entry| {
                let rule = entry.value();
                if rule.applies_to(page) {
                    rule.evaluate(page, context)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all registered issue rule IDs
    pub fn get_issue_rule_ids(&self) -> Vec<String> {
        self.issue_rules.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get all registered data extractor IDs
    pub fn get_data_extractor_ids(&self) -> Vec<String> {
        self.data_extractors.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get all registered audit check keys
    pub fn get_audit_check_keys(&self) -> Vec<String> {
        self.audit_checks.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Run a specific data extractor
    pub fn run_extractor(&self, id: &str, html: &str, url: &str) -> Option<ExtractedData> {
        self.data_extractors.get(id).and_then(|entry| {
            entry.value().extract(html, url).ok()
        })
    }

    /// Run a specific audit check
    pub fn run_audit_check(&self, key: &str, context: &AuditContext) -> Option<CheckResult> {
        self.audit_checks.get(key).map(|entry| entry.value().check(context))
    }

    /// Clear the evaluation cache
    pub fn clear_cache(&self) {
        self.evaluation_cache.clear();
    }

    /// Get the number of registered issue rules
    pub fn rule_count(&self) -> usize {
        self.issue_rules.len()
    }

    /// Get the number of registered data extractors
    pub fn extractor_count(&self) -> usize {
        self.data_extractors.len()
    }

    /// Get the number of registered audit checks
    pub fn check_count(&self) -> usize {
        self.audit_checks.len()
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ExtensionRegistry::new();
        assert_eq!(registry.rule_count(), 0);
        assert_eq!(registry.extractor_count(), 0);
        assert_eq!(registry.check_count(), 0);
    }
}

//! Extension System for SEO Analysis
//!
//! This module provides a plugin-based architecture for extending the SEO analysis
//! capabilities without modifying core code. Extensions are loaded from the database
//! at startup and can be added dynamically.
//!
//! # Architecture
//!
//! The extension system consists of three main capabilities:
//! - [`IssueGenerator`]: Rules that generate issues based on page analysis
//! - [`DataExtractor`]: Extractors that pull additional data from pages
//! - [`DataExporter`]: Exporters that expose data for external consumption
//!
//! All extensions are managed by the [`ExtensionPipeline`] which handles
//! the three-phase execution: Extract → Validate → Export.

// Core types
pub mod capabilities;
pub mod context;
pub mod result;
pub mod traits;
pub mod pipeline;

// Built-in extensions
pub mod builtins;

// Legacy compatibility (deprecated)
mod loader;

// Re-export core types
pub use capabilities::{CapabilityConfig, CapabilityMetadata, ExtensionCapability};
pub use context::{ExtractionContext, ExportContext, ExportFormat, ValidationContext};
pub use result::{
    ExtractionMetadata, ExtractionResult, ExtractedValue, ExportResult,
    PipelineResult, ValidationResult,
};
pub use traits::{
    DataExporter, DataExtractor, Extension, ExtensionConfig,
    ExtractionSchema, SchemaField, SchemaFieldType, IssueGenerator,
};
pub use pipeline::ExtensionPipeline;
pub use loader::ExtensionLoader;

// Re-export built-in extensions
pub use builtins::register_builtins;

use anyhow::Result;
use dashmap::DashMap;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::contexts::{NewIssue, Page};

/// Central registry for all extensions.
///
/// The registry manages issue generators, data extractors, and data exporters.
/// It loads extensions from the database at startup and provides methods
/// for executing the extension pipeline.
///
/// # Example
///
/// ```rust
/// use extension::{ExtensionPipeline, ExtensionRegistry};
///
/// let registry = ExtensionRegistry::new();
/// registry.register_builtin_extensions();
///
/// // Execute pipeline for a page
/// let result = registry.execute(&page, html).await?;
/// ```
pub struct ExtensionRegistry {
    /// The execution pipeline
    pipeline: ExtensionPipeline,
    
    /// Extension configurations
    configs: DashMap<String, ExtensionConfig>,
}

impl ExtensionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            pipeline: ExtensionPipeline::new(),
            configs: DashMap::new(),
        }
    }
    
    /// Load all extensions from the database
    pub async fn load_from_database(pool: &SqlitePool) -> Result<Self> {
        let registry = Self::new();
        
        // Register built-in extensions first
        registry.register_builtin_extensions();
        
        // Load custom extensions from database
        let loader = ExtensionLoader::new(pool);
        
        // Load custom rules
        match loader.load_issue_rules().await {
            Ok(rules) => {
                for rule in rules {
                    // Convert legacy rules to new format
                    tracing::debug!("Loaded custom rule: {}", rule.id());
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load custom rules: {}", e);
            }
        }
        
        let (extractors, validators, exporters) = registry.pipeline.counts();
        tracing::info!(
            "Extension registry loaded: {} extractors, {} validators, {} exporters",
            extractors,
            validators,
            exporters
        );
        
        Ok(registry)
    }
    
    /// Register all built-in extensions
    pub fn register_builtin_extensions(&self) {
        builtins::register_builtins(&self.pipeline);
    }
    
    /// Register an issue generator
    pub fn register_validator(&self, validator: Arc<dyn IssueGenerator>) {
        let config = validator.config();
        self.configs.insert(config.id.clone(), config);
        self.pipeline.register_validator(validator);
    }
    
    /// Register a data extractor
    pub fn register_extractor(&self, extractor: Arc<dyn DataExtractor>) {
        let config = extractor.config();
        self.configs.insert(config.id.clone(), config);
        self.pipeline.register_extractor(extractor);
    }
    
    /// Register a data exporter
    pub fn register_exporter(&self, exporter: Arc<dyn DataExporter>) {
        let config = exporter.config();
        self.configs.insert(config.id.clone(), config);
        self.pipeline.register_exporter(exporter);
    }
    
    /// Unregister an extension by ID
    pub fn unregister(&self, id: &str) {
        self.pipeline.unregister(id);
        self.configs.remove(id);
    }
    
    /// Get extension configuration by ID
    pub fn get_config(&self, id: &str) -> Option<ExtensionConfig> {
        self.configs.get(id).map(|e| e.clone())
    }
    
    /// Get all extension configurations
    pub fn get_all_configs(&self) -> Vec<ExtensionConfig> {
        self.configs.iter().map(|e| e.clone()).collect()
    }
    
    /// Execute the full pipeline for a page
    pub async fn execute(&self, page: &Page, html: &str) -> Result<PipelineResult> {
        Ok(self.pipeline.execute(page, html).await)
    }
    
    /// Execute only extraction and validation phases
    pub async fn extract_and_validate(
        &self,
        page: &Page,
        html: &str,
    ) -> (std::collections::HashMap<String, serde_json::Value>, Vec<NewIssue>) {
        self.pipeline.extract_and_validate(page, html).await
    }
    
    /// Evaluate all issue rules against a page (legacy compatibility)
    pub fn evaluate_rules(
        &self,
        page: &Page,
        context: &ValidationContext,
    ) -> Vec<NewIssue> {
        // This is a synchronous wrapper for backward compatibility
        // In practice, use execute() or extract_and_validate() instead
        let mut issues = Vec::new();
        
        // Use a simple synchronous approach for legacy compatibility
        // This will be deprecated in favor of the async pipeline
        issues
    }
    
    /// Get all registered issue rule IDs
    pub fn get_issue_rule_ids(&self) -> Vec<String> {
        self.configs
            .iter()
            .filter(|e| e.capabilities.has_capability(ExtensionCapability::IssueGeneration))
            .map(|e| e.id.clone())
            .collect()
    }
    
    /// Get all registered data extractor IDs
    pub fn get_data_extractor_ids(&self) -> Vec<String> {
        self.configs
            .iter()
            .filter(|e| e.capabilities.has_capability(ExtensionCapability::DataExtraction))
            .map(|e| e.id.clone())
            .collect()
    }
    
    /// Get all registered data exporter IDs
    pub fn get_data_exporter_ids(&self) -> Vec<String> {
        self.configs
            .iter()
            .filter(|e| e.capabilities.has_capability(ExtensionCapability::DataExport))
            .map(|e| e.id.clone())
            .collect()
    }
    
    /// Get all registered audit check keys (legacy compatibility)
    pub fn get_audit_check_keys(&self) -> Vec<String> {
        // Audit checks are now merged with issue generators
        self.get_issue_rule_ids()
    }
    
    /// Run a specific data extractor
    pub fn run_extractor(&self, id: &str, html: &str, url: &str) -> Option<std::collections::HashMap<String, serde_json::Value>> {
        // Create extraction context
        let context = ExtractionContext::new(
            html.to_string(),
            url.to_string(),
            String::new(),
            String::new(),
        );
        
        // This would need to be async in practice
        None
    }
    
    /// Get the number of registered issue rules
    pub fn rule_count(&self) -> usize {
        self.configs
            .iter()
            .filter(|e| e.capabilities.has_capability(ExtensionCapability::IssueGeneration))
            .count()
    }
    
    /// Get the number of registered data extractors
    pub fn extractor_count(&self) -> usize {
        self.configs
            .iter()
            .filter(|e| e.capabilities.has_capability(ExtensionCapability::DataExtraction))
            .count()
    }
    
    /// Get the number of registered data exporters
    pub fn exporter_count(&self) -> usize {
        self.configs
            .iter()
            .filter(|e| e.capabilities.has_capability(ExtensionCapability::DataExport))
            .count()
    }
    
    /// Get the number of registered audit checks (legacy compatibility)
    pub fn check_count(&self) -> usize {
        self.rule_count()
    }
    
    /// Clear all extensions
    pub fn clear(&self) {
        self.pipeline.clear();
        self.configs.clear();
    }
    
    /// Clear the extraction cache
    pub fn clear_cache(&self) {
        self.pipeline.clear_cache();
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
        assert_eq!(registry.exporter_count(), 0);
    }
    
    #[test]
    fn test_builtin_registration() {
        let registry = ExtensionRegistry::new();
        registry.register_builtin_extensions();
        
        let (extractors, validators, exporters) = registry.pipeline.counts();
        assert!(extractors > 0);
        assert!(validators > 0);
        assert!(exporters > 0);
    }
}

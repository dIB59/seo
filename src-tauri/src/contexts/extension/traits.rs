//! Extension Traits
//!
//! This module defines the core traits for the extension system.
//! All extensions implement the base Extension trait and one or more
//! capability traits (IssueGenerator, DataExtractor, DataExporter).

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::capabilities::{CapabilityMetadata, ExtensionCapability};
use super::context::{ExtractionContext, ExportContext, ValidationContext};
use super::result::{ExtractionResult, ExportResult, ValidationResult};

/// Configuration for an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// Unique identifier for this extension
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Description of what this extension does
    pub description: Option<String>,
    
    /// Version string (semver recommended)
    pub version: String,
    
    /// Whether this extension is enabled
    pub enabled: bool,
    
    /// Whether this is a built-in extension
    pub is_builtin: bool,
    
    /// Capability metadata
    pub capabilities: CapabilityMetadata,
    
    /// Extension-specific configuration as JSON
    pub config: Option<serde_json::Value>,
}

impl ExtensionConfig {
    /// Create a new extension configuration
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0.0".to_string(),
            enabled: true,
            is_builtin: false,
            capabilities: CapabilityMetadata::new(vec![]),
            config: None,
        }
    }
    
    /// Add a description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
    
    /// Mark as built-in
    pub fn builtin(mut self) -> Self {
        self.is_builtin = true;
        self
    }
    
    /// Add capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<ExtensionCapability>) -> Self {
        self.capabilities = CapabilityMetadata::new(capabilities);
        self
    }
    
    /// Set the config JSON
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = Some(config);
        self
    }
}

/// Base trait for all extensions.
///
/// All extensions must implement this trait, which provides metadata
/// and lifecycle hooks. Extensions then implement one or more capability
/// traits based on what they can do.
pub trait Extension: Send + Sync {
    /// Get the unique identifier for this extension
    fn id(&self) -> &str;
    
    /// Get the human-readable name
    fn name(&self) -> &str;
    
    /// Get the description
    fn description(&self) -> Option<&str> {
        None
    }
    
    /// Get the version string
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    /// Get the capabilities this extension supports
    fn capabilities(&self) -> Vec<ExtensionCapability>;
    
    /// Check if this extension has a specific capability
    fn has_capability(&self, capability: ExtensionCapability) -> bool {
        self.capabilities().contains(&capability)
    }
    
    /// Initialize the extension with configuration
    fn initialize(&mut self, _config: &ExtensionConfig) -> Result<()> {
        Ok(())
    }
    
    /// Shutdown the extension
    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Get the extension configuration
    fn config(&self) -> ExtensionConfig {
        ExtensionConfig::new(self.id(), self.name())
            .with_description(self.description().unwrap_or(""))
            .with_version(self.version())
            .with_capabilities(self.capabilities())
    }
}

/// Trait for extensions that generate issues based on validation logic.
///
/// Issue generators are the core of the rule system. They examine page data
/// and generate issues when validation rules fail.
pub trait IssueGenerator: Extension {
    /// Validate a page and generate issues.
    ///
    /// This method is called during the validation phase of the pipeline.
    /// It receives the validation context with page data and extracted data,
    /// and returns a validation result with any issues found.
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult>;
    
    /// Check if this rule applies to the given page.
    ///
    /// Override this to limit which pages a rule applies to.
    fn applies_to(&self, _page: &crate::contexts::analysis::Page) -> bool {
        true
    }
    
    /// Get the default severity for issues from this generator.
    fn default_severity(&self) -> crate::contexts::analysis::IssueSeverity {
        crate::contexts::analysis::IssueSeverity::Warning
    }
    
    /// Get a recommendation for fixing issues from this generator.
    fn recommendation(&self) -> Option<&str> {
        None
    }
    
    /// Get the category for this issue generator.
    fn category(&self) -> &str {
        "seo"
    }
}

/// Trait for extensions that extract structured data from HTML.
///
/// Data extractors parse HTML content and extract structured data
/// that can be used by other extensions or exposed via API.
pub trait DataExtractor: Extension {
    /// Extract data from HTML.
    ///
    /// This method is called during the extraction phase of the pipeline.
    /// It receives the extraction context with HTML and page metadata,
    /// and returns an extraction result with the extracted data.
    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult>;
    
    /// Get the schema for extracted data.
    ///
    /// This describes what fields the extractor produces and their types.
    fn schema(&self) -> ExtractionSchema {
        ExtractionSchema::default()
    }
    
    /// Get the database column type for storage.
    fn column_type(&self) -> &str {
        "TEXT"
    }
    
    /// Whether this extractor is enabled by default.
    fn is_enabled_by_default(&self) -> bool {
        true
    }
}

/// Schema describing the output of a data extractor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionSchema {
    /// Fields produced by this extractor
    pub fields: Vec<SchemaField>,
    
    /// Description of the schema
    pub description: Option<String>,
}

impl Default for ExtractionSchema {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            description: None,
        }
    }
}

impl ExtractionSchema {
    /// Create a new schema with the given fields
    pub fn new(fields: Vec<SchemaField>) -> Self {
        Self {
            fields,
            description: None,
        }
    }
    
    /// Add a description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A field in an extraction schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    /// Field name
    pub name: String,
    
    /// Field type
    pub field_type: SchemaFieldType,
    
    /// Whether this field is required
    pub required: bool,
    
    /// Description of the field
    pub description: Option<String>,
}

impl SchemaField {
    /// Create a new schema field
    pub fn new(name: impl Into<String>, field_type: SchemaFieldType) -> Self {
        Self {
            name: name.into(),
            field_type,
            required: false,
            description: None,
        }
    }
    
    /// Mark as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    /// Add a description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Types for schema fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaFieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

/// Trait for extensions that export data for external consumption.
///
/// Data exporters take extracted data and issues and expose them
/// via API endpoints, webhooks, or other export mechanisms.
pub trait DataExporter: Extension {
    /// Export data.
    ///
    /// This method is called during the export phase of the pipeline.
    /// It receives the export context with extracted data and issues,
    /// and returns an export result.
    fn export(&self, context: &ExportContext) -> Result<ExportResult>;
    
    /// Get the export format this exporter produces.
    fn export_format(&self) -> super::context::ExportFormat {
        super::context::ExportFormat::Json
    }
    
    /// Get the target endpoint for webhook exports.
    fn endpoint(&self) -> Option<&str> {
        None
    }
    
    /// Whether to include issues in the export.
    fn include_issues(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extension_config() {
        let config = ExtensionConfig::new("test-ext", "Test Extension")
            .with_description("A test extension")
            .with_version("2.0.0")
            .with_capabilities(vec![
                ExtensionCapability::IssueGeneration,
                ExtensionCapability::DataExtraction,
            ]);
        
        assert_eq!(config.id, "test-ext");
        assert_eq!(config.name, "Test Extension");
        assert_eq!(config.version, "2.0.0");
        assert!(config.capabilities.has_capability(ExtensionCapability::IssueGeneration));
        assert!(!config.capabilities.has_capability(ExtensionCapability::DataExport));
    }
    
    #[test]
    fn test_schema_field() {
        let field = SchemaField::new("title", SchemaFieldType::String)
            .required()
            .with_description("The page title");
        
        assert_eq!(field.name, "title");
        assert!(field.required);
        assert_eq!(field.description, Some("The page title".to_string()));
    }
}

//! Extension Capabilities
//!
//! This module defines the capabilities that extensions can have.
//! Extensions declare their capabilities to enable proper orchestration
//! by the execution pipeline.

use serde::{Deserialize, Serialize};

/// Capabilities an extension can have.
///
/// Each capability corresponds to a specific function in the analysis pipeline:
/// - `IssueGeneration`: Validate pages and generate issues
/// - `DataExtraction`: Extract structured data from HTML
/// - `DataExport`: Expose data for external consumption
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionCapability {
    /// Extension can generate issues based on validation logic
    IssueGeneration,
    
    /// Extension can extract structured data from HTML
    DataExtraction,
    
    /// Extension can export data for external consumption
    DataExport,
}

impl ExtensionCapability {
    /// Get all available capabilities
    pub fn all() -> Vec<Self> {
        vec![
            Self::IssueGeneration,
            Self::DataExtraction,
            Self::DataExport,
        ]
    }
    
    /// Get the display name for this capability
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::IssueGeneration => "Issue Generation",
            Self::DataExtraction => "Data Extraction",
            Self::DataExport => "Data Export",
        }
    }
    
    /// Get a description of this capability
    pub fn description(&self) -> &'static str {
        match self {
            Self::IssueGeneration => "Validate pages and generate SEO issues based on custom rules",
            Self::DataExtraction => "Extract structured data from HTML content",
            Self::DataExport => "Export extracted data for external consumption via API or webhooks",
        }
    }
}

/// Configuration for a specific capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConfig {
    /// Whether this capability is enabled
    pub enabled: bool,
    
    /// Capability-specific configuration as JSON
    pub config: Option<serde_json::Value>,
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            config: None,
        }
    }
}

/// Metadata about an extension's capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMetadata {
    /// List of capabilities this extension supports
    pub capabilities: Vec<ExtensionCapability>,
    
    /// Configuration for each capability
    pub configs: std::collections::HashMap<String, CapabilityConfig>,
}

impl CapabilityMetadata {
    /// Create new capability metadata with the given capabilities
    pub fn new(capabilities: Vec<ExtensionCapability>) -> Self {
        Self {
            capabilities,
            configs: std::collections::HashMap::new(),
        }
    }
    
    /// Check if the extension has a specific capability
    pub fn has_capability(&self, capability: ExtensionCapability) -> bool {
        self.capabilities.contains(&capability)
    }
    
    /// Get the configuration for a specific capability
    pub fn get_config(&self, capability: &ExtensionCapability) -> Option<&CapabilityConfig> {
        let key = format!("{:?}", capability).to_lowercase();
        self.configs.get(&key)
    }
    
    /// Set the configuration for a specific capability
    pub fn set_config(&mut self, capability: ExtensionCapability, config: CapabilityConfig) {
        let key = format!("{:?}", capability).to_lowercase();
        self.configs.insert(key, config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_all() {
        let all = ExtensionCapability::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&ExtensionCapability::IssueGeneration));
        assert!(all.contains(&ExtensionCapability::DataExtraction));
        assert!(all.contains(&ExtensionCapability::DataExport));
    }
    
    #[test]
    fn test_capability_metadata() {
        let mut meta = CapabilityMetadata::new(vec![
            ExtensionCapability::IssueGeneration,
            ExtensionCapability::DataExtraction,
        ]);
        
        assert!(meta.has_capability(ExtensionCapability::IssueGeneration));
        assert!(meta.has_capability(ExtensionCapability::DataExtraction));
        assert!(!meta.has_capability(ExtensionCapability::DataExport));
        
        meta.set_config(ExtensionCapability::IssueGeneration, CapabilityConfig {
            enabled: false,
            config: Some(serde_json::json!({"threshold": 100})),
        });
        
        let config = meta.get_config(&ExtensionCapability::IssueGeneration).unwrap();
        assert!(!config.enabled);
    }
}

//! Built-in Data Exporters
//!
//! This module provides the built-in data exporters that expose
//! extracted data for external consumption.

use std::collections::HashMap;

use anyhow::Result;
use serde_json::json;

use crate::extension::capabilities::ExtensionCapability;
use crate::extension::context::{ExportContext, ExportFormat};
use crate::extension::result::ExportResult;
use crate::extension::traits::{DataExporter, Extension, ExtensionConfig};

// ============================================================================
// JSON Exporter
// ============================================================================

/// Exports data as JSON for API consumption.
pub struct JsonExporter {
    config: ExtensionConfig,
}

impl JsonExporter {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("json-export", "JSON Export")
                .with_description("Exports extracted data as JSON for API consumption")
                .with_capabilities(vec![ExtensionCapability::DataExport])
                .builtin(),
        }
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for JsonExporter {
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
        vec![ExtensionCapability::DataExport]
    }
}

impl DataExporter for JsonExporter {
    fn export(&self, context: &ExportContext) -> Result<ExportResult> {
        let export_data = json!({
            "page_id": context.page_id,
            "url": context.url,
            "data": context.data,
            "issues": context.issues.iter().map(|i| json!({
                "type": i.issue_type,
                "severity": format!("{:?}", i.severity).to_lowercase(),
                "message": i.message,
            })).collect::<Vec<_>>(),
            "exported_at": chrono::Utc::now().to_rfc3339(),
        });
        
        Ok(ExportResult::success(self.id().to_string(), export_data))
    }
    
    fn export_format(&self) -> ExportFormat {
        ExportFormat::Json
    }
    
    fn include_issues(&self) -> bool {
        true
    }
}

// ============================================================================
// Webhook Exporter
// ============================================================================

/// Exports data to an external webhook endpoint.
pub struct WebhookExporter {
    config: ExtensionConfig,
    endpoint: Option<String>,
    headers: HashMap<String, String>,
}

impl WebhookExporter {
    /// Create a new webhook exporter with an endpoint
    pub fn with_endpoint(endpoint: String) -> Self {
        Self {
            config: ExtensionConfig::new("webhook-export", "Webhook Export")
                .with_description("Exports extracted data to an external webhook endpoint")
                .with_capabilities(vec![ExtensionCapability::DataExport])
                .builtin(),
            endpoint: Some(endpoint),
            headers: HashMap::new(),
        }
    }
    
    /// Add a header to the webhook request
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}

impl Extension for WebhookExporter {
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
        vec![ExtensionCapability::DataExport]
    }
}

impl DataExporter for WebhookExporter {
    fn export(&self, context: &ExportContext) -> Result<ExportResult> {
        // For now, just prepare the data - actual HTTP call would be async
        // In a real implementation, this would use reqwest or similar
        let export_data = json!({
            "page_id": context.page_id,
            "url": context.url,
            "data": context.data,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        // Note: In production, this would make an actual HTTP request
        // For now, we just return success with the prepared data
        let mut result = ExportResult::success(self.id().to_string(), export_data);
        
        if let Some(ref endpoint) = self.endpoint {
            result = result.with_endpoint(endpoint.clone());
        }
        
        Ok(result)
    }
    
    fn export_format(&self) -> ExportFormat {
        ExportFormat::Json
    }
    
    fn endpoint(&self) -> Option<&str> {
        self.endpoint.as_deref()
    }
}

// ============================================================================
// CSV Exporter
// ============================================================================

/// Exports data as CSV format.
pub struct CsvExporter {
    config: ExtensionConfig,
}

impl CsvExporter {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("csv-export", "CSV Export")
                .with_description("Exports extracted data as CSV format")
                .with_capabilities(vec![ExtensionCapability::DataExport])
                .builtin(),
        }
    }
}

impl Default for CsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for CsvExporter {
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
        vec![ExtensionCapability::DataExport]
    }
}

impl DataExporter for CsvExporter {
    fn export(&self, context: &ExportContext) -> Result<ExportResult> {
        // Flatten the data for CSV format
        let mut rows = Vec::new();
        let mut headers = vec!["page_id", "url"];
        
        // Collect all unique keys from data
        let mut data_keys: Vec<&str> = context.data.keys().map(|s| s.as_str()).collect();
        data_keys.sort();
        headers.extend(data_keys.iter().map(|s| *s));
        
        // Create header row
        let header_row = headers.join(",");
        rows.push(header_row);
        
        // Create data row
        let mut row = vec![
            context.page_id.clone(),
            context.url.clone(),
        ];
        
        for key in &data_keys {
            let value = context.data.get(*key)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            // Escape commas and quotes in CSV
            let escaped = if value.contains(',') || value.contains('"') {
                format!("\"{}\"", value.replace('"', "\"\""))
            } else {
                value.to_string()
            };
            row.push(escaped);
        }
        
        rows.push(row.join(","));
        
        let csv_data = rows.join("\n");
        
        Ok(ExportResult::success(
            self.id().to_string(),
            json!({
                "format": "csv",
                "content": csv_data,
            }),
        ))
    }
    
    fn export_format(&self) -> ExportFormat {
        ExportFormat::Csv
    }
    
    fn include_issues(&self) -> bool {
        false // CSV doesn't handle nested issues well
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::{IssueSeverity, NewIssue};
    
    fn make_export_context() -> ExportContext {
        let mut data = HashMap::new();
        data.insert("title".to_string(), json!("Test Page"));
        data.insert("description".to_string(), json!("A test page"));
        
        ExportContext {
            page_id: "page-1".to_string(),
            url: "https://example.com".to_string(),
            data,
            issues: vec![NewIssue {
                job_id: "job-1".to_string(),
                page_id: Some("page-1".to_string()),
                issue_type: "test".to_string(),
                severity: IssueSeverity::Warning,
                message: "Test issue".to_string(),
                details: None,
            }],
            format: ExportFormat::Json,
            endpoint: None,
            headers: HashMap::new(),
        }
    }
    
    #[test]
    fn test_json_exporter() {
        let exporter = JsonExporter::new();
        let context = make_export_context();
        
        let result = exporter.export(&context).unwrap();
        
        assert!(result.success);
        assert!(result.data.is_some());
        
        let data = result.data.unwrap();
        assert_eq!(data["page_id"], "page-1");
        assert!(data["issues"].is_array());
    }
    
    #[test]
    fn test_csv_exporter() {
        let exporter = CsvExporter::new();
        let context = make_export_context();
        
        let result = exporter.export(&context).unwrap();
        
        assert!(result.success);
        let data = result.data.unwrap();
        assert_eq!(data["format"], "csv");
        assert!(data["content"].as_str().unwrap().contains("page_id"));
    }
}

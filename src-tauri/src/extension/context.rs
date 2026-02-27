//! Extension Context Types
//!
//! This module defines the context structs passed to extensions during execution.
//! Each phase of the pipeline has its own context type with relevant data.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::contexts::{LighthouseData, Page};

/// Context for data extraction phase.
///
/// Provides all necessary information for extracting data from HTML.
#[derive(Debug, Clone)]
pub struct ExtractionContext {
    /// The HTML content to extract data from
    pub html: String,
    
    /// The URL of the page being processed
    pub url: String,
    
    /// Unique identifier for the page
    pub page_id: String,
    
    /// Job ID for tracking
    pub job_id: String,
    
    /// Depth of the page in the crawl tree
    pub depth: i64,
    
    /// HTTP response headers
    pub headers: HashMap<String, String>,
    
    /// HTTP status code
    pub status_code: Option<i64>,
}

impl ExtractionContext {
    /// Create a new extraction context
    pub fn new(html: String, url: String, page_id: String, job_id: String) -> Self {
        Self {
            html,
            url,
            page_id,
            job_id,
            depth: 0,
            headers: HashMap::new(),
            status_code: None,
        }
    }
    
    /// Add headers to the context
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
    
    /// Set the depth
    pub fn with_depth(mut self, depth: i64) -> Self {
        self.depth = depth;
        self
    }
    
    /// Set the status code
    pub fn with_status_code(mut self, status_code: i64) -> Self {
        self.status_code = Some(status_code);
        self
    }
}

/// Context for validation/issue generation phase.
///
/// Provides page data, extracted data, and additional context for validation.
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// The page being validated
    pub page: Page,
    
    /// Raw HTML content (optional, may not be available for all pages)
    pub html: Option<String>,
    
    /// HTTP response headers
    pub headers: HashMap<String, String>,
    
    /// Data extracted during the extraction phase
    pub extracted_data: HashMap<String, serde_json::Value>,
    
    /// Lighthouse audit data if available
    pub lighthouse_data: Option<LighthouseData>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ValidationContext {
    /// Create a new validation context from a page
    pub fn new(page: Page) -> Self {
        Self {
            page,
            html: None,
            headers: HashMap::new(),
            extracted_data: HashMap::new(),
            lighthouse_data: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Add HTML content
    pub fn with_html(mut self, html: String) -> Self {
        self.html = Some(html);
        self
    }
    
    /// Add headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
    
    /// Add extracted data from the extraction phase
    pub fn with_extracted_data(mut self, data: HashMap<String, serde_json::Value>) -> Self {
        self.extracted_data = data;
        self
    }
    
    /// Add Lighthouse data
    pub fn with_lighthouse(mut self, data: LighthouseData) -> Self {
        self.lighthouse_data = Some(data);
        self
    }
    
    /// Get a specific extracted data value
    pub fn get_extracted(&self, key: &str) -> Option<&serde_json::Value> {
        self.extracted_data.get(key)
    }
}

/// Context for data export phase.
///
/// Provides extracted data and configuration for export.
#[derive(Debug, Clone)]
pub struct ExportContext {
    /// The page ID this data belongs to
    pub page_id: String,
    
    /// The URL of the page
    pub url: String,
    
    /// Extracted data to export
    pub data: HashMap<String, serde_json::Value>,
    
    /// Issues generated for this page
    pub issues: Vec<crate::contexts::NewIssue>,
    
    /// Export format requested
    pub format: ExportFormat,
    
    /// Target endpoint for webhook exports
    pub endpoint: Option<String>,
    
    /// Additional headers to include in export
    pub headers: HashMap<String, String>,
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// JSON format
    Json,
    
    /// CSV format (flattened)
    Csv,
    
    /// XML format
    Xml,
    
    /// NDJSON (newline-delimited JSON) for streaming
    Ndjson,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Json
    }
}

impl ExportContext {
    /// Create a new export context
    pub fn new(page_id: String, url: String) -> Self {
        Self {
            page_id,
            url,
            data: HashMap::new(),
            issues: Vec::new(),
            format: ExportFormat::default(),
            endpoint: None,
            headers: HashMap::new(),
        }
    }
    
    /// Add extracted data
    pub fn with_data(mut self, data: HashMap<String, serde_json::Value>) -> Self {
        self.data = data;
        self
    }
    
    /// Add issues
    pub fn with_issues(mut self, issues: Vec<crate::contexts::NewIssue>) -> Self {
        self.issues = issues;
        self
    }
    
    /// Set export format
    pub fn with_format(mut self, format: ExportFormat) -> Self {
        self.format = format;
        self
    }
    
    /// Set target endpoint
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    fn make_test_page() -> Page {
        Page {
            id: "test-page".to_string(),
            job_id: "test-job".to_string(),
            url: "https://example.com".to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: Some("text/html".to_string()),
            title: Some("Test Title".to_string()),
            meta_description: Some("Test description".to_string()),
            canonical_url: None,
            robots_meta: None,
            word_count: Some(100),
            load_time_ms: Some(1000),
            response_size_bytes: Some(512),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        }
    }
    
    #[test]
    fn test_extraction_context() {
        let ctx = ExtractionContext::new(
            "<html></html>".to_string(),
            "https://example.com".to_string(),
            "page-1".to_string(),
            "job-1".to_string(),
        )
        .with_depth(1)
        .with_status_code(200);
        
        assert_eq!(ctx.html, "<html></html>");
        assert_eq!(ctx.depth, 1);
        assert_eq!(ctx.status_code, Some(200));
    }
    
    #[test]
    fn test_validation_context() {
        let page = make_test_page();
        let ctx = ValidationContext::new(page.clone())
            .with_html("<html></html>".to_string());
        
        assert_eq!(ctx.page.id, "test-page");
        assert!(ctx.html.is_some());
    }
    
    #[test]
    fn test_export_context() {
        let ctx = ExportContext::new("page-1".to_string(), "https://example.com".to_string())
            .with_format(ExportFormat::Json);
        
        assert_eq!(ctx.page_id, "page-1");
        assert_eq!(ctx.format, ExportFormat::Json);
    }
}

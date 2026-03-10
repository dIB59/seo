//! Built-in Issue Generation Rules
//!
//! This module provides the built-in issue generators that validate pages
//! and generate SEO issues.

use anyhow::Result;

use crate::contexts::analysis::{IssueSeverity, NewIssue};
use super::super::capabilities::ExtensionCapability;
use super::super::context::ValidationContext;
use super::super::result::ValidationResult;
use super::super::traits::{Extension, ExtensionConfig, IssueGenerator};

// ============================================================================
// Title Rules
// ============================================================================

/// Rule that checks for missing page titles.
pub struct TitlePresenceRule {
    config: ExtensionConfig,
}

impl TitlePresenceRule {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("missing-title", "Missing Title")
                .with_description("Checks that pages have a title tag")
                .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                .builtin(),
        }
    }
}

impl Default for TitlePresenceRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for TitlePresenceRule {
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

impl IssueGenerator for TitlePresenceRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let has_title = context
            .page
            .title
            .as_deref()
            .is_some_and(|title| !title.is_empty());
        
        let result = if has_title {
            ValidationResult::new(self.id().to_string())
        } else {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: "Missing Title".to_string(),
                    severity: IssueSeverity::Critical,
                    message: "Page is missing a title tag".to_string(),
                    details: Some("Add a descriptive <title> tag to the page head section.".to_string()),
                })
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Critical
    }
    
    fn recommendation(&self) -> Option<&str> {
        Some("Add a descriptive <title> tag to the page head section.")
    }
}

/// Rule that checks title length.
pub struct TitleLengthRule {
    config: ExtensionConfig,
    min_length: usize,
    max_length: usize,
}

impl TitleLengthRule {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("title-length", "Title Length")
                .with_description("Checks that title length is within recommended range (30-60 characters)")
                .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                .builtin(),
            min_length: 30,
            max_length: 60,
        }
    }
}

impl Default for TitleLengthRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for TitleLengthRule {
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

impl IssueGenerator for TitleLengthRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let title = match context.page.title.as_deref() {
            Some(title) if !title.is_empty() => title,
            _ => return Ok(ValidationResult::new(self.id().to_string())), // No title - handled by presence rule
        };
        
        let len = title.len();
        
        let result = if len < self.min_length {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: "Title Too Short".to_string(),
                    severity: IssueSeverity::Warning,
                    message: format!("Title is too short ({} chars, recommend {}-{})", len, self.min_length, self.max_length),
                    details: Some("Expand the title to better describe the page content.".to_string()),
                })
        } else if len > self.max_length {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: "Title Too Long".to_string(),
                    severity: IssueSeverity::Warning,
                    message: format!("Title is too long ({} chars, recommend {}-{})", len, self.min_length, self.max_length),
                    details: Some("Shorten the title to prevent truncation in search results.".to_string()),
                })
        } else {
            ValidationResult::new(self.id().to_string())
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Warning
    }
    
    fn recommendation(&self) -> Option<&str> {
        Some("Keep title between 30-60 characters for optimal display in search results.")
    }
}

// ============================================================================
// Meta Description Rules
// ============================================================================

/// Rule that checks for missing meta description.
pub struct MetaDescriptionPresenceRule {
    config: ExtensionConfig,
}

impl MetaDescriptionPresenceRule {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("missing-meta-description", "Missing Meta Description")
                .with_description("Checks that pages have a meta description")
                .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                .builtin(),
        }
    }
}

impl Default for MetaDescriptionPresenceRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for MetaDescriptionPresenceRule {
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

impl IssueGenerator for MetaDescriptionPresenceRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let has_description = context
            .page
            .meta_description
            .as_deref()
            .is_some_and(|description| !description.is_empty());
        
        let result = if has_description {
            ValidationResult::new(self.id().to_string())
        } else {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: "Missing Meta Description".to_string(),
                    severity: IssueSeverity::Warning,
                    message: "Page is missing a meta description".to_string(),
                    details: Some("Add a meta description tag to improve click-through rates in search results.".to_string()),
                })
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Warning
    }
    
    fn recommendation(&self) -> Option<&str> {
        Some("Add a compelling meta description (70-160 characters) to improve click-through rates.")
    }
}

/// Rule that checks meta description length.
pub struct MetaDescriptionLengthRule {
    config: ExtensionConfig,
    min_length: usize,
    max_length: usize,
}

impl MetaDescriptionLengthRule {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("meta-description-length", "Meta Description Length")
                .with_description("Checks that meta description length is within recommended range (70-160 characters)")
                .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                .builtin(),
            min_length: 70,
            max_length: 160,
        }
    }
}

impl Default for MetaDescriptionLengthRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for MetaDescriptionLengthRule {
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

impl IssueGenerator for MetaDescriptionLengthRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let description = match context.page.meta_description.as_deref() {
            Some(description) if !description.is_empty() => description,
            _ => return Ok(ValidationResult::new(self.id().to_string())),
        };
        
        let len = description.len();
        
        let result = if len < self.min_length {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: "Meta Description Too Short".to_string(),
                    severity: IssueSeverity::Warning,
                    message: format!("Meta description is too short ({} chars, recommend {}-{})", len, self.min_length, self.max_length),
                    details: Some("Expand the meta description to provide more context.".to_string()),
                })
        } else if len > self.max_length {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: "Meta Description Too Long".to_string(),
                    severity: IssueSeverity::Warning,
                    message: format!("Meta description is too long ({} chars, recommend {}-{})", len, self.min_length, self.max_length),
                    details: Some("Shorten the meta description to prevent truncation.".to_string()),
                })
        } else {
            ValidationResult::new(self.id().to_string())
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Warning
    }
}

// ============================================================================
// HTTP Status Rule
// ============================================================================

/// Rule that checks for error HTTP status codes.
pub struct HttpStatusCodeRule {
    config: ExtensionConfig,
    error_codes: Vec<u16>,
}

impl HttpStatusCodeRule {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("http-error", "HTTP Error Status")
                .with_description("Checks for HTTP error status codes (4xx, 5xx)")
                .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                .builtin(),
            error_codes: vec![400, 401, 403, 404, 500, 502, 503, 504],
        }
    }
}

impl Default for HttpStatusCodeRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for HttpStatusCodeRule {
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

impl IssueGenerator for HttpStatusCodeRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let status = match context.page.status_code {
            Some(s) => s as u16,
            None => return Ok(ValidationResult::new(self.id().to_string())),
        };
        
        let result = if self.error_codes.contains(&status) {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: "HTTP Error".to_string(),
                    severity: IssueSeverity::Critical,
                    message: format!("Page returned HTTP status code {}", status),
                    details: Some(format!("Fix the HTTP {} error to make this page accessible.", status)),
                })
        } else {
            ValidationResult::new(self.id().to_string())
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Critical
    }
    
    fn recommendation(&self) -> Option<&str> {
        Some("Fix HTTP errors to ensure pages are accessible to users and search engines.")
    }
}

// ============================================================================
// Word Count Rule
// ============================================================================

/// Rule that checks for low word count.
pub struct WordCountRule {
    config: ExtensionConfig,
    min_words: usize,
}

impl WordCountRule {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("low-word-count", "Low Word Count")
                .with_description("Checks that pages have sufficient content (300+ words)")
                .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                .builtin(),
            min_words: 300,
        }
    }
}

impl Default for WordCountRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for WordCountRule {
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

impl IssueGenerator for WordCountRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let word_count = match context.page.word_count {
            Some(w) => w,
            None => return Ok(ValidationResult::new(self.id().to_string())),
        };
        
        let result = if (word_count as usize) < self.min_words {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: "Low Word Count".to_string(),
                    severity: IssueSeverity::Info,
                    message: format!("Page has low word count ({} words, recommend {}+)", word_count, self.min_words),
                    details: Some("Consider adding more content to provide value to users.".to_string()),
                })
        } else {
            ValidationResult::new(self.id().to_string())
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Info
    }
    
    fn recommendation(&self) -> Option<&str> {
        Some("Add more content (300+ words) to provide value and improve SEO.")
    }
}

// ============================================================================
// Load Time Rule
// ============================================================================

/// Rule that checks for slow page load times.
pub struct LoadTimeRule {
    config: ExtensionConfig,
    max_load_time_ms: i64,
}

impl LoadTimeRule {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("slow-load-time", "Slow Page Load")
                .with_description("Checks that pages load within acceptable time (under 3 seconds)")
                .with_capabilities(vec![ExtensionCapability::IssueGeneration])
                .builtin(),
            max_load_time_ms: 3000,
        }
    }
}

impl Default for LoadTimeRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for LoadTimeRule {
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

impl IssueGenerator for LoadTimeRule {
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let load_time = match context.page.load_time_ms {
            Some(t) => t,
            None => return Ok(ValidationResult::new(self.id().to_string())),
        };
        
        let result = if load_time > self.max_load_time_ms {
            ValidationResult::new(self.id().to_string())
                .with_issue(NewIssue {
                    job_id: context.page.job_id.clone(),
                    page_id: Some(context.page.id.clone()),
                    issue_type: "Slow Page Load".to_string(),
                    severity: IssueSeverity::Warning,
                    message: format!("Page loads slowly ({}ms, recommend under {}ms)", load_time, self.max_load_time_ms),
                    details: Some("Optimize images, minify CSS/JS, and consider using a CDN.".to_string()),
                })
        } else {
            ValidationResult::new(self.id().to_string())
        };
        
        Ok(result)
    }
    
    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Warning
    }
    
    fn recommendation(&self) -> Option<&str> {
        Some("Optimize page load time to under 3 seconds for better user experience and SEO.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::contexts::analysis::Page;
    
    fn make_test_page() -> Page {
        Page {
            id: "test-page".to_string(),
            job_id: "test-job".to_string(),
            url: "https://example.com".to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: Some("text/html".to_string()),
            title: Some("Test Title".to_string()),
            meta_description: Some("Test description for the page".to_string()),
            canonical_url: None,
            robots_meta: None,
            word_count: Some(500),
            load_time_ms: Some(1500),
            response_size_bytes: Some(512),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        }
    }
    
    #[test]
    fn test_title_presence_rule() {
        let rule = TitlePresenceRule::new();
        let mut page = make_test_page();
        let context = ValidationContext::new(page.clone());
        
        let result = rule.validate(&context).unwrap();
        assert!(!result.has_issues());
        
        page.title = None;
        let context = ValidationContext::new(page);
        let result = rule.validate(&context).unwrap();
        assert!(result.has_issues());
    }
    
    #[test]
    fn test_title_length_rule() {
        let rule = TitleLengthRule::new();
        let mut page = make_test_page();
        page.title = Some("Short".to_string());
        
        let context = ValidationContext::new(page);
        let result = rule.validate(&context).unwrap();
        assert!(result.has_issues());
    }
    
    #[test]
    fn test_http_status_rule() {
        let rule = HttpStatusCodeRule::new();
        let mut page = make_test_page();
        page.status_code = Some(404);
        
        let context = ValidationContext::new(page);
        let result = rule.validate(&context).unwrap();
        assert!(result.has_issues());
        assert_eq!(result.issues[0].severity, IssueSeverity::Critical);
    }
}

//! Custom HTML Extraction Rule Implementation
//!
//! This module provides a custom rule implementation that extracts data from HTML
//! using CSS selectors and validates the extracted data to determine if an issue exists.

use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use crate::contexts::{IssueSeverity, NewIssue, Page};
use crate::contexts::extension::domain::issue_rule::{EvaluationContext, IssueRule};

/// Configuration for HTML extraction rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlExtractionConfig {
    /// CSS selector to extract data from HTML
    pub selector: String,
    
    /// Attribute to extract (if None, extracts text content)
    pub attribute: Option<String>,
    
    /// Whether to extract multiple values
    pub multiple: bool,
    
    /// Expected value (if specified, checks if extracted value matches)
    pub expected_value: Option<String>,
    
    /// Minimum count expected (for multiple extraction)
    pub min_count: Option<usize>,
    
    /// Maximum count allowed (for multiple extraction)
    pub max_count: Option<usize>,
    
    /// Minimum length for string values
    pub min_length: Option<usize>,
    
    /// Maximum length for string values
    pub max_length: Option<usize>,
    
    /// Regex pattern to validate extracted value
    pub pattern: Option<String>,
    
    /// Whether to negate the condition (issue if condition is NOT met)
    pub negate: bool,
}

/// Custom rule that extracts data from HTML and validates it
pub struct HtmlExtractionRule {
    id: String,
    name: String,
    category: String,
    severity: IssueSeverity,
    config: HtmlExtractionConfig,
    recommendation: Option<String>,
}

impl HtmlExtractionRule {
    /// Create a new HTML extraction rule
    pub fn new(
        id: &str,
        name: &str,
        category: &str,
        severity: IssueSeverity,
        config: HtmlExtractionConfig,
        recommendation: Option<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category: category.to_string(),
            severity,
            config,
            recommendation,
        }
    }
    
    /// Extract data from HTML using the configured selector
    pub fn extract_from_html(&self, html: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html);
        
        let selector = Selector::parse(&self.config.selector)
            .map_err(|e| anyhow!("Invalid CSS selector '{}': {:?}", self.config.selector, e))?;
        
        let mut values = Vec::new();
        
        for element in document.select(&selector) {
            let value = if let Some(attr) = &self.config.attribute {
                // Extract attribute value
                element.value().attr(attr).map(|s| s.to_string())
            } else {
                // Extract text content
                let text = element.text().collect::<String>();
                let trimmed = text.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            };
            
            if let Some(v) = value {
                values.push(v);
            }
        }
        
        Ok(values)
    }
    
    /// Validate extracted values against the configured conditions
    pub fn validate_extracted(&self, values: &[String]) -> ValidationResult {
        // Handle negate case - issue if condition is NOT met
        if self.config.negate {
            return self.validate_with_negate(values);
        }
        
        // Count validation
        if let Some(min_count) = self.config.min_count {
            if values.len() < min_count {
                return ValidationResult::Invalid(ValidationReason::InsufficientCount {
                    expected: min_count,
                    actual: values.len(),
                });
            }
        }
        
        if let Some(max_count) = self.config.max_count {
            if values.len() > max_count {
                return ValidationResult::Invalid(ValidationReason::ExcessiveCount {
                    expected: max_count,
                    actual: values.len(),
                });
            }
        }
        
        // For empty extraction, if min_count is set and values is empty, it's invalid
        if values.is_empty() && self.config.min_count.is_some() {
            return ValidationResult::Invalid(ValidationReason::NoValuesExtracted);
        }
        
        // Single value validation (for first value)
        if let Some(first_value) = values.first() {
            // Length validation
            if let Some(min_length) = self.config.min_length {
                if first_value.len() < min_length {
                    return ValidationResult::Invalid(ValidationReason::TooShort {
                        min: min_length,
                        actual: first_value.len(),
                    });
                }
            }
            
            if let Some(max_length) = self.config.max_length {
                if first_value.len() > max_length {
                    return ValidationResult::Invalid(ValidationReason::TooLong {
                        max: max_length,
                        actual: first_value.len(),
                    });
                }
            }
            
            // Pattern validation
            if let Some(pattern) = &self.config.pattern {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    if !regex.is_match(first_value) {
                        return ValidationResult::Invalid(ValidationReason::PatternMismatch {
                            pattern: pattern.clone(),
                            value: first_value.clone(),
                        });
                    }
                }
            }
            
            // Expected value validation
            if let Some(expected) = &self.config.expected_value {
                if first_value != expected {
                    return ValidationResult::Invalid(ValidationReason::ValueMismatch {
                        expected: expected.clone(),
                        actual: first_value.clone(),
                    });
                }
            }
        }
        
        ValidationResult::Valid
    }
    
    /// Handle negated validation
    fn validate_with_negate(&self, values: &[String]) -> ValidationResult {
        // For negate: we want to generate an issue if the condition is NOT met
        // e.g., "should have og:title" -> issue if no og:title
        
        if values.is_empty() {
            // Negate + empty = issue (expected something but got nothing)
            return ValidationResult::Invalid(ValidationReason::NoValuesExtracted);
        }
        
        if let Some(min_count) = self.config.min_count {
            if values.len() < min_count {
                return ValidationResult::Invalid(ValidationReason::InsufficientCount {
                    expected: min_count,
                    actual: values.len(),
                });
            }
        }
        
        // If we got here with negate, condition is met (we have values)
        ValidationResult::Valid
    }
}

impl IssueRule for HtmlExtractionRule {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn category(&self) -> &str {
        &self.category
    }
    
    fn severity(&self) -> IssueSeverity {
        self.severity
    }
    
    fn evaluate(&self, page: &Page, context: &EvaluationContext) -> Option<NewIssue> {
        // Get HTML from context
        let html = match &context.html {
            Some(h) => h,
            None => return None,
        };
        
        // Extract data from HTML
        let extracted = match self.extract_from_html(html) {
            Ok(values) => values,
            Err(e) => {
                // Log error but don't generate issue for parsing errors
                tracing::debug!("HTML extraction error for rule {}: {}", self.id, e);
                return None;
            }
        };
        
        // Validate extracted data
        let validation_result = self.validate_extracted(&extracted);
        
        // Generate issue if validation failed
        if let ValidationResult::Invalid(reason) = validation_result {
            let message = reason.to_message(&self.config.selector);
            
            return Some(NewIssue {
                job_id: page.job_id.clone(),
                page_id: Some(page.id.clone()),
                issue_type: self.name.clone(),
                severity: self.severity,
                message,
                details: self.recommendation.clone(),
            });
        }
        
        None
    }
    
    fn recommendation(&self) -> Option<&str> {
        self.recommendation.as_deref()
    }
}

/// Result of validation
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    Invalid(ValidationReason),
}

/// Reason for validation failure
#[derive(Debug, Clone)]
pub enum ValidationReason {
    NoValuesExtracted,
    InsufficientCount { expected: usize, actual: usize },
    ExcessiveCount { expected: usize, actual: usize },
    TooShort { min: usize, actual: usize },
    TooLong { max: usize, actual: usize },
    PatternMismatch { pattern: String, value: String },
    ValueMismatch { expected: String, actual: String },
}

impl ValidationReason {
    /// Convert reason to user-friendly message
    pub fn to_message(&self, selector: &str) -> String {
        match self {
            Self::NoValuesExtracted => {
                format!("No content found matching selector '{}'", selector)
            }
            Self::InsufficientCount { expected, actual } => {
                format!(
                    "Expected at least {} element(s) for '{}', but found {}",
                    expected, selector, actual
                )
            }
            Self::ExcessiveCount { expected, actual } => {
                format!(
                    "Expected at most {} element(s) for '{}', but found {}",
                    expected, selector, actual
                )
            }
            Self::TooShort { min, actual } => {
                format!(
                    "Content too short ({} chars) for '{}', expected at least {}",
                    actual, selector, min
                )
            }
            Self::TooLong { max, actual } => {
                format!(
                    "Content too long ({} chars) for '{}', expected at most {}",
                    actual, selector, max
                )
            }
            Self::PatternMismatch { pattern, value } => {
                format!(
                    "Value '{}' for '{}' does not match pattern '{}'",
                    value, selector, pattern
                )
            }
            Self::ValueMismatch { expected, actual } => {
                format!(
                    "Expected '{}' for '{}', but found '{}'",
                    expected, selector, actual
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test HTML with Open Graph tags for testing
    const TEST_HTML_WITH_OG: &str = r#"
        <html>
        <head>
            <meta property="og:title" content="Test Title" />
            <meta property="og:description" content="Test Description" />
            <meta property="og:image" content="https://example.com/image.png" />
        </head>
        <body></body>
        </html>
    "#;
    
    /// Test HTML without Open Graph tags
    const TEST_HTML_WITHOUT_OG: &str = r#"
        <html>
        <head>
            <title>Test Title</title>
        </head>
        <body></body>
        </html>
    "#;
    
    /// Test HTML with multiple images
    const TEST_HTML_WITH_IMAGES: &str = r#"
        <html>
        <body>
            <img src="image1.png" />
            <img src="image2.png" />
            <img src="image3.png" />
        </body>
        </html>
    "#;
    
    fn make_test_page() -> Page {
        Page {
            id: "test-page".to_string(),
            job_id: "test-job".to_string(),
            url: "https://example.com".to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: Some("text/html".to_string()),
            title: Some("Test".to_string()),
            meta_description: Some("Description".to_string()),
            canonical_url: None,
            robots_meta: None,
            word_count: Some(100),
            load_time_ms: Some(500),
            response_size_bytes: Some(1024),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: chrono::Utc::now(),
        }
    }
    
    #[test]
    fn test_extract_og_title() {
        let rule = HtmlExtractionRule::new(
            "test-og-title",
            "OG Title Presence",
            "seo",
            IssueSeverity::Warning,
            HtmlExtractionConfig {
                selector: "meta[property=\"og:title\"]".to_string(),
                attribute: Some("content".to_string()),
                multiple: false,
                expected_value: None,
                min_count: Some(1),
                max_count: None,
                min_length: None,
                max_length: None,
                pattern: None,
                negate: false,
            },
            Some("Add an og:title meta tag".to_string()),
        );
        
        let values = rule.extract_from_html(TEST_HTML_WITH_OG).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], "Test Title");
    }
    
    #[test]
    fn test_extract_no_og_title() {
        let rule = HtmlExtractionRule::new(
            "test-og-title",
            "OG Title Presence",
            "seo",
            IssueSeverity::Warning,
            HtmlExtractionConfig {
                selector: "meta[property=\"og:title\"]".to_string(),
                attribute: Some("content".to_string()),
                multiple: false,
                expected_value: None,
                min_count: Some(1),
                max_count: None,
                min_length: None,
                max_length: None,
                pattern: None,
                negate: false,
            },
            Some("Add an og:title meta tag".to_string()),
        );
        
        let values = rule.extract_from_html(TEST_HTML_WITHOUT_OG).unwrap();
        assert!(values.is_empty());
    }
    
    #[test]
    fn test_extract_multiple_images() {
        let rule = HtmlExtractionRule::new(
            "test-images",
            "Image Count",
            "seo",
            IssueSeverity::Info,
            HtmlExtractionConfig {
                selector: "img".to_string(),
                attribute: Some("src".to_string()),
                multiple: true,
                expected_value: None,
                min_count: Some(2),
                max_count: Some(10),
                min_length: None,
                max_length: None,
                pattern: None,
                negate: false,
            },
            None,
        );
        
        let values = rule.extract_from_html(TEST_HTML_WITH_IMAGES).unwrap();
        assert_eq!(values.len(), 3);
    }
    
    #[test]
    fn test_validate_with_min_count() {
        let rule = HtmlExtractionRule::new(
            "test-images",
            "Image Count",
            "seo",
            IssueSeverity::Info,
            HtmlExtractionConfig {
                selector: "img".to_string(),
                attribute: Some("src".to_string()),
                multiple: true,
                expected_value: None,
                min_count: Some(2),
                max_count: None,
                min_length: None,
                max_length: None,
                pattern: None,
                negate: false,
            },
            None,
        );
        
        // Test with 3 images (should pass)
        let values = vec!["image1.png".to_string(), "image2.png".to_string(), "image3.png".to_string()];
        let result = rule.validate_extracted(&values);
        assert!(matches!(result, ValidationResult::Valid));
        
        // Test with 1 image (should fail)
        let values = vec!["image1.png".to_string()];
        let result = rule.validate_extracted(&values);
        assert!(matches!(result, ValidationResult::Invalid(ValidationReason::InsufficientCount { .. })));
    }
    
    #[test]
    fn test_validate_with_length() {
        let rule = HtmlExtractionRule::new(
            "test-title",
            "Title Length",
            "seo",
            IssueSeverity::Warning,
            HtmlExtractionConfig {
                selector: "title".to_string(),
                attribute: None,
                multiple: false,
                expected_value: None,
                min_count: None,
                max_count: None,
                min_length: Some(10),
                max_length: Some(60),
                pattern: None,
                negate: false,
            },
            None,
        );
        
        // Test with short title
        let values = vec!["Short".to_string()];
        let result = rule.validate_extracted(&values);
        assert!(matches!(result, ValidationResult::Invalid(ValidationReason::TooShort { .. })));
        
        // Test with valid length
        let values = vec!["This is a valid title".to_string()];
        let result = rule.validate_extracted(&values);
        assert!(matches!(result, ValidationResult::Valid));
    }
    
    #[test]
    fn test_validate_with_pattern() {
        let rule = HtmlExtractionRule::new(
            "test-url",
            "Valid URL",
            "seo",
            IssueSeverity::Warning,
            HtmlExtractionConfig {
                selector: "meta[property=\"og:image\"]".to_string(),
                attribute: Some("content".to_string()),
                multiple: false,
                expected_value: None,
                min_count: None,
                max_count: None,
                min_length: None,
                max_length: None,
                pattern: Some(r"^https://.*".to_string()),
                negate: false,
            },
            None,
        );
        
        // Test with valid URL
        let values = vec!["https://example.com/image.png".to_string()];
        let result = rule.validate_extracted(&values);
        assert!(matches!(result, ValidationResult::Valid));
        
        // Test with invalid URL
        let values = vec!["invalid-url".to_string()];
        let result = rule.validate_extracted(&values);
        assert!(matches!(result, ValidationResult::Invalid(ValidationReason::PatternMismatch { .. })));
    }
    
    #[test]
    fn test_negate_condition() {
        let rule = HtmlExtractionRule::new(
            "test-og-title",
            "OG Title Required",
            "seo",
            IssueSeverity::Warning,
            HtmlExtractionConfig {
                selector: "meta[property=\"og:title\"]".to_string(),
                attribute: Some("content".to_string()),
                multiple: false,
                expected_value: None,
                min_count: Some(1),
                max_count: None,
                min_length: None,
                max_length: None,
                pattern: None,
                negate: true, // Issue if og:title is NOT present
            },
            Some("Add an og:title meta tag".to_string()),
        );
        
        // With og:title (negate + has value = valid)
        let values = vec!["Test Title".to_string()];
        let result = rule.validate_extracted(&values);
        assert!(matches!(result, ValidationResult::Valid));
        
        // Without og:title (negate + no value = invalid)
        let values: Vec<String> = vec![];
        let result = rule.validate_extracted(&values);
        assert!(matches!(result, ValidationResult::Invalid(ValidationReason::NoValuesExtracted)));
    }
    
    #[test]
    fn test_issue_rule_evaluate_with_issue() {
        let rule = HtmlExtractionRule::new(
            "test-og-title",
            "OG Title Presence",
            "seo",
            IssueSeverity::Warning,
            HtmlExtractionConfig {
                selector: "meta[property=\"og:title\"]".to_string(),
                attribute: Some("content".to_string()),
                multiple: false,
                expected_value: None,
                min_count: Some(1),
                max_count: None,
                min_length: None,
                max_length: None,
                pattern: None,
                negate: false,
            },
            Some("Add an og:title meta tag".to_string()),
        );
        
        let page = make_test_page();
        let context = EvaluationContext::new()
            .with_html(TEST_HTML_WITHOUT_OG.to_string());
        
        // Should generate issue because og:title is missing
        let issue = rule.evaluate(&page, &context);
        assert!(issue.is_some());
        
        let issue = issue.unwrap();
        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert!(issue.message.contains("og:title"));
    }
    
    #[test]
    fn test_issue_rule_evaluate_without_issue() {
        let rule = HtmlExtractionRule::new(
            "test-og-title",
            "OG Title Presence",
            "seo",
            IssueSeverity::Warning,
            HtmlExtractionConfig {
                selector: "meta[property=\"og:title\"]".to_string(),
                attribute: Some("content".to_string()),
                multiple: false,
                expected_value: None,
                min_count: Some(1),
                max_count: None,
                min_length: None,
                max_length: None,
                pattern: None,
                negate: false,
            },
            Some("Add an og:title meta tag".to_string()),
        );
        
        let page = make_test_page();
        let context = EvaluationContext::new()
            .with_html(TEST_HTML_WITH_OG.to_string());
        
        // Should NOT generate issue because og:title is present
        let issue = rule.evaluate(&page, &context);
        assert!(issue.is_none());
    }
    
    #[test]
    fn test_invalid_selector_handling() {
        let rule = HtmlExtractionRule::new(
            "test-invalid",
            "Test Rule",
            "seo",
            IssueSeverity::Warning,
            HtmlExtractionConfig {
                selector: "".to_string(), // Empty selector should fail
                attribute: None,
                multiple: false,
                expected_value: None,
                min_count: None,
                max_count: None,
                min_length: None,
                max_length: None,
                pattern: None,
                negate: false,
            },
            None,
        );
        
        let result = rule.extract_from_html(TEST_HTML_WITH_OG);
        
        // Should return error for empty selector
        assert!(result.is_err());
    }
    
    #[test]
    fn test_html_extraction_with_mockdiscord() {
        // This test uses the mockdiscord.html file
        let html = std::fs::read_to_string("src/test_utils/mockdiscord.html")
            .expect("Failed to read mockdiscord.html");
        
        let rule = HtmlExtractionRule::new(
            "discord-title",
            "Discord Page Title",
            "seo",
            IssueSeverity::Info,
            HtmlExtractionConfig {
                selector: "title".to_string(),
                attribute: None,
                multiple: false,
                expected_value: None,
                min_count: Some(1),
                max_count: None,
                min_length: None,
                max_length: None,
                pattern: None,
                negate: false,
            },
            None,
        );
        
        let values = rule.extract_from_html(&html).unwrap();
        assert!(!values.is_empty());
        println!("Found title: {}", values[0]);
    }
}

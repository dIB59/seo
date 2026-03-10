//! Audit Check System
//!
//! This module defines the trait and implementations for SEO audit checks.
//! Audit checks contribute to the overall SEO score and provide detailed
//! feedback on specific aspects of page quality.

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use url::Url;

use crate::contexts::analysis::Page;

/// Context provided to audit checks
#[derive(Debug, Clone)]
pub struct AuditContext {
    /// The page being audited
    pub page: Page,

    /// Parsed URL
    pub url: Url,

    /// HTTP response headers
    pub response_headers: HashMap<String, String>,

    /// Raw HTML string
    pub raw_html: String,
}

impl AuditContext {
    pub fn new(page: Page, html: &str) -> Self {
        let url = Url::parse(&page.url).unwrap_or_else(|_| Url::parse("about:blank").unwrap());
        Self {
            url,
            page,
            response_headers: HashMap::new(),
            raw_html: html.to_string(),
        }
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.response_headers = headers;
        self
    }

    /// Parse HTML on demand
    pub fn parse_html(&self) -> Html {
        Html::parse_document(&self.raw_html)
    }
}

/// Result of an audit check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Whether the check passed
    pub passed: bool,

    /// Score from 0.0 to 1.0
    pub score: f64,

    /// Value extracted during check
    pub value: Option<String>,

    /// Human-readable description
    pub description: Option<String>,

    /// Additional details for debugging
    pub details: Option<serde_json::Value>,
}

impl Default for CheckResult {
    fn default() -> Self {
        Self {
            passed: true,
            score: 1.0,
            value: None,
            description: None,
            details: None,
        }
    }
}

impl CheckResult {
    pub fn pass() -> Self {
        Self::default()
    }

    pub fn fail() -> Self {
        Self {
            passed: false,
            score: 0.0,
            ..Default::default()
        }
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self.passed = score >= 0.5;
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Trait for audit checks
pub trait AuditCheck: Send + Sync {
    /// Unique key for this check (machine-readable)
    fn key(&self) -> &str;

    /// Human-readable label
    fn label(&self) -> &str;

    /// Category for grouping (seo, performance, accessibility)
    fn category(&self) -> &str;

    /// Weight in overall score calculation (0.0 to 1.0)
    fn weight(&self) -> f64 {
        1.0
    }

    /// Perform the check (synchronous for thread safety)
    fn check(&self, context: &AuditContext) -> CheckResult;

    /// Whether this check is enabled by default
    fn is_enabled_by_default(&self) -> bool {
        true
    }
}

/// Check for document title
pub struct TitleCheck;

impl TitleCheck {
    pub fn new() -> Self {
        Self
    }

    fn get_title(html: &Html) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("title").unwrap());

        html.select(selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
    }
}

impl Default for TitleCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for TitleCheck {
    fn key(&self) -> &str {
        "document_title"
    }

    fn label(&self) -> &str {
        "Document Title"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        1.0
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        let html = context.parse_html();
        let title = Self::get_title(&html);

        match title {
            Some(t) if !t.is_empty() => {
                let len = t.len();
                let (score, desc) = if len < 30 {
                    (0.5, format!("Title too short ({} chars, recommend 30-60)", len))
                } else if len > 60 {
                    (0.7, format!("Title too long ({} chars, recommend 30-60)", len))
                } else {
                    (1.0, format!("Title length is good ({} chars)", len))
                };

                CheckResult::default()
                    .with_score(score)
                    .with_value(t)
                    .with_description(desc)
            }
            _ => CheckResult::fail()
                .with_description("Missing document title"),
        }
    }
}

/// Check for meta description
pub struct MetaDescriptionCheck;

impl MetaDescriptionCheck {
    pub fn new() -> Self {
        Self
    }

    fn get_description(html: &Html) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("meta[name='description']").unwrap());

        html.select(selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string())
    }
}

impl Default for MetaDescriptionCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for MetaDescriptionCheck {
    fn key(&self) -> &str {
        "meta_description"
    }

    fn label(&self) -> &str {
        "Meta Description"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        1.0
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        let html = context.parse_html();
        let description = Self::get_description(&html);

        match description {
            Some(d) if !d.is_empty() => {
                let len = d.len();
                let (score, desc) = if len < 70 {
                    (0.5, format!("Description too short ({} chars, recommend 70-160)", len))
                } else if len > 160 {
                    (0.7, format!("Description too long ({} chars, recommend 70-160)", len))
                } else {
                    (1.0, format!("Description length is good ({} chars)", len))
                };

                CheckResult::default()
                    .with_score(score)
                    .with_value(d)
                    .with_description(desc)
            }
            _ => CheckResult::fail()
                .with_description("Missing meta description"),
        }
    }
}

/// Check for viewport meta tag
pub struct ViewportCheck;

impl ViewportCheck {
    pub fn new() -> Self {
        Self
    }

    fn get_viewport(html: &Html) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("meta[name='viewport']").unwrap());

        html.select(selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string())
    }
}

impl Default for ViewportCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for ViewportCheck {
    fn key(&self) -> &str {
        "viewport"
    }

    fn label(&self) -> &str {
        "Viewport Meta Tag"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        1.0
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        let html = context.parse_html();
        let viewport = Self::get_viewport(&html);

        match viewport {
            Some(v) if v.contains("width=device-width") => {
                CheckResult::pass()
                    .with_value(v)
                    .with_description("Viewport is properly configured")
            }
            Some(v) => {
                CheckResult::default()
                    .with_score(0.5)
                    .with_value(v)
                    .with_description("Viewport missing width=device-width")
            }
            None => CheckResult::fail()
                .with_description("Missing viewport meta tag"),
        }
    }
}

/// Check for canonical URL
pub struct CanonicalCheck;

impl CanonicalCheck {
    pub fn new() -> Self {
        Self
    }

    fn get_canonical(html: &Html) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("link[rel='canonical']").unwrap());

        html.select(selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string())
    }
}

impl Default for CanonicalCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for CanonicalCheck {
    fn key(&self) -> &str {
        "canonical"
    }

    fn label(&self) -> &str {
        "Canonical URL"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        0.8
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        let html = context.parse_html();
        let canonical = Self::get_canonical(&html);

        match canonical {
            Some(c) if !c.is_empty() => {
                let matches = c == context.url.as_str()
                    || context.url.join(&c).map(|u| u.as_str() == context.url.as_str()).unwrap_or(false);

                CheckResult::pass()
                    .with_value(c)
                    .with_description(if matches {
                        "Canonical URL matches page URL"
                    } else {
                        "Canonical URL points to different page"
                    })
            }
            _ => CheckResult::fail()
                .with_description("Missing canonical URL"),
        }
    }
}

/// Check for image alt attributes
pub struct ImageAltCheck;

impl ImageAltCheck {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ImageAltCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for ImageAltCheck {
    fn key(&self) -> &str {
        "image_alt"
    }

    fn label(&self) -> &str {
        "Image Alt Attributes"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        0.8
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("img").unwrap());

        let html = context.parse_html();
        let mut total = 0;
        let mut missing_alt = 0;

        for img in html.select(selector) {
            total += 1;
            let alt = img.value().attr("alt");
            if alt.is_none() || alt.map(|a| a.trim().is_empty()).unwrap_or(true) {
                missing_alt += 1;
            }
        }

        if total == 0 {
            return CheckResult::pass()
                .with_value("0 images")
                .with_description("No images found on page");
        }

        let with_alt = total - missing_alt;
        let score = with_alt as f64 / total as f64;
        let passed = missing_alt == 0;

        CheckResult::default()
            .with_score(score)
            .with_value(format!("{}/{} with alt", with_alt, total))
            .with_description(if missing_alt > 0 {
                format!("{} images missing alt attribute", missing_alt)
            } else {
                "All images have alt attributes".to_string()
            })
            .with_passed(passed)
    }
}

/// Check for crawlable anchors
pub struct CrawlableAnchorsCheck;

impl CrawlableAnchorsCheck {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CrawlableAnchorsCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for CrawlableAnchorsCheck {
    fn key(&self) -> &str {
        "crawlable_anchors"
    }

    fn label(&self) -> &str {
        "Crawlable Anchors"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        0.7
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("a[href]").unwrap());

        let html = context.parse_html();
        let mut total = 0;
        let mut uncrawlable = 0;

        for anchor in html.select(selector) {
            total += 1;
            let href = anchor.value().attr("href").unwrap_or("");

            if href.starts_with("javascript:")
                || (href.starts_with('#') && href.len() == 1)
                || href.is_empty()
            {
                uncrawlable += 1;
            }
        }

        if total == 0 {
            return CheckResult::pass()
                .with_value("0 links")
                .with_description("No links found on page");
        }

        let crawlable_pct = ((total - uncrawlable) as f64 / total as f64) * 100.0;
        let passed = uncrawlable == 0;
        let score = crawlable_pct / 100.0;

        CheckResult::default()
            .with_score(score)
            .with_value(format!("{}/{} crawlable", total - uncrawlable, total))
            .with_description(if uncrawlable > 0 {
                format!("{} links are not crawlable (javascript: or empty href)", uncrawlable)
            } else {
                "All links are crawlable".to_string()
            })
            .with_passed(passed)
    }
}

/// Check for descriptive link text
pub struct LinkTextCheck;

impl LinkTextCheck {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinkTextCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for LinkTextCheck {
    fn key(&self) -> &str {
        "link_text"
    }

    fn label(&self) -> &str {
        "Descriptive Link Text"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        0.6
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("a[href]").unwrap());

        static IMG_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let img_selector = IMG_SELECTOR.get_or_init(|| Selector::parse("img").unwrap());

        let bad_texts = ["click here", "read more", "learn more", "here", "link"];
        let mut total = 0;
        let mut poor_text = 0;

        let html = context.parse_html();

        for anchor in html.select(selector) {
            total += 1;
            let mut text = anchor.text().collect::<String>().trim().to_lowercase();

            // Fallbacks
            if text.is_empty() {
                if let Some(attr) = anchor.value().attr("aria-label").or_else(|| anchor.value().attr("title")) {
                    text = attr.trim().to_lowercase();
                }
            }

            if text.is_empty() {
                for img in anchor.select(img_selector) {
                    if let Some(alt) = img.value().attr("alt") {
                        if !alt.trim().is_empty() {
                            text = alt.trim().to_lowercase();
                            break;
                        }
                    }
                }
            }

            let normalized = text
                .chars()
                .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();

            if normalized.is_empty() || bad_texts.iter().any(|b| normalized.contains(b)) {
                poor_text += 1;
            }
        }

        if total == 0 {
            return CheckResult::pass()
                .with_description("No links found");
        }

        let good_pct = ((total - poor_text) as f64 / total as f64) * 100.0;
        let passed = poor_text == 0;

        CheckResult::default()
            .with_score(good_pct / 100.0)
            .with_value(format!("{}/{} with good text", total - poor_text, total))
            .with_description(if poor_text > 0 {
                format!("{} links have generic/empty text", poor_text)
            } else {
                "All links have descriptive text".to_string()
            })
            .with_passed(passed)
    }
}

/// Check for robots meta tag
pub struct RobotsMetaCheck;

impl RobotsMetaCheck {
    pub fn new() -> Self {
        Self
    }

    fn get_robots(html: &Html) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("meta[name='robots']").unwrap());

        html.select(selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_lowercase())
    }
}

impl Default for RobotsMetaCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for RobotsMetaCheck {
    fn key(&self) -> &str {
        "is_crawlable"
    }

    fn label(&self) -> &str {
        "Page is Crawlable"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        1.0
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        let html = context.parse_html();
        let robots = Self::get_robots(&html);

        match robots {
            Some(r) if r.contains("noindex") => {
                CheckResult::fail()
                    .with_value(r)
                    .with_description("Page has noindex directive")
            }
            Some(r) => {
                CheckResult::pass()
                    .with_value(r)
                    .with_description("Page is crawlable")
            }
            None => {
                CheckResult::pass()
                    .with_description("No robots meta tag (page is crawlable by default)")
            }
        }
    }
}

/// Check for hreflang tags
pub struct HreflangCheck;

impl HreflangCheck {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HreflangCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for HreflangCheck {
    fn key(&self) -> &str {
        "hreflang"
    }

    fn label(&self) -> &str {
        "Hreflang Tags"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        0.5
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("link[rel='alternate'][hreflang]").unwrap());

        let html = context.parse_html();
        let count = html.select(selector).count();

        if count > 0 {
            CheckResult::pass()
                .with_value(format!("{} hreflang tags", count))
                .with_description(format!("Found {} hreflang tags for internationalization", count))
        } else {
            CheckResult::pass()
                .with_description("No hreflang tags (optional for single-language sites)")
        }
    }
}

/// Check for HTTP status code
pub struct HttpStatusCodeCheck;

impl HttpStatusCodeCheck {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpStatusCodeCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditCheck for HttpStatusCodeCheck {
    fn key(&self) -> &str {
        "http_status_code"
    }

    fn label(&self) -> &str {
        "HTTP Status Code"
    }

    fn category(&self) -> &str {
        "seo"
    }

    fn weight(&self) -> f64 {
        1.0
    }

    fn check(&self, context: &AuditContext) -> CheckResult {
        match context.page.status_code {
            Some(code) if code >= 400 => {
                CheckResult::fail()
                    .with_value(code.to_string())
                    .with_description(format!("HTTP error status: {}", code))
            }
            Some(code) => {
                CheckResult::pass()
                    .with_value(code.to_string())
                    .with_description(format!("HTTP status: {}", code))
            }
            None => {
                CheckResult::pass()
                    .with_description("HTTP status not checked")
            }
        }
    }
}

// Helper trait implementation for CheckResult
trait WithPassed {
    fn with_passed(self, passed: bool) -> Self;
}

impl WithPassed for CheckResult {
    fn with_passed(mut self, passed: bool) -> Self {
        self.passed = passed;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_context(html: &str, url: &str) -> AuditContext {
        let page = Page {
            id: "test".to_string(),
            job_id: "job".to_string(),
            url: url.to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: Some("text/html".to_string()),
            title: None,
            meta_description: None,
            canonical_url: None,
            robots_meta: None,
            word_count: Some(100),
            load_time_ms: Some(1000),
            response_size_bytes: Some(500),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        };
        AuditContext::new(page, html)
    }

    #[test]
    fn test_title_check_pass() {
        let html = r#"<html><head><title>This is a good title with proper length</title></head></html>"#;
        let context = make_context(html, "https://example.com");

        let check = TitleCheck::new();
        let result = check.check(&context);

        assert!(result.passed);
        assert!(result.score > 0.9, "Score was {}, expected > 0.9", result.score);
    }

    #[test]
    fn test_title_check_too_short() {
        let html = r#"<html><head><title>Hi</title></head></html>"#;
        let context = make_context(html, "https://example.com");

        let check = TitleCheck::new();
        let result = check.check(&context);

        // Title "Hi" is 2 chars, which gets score 0.5, and passed = score >= 0.5
        assert!(result.score == 0.5, "Score was {}, expected 0.5", result.score);
        assert!(result.description.unwrap().contains("too short"));
    }

    #[test]
    fn test_meta_description_check_missing() {
        let html = r#"<html><head></head></html>"#;
        let context = make_context(html, "https://example.com");

        let check = MetaDescriptionCheck::new();
        let result = check.check(&context);

        assert!(!result.passed);
        assert!(result.description.unwrap().contains("Missing"));
    }

    #[test]
    fn test_viewport_check_pass() {
        let html = r#"<html><head><meta name="viewport" content="width=device-width, initial-scale=1"></head></html>"#;
        let context = make_context(html, "https://example.com");

        let check = ViewportCheck::new();
        let result = check.check(&context);

        assert!(result.passed);
    }

    #[test]
    fn test_canonical_check() {
        let html = r#"<html><head><link rel="canonical" href="https://example.com/"></head></html>"#;
        let context = make_context(html, "https://example.com/");

        let check = CanonicalCheck::new();
        let result = check.check(&context);

        assert!(result.passed);
    }

    #[test]
    fn test_image_alt_check() {
        let html = r#"<html><body><img src="test.jpg" alt="Test"><img src="no-alt.jpg"></body></html>"#;
        let context = make_context(html, "https://example.com");

        let check = ImageAltCheck::new();
        let result = check.check(&context);

        assert!(!result.passed); // One image missing alt
        assert!(result.value.unwrap().contains("1/2"));
    }
}

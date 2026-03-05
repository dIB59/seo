//! Page Data Extractor System
//!
//! This module defines the trait and implementations for extracting
//! additional data from web pages. Extractors can be added dynamically
//! to collect new types of data without modifying core code.

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::Result;

/// Data extracted from a page
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtractedData {
    /// Single text value
    Text(String),

    /// Numeric value
    Number(f64),

    /// Boolean value
    Boolean(bool),

    /// JSON value for complex data
    Json(serde_json::Value),

    /// List of strings
    List(Vec<String>),

    /// Key-value map
    KeyValue(HashMap<String, String>),

    /// List of key-value maps (for tabular data)
    Table(Vec<HashMap<String, String>>),
}

impl ExtractedData {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[String]> {
        match self {
            Self::List(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json(v) => Some(v),
            _ => None,
        }
    }
}

impl Default for ExtractedData {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

/// Trait for page data extractors
pub trait PageDataExtractor: Send + Sync {
    /// Unique identifier for this extractor
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Description of what this extractor does
    fn description(&self) -> Option<&str> {
        None
    }

    /// Extract data from HTML (synchronous for thread safety)
    fn extract(&self, html: &str, url: &str) -> Result<ExtractedData>;

    /// Database column type for storage
    fn column_type(&self) -> &str {
        "TEXT"
    }

    /// Whether this extractor requires additional processing
    fn requires_processing(&self) -> bool {
        false
    }

    /// Whether this extractor is enabled by default
    fn is_enabled_by_default(&self) -> bool {
        true
    }
}

/// Extractor that uses CSS selectors to extract data
pub struct CssSelectorExtractor {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub selector: String,
    pub attribute: Option<String>,
    pub multiple: bool,
}

impl CssSelectorExtractor {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        selector: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            selector: selector.into(),
            attribute: None,
            multiple: false,
        }
    }

    pub fn with_attribute(mut self, attr: impl Into<String>) -> Self {
        self.attribute = Some(attr.into());
        self
    }

    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    fn get_selector(&self) -> Result<Selector> {
        Selector::parse(&self.selector)
            .map_err(|e| anyhow::anyhow!("Invalid selector '{}': {:?}", self.selector, e))
    }
}

impl PageDataExtractor for CssSelectorExtractor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn extract(&self, html: &str, _url: &str) -> Result<ExtractedData> {
        let document = Html::parse_document(html);
        let selector = self.get_selector()?;

        if self.multiple {
            let values: Vec<String> = document
                .select(&selector)
                .filter_map(|el| {
                    if let Some(attr) = &self.attribute {
                        el.value().attr(attr).map(|s| s.to_string())
                    } else {
                        let text = el.text().collect::<String>();
                        let trimmed = text.trim().to_string();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed)
                        }
                    }
                })
                .collect();
            Ok(ExtractedData::List(values))
        } else {
            let value = document.select(&selector).next().and_then(|el| {
                if let Some(attr) = &self.attribute {
                    el.value().attr(attr).map(|s| s.to_string())
                } else {
                    let text = el.text().collect::<String>();
                    let trimmed = text.trim().to_string();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    }
                }
            });

            match value {
                Some(v) => Ok(ExtractedData::Text(v)),
                None => Ok(ExtractedData::Text(String::new())),
            }
        }
    }
}

/// Extractor for Open Graph meta tags
pub struct OpenGraphExtractor {
    pub id: String,
}

impl OpenGraphExtractor {
    pub fn new() -> Self {
        Self {
            id: "open-graph".to_string(),
        }
    }

    fn get_og_property(html: &Html, property: &str) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("meta[property^='og:']").unwrap()
        });

        html.select(selector)
            .find(|el| {
                el.value()
                    .attr("property")
                    .is_some_and(|p| p == property)
            })
            .and_then(|el| el.value().attr("content").map(|s| s.to_string()))
    }
}

impl Default for OpenGraphExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PageDataExtractor for OpenGraphExtractor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Open Graph Tags"
    }

    fn description(&self) -> Option<&str> {
        Some("Extracts Open Graph meta tags from the page")
    }

    fn extract(&self, html: &str, _url: &str) -> Result<ExtractedData> {
        let document = Html::parse_document(html);
        let mut og_data = HashMap::new();

        // Common Open Graph properties
        let properties = [
            "og:title",
            "og:description",
            "og:image",
            "og:url",
            "og:type",
            "og:site_name",
            "og:locale",
            "og:image:width",
            "og:image:height",
            "og:image:alt",
        ];

        for prop in properties {
            if let Some(value) = Self::get_og_property(&document, prop) {
                og_data.insert(prop.to_string(), value);
            }
        }

        Ok(ExtractedData::KeyValue(og_data))
    }

    fn column_type(&self) -> &str {
        "JSON"
    }
}

/// Extractor for Twitter Card meta tags
pub struct TwitterCardExtractor {
    pub id: String,
}

impl TwitterCardExtractor {
    pub fn new() -> Self {
        Self {
            id: "twitter-card".to_string(),
        }
    }

    fn get_twitter_property(html: &Html, name: &str) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("meta[name^='twitter:']").unwrap()
        });

        html.select(selector)
            .find(|el| {
                el.value()
                    .attr("name")
                    .is_some_and(|n| n == name)
            })
            .and_then(|el| el.value().attr("content").map(|s| s.to_string()))
    }
}

impl Default for TwitterCardExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PageDataExtractor for TwitterCardExtractor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Twitter Card Tags"
    }

    fn description(&self) -> Option<&str> {
        Some("Extracts Twitter Card meta tags from the page")
    }

    fn extract(&self, html: &str, _url: &str) -> Result<ExtractedData> {
        let document = Html::parse_document(html);
        let mut twitter_data = HashMap::new();

        let properties = [
            "twitter:card",
            "twitter:site",
            "twitter:creator",
            "twitter:title",
            "twitter:description",
            "twitter:image",
            "twitter:image:alt",
        ];

        for prop in properties {
            if let Some(value) = Self::get_twitter_property(&document, prop) {
                twitter_data.insert(prop.to_string(), value);
            }
        }

        Ok(ExtractedData::KeyValue(twitter_data))
    }

    fn column_type(&self) -> &str {
        "JSON"
    }
}

/// Extractor for href tags from the head section
pub struct HrefTagExtractor {
    pub id: String,
}

impl HrefTagExtractor {
    pub fn new() -> Self {
        Self {
            id: "href-tags".to_string(),
        }
    }
}

impl Default for HrefTagExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PageDataExtractor for HrefTagExtractor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Href Tags"
    }

    fn description(&self) -> Option<&str> {
        Some("Extracts link tags from the head section (stylesheets, icons, etc.)")
    }

    fn extract(&self, html: &str, _url: &str) -> Result<ExtractedData> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("head link[href]").unwrap()
        });

        let document = Html::parse_document(html);
        let mut href_tags = Vec::new();

        for el in document.select(selector) {
            let mut tag = HashMap::new();

            if let Some(rel) = el.value().attr("rel") {
                tag.insert("rel".to_string(), rel.to_string());
            }
            if let Some(href) = el.value().attr("href") {
                tag.insert("href".to_string(), href.to_string());
            }
            if let Some(type_) = el.value().attr("type") {
                tag.insert("type".to_string(), type_.to_string());
            }
            if let Some(sizes) = el.value().attr("sizes") {
                tag.insert("sizes".to_string(), sizes.to_string());
            }
            if let Some(media) = el.value().attr("media") {
                tag.insert("media".to_string(), media.to_string());
            }

            if !tag.is_empty() {
                href_tags.push(tag);
            }
        }

        Ok(ExtractedData::Table(href_tags))
    }

    fn column_type(&self) -> &str {
        "JSON"
    }
}

/// Extractor for keywords from page content
pub struct KeywordExtractor {
    pub id: String,
    pub min_word_length: usize,
    pub max_keywords: usize,
}

impl KeywordExtractor {
    pub fn new() -> Self {
        Self {
            id: "keywords".to_string(),
            min_word_length: 4,
            max_keywords: 20,
        }
    }

    pub fn with_min_word_length(mut self, len: usize) -> Self {
        self.min_word_length = len;
        self
    }

    pub fn with_max_keywords(mut self, max: usize) -> Self {
        self.max_keywords = max;
        self
    }

    /// Common stop words to filter out
    fn is_stop_word(word: &str) -> bool {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
            "be", "have", "has", "had", "do", "does", "did", "will", "would",
            "could", "should", "may", "might", "must", "shall", "can", "need",
            "this", "that", "these", "those", "it", "its", "they", "them",
            "their", "we", "our", "you", "your", "he", "she", "him", "her",
            "his", "i", "me", "my", "who", "which", "what", "where", "when",
            "why", "how", "all", "each", "every", "both", "few", "more", "most",
            "other", "some", "such", "no", "not", "only", "own", "same", "so",
            "than", "too", "very", "just", "also", "now", "here", "there",
        ];
        stop_words.contains(&word.to_lowercase().as_str())
    }
}

impl Default for KeywordExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PageDataExtractor for KeywordExtractor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Keywords"
    }

    fn description(&self) -> Option<&str> {
        Some("Extracts keywords from page content based on frequency")
    }

    fn extract(&self, html: &str, _url: &str) -> Result<ExtractedData> {
        static BODY_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = BODY_SELECTOR.get_or_init(|| {
            Selector::parse("body").unwrap()
        });

        let document = Html::parse_document(html);

        // Get text content from body
        let text = document
            .select(selector)
            .next()
            .map(|body| body.text().collect::<String>())
            .unwrap_or_default();

        // Count word frequencies
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for word in text.split_whitespace() {
            let word = word
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();

            if word.len() >= self.min_word_length && !Self::is_stop_word(&word) {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }

        // Sort by frequency and take top N
        let mut keywords: Vec<(String, usize)> = word_counts.into_iter().collect();
        keywords.sort_by(|a, b| b.1.cmp(&a.1));
        keywords.truncate(self.max_keywords);

        let keyword_list: Vec<String> = keywords
            .into_iter()
            .map(|(word, count)| format!("{}:{}", word, count))
            .collect();

        Ok(ExtractedData::List(keyword_list))
    }

    fn column_type(&self) -> &str {
        "JSON"
    }
}

/// Extractor for structured data (JSON-LD)
pub struct StructuredDataExtractor {
    pub id: String,
}

impl StructuredDataExtractor {
    pub fn new() -> Self {
        Self {
            id: "structured-data".to_string(),
        }
    }
}

impl Default for StructuredDataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PageDataExtractor for StructuredDataExtractor {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Structured Data"
    }

    fn description(&self) -> Option<&str> {
        Some("Extracts JSON-LD structured data from the page")
    }

    fn extract(&self, html: &str, _url: &str) -> Result<ExtractedData> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("script[type='application/ld+json']").unwrap()
        });

        let document = Html::parse_document(html);
        let mut structured_data = Vec::new();

        for el in document.select(selector) {
            let json_text = el.text().collect::<String>();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_text) {
                structured_data.push(json);
            }
        }

        if structured_data.is_empty() {
            Ok(ExtractedData::Json(serde_json::Value::Null))
        } else if structured_data.len() == 1 {
            Ok(ExtractedData::Json(structured_data.remove(0)))
        } else {
            Ok(ExtractedData::Json(serde_json::Value::Array(structured_data)))
        }
    }

    fn column_type(&self) -> &str {
        "JSON"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_selector_extractor_text() {
        let html = r#"<html><body><h1>Hello World</h1></body></html>"#;
        let extractor = CssSelectorExtractor::new("test", "Test", "h1");

        let result = extractor.extract(html, "https://example.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_text(), Some("Hello World"));
    }

    #[test]
    fn test_css_selector_extractor_attribute() {
        let html = r#"<html><body><a href="https://example.com">Link</a></body></html>"#;
        let extractor = CssSelectorExtractor::new("test", "Test", "a")
            .with_attribute("href");

        let result = extractor.extract(html, "https://example.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_text(), Some("https://example.com"));
    }

    #[test]
    fn test_css_selector_extractor_multiple() {
        let html = r#"<html><body><p>First</p><p>Second</p></body></html>"#;
        let extractor = CssSelectorExtractor::new("test", "Test", "p").multiple();

        let result = extractor.extract(html, "https://example.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_list(), Some(&["First".to_string(), "Second".to_string()][..]));
    }

    #[test]
    fn test_open_graph_extractor() {
        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="Test Title">
                <meta property="og:description" content="Test Description">
                <meta property="og:image" content="https://example.com/image.jpg">
            </head>
            </html>
        "#;

        let extractor = OpenGraphExtractor::new();
        let result = extractor.extract(html, "https://example.com");

        assert!(result.is_ok());
        if let ExtractedData::KeyValue(data) = result.unwrap() {
            assert_eq!(data.get("og:title"), Some(&"Test Title".to_string()));
            assert_eq!(data.get("og:description"), Some(&"Test Description".to_string()));
        } else {
            panic!("Expected KeyValue");
        }
    }

    #[test]
    fn test_twitter_card_extractor() {
        let html = r#"
            <html>
            <head>
                <meta name="twitter:card" content="summary_large_image">
                <meta name="twitter:title" content="Test Title">
            </head>
            </html>
        "#;

        let extractor = TwitterCardExtractor::new();
        let result = extractor.extract(html, "https://example.com");

        assert!(result.is_ok());
        if let ExtractedData::KeyValue(data) = result.unwrap() {
            assert_eq!(data.get("twitter:card"), Some(&"summary_large_image".to_string()));
        } else {
            panic!("Expected KeyValue");
        }
    }

    #[test]
    fn test_href_tag_extractor() {
        let html = r#"
            <html>
            <head>
                <link rel="stylesheet" href="/style.css" type="text/css">
                <link rel="icon" href="/favicon.ico" sizes="32x32">
            </head>
            </html>
        "#;

        let extractor = HrefTagExtractor::new();
        let result = extractor.extract(html, "https://example.com");

        assert!(result.is_ok());
        if let ExtractedData::Table(tags) = result.unwrap() {
            assert_eq!(tags.len(), 2);
            assert_eq!(tags[0].get("rel"), Some(&"stylesheet".to_string()));
            assert_eq!(tags[1].get("rel"), Some(&"icon".to_string()));
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn test_structured_data_extractor() {
        let html = r#"
            <html>
            <head>
                <script type="application/ld+json">
                {"@type": "Organization", "name": "Test"}
                </script>
            </head>
            </html>
        "#;

        let extractor = StructuredDataExtractor::new();
        let result = extractor.extract(html, "https://example.com");

        assert!(result.is_ok());
        if let ExtractedData::Json(json) = result.unwrap() {
            assert_eq!(json.get("@type").unwrap().as_str().unwrap(), "Organization");
        } else {
            panic!("Expected Json");
        }
    }
}

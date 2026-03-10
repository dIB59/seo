//! Built-in Data Extractors
//!
//! This module provides the built-in data extractors that extract
//! structured data from HTML content.

use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::Result;
use scraper::{Html, Selector};
use serde_json::json;

use super::super::capabilities::ExtensionCapability;
use super::super::context::ExtractionContext;
use super::super::result::{ExtractedValue, ExtractionMetadata, ExtractionResult};
use super::super::traits::{
    DataExtractor, Extension, ExtensionConfig, ExtractionSchema, SchemaField, SchemaFieldType,
};

// ============================================================================
// Open Graph Extractor
// ============================================================================

/// Extracts Open Graph meta tags from HTML.
pub struct OpenGraphExtractor {
    config: ExtensionConfig,
}

impl OpenGraphExtractor {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("open-graph", "Open Graph Tags")
                .with_description("Extracts Open Graph meta tags from the page")
                .with_capabilities(vec![ExtensionCapability::DataExtraction])
                .builtin(),
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

impl Extension for OpenGraphExtractor {
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
        vec![ExtensionCapability::DataExtraction]
    }
}

impl DataExtractor for OpenGraphExtractor {
    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult> {
        let document = Html::parse_document(&context.html);
        let mut data = HashMap::new();
        
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
                data.insert(prop.to_string(), ExtractedValue::Text(value));
            }
        }
        
        Ok(ExtractionResult {
            extension_id: self.id().to_string(),
            data,
            metadata: ExtractionMetadata::default(),
        })
    }
    
    fn schema(&self) -> ExtractionSchema {
        ExtractionSchema::new(vec![
            SchemaField::new("og:title", SchemaFieldType::String)
                .with_description("The title of the page as it should appear in social media"),
            SchemaField::new("og:description", SchemaFieldType::String)
                .with_description("A description of the page"),
            SchemaField::new("og:image", SchemaFieldType::String)
                .with_description("The URL of an image to represent the page"),
        ])
        .with_description("Open Graph meta tags extracted from the page")
    }
    
    fn column_type(&self) -> &str {
        "JSON"
    }
}

// ============================================================================
// Twitter Card Extractor
// ============================================================================

/// Extracts Twitter Card meta tags from HTML.
pub struct TwitterCardExtractor {
    config: ExtensionConfig,
}

impl TwitterCardExtractor {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("twitter-card", "Twitter Card Tags")
                .with_description("Extracts Twitter Card meta tags from the page")
                .with_capabilities(vec![ExtensionCapability::DataExtraction])
                .builtin(),
        }
    }
    
    fn selector() -> &'static Selector {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        SELECTOR.get_or_init(|| Selector::parse("meta[name^='twitter:']").unwrap())
    }
}

impl Default for TwitterCardExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for TwitterCardExtractor {
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
        vec![ExtensionCapability::DataExtraction]
    }
}

impl DataExtractor for TwitterCardExtractor {
    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult> {
        let document = Html::parse_document(&context.html);
        let mut data = HashMap::new();

        for element in document.select(Self::selector()) {
            let name = element
                .value()
                .attr("name")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty());
            let content = element
                .value()
                .attr("content")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty());

            if let (Some(name), Some(content)) = (name, content) {
                data.insert(name.to_string(), ExtractedValue::Text(content.to_string()));
            }
        }
        
        Ok(ExtractionResult {
            extension_id: self.id().to_string(),
            data,
            metadata: ExtractionMetadata::default(),
        })
    }
    
    fn schema(&self) -> ExtractionSchema {
        ExtractionSchema::new(vec![
            SchemaField::new("twitter:*", SchemaFieldType::String)
                .with_description("Any twitter:* meta tag name/content pair discovered on the page"),
        ])
        .with_description("Dynamically extracts all Twitter meta tags from the page")
    }
    
    fn column_type(&self) -> &str {
        "JSON"
    }
}

// ============================================================================
// Structured Data Extractor
// ============================================================================

/// Extracts JSON-LD structured data from HTML.
pub struct StructuredDataExtractor {
    config: ExtensionConfig,
}

impl StructuredDataExtractor {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("structured-data", "Structured Data")
                .with_description("Extracts JSON-LD structured data from the page")
                .with_capabilities(vec![ExtensionCapability::DataExtraction])
                .builtin(),
        }
    }
}

impl Default for StructuredDataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for StructuredDataExtractor {
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
        vec![ExtensionCapability::DataExtraction]
    }
}

impl DataExtractor for StructuredDataExtractor {
    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("script[type='application/ld+json']").unwrap()
        });
        
        let document = Html::parse_document(&context.html);
        let mut structured_data = Vec::new();
        
        for el in document.select(selector) {
            let json_text = el.text().collect::<String>();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
                structured_data.push(parsed);
            }
        }
        
        let mut data = HashMap::new();
        if !structured_data.is_empty() {
            data.insert(
                "json_ld".to_string(),
                ExtractedValue::Json(json!(structured_data)),
            );
        }
        
        Ok(ExtractionResult {
            extension_id: self.id().to_string(),
            data,
            metadata: ExtractionMetadata {
                items_count: structured_data.len(),
                ..Default::default()
            },
        })
    }
    
    fn schema(&self) -> ExtractionSchema {
        ExtractionSchema::new(vec![
            SchemaField::new("json_ld", SchemaFieldType::Array)
                .with_description("Array of JSON-LD structured data objects"),
        ])
        .with_description("JSON-LD structured data extracted from the page")
    }
    
    fn column_type(&self) -> &str {
        "JSON"
    }
}

// ============================================================================
// Href Tags Extractor
// ============================================================================

/// Extracts link tags from the head section.
pub struct HrefTagsExtractor {
    config: ExtensionConfig,
}

impl HrefTagsExtractor {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("href-tags", "Href Tags")
                .with_description("Extracts link tags from the head section (stylesheets, icons, etc.)")
                .with_capabilities(vec![ExtensionCapability::DataExtraction])
                .builtin(),
        }
    }
}

impl Default for HrefTagsExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for HrefTagsExtractor {
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
        vec![ExtensionCapability::DataExtraction]
    }
}

impl DataExtractor for HrefTagsExtractor {
    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("head link[href]").unwrap()
        });
        
        let document = Html::parse_document(&context.html);
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
        
        let mut data = HashMap::new();
        data.insert("links".to_string(), ExtractedValue::Table(href_tags));
        
        Ok(ExtractionResult {
            extension_id: self.id().to_string(),
            data,
            metadata: ExtractionMetadata::default(),
        })
    }
    
    fn schema(&self) -> ExtractionSchema {
        ExtractionSchema::new(vec![
            SchemaField::new("links", SchemaFieldType::Array)
                .with_description("Array of link tags with rel, href, type, sizes, and media attributes"),
        ])
        .with_description("Link tags extracted from the page head")
    }
    
    fn column_type(&self) -> &str {
        "JSON"
    }
}

// ============================================================================
// Keywords Extractor
// ============================================================================

/// Extracts keywords from page content based on frequency.
pub struct KeywordsExtractor {
    config: ExtensionConfig,
    min_word_length: usize,
    max_keywords: usize,
}

impl KeywordsExtractor {
    pub fn new() -> Self {
        Self {
            config: ExtensionConfig::new("keywords", "Keywords")
                .with_description("Extracts keywords from page content based on frequency")
                .with_capabilities(vec![ExtensionCapability::DataExtraction])
                .builtin(),
            min_word_length: 4,
            max_keywords: 20,
        }
    }
    
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

impl Default for KeywordsExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Extension for KeywordsExtractor {
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
        vec![ExtensionCapability::DataExtraction]
    }
}

impl DataExtractor for KeywordsExtractor {
    fn extract(&self, context: &ExtractionContext) -> Result<ExtractionResult> {
        static BODY_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = BODY_SELECTOR.get_or_init(|| {
            Selector::parse("body").unwrap()
        });
        
        let document = Html::parse_document(&context.html);
        
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
        
        let mut data = HashMap::new();
        data.insert("keywords".to_string(), ExtractedValue::List(keyword_list));
        
        Ok(ExtractionResult {
            extension_id: self.id().to_string(),
            data,
            metadata: ExtractionMetadata::default(),
        })
    }
    
    fn schema(&self) -> ExtractionSchema {
        ExtractionSchema::new(vec![
            SchemaField::new("keywords", SchemaFieldType::Array)
                .with_description("Top keywords with frequency counts (format: word:count)"),
        ])
        .with_description("Keywords extracted from page content")
    }
    
    fn column_type(&self) -> &str {
        "JSON"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_open_graph_extractor() {
        let html = r#"
            <html>
                <head>
                    <meta property="og:title" content="Test Title" />
                    <meta property="og:description" content="Test Description" />
                </head>
            </html>
        "#;
        
        let context = ExtractionContext::new(
            html.to_string(),
            "https://example.com".to_string(),
            "page-1".to_string(),
            "job-1".to_string(),
        );
        
        let extractor = OpenGraphExtractor::new();
        let result = extractor.extract(&context).unwrap();
        
        assert_eq!(result.extension_id, "open-graph");
        assert!(result.data.contains_key("og:title"));
        assert!(result.data.contains_key("og:description"));
    }
    
    #[test]
    fn test_structured_data_extractor() {
        let html = r#"
            <html>
                <head>
                    <script type="application/ld+json">
                    {"@type": "Article", "headline": "Test"}
                    </script>
                </head>
            </html>
        "#;
        
        let context = ExtractionContext::new(
            html.to_string(),
            "https://example.com".to_string(),
            "page-1".to_string(),
            "job-1".to_string(),
        );
        
        let extractor = StructuredDataExtractor::new();
        let result = extractor.extract(&context).unwrap();
        
        assert_eq!(result.extension_id, "structured-data");
        assert!(result.data.contains_key("json_ld"));
    }
}

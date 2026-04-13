use std::collections::HashMap;
use serde_json::Value;

pub mod selector;

/// Configuration for a single CSS-selector-based extractor.
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    /// The tag under which the result is stored in `extracted_data`.
    /// Matches `CustomExtractor.tag` for user-defined extractors.
    pub tag: String,
    /// CSS selector to match elements.
    pub selector: String,
    /// HTML attribute to read. `None` means use the element's text content.
    pub attribute: Option<String>,
    /// Collect all matches (`true`) or only the first (`false`).
    pub multiple: bool,
}

impl ExtractorConfig {
    pub fn text(tag: &str, selector: &str) -> Self {
        Self { tag: tag.into(), selector: selector.into(), attribute: None, multiple: false }
    }

    pub fn attr(tag: &str, selector: &str, attribute: &str) -> Self {
        Self {
            tag: tag.into(),
            selector: selector.into(),
            attribute: Some(attribute.into()),
            multiple: false,
        }
    }

    pub fn multi_text(tag: &str, selector: &str) -> Self {
        Self { tag: tag.into(), selector: selector.into(), attribute: None, multiple: true }
    }

    pub fn multi_attr(tag: &str, selector: &str, attribute: &str) -> Self {
        Self {
            tag: tag.into(),
            selector: selector.into(),
            attribute: Some(attribute.into()),
            multiple: true,
        }
    }
}

/// Extracts structured data from raw HTML.
pub trait DataExtractor: Send + Sync {
    fn id(&self) -> &str;
    fn extract(&self, html: &str) -> HashMap<String, Value>;
}

/// Runs all registered extractors against raw HTML and merges their results.
#[derive(Default)]
pub struct ExtractorRegistry {
    extractors: Vec<Box<dyn DataExtractor>>,
}

impl ExtractorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, extractor: Box<dyn DataExtractor>) {
        self.extractors.push(extractor);
    }

    /// Run all extractors and merge results. Later extractors win on tag conflicts.
    pub fn run(&self, html: &str) -> HashMap<String, Value> {
        let mut result = HashMap::new();
        for extractor in &self.extractors {
            result.extend(extractor.extract(html));
        }
        result
    }

    pub fn len(&self) -> usize {
        self.extractors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.extractors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::selector::SelectorExtractor;

    const HTML: &str = r#"
        <html>
          <head>
            <title>Test Page</title>
            <meta property="og:title" content="OG Title" />
            <meta property="og:description" content="OG Desc" />
          </head>
          <body>
            <h1>Main Heading</h1>
            <h2>Sub Heading One</h2>
            <h2>Sub Heading Two</h2>
            <a href="/page1">Link One</a>
            <a href="/page2">Link Two</a>
          </body>
        </html>
    "#;

    #[test]
    fn empty_registry_returns_empty_map() {
        let registry = ExtractorRegistry::new();
        let result = registry.run(HTML);
        assert!(result.is_empty());
    }

    #[test]
    fn registry_merges_results_from_multiple_extractors() {
        let mut registry = ExtractorRegistry::new();
        registry.register(Box::new(SelectorExtractor::new(
            ExtractorConfig::text("title", "title"),
        )));
        registry.register(Box::new(SelectorExtractor::new(
            ExtractorConfig::text("h1", "h1"),
        )));

        let result = registry.run(HTML);
        assert!(result.contains_key("title"));
        assert!(result.contains_key("h1"));
    }

    #[test]
    fn later_extractor_wins_on_tag_conflict() {
        let mut registry = ExtractorRegistry::new();
        registry.register(Box::new(SelectorExtractor::new(
            ExtractorConfig::text("heading", "h1"),
        )));
        registry.register(Box::new(SelectorExtractor::new(
            ExtractorConfig::text("heading", "h2"),
        )));

        let result = registry.run(HTML);
        // h2 extractor registered last — its value wins
        let val = result["heading"].as_str().unwrap();
        assert_eq!(val, "Sub Heading One");
    }

    #[test]
    fn extractor_config_text_constructor() {
        let cfg = ExtractorConfig::text("k", "h1");
        assert_eq!(cfg.tag, "k");
        assert_eq!(cfg.selector, "h1");
        assert!(cfg.attribute.is_none());
        assert!(!cfg.multiple);
    }

    #[test]
    fn extractor_config_attr_constructor() {
        let cfg = ExtractorConfig::attr("og_title", "meta[property='og:title']", "content");
        assert_eq!(cfg.attribute, Some("content".into()));
        assert!(!cfg.multiple);
    }

    #[test]
    fn extractor_config_multi_constructors() {
        let cfg = ExtractorConfig::multi_text("h2s", "h2");
        assert!(cfg.multiple);
        let cfg = ExtractorConfig::multi_attr("hrefs", "a", "href");
        assert!(cfg.multiple);
        assert_eq!(cfg.attribute, Some("href".into()));
    }
}

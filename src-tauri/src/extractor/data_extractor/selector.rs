use std::collections::HashMap;
use scraper::{Html, Selector};
use serde_json::Value;

use super::{DataExtractor, ExtractorConfig};

/// Extracts data from HTML using a CSS selector.
///
/// - Single mode: returns the first matching element's text or attribute as a JSON string.
/// - Multiple mode: returns all matching elements as a JSON array of strings.
/// - If no element matches, the key is omitted from the result (not inserted as null).
///
/// The CSS selector is parsed once at construction time and reused for
/// every `extract` call. Previously the selector was re-parsed per page,
/// which is wasted work in a multi-page crawl. An invalid selector is
/// stored as `None` and silently produces an empty result, matching the
/// previous fall-through behavior.
pub struct SelectorExtractor {
    config: ExtractorConfig,
    parsed_selector: Option<Selector>,
}

impl SelectorExtractor {
    pub fn new(config: ExtractorConfig) -> Self {
        let parsed_selector = Selector::parse(&config.selector).ok();
        Self {
            config,
            parsed_selector,
        }
    }

    fn read_element(el: scraper::ElementRef, attribute: Option<&str>) -> Option<String> {
        match attribute {
            Some(attr) => el.value().attr(attr).map(|s| s.trim().to_string()),
            None => {
                let text = el.text().collect::<String>();
                let trimmed = text.trim().to_string();
                if trimmed.is_empty() { None } else { Some(trimmed) }
            }
        }
    }
}

impl DataExtractor for SelectorExtractor {
    fn id(&self) -> &str {
        &self.config.tag
    }

    fn extract(&self, html: &str) -> HashMap<String, Value> {
        let Some(selector) = self.parsed_selector.as_ref() else {
            return HashMap::new();
        };

        let document = Html::parse_document(html);
        let attribute = self.config.attribute.as_deref();

        if self.config.multiple {
            let values: Vec<Value> = document
                .select(selector)
                .filter_map(|el| Self::read_element(el, attribute))
                .map(Value::String)
                .collect();

            if values.is_empty() {
                return HashMap::new();
            }

            let mut result = HashMap::new();
            result.insert(self.config.tag.clone(), Value::Array(values));
            result
        } else {
            let value = document
                .select(selector)
                .find_map(|el| Self::read_element(el, attribute));

            match value {
                Some(v) => {
                    let mut result = HashMap::new();
                    result.insert(self.config.tag.clone(), Value::String(v));
                    result
                }
                None => HashMap::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HTML: &str = r#"
        <html>
          <head>
            <title>Test Page Title</title>
            <meta property="og:title" content="OG Title Value" />
            <meta property="og:description" content="OG Description Value" />
          </head>
          <body>
            <h1>Main Heading</h1>
            <h2>First H2</h2>
            <h2>Second H2</h2>
            <a href="/one">Link One</a>
            <a href="/two">Link Two</a>
            <img src="img.jpg" alt="An image" />
            <p>  whitespace  </p>
          </body>
        </html>
    "#;

    fn extract(config: ExtractorConfig) -> HashMap<String, Value> {
        SelectorExtractor::new(config).extract(HTML)
    }

    // --- Single text extraction ---

    #[test]
    fn extracts_title_text() {
        let result = extract(ExtractorConfig::text("title", "title"));
        assert_eq!(result["title"], Value::String("Test Page Title".into()));
    }

    #[test]
    fn extracts_h1_text() {
        let result = extract(ExtractorConfig::text("h1", "h1"));
        assert_eq!(result["h1"], Value::String("Main Heading".into()));
    }

    #[test]
    fn single_mode_returns_first_match_only() {
        let result = extract(ExtractorConfig::text("h2", "h2"));
        assert_eq!(result["h2"], Value::String("First H2".into()));
    }

    #[test]
    fn trims_whitespace_from_text() {
        let result = extract(ExtractorConfig::text("para", "p"));
        assert_eq!(result["para"], Value::String("whitespace".into()));
    }

    // --- Single attribute extraction ---

    #[test]
    fn extracts_og_title_attribute() {
        let result = extract(ExtractorConfig::attr(
            "og_title",
            "meta[property='og:title']",
            "content",
        ));
        assert_eq!(result["og_title"], Value::String("OG Title Value".into()));
    }

    #[test]
    fn extracts_img_alt_attribute() {
        let result = extract(ExtractorConfig::attr("img_alt", "img", "alt"));
        assert_eq!(result["img_alt"], Value::String("An image".into()));
    }

    #[test]
    fn extracts_first_href_attribute() {
        let result = extract(ExtractorConfig::attr("first_link", "a", "href"));
        assert_eq!(result["first_link"], Value::String("/one".into()));
    }

    // --- Multiple text extraction ---

    #[test]
    fn multi_text_collects_all_h2s() {
        let result = extract(ExtractorConfig::multi_text("h2s", "h2"));
        let arr = result["h2s"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], Value::String("First H2".into()));
        assert_eq!(arr[1], Value::String("Second H2".into()));
    }

    // --- Multiple attribute extraction ---

    #[test]
    fn multi_attr_collects_all_hrefs() {
        let result = extract(ExtractorConfig::multi_attr("hrefs", "a", "href"));
        let arr = result["hrefs"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert!(arr.contains(&Value::String("/one".into())));
        assert!(arr.contains(&Value::String("/two".into())));
    }

    // --- Missing elements ---

    #[test]
    fn missing_element_returns_empty_map_single() {
        let result = extract(ExtractorConfig::text("nope", "section.nonexistent"));
        assert!(result.is_empty());
    }

    #[test]
    fn missing_element_returns_empty_map_multiple() {
        let result = extract(ExtractorConfig::multi_text("nope", "section.nonexistent"));
        assert!(result.is_empty());
    }

    // --- Missing attribute ---

    #[test]
    fn missing_attribute_skips_element() {
        // <title> has no "content" attribute — should return empty
        let result = extract(ExtractorConfig::attr("nope", "title", "content"));
        assert!(result.is_empty());
    }

    // --- Invalid selector ---

    #[test]
    fn invalid_selector_returns_empty_map() {
        let result = extract(ExtractorConfig::text("bad", "[[invalid]]"));
        assert!(result.is_empty());
    }

    // --- Key naming ---

    #[test]
    fn result_uses_configured_key_not_selector() {
        let result = extract(ExtractorConfig::text("my_custom_key", "title"));
        assert!(result.contains_key("my_custom_key"));
        assert!(!result.contains_key("title"));
    }

    // --- Empty text content ---

    #[test]
    fn empty_text_element_skipped_in_single_mode() {
        let html = "<html><body><p></p></body></html>";
        let result = SelectorExtractor::new(ExtractorConfig::text("p", "p")).extract(html);
        assert!(result.is_empty());
    }

    #[test]
    fn empty_text_elements_filtered_in_multi_mode() {
        let html = "<html><body><p>has text</p><p></p><p>more text</p></body></html>";
        let result = SelectorExtractor::new(ExtractorConfig::multi_text("ps", "p")).extract(html);
        let arr = result["ps"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }
}

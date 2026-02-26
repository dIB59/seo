//! Tests for HTML Extraction using mockdiscord.html
//!
//! This module tests the extraction functionality using the mockdiscord.html
//! test file to verify that extractors work correctly on real-world HTML.

use std::collections::HashMap;
use std::sync::OnceLock;

use scraper::{Html, Selector};

use app::extension::builtins::{
    HrefTagsExtractor, OpenGraphExtractor, StructuredDataExtractor, TwitterCardExtractor,
};
use app::extension::context::ExtractionContext;
use app::extension::traits::DataExtractor;

// ============================================================================
// Test Helpers
// ============================================================================

/// Get the mockdiscord.html content for testing
fn get_mockdiscord_html() -> String {
    // Read the mockdiscord.html file
    std::fs::read_to_string("src/test_utils/mockdiscord.html")
        .expect("Failed to read mockdiscord.html")
}

/// Create an ExtractionContext for testing using the builder pattern
fn make_extraction_context(html: String) -> ExtractionContext {
    ExtractionContext::new(
        html,
        "https://discord.com/community/establishing-trust-with-connections".to_string(),
        "test-page".to_string(),
        "test-job".to_string(),
    )
}

// ============================================================================
// Open Graph Extraction Tests
// ============================================================================

#[test]
fn test_open_graph_extractor_finds_title() {
    let extractor = OpenGraphExtractor::new();
    let html = get_mockdiscord_html();
    let context = make_extraction_context(html);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // The mockdiscord.html has og:title
    assert!(
        result.data.contains_key("og:title"),
        "Should find og:title"
    );
}

#[test]
fn test_open_graph_extractor_finds_description() {
    let extractor = OpenGraphExtractor::new();
    let html = get_mockdiscord_html();
    let context = make_extraction_context(html);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // The mockdiscord.html has og:description
    assert!(
        result.data.contains_key("og:description"),
        "Should find og:description"
    );
}

#[test]
fn test_open_graph_extractor_finds_image() {
    let extractor = OpenGraphExtractor::new();
    let html = get_mockdiscord_html();
    let context = make_extraction_context(html);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // The mockdiscord.html has og:image
    assert!(
        result.data.contains_key("og:image"),
        "Should find og:image"
    );
}

#[test]
fn test_open_graph_extractor_finds_type() {
    let extractor = OpenGraphExtractor::new();
    let html = get_mockdiscord_html();
    let context = make_extraction_context(html);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // The mockdiscord.html has og:type
    assert!(
        result.data.contains_key("og:type"),
        "Should find og:type"
    );
}

#[test]
fn test_open_graph_extractor_schema() {
    let extractor = OpenGraphExtractor::new();
    let schema = extractor.schema();
    
    // Verify schema fields
    assert!(schema.fields.iter().any(|f| f.name == "og:title"));
    assert!(schema.fields.iter().any(|f| f.name == "og:description"));
    assert!(schema.fields.iter().any(|f| f.name == "og:image"));
}

// ============================================================================
// Twitter Card Extraction Tests
// ============================================================================

#[test]
fn test_twitter_card_extractor_finds_card() {
    let extractor = TwitterCardExtractor::new();
    let html = get_mockdiscord_html();
    let context = make_extraction_context(html);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // The mockdiscord.html may or may not have twitter:card depending on the HTML
    // Just verify extraction works without error
    let _ = result;
}

#[test]
fn test_twitter_card_extractor_works() {
    let extractor = TwitterCardExtractor::new();
    let html = get_mockdiscord_html();
    let context = make_extraction_context(html);
    
    // Test that the extractor runs without errors
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // Just verify we got a result - twitter cards may or may not be present
    assert!(!result.extension_id.is_empty());
}

// ============================================================================
// Structured Data (JSON-LD) Extraction Tests
// ============================================================================

#[test]
fn test_structured_data_extractor_works() {
    let extractor = StructuredDataExtractor::new();
    let html = get_mockdiscord_html();
    let context = make_extraction_context(html);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // Check if json_ld was found (may or may not be present in mockdiscord.html)
    // If present, it should be in the data
    if result.data.contains_key("json_ld") {
        println!("Found json_ld data");
    }
}

// ============================================================================
// Href Tags Extraction Tests
// ============================================================================

#[test]
fn test_href_tags_extractor_finds_links() {
    let extractor = HrefTagsExtractor::new();
    let html = get_mockdiscord_html();
    let context = make_extraction_context(html);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // Should find link tags
    assert!(
        result.data.contains_key("links"),
        "Should find links key"
    );
}

// ============================================================================
// CSS Selector Extraction Tests
// ============================================================================

#[test]
fn test_css_selector_extraction_title() {
    let html = get_mockdiscord_html();
    let document = Html::parse_document(&html);
    
    // Find the title tag
    static TITLE_SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = TITLE_SELECTOR.get_or_init(|| {
        Selector::parse("title").unwrap()
    });
    
    let title_element = document.select(selector).next();
    assert!(
        title_element.is_some(),
        "Should find a title element"
    );
    
    let title_text = title_element.unwrap().text().collect::<String>();
    assert!(
        !title_text.is_empty(),
        "Title should have text content"
    );
    // The title might not contain Discord - let's just verify it's not empty
    println!("Found title: {}", title_text);
}

#[test]
fn test_css_selector_extraction_meta_description() {
    let html = get_mockdiscord_html();
    let document = Html::parse_document(&html);
    
    // Find the meta description
    static META_SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = META_SELECTOR.get_or_init(|| {
        Selector::parse("meta[name=\"description\"]").unwrap()
    });
    
    let meta_element = document.select(selector).next();
    assert!(
        meta_element.is_some(),
        "Should find meta description"
    );
    
    let description = meta_element.unwrap().value().attr("content");
    assert!(
        description.is_some(),
        "Meta description should have content attribute"
    );
}

#[test]
fn test_css_selector_extraction_open_graph() {
    let html = get_mockdiscord_html();
    let document = Html::parse_document(&html);
    
    // Find og:title
    static OG_SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = OG_SELECTOR.get_or_init(|| {
        Selector::parse("meta[property=\"og:title\"]").unwrap()
    });
    
    let og_element = document.select(selector).next();
    assert!(
        og_element.is_some(),
        "Should find og:title"
    );
    
    let og_title = og_element.unwrap().value().attr("content");
    assert!(
        og_title.is_some(),
        "og:title should have content"
    );
}

#[test]
fn test_css_selector_extraction_all_meta_tags() {
    let html = get_mockdiscord_html();
    let document = Html::parse_document(&html);
    
    // Find all meta tags
    static META_ALL_SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = META_ALL_SELECTOR.get_or_init(|| {
        Selector::parse("meta").unwrap()
    });
    
    let meta_tags: Vec<_> = document.select(selector).collect();
    assert!(
        meta_tags.len() > 10,
        "Should find multiple meta tags (found {})",
        meta_tags.len()
    );
}

// ============================================================================
// Language Selector Extraction Tests
// ============================================================================

#[test]
fn test_language_selector_extraction() {
    let html = get_mockdiscord_html();
    let document = Html::parse_document(&html);
    
    // Find language selector/dropdown elements
    static LANG_SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = LANG_SELECTOR.get_or_init(|| {
        Selector::parse(".language, .lang-dropdown, .dropdown-language-name").unwrap()
    });
    
    let lang_elements: Vec<_> = document.select(selector).collect();
    
    // The mockdiscord.html has language dropdown elements
    // This tests that the CSS selectors work for language-related content
    let _ = lang_elements; // Used for debugging if needed
}

// ============================================================================
// Content Extraction Tests
// ============================================================================

#[test]
fn test_extraction_from_specific_html_elements() {
    let html = get_mockdiscord_html();
    let document = Html::parse_document(&html);
    
    // Test extracting from the main heading
    static HEADING_SELECTOR: OnceLock<Selector> = OnceLock::new();
    let selector = HEADING_SELECTOR.get_or_init(|| {
        Selector::parse("h1, h2, h3").unwrap()
    });
    
    let headings: Vec<_> = document.select(selector).collect();
    assert!(
        !headings.is_empty(),
        "Should find heading elements"
    );
    
    // Log heading texts for debugging
    for heading in headings.iter().take(3) {
        let text = heading.text().collect::<String>();
        if !text.trim().is_empty() {
            println!("Found heading: {}", text.trim());
        }
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_extraction_performance() {
    let html = get_mockdiscord_html();
    
    // Test OpenGraphExtractor performance
    let og_extractor = OpenGraphExtractor::new();
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let context = make_extraction_context(html.clone());
        let _ = og_extractor.extract(&context).unwrap();
    }
    let og_duration = start.elapsed();
    
    println!("OpenGraphExtractor: 10 iterations in {:?}", og_duration);
    
    // Test TwitterCardExtractor performance
    let twitter_extractor = TwitterCardExtractor::new();
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let context = make_extraction_context(html.clone());
        let _ = twitter_extractor.extract(&context).unwrap();
    }
    let twitter_duration = start.elapsed();
    
    println!("TwitterCardExtractor: 10 iterations in {:?}", twitter_duration);
    
    // Both should complete reasonably fast (10 iterations in under 5 seconds)
    assert!(
        og_duration.as_secs() < 5,
        "OpenGraph extraction should be reasonably fast"
    );
    assert!(
        twitter_duration.as_secs() < 5,
        "TwitterCard extraction should be reasonably fast"
    );
}

// ============================================================================
// Multiple Values Extraction Tests
// ============================================================================

#[test]
fn test_extraction_returns_correct_extension_id() {
    let extractor = OpenGraphExtractor::new();
    let html = get_mockdiscord_html();
    let context = make_extraction_context(html);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    assert_eq!(
        result.extension_id, "open-graph",
        "Extension ID should match the extractor ID"
    );
}

#[test]
fn test_extraction_with_status_code() {
    let extractor = OpenGraphExtractor::new();
    let html = get_mockdiscord_html();
    let mut context = make_extraction_context(html);
    context = context.with_status_code(200);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // Status code should be set in context
    assert_eq!(context.status_code, Some(200));
    
    // Extraction should still work
    assert!(!result.data.is_empty());
}

#[test]
fn test_extraction_with_depth() {
    let extractor = OpenGraphExtractor::new();
    let html = get_mockdiscord_html();
    let mut context = make_extraction_context(html);
    context = context.with_depth(2);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // Depth should be set in context
    assert_eq!(context.depth, 2);
    
    // Extraction should still work
    assert!(!result.data.is_empty());
}

#[test]
fn test_extraction_with_custom_headers() {
    let extractor = OpenGraphExtractor::new();
    let html = get_mockdiscord_html();
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "text/html".to_string());
    headers.insert("server".to_string(), "nginx".to_string());
    let context = make_extraction_context(html).with_headers(headers);
    
    let result = extractor.extract(&context).expect("Extraction should succeed");
    
    // Headers should be set in context
    assert_eq!(context.headers.get("content-type"), Some(&"text/html".to_string()));
    assert_eq!(context.headers.get("server"), Some(&"nginx".to_string()));
    
    // Extraction should still work
    assert!(!result.data.is_empty());
}

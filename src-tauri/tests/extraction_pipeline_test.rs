//! Integration tests for the custom extractor pipeline.
//!
//! Tests the full data path:
//!   ExtensionRepository (DB) → ExtractorRegistry → HTML → Page.extracted_data → PageRepository (DB)

use app::{
    contexts::analysis::Page,
    contexts::extension::CustomExtractorParams,
    extractor::data_extractor::{ExtractorConfig, ExtractorRegistry},
    extractor::data_extractor::selector::SelectorExtractor,
    repository::{sqlite_extension_repo, sqlite_page_repo},
};
use chrono::Utc;
use serde_json::Value;
use sqlx::SqlitePool;
async fn setup_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

fn sample_html() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<head>
  <title>Integration Test Page</title>
  <meta property="og:title" content="OG Test Title" />
  <meta property="og:image" content="https://example.com/image.jpg" />
  <link rel="canonical" href="https://example.com/page" />
  <link rel="alternate" hreflang="en-US" href="https://example.com/en/" />
  <link rel="alternate" hreflang="fr" href="https://example.com/fr/" />
</head>
<body>
  <h1>Main Heading</h1>
  <h2>Sub Heading One</h2>
  <h2>Sub Heading Two</h2>
</body>
</html>"#
}

// ---------------------------------------------------------------------------
// Layer 1: ExtractorRegistry runs correctly without touching the DB
// ---------------------------------------------------------------------------

#[test]
fn extractor_registry_produces_data_from_html() {
    let mut registry = ExtractorRegistry::new();
    registry.register(Box::new(SelectorExtractor::new(ExtractorConfig {
        key: "og_title".into(),
        selector: "meta[property='og:title']".into(),
        attribute: Some("content".into()),
        multiple: false,
    })));
    registry.register(Box::new(SelectorExtractor::new(ExtractorConfig {
        key: "hreflang".into(),
        selector: "link[rel='alternate'][hreflang]".into(),
        attribute: Some("hreflang".into()),
        multiple: true,
    })));

    let result = registry.run(sample_html());

    assert_eq!(
        result.get("og_title"),
        Some(&Value::String("OG Test Title".into())),
        "og_title should be extracted"
    );

    let hreflang = result.get("hreflang").expect("hreflang should be present");
    let arr = hreflang.as_array().expect("hreflang should be an array");
    assert_eq!(arr.len(), 2, "should have 2 hreflang values");
    assert!(arr.contains(&Value::String("en-US".into())));
    assert!(arr.contains(&Value::String("fr".into())));
}

// ---------------------------------------------------------------------------
// Layer 2: ExtensionRepository stores and retrieves custom extractors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn extension_repo_persists_and_retrieves_extractors() {
    let pool = setup_db().await;
    let repo = sqlite_extension_repo(pool.clone());

    // Create two extractors
    repo.create_extractor(&CustomExtractorParams {
        name: "OG Title".into(),
        key: "og_title".into(),
        selector: "meta[property='og:title']".into(),
        attribute: Some("content".into()),
        multiple: false,
        enabled: true,
    })
    .await
    .expect("create og_title extractor");

    repo.create_extractor(&CustomExtractorParams {
        name: "Hreflang (disabled)".into(),
        key: "hreflang".into(),
        selector: "link[rel='alternate'][hreflang]".into(),
        attribute: Some("hreflang".into()),
        multiple: true,
        enabled: false, // disabled — should not appear in list_enabled
    })
    .await
    .expect("create hreflang extractor");

    // list_extractors returns all
    let all = repo.list_extractors().await.expect("list_extractors");
    assert_eq!(all.len(), 2, "list_extractors should return 2");

    // list_enabled_extractors returns only the enabled one
    let enabled = repo.list_enabled_extractors().await.expect("list_enabled_extractors");
    assert_eq!(enabled.len(), 1, "only 1 extractor should be enabled");
    assert_eq!(enabled[0].key, "og_title");
    assert_eq!(enabled[0].selector, "meta[property='og:title']");
    assert_eq!(enabled[0].attribute, Some("content".into()));
    assert!(!enabled[0].multiple);
}

// ---------------------------------------------------------------------------
// Layer 3: DB → ExtractorRegistry → HTML produces correct output
// ---------------------------------------------------------------------------

#[tokio::test]
async fn registry_built_from_db_extracts_correct_data() {
    let pool = setup_db().await;
    let repo = sqlite_extension_repo(pool.clone());

    // Persist two enabled extractors
    repo.create_extractor(&CustomExtractorParams {
        name: "OG Title".into(),
        key: "og_title".into(),
        selector: "meta[property='og:title']".into(),
        attribute: Some("content".into()),
        multiple: false,
        enabled: true,
    })
    .await
    .unwrap();

    repo.create_extractor(&CustomExtractorParams {
        name: "Hreflang".into(),
        key: "hreflang".into(),
        selector: "link[rel='alternate'][hreflang]".into(),
        attribute: Some("hreflang".into()),
        multiple: true,
        enabled: true,
    })
    .await
    .unwrap();

    // Rebuild registry from DB (mirrors what AppState::new does)
    let mut registry = ExtractorRegistry::new();
    let extractors = repo.list_enabled_extractors().await.unwrap();
    assert_eq!(extractors.len(), 2, "2 extractors should be loaded from DB");

    for ext in extractors {
        let config = ExtractorConfig {
            key: ext.key,
            selector: ext.selector,
            attribute: ext.attribute,
            multiple: ext.multiple,
        };
        registry.register(Box::new(SelectorExtractor::new(config)));
    }

    // Run against sample HTML
    let data = registry.run(sample_html());

    assert_eq!(
        data.get("og_title"),
        Some(&Value::String("OG Test Title".into())),
        "og_title extracted from DB-backed registry"
    );

    let arr = data["hreflang"].as_array().unwrap();
    assert_eq!(arr.len(), 2, "hreflang should have 2 values");
}

// ---------------------------------------------------------------------------
// Layer 4: extracted_data survives Page insert → retrieval round-trip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn extracted_data_survives_page_db_round_trip() {
    let pool = setup_db().await;
    let page_repo = sqlite_page_repo(pool.clone());

    // Build extracted data as if the registry ran
    let mut extracted_data = std::collections::HashMap::new();
    extracted_data.insert("og_title".to_string(), Value::String("OG Test Title".into()));
    extracted_data.insert(
        "hreflang".to_string(),
        Value::Array(vec![
            Value::String("en-US".into()),
            Value::String("fr".into()),
        ]),
    );

    // Mimic a job ID
    let job_id = "test-job-001";

    // Insert a page carrying that data
    let page = Page {
        id: "test-page-001".into(),
        job_id: job_id.into(),
        url: "https://example.com/".into(),
        depth: 0,
        status_code: Some(200),
        content_type: None,
        title: Some("Integration Test Page".into()),
        meta_description: None,
        canonical_url: None,
        robots_meta: None,
        word_count: Some(100),
        load_time_ms: Some(250),
        response_size_bytes: Some(5000),
        has_viewport: true,
        has_structured_data: false,
        crawled_at: Utc::now(),
        extracted_data,
    };

    // Insert needs the job to exist first due to FK — insert directly into jobs table
    sqlx::query("INSERT INTO jobs (id, url, status) VALUES (?, ?, 'pending')")
        .bind(job_id)
        .bind("https://example.com/")
        .execute(&pool)
        .await
        .expect("insert job");

    page_repo.insert(&page).await.expect("insert page");

    // Retrieve and verify
    let retrieved = page_repo
        .get_by_id("test-page-001")
        .await
        .expect("get page by id");

    assert_eq!(
        retrieved.extracted_data.get("og_title"),
        Some(&Value::String("OG Test Title".into())),
        "og_title must survive DB round-trip"
    );

    let hreflang = retrieved.extracted_data.get("hreflang")
        .expect("hreflang must survive DB round-trip");
    let arr = hreflang.as_array().unwrap();
    assert_eq!(arr.len(), 2, "hreflang array must have 2 items after round-trip");
    assert!(arr.contains(&Value::String("en-US".into())));
    assert!(arr.contains(&Value::String("fr".into())));
}

// ---------------------------------------------------------------------------
// Layer 5: Full pipeline — DB extractor config → registry → page → DB
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_pipeline_extractor_config_to_stored_page() {
    let pool = setup_db().await;
    let ext_repo = sqlite_extension_repo(pool.clone());
    let page_repo = sqlite_page_repo(pool.clone());

    // Step 1: user creates a custom extractor
    ext_repo
        .create_extractor(&CustomExtractorParams {
            name: "OG Title".into(),
            key: "og_title".into(),
            selector: "meta[property='og:title']".into(),
            attribute: Some("content".into()),
            multiple: false,
            enabled: true,
        })
        .await
        .unwrap();

    // Step 2: app startup — build registry from DB
    let mut registry = ExtractorRegistry::new();
    for ext in ext_repo.list_enabled_extractors().await.unwrap() {
        registry.register(Box::new(SelectorExtractor::new(ExtractorConfig {
            key: ext.key,
            selector: ext.selector,
            attribute: ext.attribute,
            multiple: ext.multiple,
        })));
    }

    // Step 3: analyze_page runs the registry
    let extracted_data = registry.run(sample_html());
    assert!(
        !extracted_data.is_empty(),
        "registry must produce data — if this fails the problem is in the registry"
    );

    // Step 4: page is stored with extracted_data
    let job_id = "test-job-pipeline";
    sqlx::query("INSERT INTO jobs (id, url, status) VALUES (?, ?, 'pending')")
        .bind(job_id)
        .bind("https://example.com/")
        .execute(&pool)
        .await
        .unwrap();

    let page = Page {
        id: "test-page-pipeline".into(),
        job_id: job_id.into(),
        url: "https://example.com/".into(),
        depth: 0,
        status_code: Some(200),
        content_type: None,
        title: Some("Test".into()),
        meta_description: None,
        canonical_url: None,
        robots_meta: None,
        word_count: Some(50),
        load_time_ms: Some(100),
        response_size_bytes: Some(2000),
        has_viewport: false,
        has_structured_data: false,
        crawled_at: Utc::now(),
        extracted_data,
    };

    page_repo.insert(&page).await.unwrap();

    // Step 5: retrieve and verify the data made it through
    let retrieved = page_repo.get_by_id("test-page-pipeline").await.unwrap();

    assert_eq!(
        retrieved.extracted_data.len(),
        1,
        "extracted_data should have 1 key (og_title)"
    );
    assert_eq!(
        retrieved.extracted_data.get("og_title"),
        Some(&Value::String("OG Test Title".into())),
        "og_title must survive the full pipeline"
    );
}

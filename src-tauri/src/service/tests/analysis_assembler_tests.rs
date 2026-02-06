use chrono::Utc;
use crate::service::analysis_assembler::AnalysisAssembler;
use crate::repository::sqlite::{JobRepository, PageRepository, LinkRepository};
use crate::domain::models::{Page, LighthouseData, NewLink, LinkType, JobSettings};
use crate::test_utils::fixtures;

#[tokio::test]
async fn test_mobile_detection_and_structured_data_from_lighthouse() {
    let pool = fixtures::setup_test_db().await;

    let job_repo = JobRepository::new(pool.clone());
    let page_repo = PageRepository::new(pool.clone());

    // Create job
    let job_id = job_repo.create("https://example.com", &JobSettings::default()).await.unwrap();

    // Insert a page with large load time (4s) so fallback would be false
    let page = Page {
        id: "".to_string(),
        job_id: job_id.clone(),
        url: "https://example.com/page-1".to_string(),
        depth: 0,
        status_code: Some(200),
        content_type: None,
        title: Some("Page 1".to_string()),
        meta_description: None,
        canonical_url: None,
        robots_meta: None,
        word_count: Some(100),
        load_time_ms: Some(4000),
        response_size_bytes: Some(1024),
        crawled_at: Utc::now(),
    };

    let page_id = page_repo.insert(&page).await.unwrap();

    // Insert lighthouse raw json that indicates viewport passed and structured data present
    let raw = r#"{"seo_audits":{"viewport":{"passed":true}},"structured_data":{}}"#.to_string();

    let lh = LighthouseData {
        page_id: page_id.clone(),
        performance_score: None,
        accessibility_score: None,
        best_practices_score: None,
        seo_score: None,
        first_contentful_paint_ms: None,
        largest_contentful_paint_ms: None,
        total_blocking_time_ms: None,
        cumulative_layout_shift: None,
        speed_index: None,
        time_to_interactive_ms: None,
        raw_json: Some(raw),
    };

    page_repo.insert_lighthouse(&lh).await.unwrap();

    let assembler = AnalysisAssembler::new(pool.clone());
    let result = assembler.assemble(&job_id).await.unwrap();

    assert_eq!(result.pages.len(), 1);
    let page = &result.pages[0];

    // Lighthouse viewport passed should override slow load time
    assert!(page.mobile_friendly, "expected mobile_friendly=true from Lighthouse viewport");
    assert!(page.has_structured_data, "expected structured data detected from Lighthouse raw JSON");
}

#[tokio::test]
async fn test_mobile_detection_fallback_to_load_time() {
    let pool = fixtures::setup_test_db().await;

    let job_repo = JobRepository::new(pool.clone());
    let page_repo = PageRepository::new(pool.clone());

    let job_id = job_repo.create("https://example.com", &JobSettings::default()).await.unwrap();

    // Insert a page with short load time (1s) and no lighthouse data
    let page = Page {
        id: "".to_string(),
        job_id: job_id.clone(),
        url: "https://example.com/fast-page".to_string(),
        depth: 0,
        status_code: Some(200),
        content_type: None,
        title: Some("Fast Page".to_string()),
        meta_description: None,
        canonical_url: None,
        robots_meta: None,
        word_count: Some(200),
        load_time_ms: Some(1000),
        response_size_bytes: Some(512),
        crawled_at: Utc::now(),
    };

    page_repo.insert(&page).await.unwrap();

    let assembler = AnalysisAssembler::new(pool.clone());
    let result = assembler.assemble(&job_id).await.unwrap();

    assert_eq!(result.pages.len(), 1);
    let page = &result.pages[0];

    // No Lighthouse viewport present; fallback to load_time <= 2s
    assert!(page.mobile_friendly, "expected mobile_friendly=true from load time heuristic");
}

#[tokio::test]
async fn test_link_classification_fallback_when_target_unparsable() {
    let pool = fixtures::setup_test_db().await;

    let job_repo = JobRepository::new(pool.clone());
    let page_repo = PageRepository::new(pool.clone());
    let link_repo = LinkRepository::new(pool.clone());

    let job_id = job_repo.create("https://example.com", &JobSettings::default()).await.unwrap();

    // Insert source page
    let page = Page {
        id: "".to_string(),
        job_id: job_id.clone(),
        url: "https://example.com/page-a".to_string(),
        depth: 0,
        status_code: Some(200),
        content_type: None,
        title: Some("A".to_string()),
        meta_description: None,
        canonical_url: None,
        robots_meta: None,
        word_count: Some(10),
        load_time_ms: Some(500),
        response_size_bytes: Some(256),
        crawled_at: Utc::now(),
    };

    let page_id = page_repo.insert(&page).await.unwrap();

    // Insert links with unparsable target URL
    let links = vec![
        NewLink {
            job_id: job_id.clone(),
            source_page_id: page_id.clone(),
            target_page_id: None,
            target_url: "javascript:void(0)".to_string(),
            link_text: Some("void link".to_string()),
            link_type: LinkType::Internal,
            is_followed: true,
            status_code: None,
        },
        NewLink {
            job_id: job_id.clone(),
            source_page_id: page_id.clone(),
            target_page_id: None,
            target_url: "javascript:void(0)".to_string(),
            link_text: Some("void link external".to_string()),
            link_type: LinkType::External,
            is_followed: true,
            status_code: None,
        },
    ];

    link_repo.insert_batch(&links).await.unwrap();

    let assembler = AnalysisAssembler::new(pool.clone());
    let result = assembler.assemble(&job_id).await.unwrap();

    assert_eq!(result.pages.len(), 1);
    let page = &result.pages[0];

    // Should have two detailed links
    assert_eq!(page.detailed_links.len(), 2);

    // First link: internal link_type and unparsable target -> is_external = false
    assert_eq!(page.detailed_links[0].is_external, false);

    // Second link: external link_type and unparsable target -> is_external = true
    assert_eq!(page.detailed_links[1].is_external, true);
}

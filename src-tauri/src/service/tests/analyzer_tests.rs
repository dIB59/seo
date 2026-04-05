use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::contexts::analysis::{
    Issue, IssueSeverity, LighthouseData, NewHeading, NewImage, NewIssue, Page, PageInfo,
};
use crate::extractor::data_extractor::{ExtractorConfig, ExtractorRegistry};
use crate::extractor::data_extractor::selector::SelectorExtractor;
use crate::repository::{IssueRepository, PageRepository};
use crate::service::auditor::{AuditResult, AuditScores, Auditor, SeoAuditDetails};
use crate::service::processor::AnalyzerService;
use anyhow::Result;

// ---------------------------------------------------------------------------
// Mock repos
// ---------------------------------------------------------------------------

struct MockPageRepo {
    pub inserted_pages: Mutex<Vec<Page>>,
}

impl MockPageRepo {
    fn new() -> Self {
        Self { inserted_pages: Mutex::new(vec![]) }
    }
}

#[async_trait]
impl PageRepository for MockPageRepo {
    async fn insert(&self, page: &Page) -> Result<String> {
        let mut guard = self.inserted_pages.lock().unwrap();
        guard.push(page.clone());
        Ok(page.id.clone())
    }
    async fn insert_batch(&self, _: &[Page]) -> Result<()> { Ok(()) }
    async fn get_by_job_id(&self, _: &str) -> Result<Vec<Page>> { Ok(vec![]) }
    async fn get_info_by_job_id(&self, _: &str) -> Result<Vec<PageInfo>> { Ok(vec![]) }
    async fn get_by_id(&self, _: &str) -> Result<Page> { Err(anyhow::anyhow!("not impl")) }
    async fn replace_headings(&self, _: &str, _: &[NewHeading]) -> Result<()> { Ok(()) }
    async fn replace_images(&self, _: &str, _: &[NewImage]) -> Result<()> { Ok(()) }
    async fn count_by_job_id(&self, _: &str) -> Result<i64> { Ok(0) }
    async fn insert_lighthouse(&self, _: &LighthouseData) -> Result<()> { Ok(()) }
    async fn get_lighthouse_by_job_id(&self, _: &str) -> Result<Vec<LighthouseData>> { Ok(vec![]) }
}

struct MockIssueRepo {
    pub inserted_issues: Mutex<Vec<NewIssue>>,
}

impl MockIssueRepo {
    fn new() -> Self {
        Self { inserted_issues: Mutex::new(vec![]) }
    }
}

#[async_trait]
impl IssueRepository for MockIssueRepo {
    async fn insert_batch(&self, issues: &[NewIssue]) -> Result<()> {
        self.inserted_issues.lock().unwrap().extend_from_slice(issues);
        Ok(())
    }
    async fn get_by_job_id(&self, _: &str) -> Result<Vec<Issue>> { Ok(vec![]) }
    async fn get_by_page_id(&self, _: &str) -> Result<Vec<Issue>> { Ok(vec![]) }
    async fn get_by_job_and_severity(&self, _: &str, _: IssueSeverity) -> Result<Vec<Issue>> { Ok(vec![]) }
    async fn count_by_severity(&self, _: &str) -> Result<crate::repository::IssueCounts> { Ok(Default::default()) }
    async fn count_by_job_id(&self, _: &str) -> Result<i64> { Ok(0) }
    async fn get_grouped_by_type(&self, _: &str) -> Result<Vec<crate::repository::IssueGroup>> { Ok(vec![]) }
}

// ---------------------------------------------------------------------------
// Mock auditors
// ---------------------------------------------------------------------------

/// Minimal auditor — bare HTML with no title/description (triggers issues).
struct MockAuditor;

#[async_trait]
impl Auditor for MockAuditor {
    async fn analyze(&self, url: &str) -> Result<AuditResult> {
        let html = r#"<html><head><title></title><meta name="description" content=""></head>
                      <body><a href="/page-2">next</a></body></html>"#.into();
        Ok(AuditResult {
            url: url.to_string(),
            html,
            status_code: 200,
            load_time_ms: 100.0,
            content_size: 1000,
            scores: AuditScores { seo_details: SeoAuditDetails::default(), ..Default::default() },
        })
    }
    fn name(&self) -> &'static str { "mock" }
}

/// Auditor whose HTML contains known extractable content.
struct MockAuditorWithExtractableContent;

#[async_trait]
impl Auditor for MockAuditorWithExtractableContent {
    async fn analyze(&self, url: &str) -> Result<AuditResult> {
        let html = r#"<!DOCTYPE html>
<html>
<head>
  <title>Test Page</title>
  <meta property="og:title" content="OG Test Title" />
  <meta property="og:image" content="https://example.com/image.jpg" />
  <link rel="alternate" hreflang="en-US" href="https://example.com/en/" />
  <link rel="alternate" hreflang="fr" href="https://example.com/fr/" />
</head>
<body><h1>Heading</h1></body>
</html>"#.into();
        Ok(AuditResult {
            url: url.to_string(),
            html,
            status_code: 200,
            load_time_ms: 80.0,
            content_size: 500,
            scores: AuditScores { seo_details: SeoAuditDetails::default(), ..Default::default() },
        })
    }
    fn name(&self) -> &'static str { "mock-with-content" }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn analyze_page_insert_and_issue_generation() {
    let page_repo = Arc::new(MockPageRepo::new());
    let issue_repo = Arc::new(MockIssueRepo::new());

    let analyzer = AnalyzerService::new(
        page_repo.clone(),
        issue_repo.clone(),
        Arc::new(crate::service::spider::MockSpider { html_response: String::new(), generic_response: crate::service::spider::SpiderResponse { status: 200, body: String::new(), url: String::new() } }),
        Arc::new(ExtractorRegistry::new()),
    );
    let auditor: Arc<dyn Auditor + Send + Sync> = Arc::new(MockAuditor);

    let (result, new_urls) = analyzer
        .analyze_page("https://example.com", "job-1", 0, &auditor)
        .await
        .expect("analysis should succeed");

    let pages = page_repo.inserted_pages.lock().unwrap();
    assert_eq!(pages.len(), 1, "one page should be inserted");

    let issues = issue_repo.inserted_issues.lock().unwrap();
    assert!(!issues.is_empty(), "issues should be recorded for missing title/description");

    assert_eq!(new_urls, vec!["https://example.com/page-2".to_string()]);
    assert!(result.links.iter().any(|l| l.target_url.contains("/page-2")));
}

/// This is the critical test: proves that when `AnalyzerService` has a
/// non-empty `ExtractorRegistry`, the extracted data ends up in the stored page.
#[tokio::test]
async fn analyze_page_populates_extracted_data() {
    let page_repo = Arc::new(MockPageRepo::new());
    let issue_repo = Arc::new(MockIssueRepo::new());

    // Build a registry with two extractors
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

    let analyzer = AnalyzerService::new(
        page_repo.clone(),
        issue_repo.clone(),
        Arc::new(crate::service::spider::MockSpider { html_response: String::new(), generic_response: crate::service::spider::SpiderResponse { status: 200, body: String::new(), url: String::new() } }),
        Arc::new(registry),
    );
    let auditor: Arc<dyn Auditor + Send + Sync> = Arc::new(MockAuditorWithExtractableContent);

    analyzer
        .analyze_page("https://example.com", "job-extractor", 0, &auditor)
        .await
        .expect("analysis should succeed");

    let pages = page_repo.inserted_pages.lock().unwrap();
    assert_eq!(pages.len(), 1);

    let stored = &pages[0];

    // og_title must be extracted
    assert_eq!(
        stored.extracted_data.get("og_title"),
        Some(&serde_json::Value::String("OG Test Title".into())),
        "og_title must be in extracted_data of the stored page"
    );

    // hreflang must be extracted as an array
    let hreflang = stored.extracted_data.get("hreflang")
        .expect("hreflang must be in extracted_data");
    let arr = hreflang.as_array().expect("hreflang must be a JSON array");
    assert_eq!(arr.len(), 2, "two hreflang values expected");
    assert!(arr.contains(&serde_json::Value::String("en-US".into())));
    assert!(arr.contains(&serde_json::Value::String("fr".into())));
}

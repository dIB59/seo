use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::domain::models::{LighthouseData, NewHeading, NewImage, NewIssue, Page};
use crate::repository::{IssueRepository, PageRepository};
use crate::service::auditor::{AuditResult, AuditScores, Auditor, SeoAuditDetails};
use crate::service::processor::AnalyzerService;
use anyhow::Result;

struct MockPageRepo {
    pub inserted_pages: Mutex<Vec<Page>>,
}

impl MockPageRepo {
    fn new() -> Self {
        Self {
            inserted_pages: Mutex::new(vec![]),
        }
    }
}

#[async_trait]
impl PageRepository for MockPageRepo {
    async fn insert(&self, page: &crate::domain::models::Page) -> Result<String> {
        let mut guard = self.inserted_pages.lock().unwrap();
        guard.push(page.clone());
        Ok(page.id.clone())
    }

    async fn insert_batch(&self, _pages: &[crate::domain::models::Page]) -> Result<()> {
        Ok(())
    }

    async fn get_by_job_id(&self, _job_id: &str) -> Result<Vec<crate::domain::models::Page>> {
        Ok(vec![])
    }

    async fn get_info_by_job_id(
        &self,
        _job_id: &str,
    ) -> Result<Vec<crate::domain::models::PageInfo>> {
        Ok(vec![])
    }

    async fn get_by_id(&self, _page_id: &str) -> Result<crate::domain::models::Page> {
        Err(anyhow::anyhow!("not implemented"))
    }

    async fn replace_headings(
        &self,
        _page_id: &str,
        _headings: &[crate::domain::models::NewHeading],
    ) -> Result<()> {
        Ok(())
    }

    async fn replace_images(
        &self,
        _page_id: &str,
        _images: &[crate::domain::models::NewImage],
    ) -> Result<()> {
        Ok(())
    }

    async fn count_by_job_id(&self, _job_id: &str) -> Result<i64> {
        Ok(0)
    }

    async fn insert_lighthouse(&self, _data: &LighthouseData) -> Result<()> {
        Ok(())
    }

    async fn get_lighthouse_by_job_id(&self, _job_id: &str) -> Result<Vec<LighthouseData>> {
        Ok(vec![])
    }
}

struct MockIssueRepo {
    pub inserted_issues: Mutex<Vec<NewIssue>>,
}

impl MockIssueRepo {
    fn new() -> Self {
        Self {
            inserted_issues: Mutex::new(vec![]),
        }
    }
}

#[async_trait]
impl IssueRepository for MockIssueRepo {
    async fn insert_batch(&self, issues: &[crate::domain::models::NewIssue]) -> Result<()> {
        let mut guard = self.inserted_issues.lock().unwrap();
        guard.extend_from_slice(issues);
        Ok(())
    }

    async fn get_by_job_id(&self, _job_id: &str) -> Result<Vec<crate::domain::models::Issue>> {
        Ok(vec![])
    }

    async fn get_by_page_id(&self, _page_id: &str) -> Result<Vec<crate::domain::models::Issue>> {
        Ok(vec![])
    }

    async fn get_by_job_and_severity(
        &self,
        _job_id: &str,
        _severity: crate::domain::models::IssueSeverity,
    ) -> Result<Vec<crate::domain::models::Issue>> {
        Ok(vec![])
    }

    async fn count_by_severity(&self, _job_id: &str) -> Result<crate::repository::IssueCounts> {
        Ok(Default::default())
    }

    async fn count_by_job_id(&self, _job_id: &str) -> Result<i64> {
        Ok(0)
    }

    async fn get_grouped_by_type(
        &self,
        _job_id: &str,
    ) -> Result<Vec<crate::repository::IssueGroup>> {
        Ok(vec![])
    }
}

// Minimal mock auditor to produce deterministic HTML and scores
struct MockAuditor;

#[async_trait]
impl Auditor for MockAuditor {
    async fn analyze(&self, url: &str) -> Result<AuditResult> {
        let html = format!("<html><head><title></title><meta name=\"description\" content=\"\"></head><body><a href=\"/page-2\">next</a></body></html>");
        Ok(AuditResult {
            url: url.to_string(),
            html,
            status_code: 200,
            load_time_ms: 100.0,
            content_size: 1000,
            scores: AuditScores {
                seo_details: SeoAuditDetails::default(),
                ..Default::default()
            },
        })
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}

#[tokio::test]
async fn analyze_page_insert_and_issue_generation() {
    let page_repo = Arc::new(MockPageRepo::new());
    let issue_repo = Arc::new(MockIssueRepo::new());

    let analyzer = AnalyzerService::new(page_repo.clone(), issue_repo.clone());
    let auditor = Arc::new(MockAuditor);

    let (result, new_urls) = analyzer
        .analyze_page("https://example.com", "job-1", 0, &auditor)
        .await
        .expect("analysis should succeed");

    // Page should have been inserted
    let pages = page_repo.inserted_pages.lock().unwrap();
    assert_eq!(pages.len(), 1);

    // Issues should have been recorded (missing title & description)
    let issues = issue_repo.inserted_issues.lock().unwrap();
    assert!(issues.len() >= 1);

    // Edge/url discovery should include the /page-2 link
    assert_eq!(new_urls, vec!["/page-2".to_string()]);
    assert!(result.edges.iter().any(|e| e.to_url == "/page-2"));
}

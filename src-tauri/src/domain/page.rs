use crate::domain::{IssueSeverity, NewIssue};
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Pages with load time ≤ 2s considered mobile-friendly (speed heuristic fallback).
const SPEED_HEURISTIC_LOAD_TIME_MS: i64 = 2000;

/// A crawled page with SEO data.
/// Maps to the `pages` table in the new schema.
#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub id: String,
    pub job_id: String,
    pub url: String,
    pub depth: i64,
    pub status_code: Option<i64>,
    pub content_type: Option<String>,

    // Core SEO fields
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub canonical_url: Option<String>,
    pub robots_meta: Option<String>,

    // Content metrics
    pub word_count: Option<i64>,
    pub load_time_ms: Option<i64>,
    pub response_size_bytes: Option<i64>,

    // SEO flags (extracted from HTML, available without Lighthouse)
    pub has_viewport: bool,
    pub has_structured_data: bool,

    pub crawled_at: DateTime<Utc>,
}

impl Page {
    /// Page-level mobile-friendly heuristic: viewport meta tag present AND fast load time.
    pub fn is_mobile_friendly_heuristic(&self) -> bool {
        self.has_viewport && self.load_time_ms.unwrap_or(0) <= SPEED_HEURISTIC_LOAD_TIME_MS
    }

    /// Perform a basic SEO audit on the page and generate a list of issues.
    pub fn audit(&self) -> Vec<NewIssue> {
        let mut issues = Vec::new();

        // 1. Title checks
        if self.title.is_none() || self.title.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
            issues.push(NewIssue {
                job_id: self.job_id.clone(),
                page_id: Some(self.id.clone()),
                issue_type: "Missing Title".to_string(),
                severity: IssueSeverity::Critical,
                message: "Page has no title tag".to_string(),
                details: Some("Add a descriptive title tag".to_string()),
            });
        }

        // 2. Meta description checks
        if self.meta_description.is_none()
            || self
                .meta_description
                .as_ref()
                .map(|d| d.is_empty())
                .unwrap_or(true)
        {
            issues.push(NewIssue {
                job_id: self.job_id.clone(),
                page_id: Some(self.id.clone()),
                issue_type: "Missing Meta Description".to_string(),
                severity: IssueSeverity::Warning,
                message: "Page has no meta description".to_string(),
                details: Some("Add a meta description".to_string()),
            });
        }

        // 3. HTTP Status Check
        if let Some(status) = self.status_code {
            if status >= 400 {
                issues.push(NewIssue {
                    job_id: self.job_id.clone(),
                    page_id: Some(self.id.clone()),
                    issue_type: "HTTP Error".to_string(),
                    severity: IssueSeverity::Critical,
                    message: format!("Page returned status code {}", status),
                    details: Some("Fix the HTTP error".to_string()),
                });
            }
        }

        issues
    }
}

/// Lightweight page info for listings.
#[derive(Debug, Clone, Serialize)]
pub struct PageInfo {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<i64>,
    pub load_time_ms: Option<i64>,
    pub issue_count: i64,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::IssueSeverity;

    fn make_page(overrides: impl FnOnce(&mut Page)) -> Page {
        let mut page = Page {
            id: "1".to_string(),
            job_id: "job1".to_string(),
            url: "https://example.com".to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: None,
            title: Some("Title".to_string()),
            meta_description: Some("desc".to_string()),
            canonical_url: None,
            robots_meta: None,
            word_count: Some(100),
            load_time_ms: Some(1000),
            response_size_bytes: Some(512),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
        };
        overrides(&mut page);
        page
    }

    #[test]
    fn test_audit_missing_title() {
        let page = make_page(|p| {
            p.title = None;
        });
        let issues = page.audit();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, "Missing Title");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_audit_http_error() {
        let page = make_page(|p| {
            p.status_code = Some(404);
        });
        let issues = page.audit();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, "HTTP Error");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_mobile_friendly_heuristic_fast_page() {
        let page = make_page(|p| p.load_time_ms = Some(1500));
        assert!(page.is_mobile_friendly_heuristic());
    }

    #[test]
    fn test_mobile_friendly_heuristic_slow_page() {
        let page = make_page(|p| p.load_time_ms = Some(3000));
        assert!(!page.is_mobile_friendly_heuristic());
    }

    #[test]
    fn test_mobile_friendly_heuristic_exactly_threshold() {
        let page = make_page(|p| p.load_time_ms = Some(2000));
        assert!(page.is_mobile_friendly_heuristic());
    }
}

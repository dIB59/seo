pub mod builtin;
pub mod custom;

use crate::contexts::analysis::{IssueSeverity, NewIssue, Page};
use crate::service::auditor::SeoAuditDetails;

/// All the data a `Check` needs to evaluate a page.
pub struct CheckContext<'a> {
    pub page: &'a Page,
    pub seo_details: &'a SeoAuditDetails,
    pub job_id: &'a str,
    pub page_id: &'a str,
}

impl<'a> CheckContext<'a> {
    pub fn new(
        page: &'a Page,
        seo_details: &'a SeoAuditDetails,
        job_id: &'a str,
        page_id: &'a str,
    ) -> Self {
        Self { page, seo_details, job_id, page_id }
    }

    /// Helper to build a `NewIssue` scoped to this context.
    pub fn issue(
        &self,
        issue_type: &str,
        severity: IssueSeverity,
        message: &str,
    ) -> NewIssue {
        NewIssue {
            job_id: self.job_id.to_string(),
            page_id: Some(self.page_id.to_string()),
            issue_type: issue_type.to_string(),
            severity,
            message: message.to_string(),
            details: None,
        }
    }
}

/// A single SEO check. Returns `Some(issue)` if the check fails, `None` if it passes.
pub trait Check: Send + Sync {
    fn id(&self) -> &str;
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue>;
}

/// Runs all registered checks against a page and collects the resulting issues.
#[derive(Default)]
pub struct CheckerRegistry {
    checks: Vec<Box<dyn Check>>,
}

impl CheckerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry pre-loaded with all built-in checks.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        for check in builtin::all() {
            registry.register(check);
        }
        registry
    }

    pub fn register(&mut self, check: Box<dyn Check>) {
        self.checks.push(check);
    }

    pub fn run(&self, ctx: &CheckContext) -> Vec<NewIssue> {
        self.checks
            .iter()
            .filter_map(|c| c.check(ctx))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.checks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.checks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::auditor::{CheckResult, Score, SeoAuditDetails};
    use chrono::Utc;

    fn passing_details() -> SeoAuditDetails {
        let pass = CheckResult { passed: true, score: Score::from(1.0), value: None, description: None };
        SeoAuditDetails {
            document_title: pass.clone(),
            meta_description: pass.clone(),
            viewport: pass.clone(),
            canonical: pass.clone(),
            hreflang: pass.clone(),
            crawlable_anchors: pass.clone(),
            link_text: pass.clone(),
            image_alt: pass.clone(),
            http_status_code: pass.clone(),
            is_crawlable: pass,
        }
    }

    fn make_page() -> Page {
        Page {
            id: "p1".into(),
            job_id: "j1".into(),
            url: "https://example.com".into(),
            depth: crate::contexts::analysis::Depth::root(),
            status_code: Some(200),
            content_type: None,
            title: Some("A Good SEO Title That Is Long Enough".into()),
            meta_description: Some("A good meta description that is between 70 and 160 characters long to satisfy the length requirement here.".into()),
            canonical_url: None,
            robots_meta: None,
            word_count: Some(500),
            load_time_ms: Some(1000),
            response_size_bytes: Some(10000),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn empty_registry_returns_no_issues() {
        let registry = CheckerRegistry::new();
        let page = make_page();
        let details = passing_details();
        let ctx = CheckContext::new(&page, &details, "j1", "p1");
        assert!(registry.run(&ctx).is_empty());
    }

    #[test]
    fn registry_runs_all_registered_checks() {
        struct AlwaysFail;
        impl Check for AlwaysFail {
            fn id(&self) -> &str { "always-fail" }
            fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
                Some(ctx.issue("Test", IssueSeverity::Info, "always fails"))
            }
        }

        let mut registry = CheckerRegistry::new();
        registry.register(Box::new(AlwaysFail));
        registry.register(Box::new(AlwaysFail));

        let page = make_page();
        let details = passing_details();
        let ctx = CheckContext::new(&page, &details, "j1", "p1");
        assert_eq!(registry.run(&ctx).len(), 2);
    }

    #[test]
    fn context_issue_helper_sets_ids_correctly() {
        let page = make_page();
        let details = passing_details();
        let ctx = CheckContext::new(&page, &details, "job-1", "page-1");
        let issue = ctx.issue("Test", IssueSeverity::Warning, "msg");
        assert_eq!(issue.job_id, "job-1");
        assert_eq!(issue.page_id, Some("page-1".into()));
    }

    #[test]
    fn with_defaults_loads_builtin_checks() {
        let registry = CheckerRegistry::with_defaults();
        assert!(!registry.is_empty());
    }
}

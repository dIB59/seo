use crate::checker::{Check, CheckContext};
use crate::contexts::analysis::{IssueSeverity, NewIssue};

pub struct HttpStatusCheck;
impl Check for HttpStatusCheck {
    fn id(&self) -> &str { "http-status" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let r = &ctx.seo_details.http_status_code;
        if r.passed { return None; }
        Some(ctx.issue(
            "HTTP Error",
            IssueSeverity::Critical,
            r.description.as_deref().unwrap_or("HTTP error status code"),
        ))
    }
}

pub struct CrawlabilityCheck;
impl Check for CrawlabilityCheck {
    fn id(&self) -> &str { "crawlability" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let r = &ctx.seo_details.is_crawlable;
        if r.passed { return None; }
        Some(ctx.issue(
            "Not Crawlable",
            IssueSeverity::Critical,
            r.description.as_deref().unwrap_or("Page has noindex directive"),
        ))
    }
}

pub struct TitleCheck;
impl Check for TitleCheck {
    fn id(&self) -> &str { "document-title" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let r = &ctx.seo_details.document_title;
        if r.passed { return None; }
        let severity = if r.score.raw() == 0.0 {
            IssueSeverity::Critical
        } else {
            IssueSeverity::Warning
        };
        Some(ctx.issue(
            "Document Title",
            severity,
            r.description.as_deref().unwrap_or("Title issue"),
        ))
    }
}

pub struct MetaDescriptionCheck;
impl Check for MetaDescriptionCheck {
    fn id(&self) -> &str { "meta-description" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let r = &ctx.seo_details.meta_description;
        if r.passed { return None; }
        let severity = if r.score.raw() == 0.0 {
            IssueSeverity::Warning
        } else {
            IssueSeverity::Info
        };
        Some(ctx.issue(
            "Meta Description",
            severity,
            r.description.as_deref().unwrap_or("Meta description issue"),
        ))
    }
}

pub struct ViewportCheck;
impl Check for ViewportCheck {
    fn id(&self) -> &str { "viewport" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let r = &ctx.seo_details.viewport;
        if r.passed { return None; }
        Some(ctx.issue(
            "Viewport",
            IssueSeverity::Warning,
            r.description.as_deref().unwrap_or("Viewport issue"),
        ))
    }
}

pub struct CanonicalCheck;
impl Check for CanonicalCheck {
    fn id(&self) -> &str { "canonical" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let r = &ctx.seo_details.canonical;
        if r.passed { return None; }
        Some(ctx.issue(
            "Canonical URL",
            IssueSeverity::Warning,
            r.description.as_deref().unwrap_or("Missing canonical URL"),
        ))
    }
}

pub struct ImageAltCheck;
impl Check for ImageAltCheck {
    fn id(&self) -> &str { "image-alt" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let r = &ctx.seo_details.image_alt;
        if r.passed { return None; }
        Some(ctx.issue(
            "Images Missing Alt",
            IssueSeverity::Warning,
            r.description.as_deref().unwrap_or("Images missing alt attributes"),
        ))
    }
}

pub struct LinkTextCheck;
impl Check for LinkTextCheck {
    fn id(&self) -> &str { "link-text" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let r = &ctx.seo_details.link_text;
        if r.passed { return None; }
        Some(ctx.issue(
            "Poor Link Text",
            IssueSeverity::Info,
            r.description.as_deref().unwrap_or("Links have generic text"),
        ))
    }
}

pub struct CrawlableAnchorsCheck;
impl Check for CrawlableAnchorsCheck {
    fn id(&self) -> &str { "crawlable-anchors" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let r = &ctx.seo_details.crawlable_anchors;
        if r.passed { return None; }
        Some(ctx.issue(
            "Uncrawlable Links",
            IssueSeverity::Info,
            r.description.as_deref().unwrap_or("Some links are not crawlable"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::auditor::{CheckResult, Score, SeoAuditDetails};
    use crate::checker::CheckContext;
    use crate::contexts::analysis::Page;
    use chrono::Utc;

    fn pass() -> CheckResult {
        CheckResult { passed: true, score: Score::from(1.0), value: None, description: None }
    }

    fn fail(score: f64, msg: &str) -> CheckResult {
        CheckResult { passed: false, score: Score::from(score), value: None, description: Some(msg.to_string()) }
    }

    fn all_pass() -> SeoAuditDetails {
        SeoAuditDetails {
            document_title: pass(), meta_description: pass(), viewport: pass(),
            canonical: pass(), hreflang: pass(), crawlable_anchors: pass(),
            link_text: pass(), image_alt: pass(), http_status_code: pass(),
            is_crawlable: pass(),
        }
    }

    fn make_page() -> Page {
        Page {
            id: "p1".into(), job_id: "j1".into(),
            url: "https://example.com".into(), depth: 0,
            status_code: Some(200), content_type: None,
            title: Some("Title".into()), meta_description: Some("Desc".into()),
            canonical_url: None, robots_meta: None,
            word_count: Some(500), load_time_ms: Some(1000),
            response_size_bytes: Some(1000), has_viewport: true,
            has_structured_data: false, crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        }
    }

    fn ctx<'a>(page: &'a Page, details: &'a SeoAuditDetails) -> CheckContext<'a> {
        CheckContext::new(page, details, "j1", "p1")
    }

    #[test]
    fn http_status_passes_on_200() {
        let page = make_page();
        let details = all_pass();
        assert!(HttpStatusCheck.check(&ctx(&page, &details)).is_none());
    }

    #[test]
    fn http_status_fails_on_error() {
        let page = make_page();
        let mut details = all_pass();
        details.http_status_code = fail(0.0, "HTTP error status: 404");
        let issue = HttpStatusCheck.check(&ctx(&page, &details)).unwrap();
        assert_eq!(issue.issue_type, "HTTP Error");
        assert_eq!(issue.severity, IssueSeverity::Critical);
        assert_eq!(issue.message, "HTTP error status: 404");
    }

    #[test]
    fn missing_title_is_critical() {
        let page = make_page();
        let mut details = all_pass();
        details.document_title = fail(0.0, "Missing document title");
        let issue = TitleCheck.check(&ctx(&page, &details)).unwrap();
        assert_eq!(issue.severity, IssueSeverity::Critical);
    }

    #[test]
    fn short_title_is_warning() {
        let page = make_page();
        let mut details = all_pass();
        details.document_title = fail(0.5, "Title too short");
        let issue = TitleCheck.check(&ctx(&page, &details)).unwrap();
        assert_eq!(issue.severity, IssueSeverity::Warning);
    }

    #[test]
    fn missing_meta_description_is_warning() {
        let page = make_page();
        let mut details = all_pass();
        details.meta_description = fail(0.0, "Missing meta description");
        let issue = MetaDescriptionCheck.check(&ctx(&page, &details)).unwrap();
        assert_eq!(issue.severity, IssueSeverity::Warning);
    }

    #[test]
    fn short_meta_description_is_info() {
        let page = make_page();
        let mut details = all_pass();
        details.meta_description = fail(0.5, "Description too short");
        let issue = MetaDescriptionCheck.check(&ctx(&page, &details)).unwrap();
        assert_eq!(issue.severity, IssueSeverity::Info);
    }

    #[test]
    fn noindex_is_critical() {
        let page = make_page();
        let mut details = all_pass();
        details.is_crawlable = fail(0.0, "Page has noindex directive");
        let issue = CrawlabilityCheck.check(&ctx(&page, &details)).unwrap();
        assert_eq!(issue.severity, IssueSeverity::Critical);
    }

    #[test]
    fn passing_checks_return_none() {
        let page = make_page();
        let details = all_pass();
        let c = &ctx(&page, &details);
        assert!(ViewportCheck.check(c).is_none());
        assert!(CanonicalCheck.check(c).is_none());
        assert!(ImageAltCheck.check(c).is_none());
        assert!(LinkTextCheck.check(c).is_none());
        assert!(CrawlableAnchorsCheck.check(c).is_none());
    }
}

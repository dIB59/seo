use crate::checker::{Check, CheckContext};
use crate::contexts::analysis::{IssueSeverity, NewIssue};

const MIN_WORD_COUNT: i64 = 300;
const MAX_LOAD_TIME_MS: i64 = 3000;

pub struct WordCountCheck;
impl Check for WordCountCheck {
    fn id(&self) -> &str { "word-count" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let count = ctx.page.word_count?;
        if count >= MIN_WORD_COUNT { return None; }
        Some(ctx.issue(
            "Thin Content",
            IssueSeverity::Info,
            &format!("Page has only {} words (recommend {}+)", count, MIN_WORD_COUNT),
        ))
    }
}

pub struct LoadTimeCheck;
impl Check for LoadTimeCheck {
    fn id(&self) -> &str { "load-time" }
    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        let ms = ctx.page.load_time_ms?;
        if ms <= MAX_LOAD_TIME_MS { return None; }
        Some(ctx.issue(
            "Slow Page Load",
            IssueSeverity::Warning,
            &format!("Page took {}ms to load (threshold: {}ms)", ms, MAX_LOAD_TIME_MS),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::CheckContext;
    use crate::contexts::analysis::Page;
    use crate::service::auditor::{CheckResult, Score, SeoAuditDetails};
    use chrono::Utc;

    fn pass() -> CheckResult {
        CheckResult { passed: true, score: Score::from(1.0), value: None, description: None }
    }

    fn all_pass() -> SeoAuditDetails {
        SeoAuditDetails {
            document_title: pass(), meta_description: pass(), viewport: pass(),
            canonical: pass(), hreflang: pass(), crawlable_anchors: pass(),
            link_text: pass(), image_alt: pass(), http_status_code: pass(),
            is_crawlable: pass(),
        }
    }

    fn make_page(word_count: Option<i64>, load_time_ms: Option<i64>) -> Page {
        Page {
            id: "p1".into(), job_id: "j1".into(),
            url: "https://example.com".into(), depth: 0,
            status_code: Some(200), content_type: None,
            title: None, meta_description: None, canonical_url: None,
            robots_meta: None, word_count, load_time_ms,
            response_size_bytes: None, has_viewport: false,
            has_structured_data: false, crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        }
    }

    fn ctx<'a>(page: &'a Page, details: &'a SeoAuditDetails) -> CheckContext<'a> {
        CheckContext::new(page, details, "j1", "p1")
    }

    #[test]
    fn word_count_passes_above_threshold() {
        let page = make_page(Some(300), None);
        let details = all_pass();
        assert!(WordCountCheck.check(&ctx(&page, &details)).is_none());
    }

    #[test]
    fn word_count_fails_below_threshold() {
        let page = make_page(Some(150), None);
        let details = all_pass();
        let issue = WordCountCheck.check(&ctx(&page, &details)).unwrap();
        assert_eq!(issue.issue_type, "Thin Content");
        assert_eq!(issue.severity, IssueSeverity::Info);
        assert!(issue.message.contains("150"));
    }

    #[test]
    fn word_count_skipped_when_none() {
        let page = make_page(None, None);
        let details = all_pass();
        assert!(WordCountCheck.check(&ctx(&page, &details)).is_none());
    }

    #[test]
    fn load_time_passes_at_threshold() {
        let page = make_page(None, Some(3000));
        let details = all_pass();
        assert!(LoadTimeCheck.check(&ctx(&page, &details)).is_none());
    }

    #[test]
    fn load_time_fails_above_threshold() {
        let page = make_page(None, Some(5000));
        let details = all_pass();
        let issue = LoadTimeCheck.check(&ctx(&page, &details)).unwrap();
        assert_eq!(issue.issue_type, "Slow Page Load");
        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert!(issue.message.contains("5000"));
    }

    #[test]
    fn load_time_skipped_when_none() {
        let page = make_page(None, None);
        let details = all_pass();
        assert!(LoadTimeCheck.check(&ctx(&page, &details)).is_none());
    }
}

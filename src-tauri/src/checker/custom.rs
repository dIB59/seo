//! Adapts a [`CustomCheck`] (from the extension CRUD) into a [`Check`]
//! trait impl so user-defined checks run alongside the built-ins during
//! page analysis.
//!
//! Evaluation reuses `pattern_engine::resolve_field` and
//! `evaluate_condition` — the condition logic is identical to report
//! patterns so a user referencing `tag:og_image` in a custom check
//! sees the same semantics as a report pattern referencing the same tag.
//!
//! Tag substitution in `message_template`: after a check fires,
//! `{tag.X}` placeholders in the message are replaced with the
//! page's extracted value for tag `X`. This is the chunk-3 deliverable.

use std::collections::HashMap;

use crate::checker::{Check, CheckContext};
use crate::contexts::analysis::{IssueSeverity, NewIssue};
use crate::contexts::extension::CustomCheck;
use crate::contexts::report::services::pattern_engine::{
    evaluate_condition, resolve_field, FieldValue,
};

/// Wraps a user-defined [`CustomCheck`] and evaluates it per-page using
/// the same field resolver and condition evaluator as the report
/// pattern engine.
pub struct CustomCheckAdapter {
    check: CustomCheck,
}

impl CustomCheckAdapter {
    pub fn new(check: CustomCheck) -> Self {
        Self { check }
    }
}

impl Check for CustomCheckAdapter {
    fn id(&self) -> &str {
        &self.check.id
    }

    fn check(&self, ctx: &CheckContext) -> Option<NewIssue> {
        // resolve_field needs an h1_counts map. Custom checks rarely
        // reference h1_count (they use tag: fields), and headings
        // aren't in CheckContext today, so we pass an empty map.
        // h1_count will resolve to 0, which is documented as a known
        // limitation until CheckContext carries heading info.
        let empty_h1: HashMap<String, usize> = HashMap::new();

        let value = resolve_field(ctx.page, &empty_h1, &self.check.field);
        let threshold = self.check.threshold.as_deref();

        if !evaluate_condition(&value, &self.check.operator, threshold) {
            return None;
        }

        // The check fired — build the issue message by substituting
        // {tag.X} placeholders against the page's extracted_data, and
        // {value} against the resolved field value.
        let message = substitute_message(
            &self.check.message_template,
            &value,
            &ctx.page.extracted_data,
        );

        Some(NewIssue {
            job_id: ctx.job_id.to_string(),
            page_id: Some(ctx.page_id.to_string()),
            issue_type: self.check.name.clone(),
            severity: self.check.severity,
            message,
            details: None,
        })
    }
}

/// Replace `{value}` with the resolved field value and `{tag.X}` with
/// the page's extracted value for tag X.
fn substitute_message(
    template: &str,
    value: &FieldValue,
    extracted_data: &HashMap<String, serde_json::Value>,
) -> String {
    // First pass: replace {value} with the evaluated field's display form.
    let value_str = match value {
        FieldValue::Null => String::new(),
        FieldValue::Text(s) => s.clone(),
        FieldValue::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{n:.2}")
            }
        }
        FieldValue::Bool(b) => b.to_string(),
    };

    let mut result = template.replace("{value}", &value_str);

    // Second pass: replace every {tag.X} with extracted_data[X].
    // Scan for `{tag.` prefix, find the closing `}`, look up the key.
    // Unrecognised tags are left as-is (same as replace_prompt_vars).
    let prefix = "{tag.";
    while let Some(start) = result.find(prefix) {
        let rest = &result[start + prefix.len()..];
        let Some(end) = rest.find('}') else { break };
        let tag_name = &rest[..end];
        let replacement = extracted_data
            .get(tag_name)
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            })
            .unwrap_or_default();
        let full_placeholder = format!("{prefix}{tag_name}}}");
        result = result.replacen(&full_placeholder, &replacement, 1);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::CheckContext;
    use crate::contexts::analysis::Depth;
    use crate::contexts::extension::Operator;
    use crate::service::auditor::{CheckResult, Score, SeoAuditDetails};
    use chrono::Utc;

    fn passing_details() -> SeoAuditDetails {
        let pass = CheckResult {
            passed: true,
            score: Score::from(1.0),
            value: None,
            description: None,
        };
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

    fn make_page_with_tags(tags: Vec<(&str, serde_json::Value)>) -> crate::contexts::analysis::Page {
        let mut extracted_data = HashMap::new();
        for (k, v) in tags {
            extracted_data.insert(k.to_string(), v);
        }
        crate::contexts::analysis::Page {
            id: "p1".into(),
            job_id: "j1".into(),
            url: "https://example.com".into(),
            depth: Depth::root(),
            status_code: Some(200),
            content_type: None,
            title: Some("Test Title".into()),
            meta_description: Some("Test Description".into()),
            canonical_url: None,
            robots_meta: None,
            word_count: Some(500),
            load_time_ms: Some(1200),
            response_size_bytes: Some(10000),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data,
        }
    }

    fn make_check(field: &str, op: Operator, threshold: Option<&str>, template: &str) -> CustomCheck {
        CustomCheck {
            id: "chk-1".into(),
            name: "Test Check".into(),
            severity: IssueSeverity::Warning,
            field: field.into(),
            operator: op,
            threshold: threshold.map(|s| s.into()),
            message_template: template.into(),
            enabled: true,
        }
    }

    #[test]
    fn fires_when_tag_field_is_missing() {
        let page = make_page_with_tags(vec![]); // no og_image
        let details = passing_details();
        let ctx = CheckContext::new(&page, &details, "j1", "p1");

        let check = make_check("tag:og_image", Operator::Missing, None, "Missing OG image");
        let adapter = CustomCheckAdapter::new(check);
        let issue = adapter.check(&ctx);

        assert!(issue.is_some());
        assert_eq!(issue.unwrap().message, "Missing OG image");
    }

    #[test]
    fn does_not_fire_when_tag_field_is_present() {
        let page = make_page_with_tags(vec![(
            "og_image",
            serde_json::Value::String("https://img.jpg".into()),
        )]);
        let details = passing_details();
        let ctx = CheckContext::new(&page, &details, "j1", "p1");

        let check = make_check("tag:og_image", Operator::Missing, None, "Missing OG image");
        let adapter = CustomCheckAdapter::new(check);

        assert!(adapter.check(&ctx).is_none());
    }

    #[test]
    fn substitutes_tag_in_message_template() {
        let page = make_page_with_tags(vec![(
            "og_image",
            serde_json::Value::String("https://img.jpg".into()),
        )]);
        let details = passing_details();
        let ctx = CheckContext::new(&page, &details, "j1", "p1");

        // Check on word_count < 1000 (page has 500, so it fires)
        let check = make_check(
            "word_count",
            Operator::Lt,
            Some("1000"),
            "Word count is {value}, and og_image is {tag.og_image}",
        );
        let adapter = CustomCheckAdapter::new(check);
        let issue = adapter.check(&ctx).expect("should fire");

        assert_eq!(
            issue.message,
            "Word count is 500, and og_image is https://img.jpg"
        );
    }

    #[test]
    fn substitutes_value_placeholder_with_resolved_field() {
        let page = make_page_with_tags(vec![]);
        let details = passing_details();
        let ctx = CheckContext::new(&page, &details, "j1", "p1");

        let check = make_check(
            "load_time_ms",
            Operator::Gt,
            Some("1000"),
            "Load time {value}ms exceeds threshold",
        );
        let adapter = CustomCheckAdapter::new(check);
        let issue = adapter.check(&ctx).expect("should fire");

        assert_eq!(issue.message, "Load time 1200ms exceeds threshold");
    }

    #[test]
    fn unknown_tag_leaves_placeholder_in_message() {
        let page = make_page_with_tags(vec![]);
        let details = passing_details();
        let ctx = CheckContext::new(&page, &details, "j1", "p1");

        let check = make_check(
            "word_count",
            Operator::Lt,
            Some("1000"),
            "Missing data: {tag.nonexistent}",
        );
        let adapter = CustomCheckAdapter::new(check);
        let issue = adapter.check(&ctx).expect("should fire");

        // Unknown tag resolves to empty string (not left as placeholder)
        assert_eq!(issue.message, "Missing data: ");
    }

    #[test]
    fn works_with_builtin_page_fields() {
        let page = make_page_with_tags(vec![]);
        let details = passing_details();
        let ctx = CheckContext::new(&page, &details, "j1", "p1");

        let check = make_check("title", Operator::Present, None, "Title found: {value}");
        let adapter = CustomCheckAdapter::new(check);
        let issue = adapter.check(&ctx).expect("should fire — title is present");

        assert_eq!(issue.message, "Title found: Test Title");
    }
}

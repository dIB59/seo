use std::collections::HashMap;

use crate::contexts::analysis::{CompleteJobResult, Heading, Page};
use crate::contexts::extension::Operator;
use crate::contexts::report::domain::{DetectedPattern, PatternCategory, PillarScores, ReportPattern};

// ── Field value ───────────────────────────────────────────────────────────────

pub(crate) enum FieldValue {
    Null,
    Text(String),
    Number(f64),
    Bool(bool),
}

impl FieldValue {
    fn as_comparable_string(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            Self::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

// ── Field resolver ────────────────────────────────────────────────────────────

/// Prefix marker for fields that look up a tag in `page.extracted_data`
/// instead of being built-ins on `Page`. The tag name is whatever the
/// user set as `CustomExtractor.tag` — rule fields written as
/// `tag:og_image` resolve against `page.extracted_data["og_image"]`.
///
/// Migration note: pre-0034 rules used the `extracted:` prefix for the
/// same thing. The 0034 migration rewrites existing rows; this is the
/// only supported prefix going forward.
pub const TAG_FIELD_PREFIX: &str = "tag:";

/// Resolve a named field from page data.
///
/// Supported built-in fields:
/// - `meta_description`, `title`, `canonical_url` — `Option<String>` → Null or Text
/// - `word_count`, `load_time_ms`, `status_code` — `Option<i64>` → Null or Number
/// - `has_viewport`, `has_structured_data` — `bool` → Bool
/// - `h1_count` — derived from pre-computed `h1_counts` map
/// - `tag:<extractor_tag>` — value from `page.extracted_data`
pub(crate) fn resolve_field(
    page: &Page,
    h1_counts: &HashMap<String, usize>,
    field: &str,
) -> FieldValue {
    match field {
        "meta_description" => opt_string_field(page.meta_description.as_deref()),
        "title" => opt_string_field(page.title.as_deref()),
        "canonical_url" => opt_string_field(page.canonical_url.as_deref()),
        "word_count" => opt_number_field(page.word_count),
        "load_time_ms" => opt_number_field(page.load_time_ms),
        "status_code" => opt_number_field(page.status_code),
        "has_viewport" => FieldValue::Bool(page.has_viewport),
        "has_structured_data" => FieldValue::Bool(page.has_structured_data),
        "h1_count" => {
            let count = h1_counts.get(&page.id).copied().unwrap_or(0);
            FieldValue::Number(count as f64)
        }
        other if other.starts_with(TAG_FIELD_PREFIX) => {
            let tag = &other[TAG_FIELD_PREFIX.len()..];
            match page.extracted_data.get(tag) {
                None => FieldValue::Null,
                Some(v) => match v {
                    serde_json::Value::Null => FieldValue::Null,
                    serde_json::Value::Bool(b) => FieldValue::Bool(*b),
                    serde_json::Value::Number(n) => {
                        FieldValue::Number(n.as_f64().unwrap_or(0.0))
                    }
                    serde_json::Value::String(s) => FieldValue::Text(s.clone()),
                    other => FieldValue::Text(other.to_string()),
                },
            }
        }
        _ => FieldValue::Null,
    }
}

fn opt_string_field(v: Option<&str>) -> FieldValue {
    match v {
        None | Some("") => FieldValue::Null,
        Some(s) => FieldValue::Text(s.to_string()),
    }
}

fn opt_number_field(v: Option<i64>) -> FieldValue {
    match v {
        None => FieldValue::Null,
        Some(n) => FieldValue::Number(n as f64),
    }
}

// ── Evaluator ─────────────────────────────────────────────────────────────────

pub(crate) fn evaluate_condition(value: &FieldValue, op: &Operator, threshold: Option<&str>) -> bool {
    match op {
        Operator::Missing => value.is_null(),
        Operator::Present => !value.is_null(),
        Operator::Eq => {
            let t = threshold.unwrap_or("");
            match value {
                FieldValue::Null => false,
                FieldValue::Bool(b) => {
                    let bstr = if *b { "true" } else { "false" };
                    bstr == t
                }
                FieldValue::Number(n) => {
                    if let Ok(tn) = t.parse::<f64>() {
                        (n - tn).abs() < f64::EPSILON
                    } else {
                        false
                    }
                }
                FieldValue::Text(s) => s.as_str() == t,
            }
        }
        // A missing or malformed threshold previously fell back to 0.0
        // — turning a typo'd rule (threshold `"1oo"`) silently into
        // "< 0" / "> 0", which is almost never what the author meant.
        // Now an unparseable threshold causes the condition to not
        // match (same as Eq) and a warning is logged so the pattern
        // author can fix the rule.
        Operator::Lt | Operator::Gt => {
            let Some(t) = threshold.and_then(|s| s.parse::<f64>().ok()) else {
                if let Some(raw) = threshold {
                    tracing::warn!(
                        "pattern_engine: unparseable {op:?} threshold '{raw}'; rule will not match",
                        op = op
                    );
                }
                return false;
            };
            let Some(n) = value.as_number() else { return false };
            match op {
                Operator::Lt => n < t,
                Operator::Gt => n > t,
                _ => unreachable!("outer match arm guarantees Lt|Gt"),
            }
        }
        Operator::Contains => {
            let t = threshold.unwrap_or("");
            value.as_comparable_string().map(|s| s.contains(t)).unwrap_or(false)
        }
        Operator::NotContains => {
            let t = threshold.unwrap_or("");
            value.as_comparable_string().map(|s| !s.contains(t)).unwrap_or(false)
        }
    }
}

// ── Pattern engine ────────────────────────────────────────────────────────────

/// Pre-compute H1 heading counts per page from the full headings list.
fn build_h1_counts(headings: &[Heading]) -> HashMap<String, usize> {
    let mut map: HashMap<String, usize> = HashMap::new();
    for h in headings {
        if h.level == 1 {
            *map.entry(h.page_id.clone()).or_insert(0) += 1;
        }
    }
    map
}

/// Evaluate all enabled patterns against the complete job result.
/// Returns only patterns that met their `min_prevalence` threshold.
pub fn evaluate_all(
    patterns: &[ReportPattern],
    result: &CompleteJobResult,
) -> Vec<DetectedPattern> {
    let pages = &result.pages;
    let total = pages.len();

    if total == 0 {
        return vec![];
    }

    let h1_counts = build_h1_counts(&result.headings);

    let mut detected: Vec<DetectedPattern> = patterns
        .iter()
        .filter_map(|pattern| {
            let threshold = pattern.threshold.as_deref();
            let mut affected_urls: Vec<String> = Vec::new();

            for page in pages {
                let value = resolve_field(page, &h1_counts, &pattern.field);
                if evaluate_condition(&value, &pattern.operator, threshold) {
                    affected_urls.push(page.url.clone());
                }
            }

            let affected = affected_urls.len();
            let prevalence = affected as f64 / total as f64;

            if prevalence < pattern.min_prevalence {
                return None;
            }

            let sample_urls: Vec<String> = affected_urls.into_iter().take(5).collect();
            let priority_score = DetectedPattern::compute_priority(pattern, prevalence);

            Some(DetectedPattern {
                pattern: pattern.clone(),
                prevalence,
                affected_pages: affected,
                total_pages: total,
                priority_score,
                sample_urls,
            })
        })
        .collect();

    // `total_cmp` gives a deterministic total ordering on f64 including
    // NaN, so we don't need the `unwrap_or(Equal)` escape hatch that
    // silently reorders NaN-containing runs.
    detected.sort_by(|a, b| b.priority_score.total_cmp(&a.priority_score));
    detected
}

// ── Pillar scores ─────────────────────────────────────────────────────────────

/// Compute pillar health scores (0–100) from the set of detected patterns.
/// Each detected pattern deducts `severity_weight × prevalence × 15` from its pillar.
pub fn compute_pillar_scores(detected: &[DetectedPattern]) -> PillarScores {
    let mut technical: f64 = 100.0;
    let mut content: f64 = 100.0;
    let mut performance: f64 = 100.0;
    let mut accessibility: f64 = 100.0;

    for d in detected {
        let deduction = d.pattern.severity.weight() * d.prevalence * 15.0;
        match d.pattern.category {
            PatternCategory::Technical => technical -= deduction,
            PatternCategory::Content => content -= deduction,
            PatternCategory::Performance => performance -= deduction,
            PatternCategory::Accessibility => accessibility -= deduction,
        }
    }

    let clamp = |v: f64| v.clamp(0.0, 100.0);
    let technical     = clamp(technical);
    let content       = clamp(content);
    let performance   = clamp(performance);
    let accessibility = clamp(accessibility);
    let overall = (technical + content + performance + accessibility) / 4.0;

    PillarScores::new(technical, content, performance, accessibility, overall)
}

#[cfg(test)]
mod tests {
    //! Characterization tests for the report pattern engine. The
    //! engine is the heart of the report generation pipeline — these
    //! tests pin the field-resolution rules, the operator dispatch,
    //! and the pillar score formula so any future refactor lands
    //! under green.

    use super::*;
    use crate::contexts::analysis::{Depth, Heading, NewLink, Page};
    use crate::contexts::report::domain::{
        BusinessImpact, FixEffort, PatternCategory, PatternSeverity, ReportPattern,
    };
    use chrono::Utc;

    fn make_pattern(
        category: PatternCategory,
        severity: PatternSeverity,
        field: &str,
        operator: Operator,
        threshold: Option<&str>,
        min_prevalence: f64,
    ) -> ReportPattern {
        ReportPattern {
            id: "p".into(),
            name: format!("{} {}", field, operator),
            description: "test".into(),
            category,
            severity,
            field: field.to_string(),
            operator,
            threshold: threshold.map(String::from),
            min_prevalence,
            business_impact: BusinessImpact::Medium,
            fix_effort: FixEffort::Medium,
            recommendation: "fix".into(),
            is_builtin: false,
            enabled: true,
        }
    }

    fn make_page(id: &str, url: &str, title: Option<&str>, word_count: Option<i64>) -> Page {
        Page {
            id: id.into(),
            job_id: "j".into(),
            url: url.into(),
            depth: Depth::root(),
            status_code: Some(200),
            content_type: None,
            title: title.map(String::from),
            meta_description: None,
            canonical_url: None,
            robots_meta: None,
            word_count,
            load_time_ms: Some(500),
            response_size_bytes: None,
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        }
    }

    fn make_complete_result(pages: Vec<Page>, headings: Vec<Heading>) -> CompleteJobResult {
        use crate::contexts::analysis::{Job, JobId, JobSettings, JobStatus, JobSummary};
        CompleteJobResult {
            job: Job {
                id: JobId::from("j"),
                url: "https://example.com".into(),
                status: JobStatus::Completed,
                settings: JobSettings::default(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                completed_at: Some(Utc::now()),
                summary: JobSummary::default(),
                progress: 100.0,
                error_message: None,
                sitemap_found: false,
                robots_txt_found: false,
            },
            pages,
            issues: Vec::new(),
            links: Vec::<crate::contexts::analysis::Link>::new(),
            lighthouse: Vec::new(),
            headings,
            images: Vec::new(),
            ai_insights: None,
            extracted_data: std::collections::HashMap::new(),
        }
    }

    // ── opt_string_field / opt_number_field ──────────────────────────────

    #[test]
    fn opt_string_field_treats_none_and_empty_as_null() {
        assert!(opt_string_field(None).is_null());
        assert!(opt_string_field(Some("")).is_null());
        assert!(!opt_string_field(Some("hello")).is_null());
    }

    #[test]
    fn opt_number_field_treats_none_as_null_some_as_number() {
        assert!(opt_number_field(None).is_null());
        let v = opt_number_field(Some(42));
        assert!(!v.is_null());
        assert_eq!(v.as_number(), Some(42.0));
    }

    // ── evaluate_condition ───────────────────────────────────────────────

    #[test]
    fn evaluate_missing_returns_true_for_null_value() {
        assert!(evaluate_condition(&FieldValue::Null, &Operator::Missing, None));
    }

    #[test]
    fn evaluate_missing_returns_false_for_non_null_value() {
        assert!(!evaluate_condition(
            &FieldValue::Text("x".into()),
            &Operator::Missing,
            None
        ));
    }

    #[test]
    fn evaluate_present_inverts_missing() {
        assert!(!evaluate_condition(&FieldValue::Null, &Operator::Present, None));
        assert!(evaluate_condition(
            &FieldValue::Number(0.0),
            &Operator::Present,
            None
        ));
    }

    #[test]
    fn evaluate_lt_compares_numbers_against_threshold() {
        assert!(evaluate_condition(
            &FieldValue::Number(50.0),
            &Operator::Lt,
            Some("100"),
        ));
        assert!(!evaluate_condition(
            &FieldValue::Number(150.0),
            &Operator::Lt,
            Some("100"),
        ));
    }

    #[test]
    fn evaluate_gt_compares_numbers_against_threshold() {
        assert!(evaluate_condition(
            &FieldValue::Number(150.0),
            &Operator::Gt,
            Some("100"),
        ));
        assert!(!evaluate_condition(
            &FieldValue::Number(50.0),
            &Operator::Gt,
            Some("100"),
        ));
    }

    #[test]
    fn evaluate_eq_matches_text_exactly() {
        assert!(evaluate_condition(
            &FieldValue::Text("hello".into()),
            &Operator::Eq,
            Some("hello"),
        ));
        assert!(!evaluate_condition(
            &FieldValue::Text("hello".into()),
            &Operator::Eq,
            Some("world"),
        ));
    }

    #[test]
    fn evaluate_eq_handles_bool_via_string_form() {
        assert!(evaluate_condition(
            &FieldValue::Bool(true),
            &Operator::Eq,
            Some("true"),
        ));
        assert!(evaluate_condition(
            &FieldValue::Bool(false),
            &Operator::Eq,
            Some("false"),
        ));
        assert!(!evaluate_condition(
            &FieldValue::Bool(true),
            &Operator::Eq,
            Some("false"),
        ));
    }

    #[test]
    fn evaluate_contains_returns_true_when_substring_present() {
        assert!(evaluate_condition(
            &FieldValue::Text("hello world".into()),
            &Operator::Contains,
            Some("world"),
        ));
        assert!(!evaluate_condition(
            &FieldValue::Text("hello world".into()),
            &Operator::Contains,
            Some("missing"),
        ));
    }

    #[test]
    fn evaluate_not_contains_inverts_contains() {
        assert!(!evaluate_condition(
            &FieldValue::Text("hello world".into()),
            &Operator::NotContains,
            Some("world"),
        ));
        assert!(evaluate_condition(
            &FieldValue::Text("hello world".into()),
            &Operator::NotContains,
            Some("missing"),
        ));
    }

    // ── build_h1_counts ──────────────────────────────────────────────────

    #[test]
    fn build_h1_counts_counts_only_level_1_headings() {
        let headings = vec![
            Heading {
                id: 1,
                page_id: "p1".into(),
                level: 1,
                text: "main".into(),
                position: 0,
            },
            Heading {
                id: 2,
                page_id: "p1".into(),
                level: 2,
                text: "sub".into(),
                position: 1,
            },
            Heading {
                id: 3,
                page_id: "p1".into(),
                level: 1,
                text: "another main".into(),
                position: 2,
            },
            Heading {
                id: 4,
                page_id: "p2".into(),
                level: 1,
                text: "main".into(),
                position: 0,
            },
        ];
        let counts = build_h1_counts(&headings);
        assert_eq!(counts.get("p1").copied(), Some(2));
        assert_eq!(counts.get("p2").copied(), Some(1));
    }

    #[test]
    fn build_h1_counts_returns_empty_for_no_h1s() {
        let headings = vec![Heading {
            id: 1,
            page_id: "p1".into(),
            level: 2,
            text: "h2".into(),
            position: 0,
        }];
        let counts = build_h1_counts(&headings);
        assert!(counts.is_empty());
    }

    // ── evaluate_all ─────────────────────────────────────────────────────

    #[test]
    fn evaluate_all_returns_empty_for_zero_pages() {
        let pattern = make_pattern(
            PatternCategory::Content,
            PatternSeverity::Warning,
            "title",
            Operator::Missing,
            None,
            0.0,
        );
        let result = make_complete_result(vec![], vec![]);
        let detected = evaluate_all(&[pattern], &result);
        assert!(detected.is_empty());
    }

    #[test]
    fn evaluate_all_detects_missing_titles_above_min_prevalence() {
        let pattern = make_pattern(
            PatternCategory::Content,
            PatternSeverity::Warning,
            "title",
            Operator::Missing,
            None,
            0.4, // 40% threshold
        );
        let pages = vec![
            make_page("p1", "https://a.test/1", None, Some(500)),       // missing → match
            make_page("p2", "https://a.test/2", None, Some(500)),       // missing → match
            make_page("p3", "https://a.test/3", Some("OK"), Some(500)), // present
            make_page("p4", "https://a.test/4", Some("OK"), Some(500)), // present
        ];
        let result = make_complete_result(pages, vec![]);
        let detected = evaluate_all(&[pattern], &result);
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].affected_pages, 2);
        assert_eq!(detected[0].total_pages, 4);
        assert_eq!(detected[0].prevalence, 0.5);
    }

    #[test]
    fn evaluate_all_filters_below_min_prevalence() {
        let pattern = make_pattern(
            PatternCategory::Content,
            PatternSeverity::Warning,
            "title",
            Operator::Missing,
            None,
            0.6, // 60% threshold
        );
        let pages = vec![
            make_page("p1", "https://a.test/1", None, Some(500)), // 1 of 4 = 25%
            make_page("p2", "https://a.test/2", Some("OK"), Some(500)),
            make_page("p3", "https://a.test/3", Some("OK"), Some(500)),
            make_page("p4", "https://a.test/4", Some("OK"), Some(500)),
        ];
        let result = make_complete_result(pages, vec![]);
        let detected = evaluate_all(&[pattern], &result);
        assert!(detected.is_empty());
    }

    #[test]
    fn evaluate_all_caps_sample_urls_at_5() {
        let pattern = make_pattern(
            PatternCategory::Content,
            PatternSeverity::Warning,
            "title",
            Operator::Missing,
            None,
            0.0,
        );
        let pages: Vec<Page> = (0..10)
            .map(|i| make_page(&format!("p{i}"), &format!("https://a.test/{i}"), None, Some(500)))
            .collect();
        let result = make_complete_result(pages, vec![]);
        let detected = evaluate_all(&[pattern], &result);
        assert_eq!(detected[0].sample_urls.len(), 5);
        assert_eq!(detected[0].affected_pages, 10);
    }

    #[test]
    fn evaluate_all_sorts_by_priority_score_descending() {
        // Two patterns; the critical one should rank above the warning.
        let critical = make_pattern(
            PatternCategory::Content,
            PatternSeverity::Critical,
            "title",
            Operator::Missing,
            None,
            0.0,
        );
        let warning = make_pattern(
            PatternCategory::Content,
            PatternSeverity::Warning,
            "word_count",
            Operator::Lt,
            Some("100"),
            0.0,
        );
        let pages = vec![
            make_page("p1", "https://a.test/1", None, Some(50)),
        ];
        let result = make_complete_result(pages, vec![]);
        let detected = evaluate_all(&[warning, critical], &result);
        // critical sorts first regardless of input order
        assert_eq!(detected.len(), 2);
        assert_eq!(detected[0].pattern.severity, PatternSeverity::Critical);
        assert_eq!(detected[1].pattern.severity, PatternSeverity::Warning);
    }

    // ── compute_pillar_scores ────────────────────────────────────────────

    fn detected_with(
        category: PatternCategory,
        severity: PatternSeverity,
        prevalence: f64,
    ) -> DetectedPattern {
        DetectedPattern {
            pattern: make_pattern(
                category,
                severity,
                "title",
                Operator::Missing,
                None,
                0.0,
            ),
            prevalence,
            affected_pages: 1,
            total_pages: 1,
            priority_score: 0.0,
            sample_urls: vec![],
        }
    }

    #[test]
    fn pillar_scores_default_to_100_with_no_detected_patterns() {
        let scores = compute_pillar_scores(&[]);
        assert_eq!(scores.technical(), 100.0);
        assert_eq!(scores.content(), 100.0);
        assert_eq!(scores.performance(), 100.0);
        assert_eq!(scores.accessibility(), 100.0);
        assert_eq!(scores.overall(), 100.0);
    }

    #[test]
    fn pillar_scores_deduct_per_pattern_using_severity_weight() {
        // Critical(3) × prevalence(0.5) × 15 = 22.5 deduction.
        let detected = vec![detected_with(
            PatternCategory::Technical,
            PatternSeverity::Critical,
            0.5,
        )];
        let scores = compute_pillar_scores(&detected);
        assert_eq!(scores.technical(), 77.5);
        // Other pillars unaffected.
        assert_eq!(scores.content(), 100.0);
        assert_eq!(scores.performance(), 100.0);
        assert_eq!(scores.accessibility(), 100.0);
    }

    #[test]
    fn pillar_scores_route_each_category_to_its_own_pillar() {
        let detected = vec![
            detected_with(PatternCategory::Technical, PatternSeverity::Warning, 0.5),
            detected_with(PatternCategory::Content, PatternSeverity::Warning, 0.5),
            detected_with(PatternCategory::Performance, PatternSeverity::Warning, 0.5),
            detected_with(PatternCategory::Accessibility, PatternSeverity::Warning, 0.5),
        ];
        // Warning(2) × 0.5 × 15 = 15 deduction per pillar
        let scores = compute_pillar_scores(&detected);
        assert_eq!(scores.technical(), 85.0);
        assert_eq!(scores.content(), 85.0);
        assert_eq!(scores.performance(), 85.0);
        assert_eq!(scores.accessibility(), 85.0);
        assert_eq!(scores.overall(), 85.0);
    }

    #[test]
    fn pillar_scores_clamp_to_zero_floor() {
        // Many critical patterns at 100% prevalence — total deduction
        // would push the score negative. Pin the clamp.
        let detected = vec![
            detected_with(PatternCategory::Technical, PatternSeverity::Critical, 1.0),
            detected_with(PatternCategory::Technical, PatternSeverity::Critical, 1.0),
            detected_with(PatternCategory::Technical, PatternSeverity::Critical, 1.0),
            detected_with(PatternCategory::Technical, PatternSeverity::Critical, 1.0),
        ];
        let scores = compute_pillar_scores(&detected);
        assert_eq!(scores.technical(), 0.0);
        assert!(scores.technical() >= 0.0);
    }

    #[test]
    fn pillar_scores_overall_is_simple_average_of_four() {
        let detected = vec![detected_with(
            PatternCategory::Technical,
            PatternSeverity::Warning,
            1.0,
        )];
        // Technical: 100 - (2 × 1 × 15) = 70
        // Others: 100 each
        // Overall: (70 + 100 + 100 + 100) / 4 = 92.5
        let scores = compute_pillar_scores(&detected);
        assert_eq!(scores.technical(), 70.0);
        assert_eq!(scores.overall(), 92.5);
    }

    // Suppress unused-import warning when the test module is the only
    // place these are referenced.
    #[allow(dead_code)]
    fn _force_link_use(_: NewLink) {}
}

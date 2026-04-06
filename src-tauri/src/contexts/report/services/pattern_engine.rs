use std::collections::HashMap;

use crate::contexts::analysis::{CompleteJobResult, Heading, Page};
use crate::contexts::extension::Operator;
use crate::contexts::report::domain::{DetectedPattern, PatternCategory, PillarScores, ReportPattern};

// ── Field value ───────────────────────────────────────────────────────────────

enum FieldValue {
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

/// Resolve a named field from page data.
///
/// Supported built-in fields:
/// - `meta_description`, `title`, `canonical_url` — `Option<String>` → Null or Text
/// - `word_count`, `load_time_ms`, `status_code` — `Option<i64>` → Null or Number
/// - `has_viewport`, `has_structured_data` — `bool` → Bool
/// - `h1_count` — derived from pre-computed `h1_counts` map
/// - `extracted:<key>` — value from `page.extracted_data`
fn resolve_field(
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
        other if other.starts_with("extracted:") => {
            let key = &other["extracted:".len()..];
            match page.extracted_data.get(key) {
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
        None => FieldValue::Null,
        Some(s) if s.is_empty() => FieldValue::Null,
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

fn evaluate_condition(value: &FieldValue, op: &Operator, threshold: Option<&str>) -> bool {
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
        Operator::Lt => {
            let t = threshold.and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            value.as_number().map(|n| n < t).unwrap_or(false)
        }
        Operator::Gt => {
            let t = threshold.and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            value.as_number().map(|n| n > t).unwrap_or(false)
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

    detected.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap_or(std::cmp::Ordering::Equal));
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

    PillarScores { technical, content, performance, accessibility, overall }
}

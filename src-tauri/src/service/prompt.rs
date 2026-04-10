//! Shared prompt utilities used by all AI backends (Gemini, local model, report brief).
//!
//! Single source of truth for:
//!   - The default persona / system prompt
//!   - Variable substitution into prompt templates
//!   - Assembling a final prompt from persona + prompt blocks
//!   - Loading the saved persona with default fallback

use crate::repository::SettingsRepository;
use crate::service::gemini::{GeminiRequest, PromptBlock};

/// Persona setting key shared by every AI backend.
pub const PERSONA_SETTING_KEY: &str = "gemini_persona";

/// Prompt-blocks setting key shared by every AI backend.
pub const PROMPT_BLOCKS_SETTING_KEY: &str = "gemini_prompt_blocks";

/// Load and parse the user's saved prompt blocks. Missing setting → empty
/// list; malformed JSON → empty list with a `warn!`. Centralized so the
/// JSON-decode + drift logging lives in one place instead of being copied
/// into every AI backend.
pub async fn load_prompt_blocks(
    settings_repo: &dyn SettingsRepository,
) -> crate::repository::RepositoryResult<Vec<PromptBlock>> {
    let blocks_json = settings_repo
        .get_setting(PROMPT_BLOCKS_SETTING_KEY)
        .await?
        .unwrap_or_else(|| "[]".to_string());
    Ok(serde_json::from_str(&blocks_json).unwrap_or_else(|e| {
        tracing::warn!(
            "prompt: invalid prompt blocks JSON in settings ({e}); defaulting to empty list"
        );
        Vec::new()
    }))
}

/// Look up the user's saved persona, falling back to [`DEFAULT_PERSONA`] if
/// missing or empty. Centralized so all AI backends share the same
/// "missing-or-empty → default" semantics — previously this match
/// expression was duplicated in `gemini.rs`, `local_model_service.rs`,
/// `report_service.rs`, and `commands/ai.rs`.
pub async fn load_persona(
    settings_repo: &dyn SettingsRepository,
) -> crate::repository::RepositoryResult<String> {
    Ok(match settings_repo.get_setting(PERSONA_SETTING_KEY).await? {
        Some(p) if !p.trim().is_empty() => p,
        _ => DEFAULT_PERSONA.to_string(),
    })
}

// Fallback persona — returned to the frontend when no custom persona is saved,
// so the user sees and can edit this text directly in Settings → AI Instructions.
pub const DEFAULT_PERSONA: &str = "\
You are a senior SEO consultant producing a professional audit report. \
Your work transforms raw audit data into clear, directive, business-focused narrative.

HOOK — Open every section with a consequence specific to this site's numbers: \
lost impressions, missed conversions, revenue at risk. Name the numbers. \
A strong hook is true only for this site — not any site.

STRUCTURE — Follow this sequence for every section:
1. Diagnosis — what is happening and how widespread it is
2. Consequence — what it costs in organic traffic, leads, or revenue
3. Priority action — exactly what to do first, stated as a directive

One priority action per issue. Not a list of options. The client needs direction.

VOICE — Short, declarative sentences. Authoritative. No hedging \
('you might want to consider', 'it could be worth'). No filler openers \
('In today's SEO landscape', 'It is important to note'). \
If a sentence can be cut without losing meaning, cut it.

EVIDENCE — Every claim must reference the specific data provided: this URL, \
these page counts, these scores. No generic advice that applies to any site.

CTA — End every response with one sentence: the single most impactful action \
this client should take this week. Not a list. One imperative, specific enough \
to act on immediately.

NEVER repeat raw data verbatim. NEVER give equal weight to all issues. \
NEVER write more than the task requires.";

// ── Variable substitution ─────────────────────────────────────────────────────

/// Replace `{variable}` placeholders in a prompt template with values from
/// the analysis request.  Unrecognised placeholders are left untouched.
///
/// Also resolves `{tag.X}` placeholders against the request's
/// `tag_values` map — a site-level aggregation of custom-extractor
/// results. Unknown tags resolve to an empty string.
pub fn replace_prompt_vars(text: &str, request: &GeminiRequest) -> String {
    let replacements: &[(&str, String)] = &[
        ("{url}", request.url.clone()),
        ("{score}", request.seo_score.to_string()),
        ("{pages_count}", request.pages_count.to_string()),
        ("{total_issues}", request.total_issues.to_string()),
        ("{critical_issues}", request.critical_issues.to_string()),
        ("{warning_issues}", request.warning_issues.to_string()),
        ("{suggestion_issues}", request.suggestion_issues.to_string()),
        ("{top_issues}", request.top_issues.join("\n")),
        ("{avg_load_time}", format!("{:.2}", request.avg_load_time)),
        ("{total_words}", request.total_words.to_string()),
        ("{ssl_certificate}", bool_yn(request.ssl_certificate).to_string()),
        ("{sitemap_found}", bool_yn(request.sitemap_found).to_string()),
        ("{robots_txt_found}", bool_yn(request.robots_txt_found).to_string()),
        // Rich context
        ("{issue_details}", request.issue_details.join("\n")),
        ("{page_summaries}", request.page_summaries.join("\n")),
        ("{missing_meta_count}", request.missing_meta_count.to_string()),
        ("{slow_pages_count}", request.slow_pages_count.to_string()),
        ("{error_pages_count}", request.error_pages_count.to_string()),
    ];

    let mut result = replacements
        .iter()
        .fold(text.to_string(), |acc, (pat, val)| acc.replace(pat, val));

    // Second pass: resolve {tag.X} against the request's tag_values map.
    result = replace_tag_vars(&result, &request.tag_values);

    result
}

/// Replace `{tag.X}` placeholders against a tag-values map. Shared
/// between the prompt builder and the custom check message substitution
/// (which uses a different input source — per-page extracted_data —
/// but the same placeholder syntax).
pub fn replace_tag_vars(
    text: &str,
    tag_values: &std::collections::HashMap<String, String>,
) -> String {
    let prefix = "{tag.";
    let mut result = text.to_string();
    while let Some(start) = result.find(prefix) {
        let rest = &result[start + prefix.len()..];
        let Some(end) = rest.find('}') else { break };
        let tag_name = &rest[..end];
        let replacement = tag_values
            .get(tag_name)
            .cloned()
            .unwrap_or_default();
        let full = format!("{prefix}{tag_name}}}");
        result = result.replacen(&full, &replacement, 1);
    }
    result
}

fn bool_yn(v: bool) -> &'static str {
    if v { "Yes" } else { "No" }
}

/// Build a site-level `tag_values` map by scanning every page's
/// `extracted_data`. For each tag, collect up to 5 distinct string
/// values and join them with `", "`. The result is suitable for
/// dropping into `GeminiRequest.tag_values` or the template engine's
/// `RenderContext`.
///
/// Call at the point where pages are available (e.g. report generation,
/// AI insight generation with full results). Callers without page data
/// (the basic prompt preview) can skip this and pass an empty map.
pub fn aggregate_tag_values(
    pages: &[crate::contexts::analysis::Page],
) -> std::collections::HashMap<String, String> {
    use std::collections::{HashMap, HashSet};

    let mut per_tag: HashMap<String, HashSet<String>> = HashMap::new();

    for page in pages {
        for (key, value) in &page.extracted_data {
            let text = match value {
                serde_json::Value::String(s) if !s.is_empty() => s.clone(),
                serde_json::Value::Array(arr) => {
                    // Multi-valued extractors: join the elements
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
                serde_json::Value::Null => continue,
                other => other.to_string(),
            };
            if text.is_empty() {
                continue;
            }
            let set = per_tag.entry(key.clone()).or_default();
            if set.len() < 5 {
                set.insert(text);
            }
        }
    }

    per_tag
        .into_iter()
        .map(|(k, vals)| {
            let mut sorted: Vec<String> = vals.into_iter().collect();
            sorted.sort();
            (k, sorted.join(", "))
        })
        .collect()
}

// ── Prompt assembly ───────────────────────────────────────────────────────────

/// Build the final prompt string from:
///   - `persona`  — system-level instructions / persona text
///   - `blocks`   — ordered prompt blocks (type "text" | "variable")
///   - `request`  — analysis data for variable substitution
///
/// This is the canonical assembly function used by every AI backend so they
/// all produce prompts in the same shape.
pub fn build_prompt_from_blocks(
    persona: &str,
    blocks: &[PromptBlock],
    request: &GeminiRequest,
) -> String {
    let persona_text = replace_prompt_vars(persona, request);

    let mut parts: Vec<String> = blocks
        .iter()
        .map(|b| replace_prompt_vars(&b.content, request))
        .collect();

    // Fallback data section when no blocks have been configured.
    // Includes the rich context fields so the AI gets real signal to
    // write about — not just bare counts.
    if parts.is_empty() {
        let mut data = format!(
            "Website: {url}\n\
            SEO Score: {score}/100\n\
            Pages analyzed: {pages}\n\
            Issues: {total} total ({critical} critical, {warnings} warnings, {suggestions} info)\n\
            Average load time: {load:.2}s\n\
            Total words: {words}\n\
            SSL: {ssl} | Sitemap: {sitemap} | Robots.txt: {robots}\n\
            Pages missing meta description: {missing_meta}\n\
            Slow pages (>3s): {slow}\n\
            Error pages (4xx/5xx): {errors}",
            url          = request.url,
            score        = request.seo_score,
            pages        = request.pages_count,
            total        = request.total_issues,
            critical     = request.critical_issues,
            warnings     = request.warning_issues,
            suggestions  = request.suggestion_issues,
            load         = request.avg_load_time,
            words        = request.total_words,
            ssl          = bool_yn(request.ssl_certificate),
            sitemap      = bool_yn(request.sitemap_found),
            robots       = bool_yn(request.robots_txt_found),
            missing_meta = request.missing_meta_count,
            slow         = request.slow_pages_count,
            errors       = request.error_pages_count,
        );

        if !request.issue_details.is_empty() {
            data.push_str("\n\nIssue details (severity | title | page | description):\n");
            for line in &request.issue_details {
                data.push_str(line);
                data.push('\n');
            }
        }

        if !request.page_summaries.is_empty() {
            data.push_str("\nPages by issue count (url | title | status | load_ms | issues):\n");
            for line in &request.page_summaries {
                data.push_str(line);
                data.push('\n');
            }
        }

        parts.push(data);
    }

    format!(
        "{}\n\nAnalyze the following SEO audit results:\n\n{}",
        persona_text,
        parts.join("\n\n"),
    )
}

#[cfg(test)]
mod tests {
    //! Characterization tests for prompt-template substitution and
    //! assembly. These pin the placeholder syntax (`{url}` etc.), the
    //! `Yes`/`No` boolean spelling, and the assembly fallback when no
    //! blocks are configured. The output is sent verbatim to LLM
    //! backends, so any change is observable.

    use super::*;
    use crate::service::gemini::{GeminiRequest, PromptBlock};

    fn fixture_request() -> GeminiRequest {
        let mut tag_values = std::collections::HashMap::new();
        tag_values.insert("og_image".into(), "https://img.jpg".into());
        GeminiRequest {
            analysis_id: "a1".to_string(),
            url: "https://example.com".to_string(),
            seo_score: 87,
            pages_count: 42,
            total_issues: 9,
            critical_issues: 2,
            warning_issues: 5,
            suggestion_issues: 2,
            top_issues: vec!["missing meta".into(), "slow load".into()],
            avg_load_time: 1234.5,
            total_words: 12_000,
            ssl_certificate: true,
            sitemap_found: true,
            robots_txt_found: false,
            issue_details: vec![
                "critical | Missing Title | /page1 | No title tag found".into(),
                "warning | Slow Load | /page2 | Load time 4200ms".into(),
            ],
            page_summaries: vec![
                "/page1 | Home | 200 | 850ms | 2 issues".into(),
                "/page2 | Blog | 200 | 4200ms | 1 issues".into(),
            ],
            missing_meta_count: 3,
            slow_pages_count: 1,
            error_pages_count: 0,
            tag_values,
        }
    }

    #[test]
    fn replace_substitutes_url_score_and_counts() {
        let req = fixture_request();
        let s = replace_prompt_vars("Site {url} scores {score} on {pages_count} pages", &req);
        assert_eq!(s, "Site https://example.com scores 87 on 42 pages");
    }

    #[test]
    fn replace_substitutes_issue_breakdown() {
        let req = fixture_request();
        let s = replace_prompt_vars(
            "{total_issues} total / {critical_issues} crit / {warning_issues} warn / {suggestion_issues} sug",
            &req,
        );
        assert_eq!(s, "9 total / 2 crit / 5 warn / 2 sug");
    }

    #[test]
    fn replace_joins_top_issues_with_newlines() {
        let req = fixture_request();
        let s = replace_prompt_vars("Top:\n{top_issues}", &req);
        assert_eq!(s, "Top:\nmissing meta\nslow load");
    }

    #[test]
    fn replace_formats_avg_load_time_to_two_decimals() {
        // The format string is "{:.2}" — pinning that 1234.5 becomes
        // "1234.50", not "1234.5" or "1234".
        let req = fixture_request();
        let s = replace_prompt_vars("Load: {avg_load_time}ms", &req);
        assert_eq!(s, "Load: 1234.50ms");
    }

    #[test]
    fn replace_uses_yes_no_for_booleans() {
        let req = fixture_request();
        // ssl_certificate=true, sitemap_found=true, robots_txt_found=false
        let s = replace_prompt_vars(
            "ssl={ssl_certificate}, sitemap={sitemap_found}, robots={robots_txt_found}",
            &req,
        );
        assert_eq!(s, "ssl=Yes, sitemap=Yes, robots=No");
    }

    #[test]
    fn replace_leaves_unrecognised_placeholders_untouched() {
        let req = fixture_request();
        let s = replace_prompt_vars("{nonsense} and {url}", &req);
        assert_eq!(s, "{nonsense} and https://example.com");
    }

    #[test]
    fn replace_handles_total_words_as_integer() {
        let req = fixture_request();
        let s = replace_prompt_vars("{total_words}", &req);
        assert_eq!(s, "12000");
    }

    #[test]
    fn build_prompt_inserts_persona_and_data_section_separator() {
        let req = fixture_request();
        let prompt = build_prompt_from_blocks("PERSONA", &[], &req);
        assert!(prompt.starts_with("PERSONA"));
        assert!(prompt.contains("Analyze the following SEO audit results:"));
    }

    #[test]
    fn build_prompt_falls_back_to_default_data_block_when_blocks_empty() {
        let req = fixture_request();
        let prompt = build_prompt_from_blocks("P", &[], &req);
        // The fallback block embeds key fields directly.
        assert!(prompt.contains("Website: https://example.com"));
        assert!(prompt.contains("SEO Score: 87/100"));
        assert!(prompt.contains("Pages analyzed: 42"));
        assert!(prompt.contains("9 total"));
        assert!(prompt.contains("2 critical"));
        assert!(prompt.contains("5 warnings"));
    }

    #[test]
    fn build_prompt_renders_each_block_with_substitution() {
        let req = fixture_request();
        let blocks = vec![
            PromptBlock {
                id: "b1".into(),
                r#type: "text".into(),
                content: "Block 1: {url}".into(),
            },
            PromptBlock {
                id: "b2".into(),
                r#type: "text".into(),
                content: "Block 2: score={score}".into(),
            },
        ];
        let prompt = build_prompt_from_blocks("P", &blocks, &req);
        assert!(prompt.contains("Block 1: https://example.com"));
        assert!(prompt.contains("Block 2: score=87"));
        // Default fallback should NOT appear when blocks are present.
        assert!(!prompt.contains("Website:"));
    }

    #[test]
    fn build_prompt_substitutes_persona_variables_too() {
        let req = fixture_request();
        let prompt = build_prompt_from_blocks("Hi {url}", &[], &req);
        assert!(prompt.starts_with("Hi https://example.com"));
    }

    #[test]
    fn replace_substitutes_tag_values() {
        let req = fixture_request();
        let s = replace_prompt_vars("OG image: {tag.og_image}", &req);
        assert_eq!(s, "OG image: https://img.jpg");
    }

    #[test]
    fn replace_unknown_tag_resolves_to_empty() {
        let req = fixture_request();
        let s = replace_prompt_vars("Missing: {tag.nonexistent}", &req);
        assert_eq!(s, "Missing: ");
    }

    #[test]
    fn aggregate_tag_values_collects_distinct_per_tag() {
        use crate::contexts::analysis::{Depth, Page};
        use chrono::Utc;

        let make_page = |data: Vec<(&str, &str)>| -> Page {
            let mut extracted_data = std::collections::HashMap::new();
            for (k, v) in data {
                extracted_data.insert(k.into(), serde_json::Value::String(v.into()));
            }
            Page {
                id: "p".into(), job_id: "j".into(), url: "u".into(),
                depth: Depth::root(), status_code: None, content_type: None,
                title: None, meta_description: None, canonical_url: None,
                robots_meta: None, word_count: None, load_time_ms: None,
                response_size_bytes: None, has_viewport: false,
                has_structured_data: false, crawled_at: Utc::now(),
                extracted_data,
            }
        };

        let pages = vec![
            make_page(vec![("og_image", "a.jpg"), ("author", "Alice")]),
            make_page(vec![("og_image", "b.jpg"), ("author", "Alice")]),
            make_page(vec![("og_image", "a.jpg")]),  // dup
        ];

        let agg = aggregate_tag_values(&pages);
        // og_image: two distinct values
        let og = agg.get("og_image").unwrap();
        assert!(og.contains("a.jpg"));
        assert!(og.contains("b.jpg"));
        // author: one distinct value
        assert_eq!(agg.get("author").unwrap(), "Alice");
    }

    #[test]
    fn default_persona_is_non_empty_and_contains_directive_keywords() {
        // Smoke test for the constant — pin that it isn't accidentally
        // wiped to an empty string in a refactor.
        assert!(!DEFAULT_PERSONA.is_empty());
        assert!(DEFAULT_PERSONA.contains("HOOK"));
        assert!(DEFAULT_PERSONA.contains("STRUCTURE"));
        assert!(DEFAULT_PERSONA.contains("VOICE"));
        assert!(DEFAULT_PERSONA.contains("CTA"));
    }
}

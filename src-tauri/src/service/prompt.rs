//! Shared prompt utilities used by all AI backends (Gemini, local model, report brief).
//!
//! Single source of truth for:
//!   - The default persona / system prompt
//!   - Variable substitution into prompt templates
//!   - Assembling a final prompt from persona + prompt blocks

use crate::service::gemini::{GeminiRequest, PromptBlock};

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
    ];

    replacements
        .iter()
        .fold(text.to_string(), |acc, (pat, val)| acc.replace(pat, val))
}

fn bool_yn(v: bool) -> &'static str {
    if v { "Yes" } else { "No" }
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

    // Fallback data section when no blocks have been configured
    if parts.is_empty() {
        parts.push(format!(
            "Website: {url}\nSEO Score: {score}/100\nPages: {pages}\n\
            Issues: {total} total ({critical} critical, {warnings} warnings)",
            url     = request.url,
            score   = request.seo_score,
            pages   = request.pages_count,
            total   = request.total_issues,
            critical = request.critical_issues,
            warnings = request.warning_issues,
        ));
    }

    format!(
        "{}\n\nAnalyze the following SEO audit results:\n\n{}",
        persona_text,
        parts.join("\n\n"),
    )
}

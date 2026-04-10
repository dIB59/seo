//! The template renderer. Walks a [`ReportTemplate`] against a
//! [`RenderContext`] and produces a list of [`RenderedFragment`]s.
//!
//! The renderer is pure: no I/O, no LLM calls. `Ai` sections are
//! emitted as opaque `RenderedFragment::AiPrompt(String)` values so a
//! downstream consumer (chunk 7) can walk the fragment list and expand
//! them asynchronously. This separation keeps the whole engine
//! deterministic and trivially unit-testable.

use super::condition::Condition;
use super::model::{ReportTemplate, TemplateSection};
use crate::contexts::analysis::Job;
use crate::contexts::report::domain::{DetectedPattern, PillarScores};

/// Everything the renderer needs to resolve variables and evaluate
/// conditions. Borrowed references only — the renderer never owns its
/// inputs, which keeps it cheap to call from a hot report-build loop.
pub struct RenderContext<'a> {
    pub job: &'a Job,
    pub detected: &'a [DetectedPattern],
    pub pillars: &'a PillarScores,
    /// SEO score as shown in the UI. Passed explicitly rather than
    /// recomputing here — the brief builder already has logic for
    /// picking between lighthouse vs issue-deduction scores.
    pub seo_score: i64,

    // ── Rich context for AI prompts ──────────────────────────────────

    /// Per-issue detail lines: "severity | title | page_url | description".
    /// Pre-formatted by the caller from the `CompleteJobResult`.
    pub issue_details: Vec<String>,

    /// Per-page summary lines: "url | title | status | load_ms | issues".
    /// Pre-formatted by the caller, sorted by issue count descending.
    pub page_summaries: Vec<String>,

    /// Diagnostic counts for the data block.
    pub missing_meta_count: i64,
    pub slow_pages_count: i64,
    pub error_pages_count: i64,
    pub avg_load_time_ms: f64,
    pub total_words: i64,

    /// Aggregated tag values from custom extractors. Each key is the
    /// tag name (e.g. `"og_image"`), each value is a comma-joined list
    /// of distinct values across all pages (capped at 5). Resolves
    /// `{tag.og_image}` in template text and AI prompts.
    pub tag_values: std::collections::HashMap<String, String>,
}

/// Errors that can surface during rendering. Kept narrow — the renderer
/// never calls I/O, so there are very few real failure modes.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RenderError {
    /// Heading level outside 1..=6.
    #[error("invalid heading level {0}, must be in 1..=6")]
    InvalidHeadingLevel(u8),
}

/// A single piece of rendered output. The final report string is the
/// concatenation of all fragments after any `AiPrompt` entries have
/// been expanded by the LLM backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderedFragment {
    /// Ready-to-ship markdown text.
    Text(String),
    /// Prompt string that should be sent to the configured LLM backend.
    /// The `label` is carried so consumers can show "Rendering AI
    /// section: 'Executive summary'…" progress. The LLM response
    /// replaces this fragment in the final output.
    AiPrompt { label: String, prompt: String },
}

impl RenderedFragment {
    /// Push the fragment's "materialized" form into a string buffer.
    /// For `AiPrompt` this is a placeholder — real expansion happens in
    /// chunk 7.
    pub fn append_to(&self, out: &mut String) {
        match self {
            Self::Text(s) => out.push_str(s),
            Self::AiPrompt { label, .. } => {
                out.push_str("\n<!-- AI: ");
                out.push_str(label);
                out.push_str(" -->\n");
            }
        }
    }
}

/// Render a template against a context. Returns a flat fragment list —
/// conditionals and nested children are already inlined.
pub fn render_template(
    template: &ReportTemplate,
    ctx: &RenderContext<'_>,
) -> Result<Vec<RenderedFragment>, RenderError> {
    let mut out = Vec::new();
    for section in &template.sections {
        render_section(section, ctx, &mut out)?;
    }
    Ok(out)
}

/// Recursive section walker. The `out` buffer is passed by `&mut` so
/// nested conditionals can append fragments in place without any
/// intermediate allocations.
fn render_section(
    section: &TemplateSection,
    ctx: &RenderContext<'_>,
    out: &mut Vec<RenderedFragment>,
) -> Result<(), RenderError> {
    match section {
        TemplateSection::Heading { level, text } => {
            if !(1..=6).contains(level) {
                return Err(RenderError::InvalidHeadingLevel(*level));
            }
            let hashes = "#".repeat(*level as usize);
            let resolved = substitute(text, ctx);
            out.push(RenderedFragment::Text(format!("{hashes} {resolved}\n\n")));
        }

        TemplateSection::Text { template } => {
            let resolved = substitute(template, ctx);
            out.push(RenderedFragment::Text(format!("{resolved}\n\n")));
        }

        TemplateSection::Ai { label, prompt } => {
            out.push(RenderedFragment::AiPrompt {
                label: label.clone(),
                prompt: substitute(prompt, ctx),
            });
        }

        TemplateSection::PatternSummary {
            filter,
            per_pattern_template,
            empty_template,
        } => {
            let matched = filter.apply(ctx.detected);
            if matched.is_empty() {
                if let Some(empty) = empty_template {
                    let resolved = substitute(empty, ctx);
                    out.push(RenderedFragment::Text(format!("{resolved}\n\n")));
                }
            } else {
                let mut buf = String::new();
                for pattern in matched {
                    let line = substitute_pattern(per_pattern_template, pattern, ctx);
                    buf.push_str(&line);
                    buf.push('\n');
                }
                buf.push('\n');
                out.push(RenderedFragment::Text(buf));
            }
        }

        TemplateSection::Conditional { when, children } => {
            if evaluate_condition(when, ctx) {
                for child in children {
                    render_section(child, ctx, out)?;
                }
            }
        }

        TemplateSection::Divider => {
            out.push(RenderedFragment::Text("---\n\n".to_string()));
        }
    }
    Ok(())
}

// ── Variable substitution ────────────────────────────────────────────────────

/// Replace `{var}` placeholders in `text` against the render context.
///
/// Builds on top of the existing `service::prompt::replace_prompt_vars`
/// engine where possible, and adds the pattern-aware variables
/// (`{top_patterns}`, `{detected_patterns_count}`, `{pillar.overall}`,
/// etc.) that are only meaningful inside a report template.
fn substitute(text: &str, ctx: &RenderContext<'_>) -> String {
    let replacements = context_variables(ctx);
    let mut result = replacements
        .iter()
        .fold(text.to_string(), |acc, (pat, val)| acc.replace(pat, val));

    // Resolve {tag.X} placeholders against aggregated tag values
    result = crate::service::prompt::replace_tag_vars(&result, &ctx.tag_values);

    result
}

/// Variant of [`substitute`] that adds per-pattern variables on top of
/// the context variables. Used by `PatternSummary` sections.
fn substitute_pattern(
    text: &str,
    pattern: &DetectedPattern,
    ctx: &RenderContext<'_>,
) -> String {
    let mut replacements = context_variables(ctx);
    let pct = (pattern.prevalence * 100.0).round() as u64;
    replacements.extend([
        ("{pattern.name}".to_string(), pattern.pattern.name.clone()),
        ("{pattern.description}".to_string(), pattern.pattern.description.clone()),
        (
            "{pattern.recommendation}".to_string(),
            pattern.pattern.recommendation.clone(),
        ),
        (
            "{pattern.category}".to_string(),
            pattern.pattern.category.as_str().to_string(),
        ),
        (
            "{pattern.severity}".to_string(),
            pattern.pattern.severity.as_str().to_string(),
        ),
        (
            "{pattern.affected_pages}".to_string(),
            pattern.affected_pages.to_string(),
        ),
        (
            "{pattern.total_pages}".to_string(),
            pattern.total_pages.to_string(),
        ),
        ("{pattern.pct}".to_string(), pct.to_string()),
    ]);
    replacements
        .iter()
        .fold(text.to_string(), |acc, (pat, val)| acc.replace(pat, val))
}

/// Build the list of placeholder → value pairs for a render context.
/// Kept as a free function so both `substitute` and `substitute_pattern`
/// share exactly one source of truth for the variable table.
fn context_variables(ctx: &RenderContext<'_>) -> Vec<(String, String)> {
    let job = ctx.job;
    let top_patterns = ctx
        .detected
        .iter()
        .take(3)
        .map(|d| format!("- {}", d.pattern.name))
        .collect::<Vec<_>>()
        .join("\n");

    vec![
        ("{url}".to_string(), job.url.clone()),
        ("{score}".to_string(), ctx.seo_score.to_string()),
        ("{pages_count}".to_string(), job.summary.total_pages().to_string()),
        (
            "{total_issues}".to_string(),
            job.summary.total_issues().to_string(),
        ),
        (
            "{critical_issues}".to_string(),
            job.summary.critical_issues().to_string(),
        ),
        (
            "{warning_issues}".to_string(),
            job.summary.warning_issues().to_string(),
        ),
        (
            "{sitemap_found}".to_string(),
            bool_yn(job.sitemap_found).to_string(),
        ),
        (
            "{robots_txt_found}".to_string(),
            bool_yn(job.robots_txt_found).to_string(),
        ),
        (
            "{pillar.technical}".to_string(),
            format!("{:.0}", ctx.pillars.technical()),
        ),
        (
            "{pillar.content}".to_string(),
            format!("{:.0}", ctx.pillars.content()),
        ),
        (
            "{pillar.performance}".to_string(),
            format!("{:.0}", ctx.pillars.performance()),
        ),
        (
            "{pillar.accessibility}".to_string(),
            format!("{:.0}", ctx.pillars.accessibility()),
        ),
        (
            "{pillar.overall}".to_string(),
            format!("{:.0}", ctx.pillars.overall()),
        ),
        (
            "{detected_patterns_count}".to_string(),
            ctx.detected.len().to_string(),
        ),
        ("{top_patterns}".to_string(), top_patterns),
        // Rich context — actual data for AI prompts
        ("{issue_details}".to_string(), ctx.issue_details.join("\n")),
        ("{page_summaries}".to_string(), ctx.page_summaries.join("\n")),
        (
            "{missing_meta_count}".to_string(),
            ctx.missing_meta_count.to_string(),
        ),
        (
            "{slow_pages_count}".to_string(),
            ctx.slow_pages_count.to_string(),
        ),
        (
            "{error_pages_count}".to_string(),
            ctx.error_pages_count.to_string(),
        ),
        (
            "{avg_load_time}".to_string(),
            format!("{:.0}", ctx.avg_load_time_ms),
        ),
        ("{total_words}".to_string(), ctx.total_words.to_string()),
        // Tag summary: all extracted tags and their aggregated values
        // in one block for AI prompts to reference.
        ("{tag_summary}".to_string(), {
            if ctx.tag_values.is_empty() {
                "(no custom extractor data)".to_string()
            } else {
                let mut sorted: Vec<_> = ctx.tag_values.iter().collect();
                sorted.sort_by_key(|(k, _)| k.clone());
                sorted
                    .iter()
                    .map(|(k, v)| format!("  {k}: {v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }),
    ]
}

fn bool_yn(v: bool) -> &'static str {
    if v {
        "Yes"
    } else {
        "No"
    }
}

// ── Condition evaluation ─────────────────────────────────────────────────────

fn evaluate_condition(condition: &Condition, ctx: &RenderContext<'_>) -> bool {
    match condition {
        Condition::PatternFired { pattern_id } => ctx
            .detected
            .iter()
            .any(|d| d.pattern.id == *pattern_id),
        Condition::AnyPatternMatches { filter } => !filter.apply(ctx.detected).is_empty(),
        Condition::ScoreLt { value } => ctx.seo_score < *value,
        Condition::CriticalIssuesGt { value } => ctx.job.summary.critical_issues() > *value,
        Condition::SitemapMissing => !ctx.job.sitemap_found,
        Condition::RobotsMissing => !ctx.job.robots_txt_found,
        Condition::TagPresent { tag } => ctx
            .tag_values
            .get(tag.as_str())
            .is_some_and(|v| !v.is_empty()),
        Condition::TagMissing { tag } => !ctx
            .tag_values
            .get(tag.as_str())
            .is_some_and(|v| !v.is_empty()),
        Condition::TagContains { tag, value } => ctx
            .tag_values
            .get(tag.as_str())
            .is_some_and(|v| v.contains(value.as_str())),
        Condition::All { children } => children.iter().all(|c| evaluate_condition(c, ctx)),
        Condition::Any { children } => children.iter().any(|c| evaluate_condition(c, ctx)),
        Condition::Not { inner } => !evaluate_condition(inner, ctx),
    }
}

/// Convenience: render and flatten the fragment list into a single
/// string, substituting an HTML comment for every AI prompt (so tests
/// and previews produce deterministic output without touching a
/// backend). The AI-executing variant lives in chunk 7.
pub fn render_template_to_string(
    template: &ReportTemplate,
    ctx: &RenderContext<'_>,
) -> Result<String, RenderError> {
    let fragments = render_template(template, ctx)?;
    let mut out = String::new();
    for fragment in &fragments {
        fragment.append_to(&mut out);
    }
    Ok(out)
}

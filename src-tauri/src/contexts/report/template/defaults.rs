//! The built-in default report template. Expresses the current
//! `brief_builder` phase1/2/3 prompt structure as a `ReportTemplate`
//! so existing report output is preserved when the user hasn't
//! customized anything.
//!
//! This is also the seed data for the `report_templates` migration.

use super::condition::{Condition, PatternFilter};
use super::model::{ReportTemplate, TemplateSection};

/// The default template id. Stable across migrations and code
/// references so `set_active_template("default")` always resolves.
pub const DEFAULT_TEMPLATE_ID: &str = "default";

/// Ground rules prepended to every AI section — anti-hallucination
/// guardrails ported verbatim from `brief_builder::GROUND_RULES`.
const GROUND_RULES: &str = "\
STRICT RULES — read carefully before responding:
1. Use ONLY the numbers, percentages and facts listed in the DATA block below.
2. NEVER invent or estimate any of the following: organic impressions, \
monthly traffic, click-through rates, conversion rates, revenue at risk, \
dollar amounts, time-to-rank, ranking positions, competitor numbers, or \
any statistic that is not present in the DATA block.
3. If you do not have a number for something, describe it qualitatively \
(\"a meaningful share\", \"the majority\", \"a small slice\") — do not guess a figure.
4. Do not refer to studies, benchmarks, or external sources.
5. Stay specific to the site and the data given. No generic SEO platitudes.
6. Write in clear, direct prose. No headings, no bullet points, no markdown inside your response.";

pub fn default_template() -> ReportTemplate {
    ReportTemplate {
        id: DEFAULT_TEMPLATE_ID.into(),
        name: "Default Report".into(),
        is_builtin: true,
        selected_tags: vec![], // empty = include all tags
        sections: vec![
            // ── Diagnosis (AI-generated) ──────────────────────────────
            TemplateSection::Heading {
                level: 2,
                text: "Diagnosis".into(),
            },
            TemplateSection::Ai {
                label: "Diagnosis".into(),
                prompt: format!(
                    "{GROUND_RULES}\n\n\
                    DATA:\n\
                    Site: {{url}}\n\
                    Overall SEO score: {{score}}/100\n\
                    Pages analysed: {{pages_count}}\n\
                    Critical issues: {{critical_issues}}\n\
                    Warning issues: {{warning_issues}}\n\
                    Pages missing meta description: {{missing_meta_count}}\n\
                    Slow pages (>3s): {{slow_pages_count}}\n\
                    Error pages (4xx/5xx): {{error_pages_count}}\n\
                    Average load time: {{avg_load_time}}ms\n\
                    Total words: {{total_words}}\n\
                    Sitemap: {{sitemap_found}}\n\
                    Robots.txt: {{robots_txt_found}}\n\
                    Pillar scores — Technical {{pillar.technical}}, Content {{pillar.content}}, \
                    Performance {{pillar.performance}}, Accessibility {{pillar.accessibility}} (out of 100)\n\
                    Top detected patterns:\n{{top_patterns}}\n\n\
                    Issue details (severity | type | page | message):\n{{issue_details}}\n\n\
                    Pages by issue count (url | title | status | load_ms | issues):\n{{page_summaries}}\n\n\
                    Custom extractor data:\n{{tag_summary}}\n\n\
                    TASK — DIAGNOSIS (3 to 4 sentences, ~110 words):\n\
                    Write a grounded read on this site's SEO health. Open by naming where it stands \
                    overall, then explain — using only the facts above — what the strongest pillar \
                    is doing right and which weak signals (sitemap, top patterns, weakest pillar) \
                    are most likely to hold the site back. Close with the qualitative business \
                    consequence of leaving these unaddressed. Be specific to this site.\n\
                    RESPONSE:"
                ),
            },

            // ── Priority Actions (pattern-driven) ─────────────────────
            TemplateSection::Heading {
                level: 2,
                text: "Priority Actions".into(),
            },
            TemplateSection::PatternSummary {
                filter: PatternFilter::TopN { n: 3 },
                per_pattern_template:
                    "**{pattern.name}** — affects {pattern.pct}% of pages. {pattern.recommendation}"
                        .into(),
                empty_template: Some("No significant patterns detected.".into()),
            },

            // ── Pillar Health ─────────────────────────────────────────
            TemplateSection::Heading {
                level: 2,
                text: "Pillar Health".into(),
            },
            TemplateSection::Text {
                template: "- Technical: {pillar.technical}/100\n\
                           - Content: {pillar.content}/100\n\
                           - Performance: {pillar.performance}/100\n\
                           - Accessibility: {pillar.accessibility}/100"
                    .into(),
            },

            // ── Roadmap (AI-generated) ────────────────────────────────
            TemplateSection::Heading {
                level: 2,
                text: "Next Steps".into(),
            },
            TemplateSection::Ai {
                label: "Roadmap".into(),
                prompt: format!(
                    "{GROUND_RULES}\n\n\
                    DATA:\n\
                    Site: {{url}}\n\
                    SEO score: {{score}}/100\n\
                    Pages: {{pages_count}} | Avg load: {{avg_load_time}}ms | Total words: {{total_words}}\n\
                    Pillar scores — Technical {{pillar.technical}}/100, Content {{pillar.content}}/100, \
                    Performance {{pillar.performance}}/100, Accessibility {{pillar.accessibility}}/100\n\
                    Critical: {{critical_issues}} | Warnings: {{warning_issues}} | Missing meta: {{missing_meta_count}} | Slow: {{slow_pages_count}} | Errors: {{error_pages_count}}\n\
                    Top detected patterns to resolve:\n{{top_patterns}}\n\n\
                    Issue details:\n{{issue_details}}\n\n\
                    TASK — ROADMAP (4 to 5 sentences, ~130 words):\n\
                    Lay out a sequenced 30-day plan in plain prose. Sentence 1: name the \
                    weakest pillar and why fixing it first moves the score most. \
                    Sentence 2 and 3: spell out the order to tackle the top patterns above \
                    — which to do this week, which next. Sentence 4: a measurable success \
                    signal the team should watch for in the next audit. Sentence 5 \
                    (optional): one closing call to action. No invented numbers.\n\
                    RESPONSE:"
                ),
            },

            // ── Conditional: sitemap missing ──────────────────────────
            TemplateSection::Conditional {
                when: Condition::SitemapMissing,
                children: vec![TemplateSection::Text {
                    template: "\n**Action required:** Generate and submit a sitemap to \
                               Google Search Console immediately."
                        .into(),
                }],
            },

            // ── Conditional: robots.txt missing ───────────────────────
            TemplateSection::Conditional {
                when: Condition::RobotsMissing,
                children: vec![TemplateSection::Text {
                    template: "\n**Action required:** Create a robots.txt file to guide \
                               search engine crawlers."
                        .into(),
                }],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_template_has_expected_id_and_name() {
        let t = default_template();
        assert_eq!(t.id, DEFAULT_TEMPLATE_ID);
        assert_eq!(t.name, "Default Report");
        assert!(t.is_builtin);
    }

    #[test]
    fn default_template_has_ai_sections() {
        let t = default_template();
        let ai_count = t.sections.iter().filter(|s| matches!(s, TemplateSection::Ai { .. })).count();
        assert_eq!(ai_count, 2, "should have Diagnosis + Roadmap AI sections");
    }

    #[test]
    fn default_template_serde_round_trips() {
        let t = default_template();
        let json = serde_json::to_string(&t).unwrap();
        let parsed: ReportTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, t);
    }
}

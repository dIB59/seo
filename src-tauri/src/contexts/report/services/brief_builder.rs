use crate::contexts::analysis::Job;
use crate::contexts::report::domain::{DetectedPattern, PatternSeverity, PillarScores};

// ── Phase prompts ─────────────────────────────────────────────────────────────
//
// Each phase is intentionally small so the full prompt (system_prompt + task)
// stays well under 2 048 tokens.  `system_prompt` is the user-configured
// persona loaded from settings ("gemini_persona") — the same text that drives
// Gemini and the local model for regular analysis.

/// Anti-hallucination ground rules prepended to every phase. Local
/// models love to fabricate concrete numbers (organic impressions,
/// dollar amounts, conversion lifts) when none are provided. We forbid
/// every quantitative claim that isn't explicitly grounded in the data
/// block we hand them.
const GROUND_RULES: &str = "\
STRICT RULES — read carefully before responding:
1. Use ONLY the numbers, percentages and facts listed in the DATA block below.
2. NEVER invent or estimate any of the following: organic impressions,
   monthly traffic, click-through rates, conversion rates, revenue at risk,
   dollar amounts, time-to-rank, ranking positions, competitor numbers, or
   any statistic that is not present in the DATA block.
3. If you do not have a number for something, describe it qualitatively
   (\"a meaningful share\", \"the majority\", \"a small slice\") — do not
   guess a figure.
4. Do not refer to studies, benchmarks, or external sources.
5. Stay specific to the site and the data given. No generic SEO platitudes.
6. Write in clear, direct prose. No headings, no bullet points, no markdown
   inside your response.
";

/// Phase 1 — Diagnosis: an evidence-based read on the site's current
/// SEO health and what the issues mean for the business.
#[allow(clippy::too_many_arguments)]
pub fn phase1_diagnosis_prompt(
    system_prompt: &str,
    url: &str,
    score: i64,
    grade: &str,
    pages: i64,
    critical: i64,
    warnings: i64,
    sitemap: bool,
    robots: bool,
    pillars: &PillarScores,
    top_issue_names: &[String],
) -> String {
    let sitemap = if sitemap { "present" } else { "missing" };
    let robots  = if robots  { "present" } else { "missing" };
    let top_list = if top_issue_names.is_empty() {
        "(none flagged)".to_string()
    } else {
        top_issue_names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("  {}. {}", i + 1, n))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let (t, c, p, a) = (pillars.technical, pillars.content, pillars.performance, pillars.accessibility);
    format!(
        "{system_prompt}\n\n\
        {GROUND_RULES}\n\
        DATA:\n\
        Site: {url}\n\
        Overall SEO score: {score}/100 ({grade})\n\
        Pages analysed: {pages}\n\
        Critical issues: {critical}\n\
        Warning issues: {warnings}\n\
        Sitemap: {sitemap}\n\
        Robots.txt: {robots}\n\
        Pillar scores — Technical {t:.0}, Content {c:.0}, Performance {p:.0}, Accessibility {a:.0} (out of 100)\n\
        Top detected patterns:\n{top_list}\n\n\
        TASK — DIAGNOSIS (3 to 4 sentences, ~110 words):\n\
        Write a grounded read on this site's SEO health. Open by naming where it stands \
        overall, then explain — using only the facts above — what the strongest pillar \
        is doing right and which weak signals (sitemap, top patterns, weakest pillar) \
        are most likely to hold the site back. Close with the qualitative business \
        consequence of leaving these unaddressed. Be specific to this site.\n\
        RESPONSE:"
    )
}

/// Phase 2 — Issue narrative: called once per issue (max 3).
/// Explains what the issue actually means for visitors and search
/// engines, and what the team should do first.
pub fn phase2_issue_prompt(
    system_prompt: &str,
    name: &str,
    description: &str,
    pct: u64,
    affected_pages: usize,
    total_pages: usize,
    business_impact: &str,
    fix_effort: &str,
    recommendation: &str,
) -> String {
    format!(
        "{system_prompt}\n\n\
        {GROUND_RULES}\n\
        DATA:\n\
        Issue: {name}\n\
        What it means: {description}\n\
        Affected pages: {affected_pages} of {total_pages} ({pct}%)\n\
        Business impact rating: {business_impact}\n\
        Fix effort rating: {fix_effort}\n\
        Recommended fix: {recommendation}\n\n\
        TASK — ISSUE NARRATIVE (2 to 3 sentences, ~75 words):\n\
        Explain what this specific issue is doing to the site's search visibility \
        and user experience — using ONLY the data above. Then say what the team \
        should do first to start unblocking it. Do not invent traffic, revenue, \
        conversion or ranking numbers.\n\
        RESPONSE:"
    )
}

/// Phase 3 — Roadmap & call to action based on pillar scores.
pub fn phase3_roadmap_prompt(
    system_prompt: &str,
    pillars: &PillarScores,
    weakest: &str,
    top_issue_names: &[String],
) -> String {
    let (t, c, p, a) = (pillars.technical, pillars.content, pillars.performance, pillars.accessibility);
    let top_list = if top_issue_names.is_empty() {
        "(none flagged)".to_string()
    } else {
        top_issue_names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("  {}. {}", i + 1, n))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "{system_prompt}\n\n\
        {GROUND_RULES}\n\
        DATA:\n\
        Pillar scores — Technical {t:.0}/100, Content {c:.0}/100, \
        Performance {p:.0}/100, Accessibility {a:.0}/100\n\
        Weakest pillar: {weakest}\n\
        Top detected patterns to resolve:\n{top_list}\n\n\
        TASK — ROADMAP (4 to 5 sentences, ~130 words):\n\
        Lay out a sequenced 30-day plan in plain prose. Sentence 1: name the \
        weakest pillar and why fixing it first moves the score most. \
        Sentence 2 and 3: spell out the order to tackle the top patterns above \
        — which to do this week, which next. Sentence 4: a measurable success \
        signal the team should watch for in the next audit. Sentence 5 \
        (optional): one closing call to action. No invented numbers.\n\
        RESPONSE:"
    )
}

// ── Fallback static brief ─────────────────────────────────────────────────────
//
// Used when no local model is active.  Still directive and structured —
// not a data dump — but template-generated rather than AI-written.

pub fn build_static_brief(
    job: &Job,
    detected: &[DetectedPattern],
    pillars: &PillarScores,
) -> String {
    let mut out = String::new();

    out.push_str("## Diagnosis\n\n");
    let critical = job.summary.critical_issues;
    let warnings = job.summary.warning_issues;
    let pages    = job.summary.total_pages;
    out.push_str(&format!(
        "This audit covered {pages} page(s) and surfaced {critical} critical \
        issue(s) and {warnings} warning(s). Critical issues block search engines \
        from indexing or ranking pages correctly — every day they remain unfixed \
        is revenue left on the table.\n\n"
    ));

    if !detected.is_empty() {
        out.push_str("## Priority Actions\n\n");
        let high_priority: Vec<_> = detected
            .iter()
            .filter(|d| {
                d.pattern.severity == PatternSeverity::Critical
                    || d.pattern.severity == PatternSeverity::Warning
            })
            .take(3)
            .collect();

        for dp in &high_priority {
            let pct = (dp.prevalence * 100.0).round() as u64;
            out.push_str(&format!(
                "**{}** — affects {}% of pages. {}\n\n",
                dp.pattern.name, pct, dp.pattern.recommendation
            ));
        }
    }

    out.push_str("## Pillar Health\n\n");
    out.push_str(&format!(
        "- Technical: {:.0}/100\n- Content: {:.0}/100\n\
        - Performance: {:.0}/100\n- Accessibility: {:.0}/100\n\n",
        pillars.technical, pillars.content, pillars.performance, pillars.accessibility,
    ));

    let weakest = weakest_pillar(pillars);
    out.push_str("## Next Steps\n\n");
    out.push_str(&format!(
        "Start with the **{weakest}** pillar — it has the largest gap and \
        addressing it will produce the fastest gains. Fix critical issues first, \
        then work through warnings in order of page coverage. Reassess scores \
        after each sprint.\n"
    ));

    if !job.sitemap_found {
        out.push_str(
            "\n**Action required:** Generate and submit a sitemap to \
            Google Search Console immediately.\n",
        );
    }

    out
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn score_grade(n: i64) -> &'static str {
    if n >= 90      { "Excellent" }
    else if n >= 70 { "Good" }
    else if n >= 50 { "Needs Attention" }
    else if n >= 30 { "Poor" }
    else            { "Critical" }
}

/// Returns the name of the pillar with the lowest score.
pub fn weakest_pillar(p: &PillarScores) -> &'static str {
    let scores = [
        ("Technical",     p.technical),
        ("Content",       p.content),
        ("Performance",   p.performance),
        ("Accessibility", p.accessibility),
    ];
    scores
        .iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(name, _)| *name)
        .unwrap_or("Technical")
}

use crate::contexts::analysis::Job;
use crate::contexts::report::domain::{DetectedPattern, PatternSeverity, PillarScores};

// ── Phase prompts ─────────────────────────────────────────────────────────────
//
// Each phase is intentionally small so the full prompt (system_prompt + task)
// stays well under 2 048 tokens.  `system_prompt` is the user-configured
// persona loaded from settings ("gemini_persona") — the same text that drives
// Gemini and the local model for regular analysis.

/// Phase 1 — Diagnosis: 2–3 sentences on the site's SEO health and the
/// business risk of leaving issues unaddressed.
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
) -> String {
    let sitemap = if sitemap { "yes" } else { "no" };
    let robots  = if robots  { "yes" } else { "no" };
    format!(
        "{system_prompt}\n\n\
        TASK — DIAGNOSIS (max 3 sentences, max 80 words):\n\
        Site: {url}\n\
        SEO score: {score}/100 — {grade}\n\
        Pages analysed: {pages}\n\
        Issues found: {critical} critical, {warnings} warnings\n\
        Sitemap present: {sitemap}. Robots.txt present: {robots}.\n\n\
        Describe the site's current SEO health and the business risk of leaving \
        these issues unaddressed. Be specific — no generic filler.\n\
        RESPONSE:"
    )
}

/// Phase 2 — Issue narrative: called once per issue (max 3).
/// Asks the model what the issue costs and what the team should do first.
pub fn phase2_issue_prompt(
    system_prompt: &str,
    name: &str,
    description: &str,
    pct: u64,
    business_impact: &str,
    fix_effort: &str,
    recommendation: &str,
) -> String {
    format!(
        "{system_prompt}\n\n\
        TASK — ISSUE NARRATIVE (max 2 sentences, max 60 words):\n\
        Issue: {name}\n\
        What it means: {description}\n\
        Affects: {pct}% of pages\n\
        Business impact: {business_impact}\n\
        Fix effort: {fix_effort}\n\
        Fix: {recommendation}\n\n\
        Write what this issue is costing the site and what the team must do first.\n\
        RESPONSE:"
    )
}

/// Phase 3 — Roadmap & call to action based on pillar scores.
pub fn phase3_roadmap_prompt(
    system_prompt: &str,
    pillars: &PillarScores,
    weakest: &str,
) -> String {
    let (t, c, p, a) = (pillars.technical, pillars.content, pillars.performance, pillars.accessibility);
    format!(
        "{system_prompt}\n\n\
        TASK — ROADMAP & CALL TO ACTION (max 3 sentences, max 80 words):\n\
        Pillar scores — Technical: {t:.0}/100, Content: {c:.0}/100, \
        Performance: {p:.0}/100, Accessibility: {a:.0}/100\n\
        Weakest pillar: {weakest}\n\n\
        Give a prioritised starting point. Tell the client exactly what to work \
        on first and why it will move the needle fastest.\n\
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

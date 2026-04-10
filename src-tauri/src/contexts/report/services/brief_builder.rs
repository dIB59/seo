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

/// Format a numbered list of issue names for inclusion in a prompt's
/// DATA block. Returns `"(none flagged)"` for the empty case so the
/// prompt template never has to handle a missing list. Centralized so
/// the phase-1 and phase-3 prompts agree on the line shape.
fn format_top_issue_list(top_issue_names: &[String]) -> String {
    if top_issue_names.is_empty() {
        return "(none flagged)".to_string();
    }
    top_issue_names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("  {}. {}", i + 1, n))
        .collect::<Vec<_>>()
        .join("\n")
}

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
    let top_list = format_top_issue_list(top_issue_names);
    let (t, c, p, a) = (pillars.technical(), pillars.content(), pillars.performance(), pillars.accessibility());
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
/// Bundle of every value [`phase2_issue_prompt`] needs. Replaces the
/// previous 9-arg positional signature so adding a new field is
/// non-breaking and call sites are self-documenting.
pub struct Phase2IssueArgs<'a> {
    pub system_prompt: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub pct: u64,
    pub affected_pages: usize,
    pub total_pages: usize,
    pub business_impact: &'a str,
    pub fix_effort: &'a str,
    pub recommendation: &'a str,
}

/// Explains what the issue actually means for visitors and search
/// engines, and what the team should do first.
pub fn phase2_issue_prompt(args: Phase2IssueArgs<'_>) -> String {
    let Phase2IssueArgs {
        system_prompt,
        name,
        description,
        pct,
        affected_pages,
        total_pages,
        business_impact,
        fix_effort,
        recommendation,
    } = args;
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
    let (t, c, p, a) = (pillars.technical(), pillars.content(), pillars.performance(), pillars.accessibility());
    let top_list = format_top_issue_list(top_issue_names);
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
    let critical = job.summary.critical_issues();
    let warnings = job.summary.warning_issues();
    let pages    = job.summary.total_pages();
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
        pillars.technical(), pillars.content(), pillars.performance(), pillars.accessibility(),
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
        ("Technical",     p.technical()),
        ("Content",       p.content()),
        ("Performance",   p.performance()),
        ("Accessibility", p.accessibility()),
    ];
    scores
        .iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(name, _)| *name)
        .unwrap_or("Technical")
}

#[cfg(test)]
mod tests {
    //! Characterization tests for the report brief builder. Pinning the
    //! grade thresholds, weakest-pillar selection, and the static
    //! fallback brief structure that ships when no local model is
    //! configured. Also smoke-tests the LLM prompt builders to catch
    //! accidental template breakage.

    use super::*;
    use crate::contexts::analysis::{Job, JobId, JobSettings, JobStatus, JobSummary};
    use crate::contexts::report::domain::{
        BusinessImpact, DetectedPattern, FixEffort, PatternCategory, PatternSeverity, ReportPattern,
    };
    use crate::contexts::extension::Operator;
    use chrono::Utc;

    fn make_pillars(t: f64, c: f64, p: f64, a: f64) -> PillarScores {
        // Use from_pillars so the test fixture exercises the same
        // overall-derivation path as production code.
        PillarScores::from_pillars(t, c, p, a)
    }

    fn make_pattern(severity: PatternSeverity, name: &str, recommendation: &str) -> ReportPattern {
        ReportPattern {
            id: "p".into(),
            name: name.into(),
            description: "d".into(),
            category: PatternCategory::Content,
            severity,
            field: "title".into(),
            operator: Operator::Missing,
            threshold: None,
            min_prevalence: 0.0,
            business_impact: BusinessImpact::Medium,
            fix_effort: FixEffort::Medium,
            recommendation: recommendation.into(),
            is_builtin: false,
            enabled: true,
        }
    }

    fn make_detected(severity: PatternSeverity, name: &str, prevalence: f64) -> DetectedPattern {
        DetectedPattern {
            pattern: make_pattern(severity, name, "fix this"),
            prevalence,
            affected_pages: 1,
            total_pages: 1,
            priority_score: 0.0,
            sample_urls: vec![],
        }
    }

    fn make_job(critical: i64, warnings: i64, pages: i64, sitemap_found: bool) -> Job {
        Job {
            id: JobId::from("j"),
            url: "https://example.com".into(),
            status: JobStatus::Completed,
            settings: JobSettings::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: Some(Utc::now()),
            summary: JobSummary::new(pages, pages, critical + warnings, critical, warnings, 0),
            progress: 100.0,
            error_message: None,
            sitemap_found,
            robots_txt_found: true,
        }
    }

    // ── score_grade ──────────────────────────────────────────────────────

    #[test]
    fn score_grade_thresholds() {
        // Pinning the exact bands.
        assert_eq!(score_grade(100), "Excellent");
        assert_eq!(score_grade(90), "Excellent");
        assert_eq!(score_grade(89), "Good");
        assert_eq!(score_grade(70), "Good");
        assert_eq!(score_grade(69), "Needs Attention");
        assert_eq!(score_grade(50), "Needs Attention");
        assert_eq!(score_grade(49), "Poor");
        assert_eq!(score_grade(30), "Poor");
        assert_eq!(score_grade(29), "Critical");
        assert_eq!(score_grade(0), "Critical");
    }

    #[test]
    fn score_grade_handles_negative() {
        // Out of normal range — pinning that it doesn't panic.
        assert_eq!(score_grade(-50), "Critical");
    }

    // ── weakest_pillar ───────────────────────────────────────────────────

    #[test]
    fn weakest_pillar_picks_lowest_score() {
        let p = make_pillars(80.0, 90.0, 60.0, 75.0);
        assert_eq!(weakest_pillar(&p), "Performance");
    }

    #[test]
    fn weakest_pillar_picks_first_on_tie() {
        // All equal — order matters: Technical comes first in the
        // declared array, so it wins ties.
        let p = make_pillars(50.0, 50.0, 50.0, 50.0);
        assert_eq!(weakest_pillar(&p), "Technical");
    }

    #[test]
    fn weakest_pillar_handles_perfect_scores() {
        let p = make_pillars(100.0, 100.0, 100.0, 100.0);
        // Tie at the top — first in the declared order wins.
        assert_eq!(weakest_pillar(&p), "Technical");
    }

    #[test]
    fn weakest_pillar_handles_accessibility_lowest() {
        let p = make_pillars(80.0, 80.0, 80.0, 30.0);
        assert_eq!(weakest_pillar(&p), "Accessibility");
    }

    // ── build_static_brief ───────────────────────────────────────────────

    #[test]
    fn static_brief_includes_diagnosis_pillar_health_and_next_steps() {
        let job = make_job(2, 5, 50, true);
        let pillars = make_pillars(80.0, 70.0, 60.0, 90.0);
        let brief = build_static_brief(&job, &[], &pillars);
        assert!(brief.contains("## Diagnosis"));
        assert!(brief.contains("## Pillar Health"));
        assert!(brief.contains("## Next Steps"));
        // Numbers come from the job summary.
        assert!(brief.contains("50 page"));
        assert!(brief.contains("2 critical"));
        assert!(brief.contains("5 warning"));
    }

    #[test]
    fn static_brief_lists_pillar_scores_with_zero_decimals() {
        let job = make_job(0, 0, 10, true);
        let pillars = make_pillars(85.5, 70.0, 60.0, 90.0);
        let brief = build_static_brief(&job, &[], &pillars);
        // Format: "- Technical: 86/100" — rounds to integer via {:.0}
        assert!(brief.contains("Technical: 86/100"));
        assert!(brief.contains("Content: 70/100"));
        assert!(brief.contains("Performance: 60/100"));
        assert!(brief.contains("Accessibility: 90/100"));
    }

    #[test]
    fn static_brief_includes_priority_actions_for_critical_and_warning_only() {
        let job = make_job(0, 0, 10, true);
        let pillars = make_pillars(80.0, 80.0, 80.0, 80.0);
        let detected = vec![
            make_detected(PatternSeverity::Critical, "Crit One", 0.8),
            make_detected(PatternSeverity::Warning, "Warn Two", 0.6),
            make_detected(PatternSeverity::Suggestion, "Suggest Three", 0.5),
        ];
        let brief = build_static_brief(&job, &detected, &pillars);
        assert!(brief.contains("## Priority Actions"));
        assert!(brief.contains("Crit One"));
        assert!(brief.contains("80% of pages"));
        assert!(brief.contains("Warn Two"));
        assert!(brief.contains("60% of pages"));
        // Suggestions are filtered out.
        assert!(!brief.contains("Suggest Three"));
    }

    #[test]
    fn static_brief_caps_priority_actions_at_three() {
        let job = make_job(0, 0, 10, true);
        let pillars = make_pillars(80.0, 80.0, 80.0, 80.0);
        let detected = vec![
            make_detected(PatternSeverity::Critical, "C1", 0.9),
            make_detected(PatternSeverity::Critical, "C2", 0.8),
            make_detected(PatternSeverity::Critical, "C3", 0.7),
            make_detected(PatternSeverity::Critical, "C4", 0.6),
            make_detected(PatternSeverity::Critical, "C5", 0.5),
        ];
        let brief = build_static_brief(&job, &detected, &pillars);
        assert!(brief.contains("C1"));
        assert!(brief.contains("C2"));
        assert!(brief.contains("C3"));
        assert!(!brief.contains("C4"));
        assert!(!brief.contains("C5"));
    }

    #[test]
    fn static_brief_omits_priority_actions_section_when_no_detected() {
        let job = make_job(0, 0, 10, true);
        let pillars = make_pillars(80.0, 80.0, 80.0, 80.0);
        let brief = build_static_brief(&job, &[], &pillars);
        assert!(!brief.contains("## Priority Actions"));
    }

    #[test]
    fn static_brief_appends_sitemap_action_when_missing() {
        let job = make_job(0, 0, 10, false); // sitemap_found=false
        let pillars = make_pillars(80.0, 80.0, 80.0, 80.0);
        let brief = build_static_brief(&job, &[], &pillars);
        assert!(brief.contains("Action required"));
        assert!(brief.contains("sitemap"));
    }

    #[test]
    fn static_brief_does_not_append_sitemap_action_when_present() {
        let job = make_job(0, 0, 10, true); // sitemap_found=true
        let pillars = make_pillars(80.0, 80.0, 80.0, 80.0);
        let brief = build_static_brief(&job, &[], &pillars);
        assert!(!brief.contains("Action required"));
    }

    #[test]
    fn static_brief_next_steps_names_weakest_pillar() {
        let job = make_job(0, 0, 10, true);
        let pillars = make_pillars(90.0, 90.0, 30.0, 90.0); // Performance weakest
        let brief = build_static_brief(&job, &[], &pillars);
        assert!(brief.contains("**Performance**"));
    }

    // ── Phase prompt smoke tests ─────────────────────────────────────────

    #[test]
    fn phase1_diagnosis_prompt_embeds_data_block() {
        let pillars = make_pillars(80.0, 70.0, 60.0, 90.0);
        let prompt = phase1_diagnosis_prompt(
            "PERSONA",
            "https://example.com",
            65,
            "Needs Attention",
            42,
            3,
            7,
            true,
            false,
            &pillars,
            &["missing meta".to_string(), "slow load".to_string()],
        );
        assert!(prompt.starts_with("PERSONA"));
        assert!(prompt.contains("Site: https://example.com"));
        assert!(prompt.contains("Overall SEO score: 65/100 (Needs Attention)"));
        assert!(prompt.contains("Pages analysed: 42"));
        assert!(prompt.contains("Critical issues: 3"));
        assert!(prompt.contains("Warning issues: 7"));
        assert!(prompt.contains("Sitemap: present"));
        assert!(prompt.contains("Robots.txt: missing"));
        assert!(prompt.contains("Technical 80"));
        assert!(prompt.contains("missing meta"));
        assert!(prompt.contains("slow load"));
    }

    #[test]
    fn phase1_diagnosis_prompt_renders_none_flagged_when_top_issues_empty() {
        let pillars = make_pillars(50.0, 50.0, 50.0, 50.0);
        let prompt = phase1_diagnosis_prompt(
            "P",
            "https://x.test",
            50,
            "Needs Attention",
            10,
            0,
            0,
            true,
            true,
            &pillars,
            &[],
        );
        assert!(prompt.contains("(none flagged)"));
    }

    #[test]
    fn phase2_issue_prompt_embeds_all_args() {
        let prompt = phase2_issue_prompt(Phase2IssueArgs {
            system_prompt: "P",
            name: "Missing Title",
            description: "no <title> tag",
            pct: 67,
            affected_pages: 4,
            total_pages: 6,
            business_impact: "High",
            fix_effort: "Low",
            recommendation: "add <title> tags",
        });
        assert!(prompt.contains("Issue: Missing Title"));
        assert!(prompt.contains("What it means: no <title> tag"));
        assert!(prompt.contains("Affected pages: 4 of 6 (67%)"));
        assert!(prompt.contains("Business impact rating: High"));
        assert!(prompt.contains("Fix effort rating: Low"));
        assert!(prompt.contains("Recommended fix: add <title> tags"));
    }

    #[test]
    fn ground_rules_constant_is_present_in_phase_prompts() {
        // Pin that the anti-hallucination rules ship inside the prompt.
        let pillars = make_pillars(80.0, 80.0, 80.0, 80.0);
        let p1 = phase1_diagnosis_prompt(
            "P", "https://x.test", 80, "Good", 10, 0, 0, true, true, &pillars, &[],
        );
        assert!(p1.contains("STRICT RULES"));
        assert!(p1.contains("NEVER invent or estimate"));

        let p2 = phase2_issue_prompt(Phase2IssueArgs {
            system_prompt: "P",
            name: "X",
            description: "Y",
            pct: 50,
            affected_pages: 1,
            total_pages: 2,
            business_impact: "Low",
            fix_effort: "Low",
            recommendation: "Z",
        });
        assert!(p2.contains("STRICT RULES"));
    }
}

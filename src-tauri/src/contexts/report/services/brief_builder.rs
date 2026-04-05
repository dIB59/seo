use crate::contexts::analysis::Job;
use crate::contexts::report::domain::{DetectedPattern, PatternSeverity, PillarScores};

/// Build a structured text summary of the analysis findings.
///
/// This brief is used:
/// 1. As context passed to an AI model to generate a narrative.
/// 2. Directly embedded in the PDF if AI is not available.
pub fn build_ai_brief(
    job: &Job,
    detected: &[DetectedPattern],
    pillars: &PillarScores,
) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "# SEO Analysis Report — {}\n\n",
        job.url
    ));

    // Executive summary
    let total = job.summary.total_pages;
    let issues = job.summary.total_issues;
    let seo_score = job.calculate_seo_score();
    out.push_str("## Executive Summary\n");
    out.push_str(&format!(
        "Analysed {} page(s). Overall SEO score: {}/100. {} issue(s) detected across all pages.\n\n",
        total, seo_score, issues
    ));

    // Pillar scores
    out.push_str("## Pillar Health Scores\n");
    out.push_str(&format!("- Technical: {:.0}/100\n", pillars.technical));
    out.push_str(&format!("- Content: {:.0}/100\n", pillars.content));
    out.push_str(&format!("- Performance: {:.0}/100\n", pillars.performance));
    out.push_str(&format!("- Accessibility: {:.0}/100\n\n", pillars.accessibility));

    // Detected patterns
    if detected.is_empty() {
        out.push_str("## Findings\nNo significant SEO patterns detected. The site appears to be in good health.\n\n");
    } else {
        let critical: Vec<_> = detected.iter().filter(|d| d.pattern.severity == PatternSeverity::Critical).collect();
        let warnings: Vec<_> = detected.iter().filter(|d| d.pattern.severity == PatternSeverity::Warning).collect();
        let suggestions: Vec<_> = detected.iter().filter(|d| d.pattern.severity == PatternSeverity::Suggestion).collect();

        out.push_str("## Detected Issues\n\n");

        if !critical.is_empty() {
            out.push_str("### Critical Issues\n");
            for d in &critical {
                write_pattern_entry(&mut out, d);
            }
        }

        if !warnings.is_empty() {
            out.push_str("### Warnings\n");
            for d in &warnings {
                write_pattern_entry(&mut out, d);
            }
        }

        if !suggestions.is_empty() {
            out.push_str("### Suggestions\n");
            for d in &suggestions {
                write_pattern_entry(&mut out, d);
            }
        }
    }

    // Top recommendations
    if !detected.is_empty() {
        out.push_str("## Top Recommendations\n");
        for (i, d) in detected.iter().take(5).enumerate() {
            out.push_str(&format!(
                "{}. **{}** — {}\n",
                i + 1,
                d.pattern.name,
                d.pattern.recommendation
            ));
        }
        out.push('\n');
    }

    // Site metadata
    out.push_str("## Site Information\n");
    out.push_str(&format!("- Sitemap found: {}\n", if job.sitemap_found { "Yes" } else { "No" }));
    out.push_str(&format!("- Robots.txt found: {}\n", if job.robots_txt_found { "Yes" } else { "No" }));

    out
}

fn write_pattern_entry(out: &mut String, d: &DetectedPattern) {
    let pct = (d.prevalence * 100.0).round() as u64;
    out.push_str(&format!(
        "**{}** — affects {}% of pages ({}/{} pages)\n",
        d.pattern.name, pct, d.affected_pages, d.total_pages
    ));
    out.push_str(&format!("  _{}_\n", d.pattern.description));
    out.push_str(&format!("  Fix: {}\n", d.pattern.recommendation));
    if !d.sample_urls.is_empty() {
        out.push_str("  Affected pages (sample):\n");
        for url in &d.sample_urls {
            out.push_str(&format!("  - {}\n", url));
        }
    }
    out.push('\n');
}

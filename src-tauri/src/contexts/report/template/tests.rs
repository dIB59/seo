//! Template engine characterization tests. Every section type, every
//! condition variant, every filter, and the full render pipeline are
//! covered here so a future refactor can't silently break the report
//! output.

use super::*;
use crate::contexts::analysis::{Job, JobId, JobSettings, JobSummary};
use crate::contexts::report::domain::{
    DetectedPattern, PatternCategory, PatternSeverity, PillarScores, ReportPattern,
    BusinessImpact, FixEffort,
};
use crate::contexts::extension::Operator;
use chrono::Utc;

// ── Fixtures ─────────────────────────────────────────────────────────────────

fn test_job() -> Job {
    Job {
        id: JobId::from("job-1"),
        url: "https://example.com".into(),
        status: crate::contexts::analysis::JobStatus::Completed,
        settings: JobSettings::default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        completed_at: Some(Utc::now()),
        summary: JobSummary::new(10, 10, 5, 2, 2, 1),
        progress: 100.0,
        error_message: None,
        sitemap_found: true,
        robots_txt_found: false,
    }
}

fn test_pillars() -> PillarScores {
    PillarScores::from_pillars(80.0, 65.0, 90.0, 75.0)
}

fn test_pattern(name: &str, severity: PatternSeverity) -> ReportPattern {
    ReportPattern {
        id: format!("p-{name}"),
        name: name.into(),
        description: format!("{name} description"),
        category: PatternCategory::Content,
        severity,
        field: "title".into(),
        operator: Operator::Missing,
        threshold: None,
        min_prevalence: 0.0,
        business_impact: BusinessImpact::High,
        fix_effort: FixEffort::Low,
        recommendation: format!("Fix {name}"),
        is_builtin: true,
        enabled: true,
    }
}

fn test_detected(name: &str, severity: PatternSeverity, prevalence: f64) -> DetectedPattern {
    let pattern = test_pattern(name, severity);
    let priority_score = DetectedPattern::compute_priority(&pattern, prevalence);
    DetectedPattern {
        pattern,
        prevalence,
        affected_pages: (prevalence * 10.0) as usize,
        total_pages: 10,
        priority_score,
        sample_urls: vec!["https://example.com/page1".into()],
    }
}

fn test_ctx<'a>(job: &'a Job, detected: &'a [DetectedPattern], pillars: &'a PillarScores) -> RenderContext<'a> {
    RenderContext {
        job,
        detected,
        pillars,
        seo_score: 78,
        issue_details: vec![
            "critical | Missing Title | /page1 | No title tag".into(),
            "warning | Slow Load | /page2 | 4200ms".into(),
        ],
        page_summaries: vec![
            "/page1 | Home | 200 | 850ms | 2 issues".into(),
            "/page2 | Blog | 200 | 4200ms | 1 issues".into(),
        ],
        missing_meta_count: 3,
        slow_pages_count: 1,
        error_pages_count: 0,
        avg_load_time_ms: 1500.0,
        total_words: 8500,
        tag_values: std::collections::HashMap::new(),
    }
}

// ── Section rendering ────────────────────────────────────────────────────────

#[test]
fn heading_renders_with_correct_level() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![
            TemplateSection::Heading { level: 1, text: "Title".into() },
            TemplateSection::Heading { level: 3, text: "Sub".into() },
        ],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("# Title"));
    assert!(result.contains("### Sub"));
}

#[test]
fn heading_rejects_invalid_level() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Heading { level: 7, text: "Bad".into() }],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let err = render_template(&template, &ctx).unwrap_err();
    assert_eq!(err, RenderError::InvalidHeadingLevel(7));
}

#[test]
fn text_section_substitutes_variables() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Text {
            template: "Site {url} scored {score}/100 with {critical_issues} critical issues.".into(),
        }],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("Site https://example.com scored 78/100 with 2 critical issues."));
}

#[test]
fn ai_section_emits_prompt_fragment() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Ai {
            label: "Diagnosis".into(),
            prompt: "Analyze {url}".into(),
        }],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let fragments = render_template(&template, &ctx).unwrap();
    assert_eq!(fragments.len(), 1);
    match &fragments[0] {
        RenderedFragment::AiPrompt { label, prompt } => {
            assert_eq!(label, "Diagnosis");
            assert_eq!(prompt, "Analyze https://example.com");
        }
        other => panic!("expected AiPrompt, got {other:?}"),
    }
}

#[test]
fn divider_renders_as_hr() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Divider],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("---"));
}

// ── Pattern summary ──────────────────────────────────────────────────────────

#[test]
fn pattern_summary_renders_matched_patterns() {
    let detected = vec![
        test_detected("Missing Title", PatternSeverity::Critical, 0.3),
        test_detected("Thin Content", PatternSeverity::Warning, 0.5),
    ];
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::PatternSummary {
            filter: PatternFilter::All,
            per_pattern_template: "**{pattern.name}** — {pattern.pct}% of pages".into(),
            empty_template: None,
        }],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &detected, &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("**Missing Title** — 30% of pages"));
    assert!(result.contains("**Thin Content** — 50% of pages"));
}

#[test]
fn pattern_summary_renders_empty_template_when_no_matches() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::PatternSummary {
            filter: PatternFilter::BySeverity { severity: PatternSeverity::Critical },
            per_pattern_template: "{pattern.name}".into(),
            empty_template: Some("No critical patterns detected.".into()),
        }],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("No critical patterns detected."));
}

#[test]
fn pattern_summary_top_n_limits_output() {
    let detected = vec![
        test_detected("A", PatternSeverity::Critical, 0.9),
        test_detected("B", PatternSeverity::Warning, 0.5),
        test_detected("C", PatternSeverity::Suggestion, 0.3),
    ];
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::PatternSummary {
            filter: PatternFilter::TopN { n: 2 },
            per_pattern_template: "{pattern.name}".into(),
            empty_template: None,
        }],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &detected, &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("A"));
    assert!(result.contains("B"));
    assert!(!result.contains("\nC\n"));
}

// ── Conditionals ─────────────────────────────────────────────────────────────

#[test]
fn conditional_renders_children_when_true() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Conditional {
            when: Condition::RobotsMissing,
            children: vec![TemplateSection::Text {
                template: "Robots.txt is missing!".into(),
            }],
        }],
    };
    let job = test_job(); // robots_txt_found = false
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("Robots.txt is missing!"));
}

#[test]
fn conditional_skips_children_when_false() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Conditional {
            when: Condition::SitemapMissing,
            children: vec![TemplateSection::Text {
                template: "Sitemap missing!".into(),
            }],
        }],
    };
    let job = test_job(); // sitemap_found = true
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(!result.contains("Sitemap missing!"));
}

#[test]
fn conditional_score_lt() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Conditional {
            when: Condition::ScoreLt { value: 80 },
            children: vec![TemplateSection::Text {
                template: "Score below 80".into(),
            }],
        }],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars); // seo_score = 78
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("Score below 80"));
}

#[test]
fn conditional_critical_gt() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Conditional {
            when: Condition::CriticalIssuesGt { value: 0 },
            children: vec![TemplateSection::Text {
                template: "Has critical issues".into(),
            }],
        }],
    };
    let job = test_job(); // critical_issues = 2
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("Has critical issues"));
}

#[test]
fn conditional_not_inverts() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Conditional {
            when: Condition::Not {
                inner: Box::new(Condition::SitemapMissing),
            },
            children: vec![TemplateSection::Text {
                template: "Sitemap present".into(),
            }],
        }],
    };
    let job = test_job(); // sitemap_found = true → SitemapMissing = false → Not = true
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("Sitemap present"));
}

#[test]
fn conditional_pattern_fired() {
    let detected = vec![test_detected("Missing Title", PatternSeverity::Critical, 0.5)];
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Conditional {
            when: Condition::PatternFired {
                pattern_id: "p-Missing Title".into(),
            },
            children: vec![TemplateSection::Text {
                template: "Title pattern fired!".into(),
            }],
        }],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &detected, &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("Title pattern fired!"));
}

// ── Pillar variables ─────────────────────────────────────────────────────────

#[test]
fn pillar_variables_resolve() {
    let template = ReportTemplate {
        id: "t".into(),
        name: "t".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![TemplateSection::Text {
            template: "Tech={pillar.technical} Content={pillar.content} Overall={pillar.overall}".into(),
        }],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &[], &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();
    assert!(result.contains("Tech=80 Content=65 Overall=78"));
}

// ── Full template render ─────────────────────────────────────────────────────

#[test]
fn full_template_produces_coherent_output() {
    let detected = vec![
        test_detected("Missing Title", PatternSeverity::Critical, 0.4),
    ];
    let template = ReportTemplate {
        id: "t".into(),
        name: "Test Template".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![
            TemplateSection::Heading { level: 1, text: "SEO Report for {url}".into() },
            TemplateSection::Text {
                template: "Score: {score}/100. {critical_issues} critical issues found.".into(),
            },
            TemplateSection::Divider,
            TemplateSection::Heading { level: 2, text: "Detected Patterns".into() },
            TemplateSection::PatternSummary {
                filter: PatternFilter::All,
                per_pattern_template: "- **{pattern.name}**: {pattern.recommendation}".into(),
                empty_template: Some("No patterns detected.".into()),
            },
            TemplateSection::Conditional {
                when: Condition::RobotsMissing,
                children: vec![TemplateSection::Text {
                    template: "**Warning:** robots.txt is missing.".into(),
                }],
            },
        ],
    };
    let job = test_job();
    let pillars = test_pillars();
    let ctx = test_ctx(&job, &detected, &pillars);
    let result = render_template_to_string(&template, &ctx).unwrap();

    assert!(result.contains("# SEO Report for https://example.com"));
    assert!(result.contains("Score: 78/100. 2 critical issues found."));
    assert!(result.contains("---"));
    assert!(result.contains("## Detected Patterns"));
    assert!(result.contains("**Missing Title**: Fix Missing Title"));
    assert!(result.contains("**Warning:** robots.txt is missing."));
}

// ── Serde round-trip ─────────────────────────────────────────────────────────

#[test]
fn template_serializes_and_deserializes() {
    let template = ReportTemplate {
        id: "t1".into(),
        name: "My Template".into(),
        is_builtin: false,
        selected_tags: vec![],
        sections: vec![
            TemplateSection::Heading { level: 2, text: "Test".into() },
            TemplateSection::Text { template: "{url}".into() },
            TemplateSection::Ai { label: "Diag".into(), prompt: "Analyze {url}".into() },
            TemplateSection::PatternSummary {
                filter: PatternFilter::TopN { n: 3 },
                per_pattern_template: "{pattern.name}".into(),
                empty_template: None,
            },
            TemplateSection::Conditional {
                when: Condition::SitemapMissing,
                children: vec![TemplateSection::Text { template: "no sitemap".into() }],
            },
            TemplateSection::Divider,
        ],
    };
    let json = serde_json::to_string(&template).unwrap();
    let parsed: ReportTemplate = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, template);
}

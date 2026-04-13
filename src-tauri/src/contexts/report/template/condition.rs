//! Conditions and pattern filters used inside a template.
//!
//! Kept deliberately small — no full expression DSL. Each variant is a
//! discrete comparison that covers the cases the brief_builder already
//! uses today, with one escape hatch (`PatternFired`) for pattern-aware
//! rendering. A future chunk can extend this enum additively.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::contexts::report::domain::{PatternCategory, PatternSeverity};

/// Runtime condition evaluated against the render context.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Condition {
    /// True if at least one detected pattern has the given id.
    PatternFired { pattern_id: String },

    /// True if at least one detected pattern matches the filter.
    AnyPatternMatches { filter: PatternFilter },

    /// True if the job's SEO score is strictly less than `value`.
    ScoreLt { value: i64 },

    /// True if the job's critical issue count is strictly greater than `value`.
    CriticalIssuesGt { value: i64 },

    /// True if the sitemap was NOT found during discovery. Shorthand
    /// for the "action required: submit a sitemap" brief section.
    SitemapMissing,

    /// True if the robots.txt was NOT found during discovery.
    RobotsMissing,

    /// True if the named tag has a non-empty aggregated value across
    /// the crawled pages. The `tag` field is the bare extractor name
    /// (e.g. `"og_image"`, not `"tag:og_image"`).
    TagPresent { tag: String },

    /// True if the named tag is absent or has no value across all pages.
    TagMissing { tag: String },

    /// True if the named tag's aggregated value contains the given
    /// substring. Useful for "if any page has og_image containing
    /// 'placeholder'…" style conditions.
    TagContains { tag: String, value: String },

    /// Logical AND of all children (empty list → true).
    All { children: Vec<Condition> },

    /// Logical OR of all children (empty list → false).
    Any { children: Vec<Condition> },

    /// Logical NOT.
    Not { inner: Box<Condition> },
}

/// Filter applied to the detected-pattern list before iterating. Used by
/// [`TemplateSection::PatternSummary`](super::TemplateSection::PatternSummary)
/// and by [`Condition::AnyPatternMatches`].
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PatternFilter {
    /// Every detected pattern.
    All,
    /// Only patterns with exactly this severity.
    BySeverity { severity: PatternSeverity },
    /// Only patterns in this category.
    ByCategory { category: PatternCategory },
    /// Top N patterns by `priority_score` (already the sort order of
    /// `detected_patterns`, so this is just a take).
    TopN { n: usize },
    /// Patterns whose prevalence is at least `min_prevalence` (0.0..=1.0).
    MinPrevalence { min_prevalence: f64 },
}

impl PatternFilter {
    /// Apply this filter to a detected-pattern list. Does not clone the
    /// patterns — returns borrowed references so the caller can decide
    /// whether to clone per fragment or stringify in place.
    pub fn apply<'a>(
        &self,
        detected: &'a [crate::contexts::report::domain::DetectedPattern],
    ) -> Vec<&'a crate::contexts::report::domain::DetectedPattern> {
        match self {
            Self::All => detected.iter().collect(),
            Self::BySeverity { severity } => detected
                .iter()
                .filter(|d| d.pattern.severity == *severity)
                .collect(),
            Self::ByCategory { category } => detected
                .iter()
                .filter(|d| d.pattern.category == *category)
                .collect(),
            Self::TopN { n } => detected.iter().take(*n).collect(),
            Self::MinPrevalence { min_prevalence } => detected
                .iter()
                .filter(|d| d.prevalence >= *min_prevalence)
                .collect(),
        }
    }
}

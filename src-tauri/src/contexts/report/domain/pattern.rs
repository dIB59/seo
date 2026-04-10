use serde::{Deserialize, Serialize};

use crate::contexts::extension::Operator;

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum PatternCategory {
    Technical,
    Content,
    Performance,
    Accessibility,
}

impl PatternCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Technical => "technical",
            Self::Content => "content",
            Self::Performance => "performance",
            Self::Accessibility => "accessibility",
        }
    }
}

crate::impl_display_via_as_str!(PatternCategory);

/// Returned by [`PatternCategory::from_str`] when the input doesn't map
/// to a known category. Carries the offending string.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("invalid pattern category: '{0}'")]
pub struct ParsePatternCategoryError(pub String);

impl std::str::FromStr for PatternCategory {
    type Err = ParsePatternCategoryError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "technical" => Ok(Self::Technical),
            "content" => Ok(Self::Content),
            "performance" => Ok(Self::Performance),
            "accessibility" => Ok(Self::Accessibility),
            other => Err(ParsePatternCategoryError(other.to_string())),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum PatternSeverity {
    Critical,
    Warning,
    Suggestion,
}

impl PatternSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Warning => "warning",
            Self::Suggestion => "suggestion",
        }
    }

    /// Weight used in priority score and pillar deduction calculations.
    pub fn weight(&self) -> f64 {
        match self {
            Self::Critical => 3.0,
            Self::Warning => 2.0,
            Self::Suggestion => 1.0,
        }
    }
}

crate::impl_display_via_as_str!(PatternSeverity);

/// Returned by [`PatternSeverity::from_str`] when the input doesn't map
/// to a known severity. Carries the offending string.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("invalid pattern severity: '{0}'")]
pub struct ParsePatternSeverityError(pub String);

impl std::str::FromStr for PatternSeverity {
    type Err = ParsePatternSeverityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Self::Critical),
            "warning" => Ok(Self::Warning),
            "suggestion" => Ok(Self::Suggestion),
            other => Err(ParsePatternSeverityError(other.to_string())),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum BusinessImpact {
    High,
    Medium,
    Low,
}

impl BusinessImpact {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Self::High => 3.0,
            Self::Medium => 2.0,
            Self::Low => 1.0,
        }
    }
}

crate::impl_display_via_as_str!(BusinessImpact);

/// Returned by [`BusinessImpact::from_str`] when the input doesn't map
/// to a known level. Carries the offending string.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("invalid business impact: '{0}'")]
pub struct ParseBusinessImpactError(pub String);

impl std::str::FromStr for BusinessImpact {
    type Err = ParseBusinessImpactError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            other => Err(ParseBusinessImpactError(other.to_string())),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum FixEffort {
    Low,
    Medium,
    High,
}

impl FixEffort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    /// Multiplier: easier fixes (Low effort) get boosted priority so they surface first.
    pub fn multiplier(&self) -> f64 {
        match self {
            Self::Low => 1.5,
            Self::Medium => 1.0,
            Self::High => 0.5,
        }
    }
}

crate::impl_display_via_as_str!(FixEffort);

/// Returned by [`FixEffort::from_str`] when the input doesn't map to a
/// known level. Carries the offending string.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("invalid fix effort: '{0}'")]
pub struct ParseFixEffortError(pub String);

impl std::str::FromStr for FixEffort {
    type Err = ParseFixEffortError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            other => Err(ParseFixEffortError(other.to_string())),
        }
    }
}

// ── Core domain types ─────────────────────────────────────────────────────────

/// A rule that, when matched against site-wide page data, indicates an SEO problem.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ReportPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: PatternCategory,
    pub severity: PatternSeverity,
    /// Page field to evaluate. Built-in fields: `meta_description`, `title`, `word_count`,
    /// `load_time_ms`, `status_code`, `has_viewport`, `has_structured_data`, `canonical_url`,
    /// `h1_count`. Custom extractor tags use `tag:<tag>`.
    pub field: String,
    pub operator: Operator,
    pub threshold: Option<String>,
    /// Minimum fraction of pages (0.0–1.0) that must match before the pattern is "detected".
    pub min_prevalence: f64,
    pub business_impact: BusinessImpact,
    pub fix_effort: FixEffort,
    pub recommendation: String,
    /// Seeded patterns cannot be deleted, only disabled.
    pub is_builtin: bool,
    pub enabled: bool,
}

/// Parameters for creating or updating a user-defined pattern.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ReportPatternParams {
    pub name: String,
    pub description: String,
    pub category: PatternCategory,
    pub severity: PatternSeverity,
    pub field: String,
    pub operator: Operator,
    pub threshold: Option<String>,
    pub min_prevalence: f64,
    pub business_impact: BusinessImpact,
    pub fix_effort: FixEffort,
    pub recommendation: String,
    pub enabled: bool,
}

/// A pattern that fired during analysis of a specific job.
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DetectedPattern {
    pub pattern: ReportPattern,
    /// Fraction of pages where the condition holds (0.0–1.0).
    pub prevalence: f64,
    pub affected_pages: usize,
    pub total_pages: usize,
    /// priority = severity_weight × impact_weight × prevalence × effort_multiplier
    pub priority_score: f64,
    /// Up to 5 representative affected URLs.
    pub sample_urls: Vec<String>,
}

impl DetectedPattern {
    pub fn compute_priority(pattern: &ReportPattern, prevalence: f64) -> f64 {
        pattern.severity.weight()
            * pattern.business_impact.weight()
            * prevalence
            * pattern.fix_effort.multiplier()
    }
}

/// Per-pillar health scores (0–100) and an overall average.
///
/// Fields are private — construct via [`PillarScores::new`] or
/// [`PillarScores::from_pillars`] and read via the typed accessors.
/// The `overall` average is derived in the constructor so the
/// invariant `overall == mean(technical, content, performance,
/// accessibility)` is enforced once at construction time.
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PillarScores {
    technical: f64,
    content: f64,
    performance: f64,
    accessibility: f64,
    overall: f64,
}

impl PillarScores {
    /// Construct from all five values explicitly. Used by repository
    /// decoders or when the caller has computed `overall` itself.
    pub fn new(
        technical: f64,
        content: f64,
        performance: f64,
        accessibility: f64,
        overall: f64,
    ) -> Self {
        Self {
            technical,
            content,
            performance,
            accessibility,
            overall,
        }
    }

    /// Construct from the four pillar scores; the `overall` average is
    /// computed automatically as the arithmetic mean. This is the
    /// preferred constructor for new code — pinning the overall
    /// invariant in one place.
    pub fn from_pillars(
        technical: f64,
        content: f64,
        performance: f64,
        accessibility: f64,
    ) -> Self {
        let overall = (technical + content + performance + accessibility) / 4.0;
        Self {
            technical,
            content,
            performance,
            accessibility,
            overall,
        }
    }

    pub fn technical(&self) -> f64 {
        self.technical
    }
    pub fn content(&self) -> f64 {
        self.content
    }
    pub fn performance(&self) -> f64 {
        self.performance
    }
    pub fn accessibility(&self) -> f64 {
        self.accessibility
    }
    pub fn overall(&self) -> f64 {
        self.overall
    }
}

/// The full output of the report engine — ready for frontend rendering / PDF export.
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ReportData {
    pub job_id: String,
    pub url: String,
    pub seo_score: i64,
    pub total_pages: i64,
    pub total_issues: i64,
    pub critical_issues: i64,
    pub warning_issues: i64,
    pub sitemap_found: bool,
    pub robots_txt_found: bool,
    pub pillar_scores: PillarScores,
    /// Patterns sorted by `priority_score` descending.
    pub detected_patterns: Vec<DetectedPattern>,
    /// Structured narrative summary for AI consumption or direct PDF inclusion.
    pub ai_brief: String,
}

#[cfg(test)]
mod tests {
    //! Characterization tests for the report pattern domain.
    //!
    //! Pinning the `as_str` ↔ `from_str` round-trips for all four enums
    //! (the SQLite report repository decoder depends on them — a broken
    //! round-trip would surface as `RepositoryError::Decode`), the
    //! weight/multiplier numerical values that drive the priority
    //! score, and the `compute_priority` formula that drives the
    //! detected-pattern ordering.
    //!
    //! All four wire formats are also `snake_case` and frontend-visible
    //! via the Tauri bindings — pinning the serde tags here.

    use super::*;
    use std::str::FromStr;

    // ── PatternCategory ──────────────────────────────────────────────────

    #[test]
    fn pattern_category_round_trips_through_str() {
        for c in [
            PatternCategory::Technical,
            PatternCategory::Content,
            PatternCategory::Performance,
            PatternCategory::Accessibility,
        ] {
            assert_eq!(PatternCategory::from_str(c.as_str()).unwrap(), c);
        }
    }

    #[test]
    fn pattern_category_from_str_rejects_unknown() {
        assert!(PatternCategory::from_str("nonsense").is_err());
        assert!(PatternCategory::from_str("Technical").is_err()); // case sensitive
    }

    #[test]
    fn pattern_category_serde_uses_snake_case() {
        let json = serde_json::to_string(&PatternCategory::Performance).unwrap();
        assert_eq!(json, "\"performance\"");
    }

    // ── PatternSeverity ──────────────────────────────────────────────────

    #[test]
    fn pattern_severity_round_trips_through_str() {
        for s in [
            PatternSeverity::Critical,
            PatternSeverity::Warning,
            PatternSeverity::Suggestion,
        ] {
            assert_eq!(PatternSeverity::from_str(s.as_str()).unwrap(), s);
        }
    }

    #[test]
    fn pattern_severity_weight_descends_critical_to_suggestion() {
        // Pinning the relative ordering — the compute_priority math
        // depends on critical > warning > suggestion.
        let crit = PatternSeverity::Critical.weight();
        let warn = PatternSeverity::Warning.weight();
        let sug = PatternSeverity::Suggestion.weight();
        assert!(crit > warn);
        assert!(warn > sug);
        assert_eq!(crit, 3.0);
        assert_eq!(warn, 2.0);
        assert_eq!(sug, 1.0);
    }

    #[test]
    fn pattern_severity_from_str_rejects_unknown() {
        assert!(PatternSeverity::from_str("info").is_err());
    }

    // ── BusinessImpact ───────────────────────────────────────────────────

    #[test]
    fn business_impact_round_trips_through_str() {
        for b in [BusinessImpact::High, BusinessImpact::Medium, BusinessImpact::Low] {
            assert_eq!(BusinessImpact::from_str(b.as_str()).unwrap(), b);
        }
    }

    #[test]
    fn business_impact_weight_descends_high_to_low() {
        assert_eq!(BusinessImpact::High.weight(), 3.0);
        assert_eq!(BusinessImpact::Medium.weight(), 2.0);
        assert_eq!(BusinessImpact::Low.weight(), 1.0);
    }

    // ── FixEffort ────────────────────────────────────────────────────────

    #[test]
    fn fix_effort_round_trips_through_str() {
        for e in [FixEffort::Low, FixEffort::Medium, FixEffort::High] {
            assert_eq!(FixEffort::from_str(e.as_str()).unwrap(), e);
        }
    }

    #[test]
    fn fix_effort_multiplier_boosts_low_effort_fixes() {
        // Lower effort gets a higher multiplier so quick wins surface
        // first in the priority ranking. Pinning this counter-intuitive
        // direction so a "fix me, I look backwards" refactor doesn't
        // accidentally invert it.
        assert_eq!(FixEffort::Low.multiplier(), 1.5);
        assert_eq!(FixEffort::Medium.multiplier(), 1.0);
        assert_eq!(FixEffort::High.multiplier(), 0.5);
        assert!(FixEffort::Low.multiplier() > FixEffort::High.multiplier());
    }

    // ── compute_priority ─────────────────────────────────────────────────

    fn make_pattern(
        severity: PatternSeverity,
        impact: BusinessImpact,
        effort: FixEffort,
    ) -> ReportPattern {
        ReportPattern {
            id: "p".into(),
            name: "n".into(),
            description: "d".into(),
            category: PatternCategory::Technical,
            severity,
            field: "title".into(),
            operator: Operator::Missing,
            threshold: None,
            min_prevalence: 0.0,
            business_impact: impact,
            fix_effort: effort,
            recommendation: "fix it".into(),
            is_builtin: false,
            enabled: true,
        }
    }

    #[test]
    fn compute_priority_multiplies_all_four_factors() {
        // critical(3) × high(3) × prevalence(0.5) × low_effort(1.5) = 6.75
        let pattern = make_pattern(
            PatternSeverity::Critical,
            BusinessImpact::High,
            FixEffort::Low,
        );
        let score = DetectedPattern::compute_priority(&pattern, 0.5);
        assert_eq!(score, 6.75);
    }

    #[test]
    fn compute_priority_zero_prevalence_yields_zero_score() {
        let pattern = make_pattern(
            PatternSeverity::Critical,
            BusinessImpact::High,
            FixEffort::Low,
        );
        assert_eq!(DetectedPattern::compute_priority(&pattern, 0.0), 0.0);
    }

    #[test]
    fn compute_priority_orders_low_effort_above_high_effort_at_same_severity() {
        // Same severity + impact + prevalence; the only difference is
        // effort. The low-effort pattern should rank higher.
        let easy = make_pattern(
            PatternSeverity::Warning,
            BusinessImpact::Medium,
            FixEffort::Low,
        );
        let hard = make_pattern(
            PatternSeverity::Warning,
            BusinessImpact::Medium,
            FixEffort::High,
        );
        let easy_score = DetectedPattern::compute_priority(&easy, 0.4);
        let hard_score = DetectedPattern::compute_priority(&hard, 0.4);
        assert!(easy_score > hard_score);
    }

    #[test]
    fn compute_priority_orders_critical_above_suggestion_at_same_effort() {
        let crit = make_pattern(
            PatternSeverity::Critical,
            BusinessImpact::Medium,
            FixEffort::Medium,
        );
        let sug = make_pattern(
            PatternSeverity::Suggestion,
            BusinessImpact::Medium,
            FixEffort::Medium,
        );
        let crit_score = DetectedPattern::compute_priority(&crit, 0.5);
        let sug_score = DetectedPattern::compute_priority(&sug, 0.5);
        assert!(crit_score > sug_score);
    }
}

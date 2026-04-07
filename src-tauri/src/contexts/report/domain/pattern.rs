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

impl std::fmt::Display for PatternCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for PatternCategory {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "technical" => Ok(Self::Technical),
            "content" => Ok(Self::Content),
            "performance" => Ok(Self::Performance),
            "accessibility" => Ok(Self::Accessibility),
            other => Err(anyhow::anyhow!("Unknown pattern category: {}", other)),
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

impl std::fmt::Display for PatternSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for PatternSeverity {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Self::Critical),
            "warning" => Ok(Self::Warning),
            "suggestion" => Ok(Self::Suggestion),
            other => Err(anyhow::anyhow!("Unknown pattern severity: {}", other)),
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

impl std::fmt::Display for BusinessImpact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for BusinessImpact {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            other => Err(anyhow::anyhow!("Unknown business impact: {}", other)),
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

impl std::fmt::Display for FixEffort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for FixEffort {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            other => Err(anyhow::anyhow!("Unknown fix effort: {}", other)),
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
    /// `h1_count`. Custom extractor fields use `extracted:<key>`.
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
#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PillarScores {
    pub technical: f64,
    pub content: f64,
    pub performance: f64,
    pub accessibility: f64,
    pub overall: f64,
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

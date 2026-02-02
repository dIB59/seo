//! Domain models for the redesigned schema (v2)
//!
//! This module contains domain models that map to the new normalized schema.
//! Key changes:
//! - `Job` consolidates job, result, and settings into one entity
//! - `Page` has direct `job_id` FK for fast queries
//! - `Issue` has direct `job_id` FK eliminating JOINs
//! - `Link` represents page edges with direct `job_id`
//! - Headings and images are normalized into separate tables

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// JOB - Consolidated job, settings, and summary
// ============================================================================

/// Status of an SEO analysis job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl std::str::FromStr for JobStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" | "queued" => Ok(Self::Pending),
            "running" | "processing" | "discovering" | "analyzing" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" | "error" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Job settings for crawl configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSettings {
    pub max_pages: i64,
    pub max_depth: i64,
    pub respect_robots_txt: bool,
    pub include_subdomains: bool,
    pub rate_limit_ms: i64,
    pub user_agent: Option<String>,
}

impl Default for JobSettings {
    fn default() -> Self {
        Self {
            max_pages: 100,
            max_depth: 3,
            respect_robots_txt: true,
            include_subdomains: false,
            rate_limit_ms: 1000,
            user_agent: None,
        }
    }
}

/// Summary statistics for a job (denormalized for fast dashboard access).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobSummary {
    pub total_pages: i64,
    pub pages_crawled: i64,
    pub total_issues: i64,
    pub critical_issues: i64,
    pub warning_issues: i64,
    pub info_issues: i64,
}

/// Consolidated job entity - combines job metadata, settings, and summary.
/// Maps to the `jobs` table in the new schema.
#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,

    // Settings (embedded)
    pub settings: JobSettings,

    // Summary stats (denormalized)
    pub summary: JobSummary,

    // Progress tracking
    pub progress: f64,
    pub current_stage: Option<String>,
    pub error_message: Option<String>,
}

impl Job {
    /// Create a new job with default settings.
    pub fn new(url: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            status: JobStatus::Pending,
            created_at: now,
            updated_at: now,
            completed_at: None,
            settings: JobSettings::default(),
            summary: JobSummary::default(),
            progress: 0.0,
            current_stage: None,
            error_message: None,
        }
    }

    /// Create a new job with custom settings.
    pub fn with_settings(url: String, settings: JobSettings) -> Self {
        let mut job = Self::new(url);
        job.settings = settings;
        job
    }
}

/// Lightweight job info for listing (without full settings/summary).
#[derive(Debug, Clone, Serialize)]
pub struct JobInfo {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub progress: f64,
    pub total_pages: i64,
    pub total_issues: i64,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// PAGE - Crawled page with direct job reference
// ============================================================================

/// A crawled page with SEO data.
/// Maps to the `pages` table in the new schema.
#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub id: String,
    pub job_id: String,
    pub url: String,
    pub depth: i64,
    pub status_code: Option<i64>,
    pub content_type: Option<String>,

    // Core SEO fields
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub canonical_url: Option<String>,
    pub robots_meta: Option<String>,

    // Content metrics
    pub word_count: Option<i64>,
    pub load_time_ms: Option<i64>,
    pub response_size_bytes: Option<i64>,

    pub crawled_at: DateTime<Utc>,
}

/// Lightweight page info for listings.
#[derive(Debug, Clone, Serialize)]
pub struct PageInfo {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<i64>,
    pub load_time_ms: Option<i64>,
    pub issue_count: i64,
}

// ============================================================================
// ISSUE - SEO issues with direct job reference
// ============================================================================

/// Severity level for SEO issues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

impl IssueSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

impl std::str::FromStr for IssueSeverity {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Self::Critical),
            "warning" => Ok(Self::Warning),
            "info" | "suggestion" => Ok(Self::Info),
            _ => Err(()),
        }
    }
}

/// An SEO issue found during analysis.
/// Maps to the `issues` table with direct `job_id` FK.
#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub id: i64,
    pub job_id: String,
    pub page_id: Option<String>,

    pub issue_type: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub details: Option<String>,

    pub created_at: DateTime<Utc>,
}

/// Builder for creating issues.
pub struct IssueBuilder {
    job_id: String,
    page_id: Option<String>,
    issue_type: String,
    severity: IssueSeverity,
    message: String,
    details: Option<String>,
}

impl IssueBuilder {
    pub fn new(job_id: String, issue_type: String, severity: IssueSeverity, message: String) -> Self {
        Self {
            job_id,
            page_id: None,
            issue_type,
            severity,
            message,
            details: None,
        }
    }

    pub fn page_id(mut self, page_id: String) -> Self {
        self.page_id = Some(page_id);
        self
    }

    pub fn details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn build(self) -> NewIssue {
        NewIssue {
            job_id: self.job_id,
            page_id: self.page_id,
            issue_type: self.issue_type,
            severity: self.severity,
            message: self.message,
            details: self.details,
        }
    }
}

/// New issue to be inserted (without auto-generated fields).
#[derive(Debug, Clone)]
pub struct NewIssue {
    pub job_id: String,
    pub page_id: Option<String>,
    pub issue_type: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub details: Option<String>,
}

// ============================================================================
// LINK - Page edges with direct job reference
// ============================================================================

/// Type of link (internal, external, resource).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Internal,
    External,
    Resource,
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::External => "external",
            Self::Resource => "resource",
        }
    }
}

impl std::str::FromStr for LinkType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "internal" => Ok(Self::Internal),
            "external" => Ok(Self::External),
            "resource" => Ok(Self::Resource),
            _ => Err(()),
        }
    }
}

/// A link between pages.
/// Maps to the `links` table with direct `job_id` FK.
#[derive(Debug, Clone, Serialize)]
pub struct Link {
    pub id: i64,
    pub job_id: String,
    pub source_page_id: String,
    pub target_page_id: Option<String>,

    pub target_url: String,
    pub link_text: Option<String>,
    pub link_type: LinkType,
    pub is_followed: bool,
    pub status_code: Option<i64>,
}

/// New link to be inserted.
#[derive(Debug, Clone)]
pub struct NewLink {
    pub job_id: String,
    pub source_page_id: String,
    pub target_page_id: Option<String>,
    pub target_url: String,
    pub link_text: Option<String>,
    pub link_type: LinkType,
    pub is_followed: bool,
    pub status_code: Option<i64>,
}

// ============================================================================
// LIGHTHOUSE DATA
// ============================================================================

/// Lighthouse performance metrics for a page.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LighthouseData {
    pub page_id: String,

    // Core Web Vitals scores (0-100)
    pub performance_score: Option<f64>,
    pub accessibility_score: Option<f64>,
    pub best_practices_score: Option<f64>,
    pub seo_score: Option<f64>,

    // Performance metrics
    pub first_contentful_paint_ms: Option<f64>,
    pub largest_contentful_paint_ms: Option<f64>,
    pub total_blocking_time_ms: Option<f64>,
    pub cumulative_layout_shift: Option<f64>,
    pub speed_index: Option<f64>,
    pub time_to_interactive_ms: Option<f64>,

    // Raw JSON for detailed analysis
    pub raw_json: Option<String>,
}

// ============================================================================
// HEADINGS & IMAGES (Normalized from JSON blobs)
// ============================================================================

/// A heading element on a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub id: i64,
    pub page_id: String,
    pub level: i64, // 1-6
    pub text: String,
    pub position: i64,
}

/// New heading to be inserted.
#[derive(Debug, Clone)]
pub struct NewHeading {
    pub page_id: String,
    pub level: i64,
    pub text: String,
    pub position: i64,
}

/// An image element on a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: i64,
    pub page_id: String,
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub loading: Option<String>,
    pub is_decorative: bool,
}

/// New image to be inserted.
#[derive(Debug, Clone)]
pub struct NewImage {
    pub page_id: String,
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub loading: Option<String>,
    pub is_decorative: bool,
}

// ============================================================================
// AI INSIGHTS
// ============================================================================

/// AI-generated insights for a job.
#[derive(Debug, Clone, Serialize)]
pub struct AiInsight {
    pub id: i64,
    pub job_id: String,
    pub summary: Option<String>,
    pub recommendations: Option<String>,
    pub raw_response: Option<String>,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// GLOBAL SETTINGS
// ============================================================================

/// Global application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub default_ai_provider: String,
    pub default_max_pages: i64,
    pub default_max_depth: i64,
    pub default_rate_limit_ms: i64,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            anthropic_api_key: None,
            google_api_key: None,
            default_ai_provider: "openai".to_string(),
            default_max_pages: 100,
            default_max_depth: 3,
            default_rate_limit_ms: 1000,
            theme: "system".to_string(),
        }
    }
}

// ============================================================================
// COMPLETE RESULT (for API responses)
// ============================================================================

/// Complete analysis result with all related data.
/// Used for API responses when fetching full job details.
#[derive(Debug, Clone, Serialize)]
pub struct CompleteJobResult {
    pub job: Job,
    pub pages: Vec<Page>,
    pub issues: Vec<Issue>,
    pub links: Vec<Link>,
    pub lighthouse: Vec<LighthouseData>,
    pub ai_insights: Option<AiInsight>,
}

// ============================================================================
// API RESPONSE TYPES (for frontend compatibility)
// ============================================================================

/// Resource status (robots.txt, sitemap, SSL)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceStatus {
    NotChecked,
    Found(String),
    NotFound,
    Unauthorized(String),
    Error,
}

impl ResourceStatus {
    /// Returns true if the resource exists (Found or Unauthorized)
    pub fn exists(&self) -> bool {
        matches!(self, Self::Found(_) | Self::Unauthorized(_))
    }
}

impl Default for ResourceStatus {
    fn default() -> Self {
        Self::NotChecked
    }
}

/// Analysis progress for frontend updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProgress {
    pub job_id: String,
    pub url: String,
    pub job_status: String,
    pub result_id: Option<String>,
    pub progress: Option<f64>,
    pub analyzed_pages: Option<i64>,
    pub total_pages: Option<i64>,
}

/// Summary of analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub analysis_id: String,
    pub seo_score: i64,
    pub avg_load_time: f64,
    pub total_words: i64,
    pub total_issues: i64,
}

/// Detailed analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResults {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub progress: f64,
    pub total_pages: i64,
    pub analyzed_pages: i64,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub sitemap_found: bool,
    pub robots_txt_found: bool,
    pub ssl_certificate: bool,
    pub created_at: DateTime<Utc>,
}

/// SEO issue (frontend-compatible format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeoIssue {
    pub page_id: String,
    pub issue_type: IssueType,
    pub title: String,
    pub description: String,
    pub page_url: String,
    pub element: Option<String>,
    pub recommendation: String,
    pub line_number: Option<i32>,
}

/// Issue severity type (frontend-compatible)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueType {
    Critical,
    Warning,
    Suggestion,
}

/// Page analysis data (frontend-compatible format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageAnalysisData {
    pub analysis_id: String,
    pub url: String,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub canonical_url: Option<String>,
    pub h1_count: i64,
    pub h2_count: i64,
    pub h3_count: i64,
    pub word_count: i64,
    pub image_count: i64,
    pub images_without_alt: i64,
    pub internal_links: i64,
    pub external_links: i64,
    pub load_time: f64,
    pub status_code: Option<i64>,
    pub content_size: i64,
    pub mobile_friendly: bool,
    pub has_structured_data: bool,
    pub lighthouse_performance: Option<f64>,
    pub lighthouse_accessibility: Option<f64>,
    pub lighthouse_best_practices: Option<f64>,
    pub lighthouse_seo: Option<f64>,
    pub lighthouse_seo_audits: Option<serde_json::Value>,
    pub lighthouse_performance_metrics: Option<serde_json::Value>,
    pub links: Vec<String>,
    pub headings: Vec<String>,
    pub images: Vec<String>,
    pub detailed_links: Vec<LinkDetail>,
}

/// Link details (frontend-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDetail {
    pub url: String,
    pub text: String,
    pub is_external: bool,
    pub is_broken: bool,
    pub status_code: Option<i64>,
}

/// Complete analysis result (frontend-compatible format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteAnalysisResult {
    pub analysis: AnalysisResults,
    pub pages: Vec<PageAnalysisData>,
    pub issues: Vec<SeoIssue>,
    pub summary: AnalysisSummary,
}

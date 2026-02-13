use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::issue::IssueSeverity;
use crate::domain::job::JobStatus;

/// Resource status (robots.txt, sitemap, SSL)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceStatus {
    #[default]
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

/// Analysis progress for frontend updates
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
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
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub page_url: String,
    pub element: Option<String>,
    pub recommendation: String,
    pub line_number: Option<i32>,
}

/// Heading element for frontend display.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HeadingElement {
    pub tag: String,
    pub text: String,
}

/// Image element for frontend display.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImageElement {
    pub src: String,
    pub alt: Option<String>,
}

/// Link details (frontend-compatible)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LinkDetail {
    #[serde(rename = "href", alias = "url")]
    pub url: String,
    pub text: String,
    pub is_external: bool,
    pub is_broken: bool,
    pub status_code: Option<i64>,
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
    pub headings: Vec<HeadingElement>,
    pub images: Vec<ImageElement>,
    pub detailed_links: Vec<LinkDetail>,
}

/// Complete analysis result (frontend-compatible format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteAnalysisResult {
    pub analysis: AnalysisResults,
    pub pages: Vec<PageAnalysisData>,
    pub issues: Vec<SeoIssue>,
    pub summary: AnalysisSummary,
}

/// Complete job result with all related data.
#[derive(Debug, Clone, Serialize)]
pub struct CompleteJobResult {
    pub job: crate::domain::job::Job,
    pub pages: Vec<crate::domain::page::Page>,
    pub issues: Vec<crate::domain::issue::Issue>,
    pub links: Vec<crate::domain::link::Link>,
    pub lighthouse: Vec<crate::domain::lighthouse::LighthouseData>,
    pub ai_insights: Option<crate::domain::ai::AiInsight>,
}

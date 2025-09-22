// src-tauri/src/seo_service.rs
// NOTE: You asked to update structs only — this file defines the data structures used
// by the Tauri commands and events. Frontend usage should remain unchanged.
// Be careful to keep field names snake_case so they serialize to the TypeScript shapes.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TauriResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueType {
    Critical,
    Warning,
    Suggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SeoIssue {
    pub id: String,
    #[serde(rename = "type")]
    pub issue_type: IssueType,
    pub title: String,
    pub description: String,
    pub page_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u32>,
    pub recommendation: String,
}

/// Lighthouse score breakdown (optional on a page)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LighthouseScore {
    pub performance: u32,
    pub accessibility: u32,
    pub best_practices: u32,
    pub seo: u32,
}

/// Per-page analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PageAnalysis {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_url: Option<String>,

    pub h1_count: u32,
    pub h2_count: u32,
    pub h3_count: u32,
    pub image_count: u32,
    pub images_without_alt: u32,
    pub internal_links: u32,
    pub external_links: u32,
    pub word_count: u32,
    pub load_time: f64,
    pub status_code: u16,
    pub content_size: u64,
    pub mobile_friendly: bool,
    pub has_structured_data: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lighthouse_score: Option<LighthouseScore>,

    #[serde(default)]
    pub issues: Vec<SeoIssue>,

    pub created_at: String,
}

/// Analysis status: 'analyzing' | 'completed' | 'error' | 'paused'
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisStatus {
    Analyzing,
    Completed,
    Error,
    Paused,
}

/// Counts of issue severities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IssueCounts {
    pub critical: u32,
    pub warnings: u32,
    pub suggestions: u32,
}

/// Summary block inside AnalysisResult.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalysisSummary {
    pub avg_load_time: f64,
    pub total_words: u64,
    pub pages_with_issues: u32,
    pub seo_score: u32, // 0-100
    pub mobile_friendly_pages: u32,
    pub pages_with_meta_description: u32,
    pub pages_with_title_issues: u32,
    pub duplicate_titles: u32,
    pub duplicate_meta_descriptions: u32,
}

/// Full analysis result for an entire site / run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalysisResult {
    pub id: String,
    pub url: String,
    pub status: AnalysisStatus,
    pub progress: u32,
    pub total_pages: u32,
    pub analyzed_pages: u32,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub pages: Vec<PageAnalysis>,
    pub issues: IssueCounts,
    pub summary: AnalysisSummary,
    pub sitemap_found: bool,
    pub robots_txt_found: bool,
    pub ssl_certificate: bool,
}

/// Simple error structure for analysis operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalysisError {
    pub message: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Settings that control how the analysis is performed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalysisSettings {
    pub max_pages: u32,
    pub include_external_links: bool,
    pub check_images: bool,
    pub mobile_analysis: bool,
    pub lighthouse_analysis: bool,
    pub delay_between_requests: u64, // milliseconds
}

/// Response for listing analyses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalysisListResponse {
    pub analyses: Vec<AnalysisResult>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

/// Event payload for progress updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalysisProgressEvent {
    pub analysis_id: String,
    pub progress: u32,
    pub analyzed_pages: u32,
    pub total_pages: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_page: Option<String>,
}

/// Event payload for analysis completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalysisCompleteEvent {
    pub analysis_id: String,
    pub result: AnalysisResult,
}

/// Event payload for analysis errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AnalysisErrorEvent {
    pub analysis_id: String,
    pub error: String,
}

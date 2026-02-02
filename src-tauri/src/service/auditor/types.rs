//! Shared types for audit results.
//!
//! These types are auditor-agnostic and work with both Light and Deep auditors.

use serde::{Deserialize, Serialize};

/// Result from an audit analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    pub url: String,
    pub html: String,
    pub status_code: u16,
    pub load_time_ms: f64,
    pub content_size: usize,
    pub scores: AuditScores,
}

/// Audit category scores (0.0 to 1.0).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditScores {
    pub performance: Option<f64>,
    pub accessibility: Option<f64>,
    pub best_practices: Option<f64>,
    pub seo: Option<f64>,
    pub seo_details: SeoAuditDetails,
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// Detailed performance metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub first_contentful_paint: Option<f64>,
    pub largest_contentful_paint: Option<f64>,
    pub speed_index: Option<f64>,
    pub time_to_interactive: Option<f64>,
    pub total_blocking_time: Option<f64>,
    pub cumulative_layout_shift: Option<f64>,
}

/// Individual check result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CheckResult {
    pub passed: bool,
    pub value: Option<String>,
    pub score: f64,
    #[serde(default)]
    pub description: Option<String>,
}

/// Detailed SEO audit results.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeoAuditDetails {
    pub document_title: CheckResult,
    pub meta_description: CheckResult,
    pub viewport: CheckResult,
    pub canonical: CheckResult,
    pub hreflang: CheckResult,
    pub robots_txt: CheckResult,
    pub crawlable_anchors: CheckResult,
    pub link_text: CheckResult,
    pub image_alt: CheckResult,
    pub http_status_code: CheckResult,
    pub is_crawlable: CheckResult,
}

impl SeoAuditDetails {
    /// Calculate overall SEO score from individual checks.
    pub fn calculate_score(&self) -> f64 {
        let checks = [
            &self.document_title,
            &self.meta_description,
            &self.viewport,
            &self.canonical,
            &self.http_status_code,
            &self.is_crawlable,
            &self.image_alt,
            &self.link_text,
            &self.crawlable_anchors,
        ];
        
        let total: f64 = checks.iter().map(|c| c.score).sum();
        total / checks.len() as f64
    }
}

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

/// A single named audit check suitable for frontend display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditCheck {
    pub key: &'static str,
    pub label: &'static str,
    pub passed: bool,
    pub score: f64,
    pub value: Option<String>,
    pub description: Option<String>,
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

    /// Return a frontend-friendly breakdown of individual SEO audit checks.
    pub fn breakdown(&self) -> Vec<AuditCheck> {
        vec![
            AuditCheck { key: "document_title", label: "Document Title", passed: self.document_title.passed, score: self.document_title.score, value: self.document_title.value.clone(), description: self.document_title.description.clone() },
            AuditCheck { key: "meta_description", label: "Meta Description", passed: self.meta_description.passed, score: self.meta_description.score, value: self.meta_description.value.clone(), description: self.meta_description.description.clone() },
            AuditCheck { key: "viewport", label: "Viewport Meta Tag", passed: self.viewport.passed, score: self.viewport.score, value: self.viewport.value.clone(), description: self.viewport.description.clone() },
            AuditCheck { key: "canonical", label: "Canonical URL", passed: self.canonical.passed, score: self.canonical.score, value: self.canonical.value.clone(), description: self.canonical.description.clone() },
            AuditCheck { key: "hreflang", label: "Hreflang Tags", passed: self.hreflang.passed, score: self.hreflang.score, value: self.hreflang.value.clone(), description: self.hreflang.description.clone() },
            AuditCheck { key: "robots_txt", label: "Robots.txt Valid", passed: self.robots_txt.passed, score: self.robots_txt.score, value: self.robots_txt.value.clone(), description: self.robots_txt.description.clone() },
            AuditCheck { key: "crawlable_anchors", label: "Crawlable Anchors", passed: self.crawlable_anchors.passed, score: self.crawlable_anchors.score, value: self.crawlable_anchors.value.clone(), description: self.crawlable_anchors.description.clone() },
            AuditCheck { key: "link_text", label: "Descriptive Link Text", passed: self.link_text.passed, score: self.link_text.score, value: self.link_text.value.clone(), description: self.link_text.description.clone() },
            AuditCheck { key: "image_alt", label: "Image Alt Attributes", passed: self.image_alt.passed, score: self.image_alt.score, value: self.image_alt.value.clone(), description: self.image_alt.description.clone() },
            AuditCheck { key: "http_status_code", label: "HTTP Status Code", passed: self.http_status_code.passed, score: self.http_status_code.score, value: self.http_status_code.value.clone(), description: self.http_status_code.description.clone() },
            AuditCheck { key: "is_crawlable", label: "Page is Crawlable", passed: self.is_crawlable.passed, score: self.is_crawlable.score, value: self.is_crawlable.value.clone(), description: self.is_crawlable.description.clone() },
        ]
    }
}

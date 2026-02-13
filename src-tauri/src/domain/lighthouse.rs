use serde::{Deserialize, Serialize};

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

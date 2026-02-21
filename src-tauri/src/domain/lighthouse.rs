use crate::service::auditor::{AuditScores, Score};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LighthouseData {
    pub page_id: String,
    pub performance_score: Option<f64>,
    pub accessibility_score: Option<f64>,
    pub best_practices_score: Option<f64>,
    pub seo_score: Option<f64>,
    pub first_contentful_paint_ms: Option<f64>,
    pub largest_contentful_paint_ms: Option<f64>,
    pub total_blocking_time_ms: Option<f64>,
    pub cumulative_layout_shift: Option<f64>,
    pub speed_index: Option<f64>,
    pub time_to_interactive_ms: Option<f64>,
    pub raw_json: Option<String>,
}

impl LighthouseData {
    pub fn from_audit_scores(page_id: &str, scores: &AuditScores) -> Self {
        let normalize = |s: Option<Score>| -> Option<f64> { s.map(|s| s.percent()) };

        let raw_json = serde_json::json!({
            "seo_audits": scores.seo_details,
            "performance_metrics": scores.performance_metrics,
        });
        let raw_json = serde_json::to_string(&raw_json).ok();

        let metrics = scores.performance_metrics.as_ref();

        Self {
            page_id: page_id.to_string(),
            performance_score: normalize(scores.performance),
            accessibility_score: normalize(scores.accessibility),
            best_practices_score: normalize(scores.best_practices),
            seo_score: normalize(scores.seo),
            first_contentful_paint_ms: metrics.and_then(|m| m.first_contentful_paint),
            largest_contentful_paint_ms: metrics.and_then(|m| m.largest_contentful_paint),
            total_blocking_time_ms: metrics.and_then(|m| m.total_blocking_time),
            cumulative_layout_shift: metrics.and_then(|m| m.cumulative_layout_shift),
            speed_index: metrics.and_then(|m| m.speed_index),
            time_to_interactive_ms: metrics.and_then(|m| m.time_to_interactive),
            raw_json,
        }
    }

    pub fn is_mobile_friendly(&self) -> bool {
        self.raw_json
            .as_deref()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
            .and_then(|v| {
                v.get("seo_audits")?
                    .get("viewport")?
                    .get("passed")?
                    .as_bool()
            })
            .unwrap_or(false)
    }

    pub fn has_structured_data(&self) -> bool {
        self.raw_json
            .as_deref()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
            .is_some_and(|v| v.get("structured_data").is_some())
    }

    /// Extracts both SEO audits and performance metrics as JSON values.
    pub fn interpret_raw(&self) -> (Option<serde_json::Value>, Option<serde_json::Value>) {
        self.raw_json
            .as_deref()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
            .map(|v| {
                (
                    v.get("seo_audits").cloned(),
                    v.get("performance_metrics").cloned(),
                )
            })
            .unwrap_or((None, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::auditor::{PerformanceMetrics, Score, SeoAuditDetails};

    #[test]
    fn from_audit_scores_normalizes_percentages() {
        let scores = AuditScores {
            performance: Some(Score::from(0.85)),
            accessibility: Some(Score::from(0.92)),
            best_practices: Some(Score::from(0.78)),
            seo: Some(Score::from(0.95)),
            seo_details: SeoAuditDetails::default(),
            performance_metrics: Some(PerformanceMetrics {
                first_contentful_paint: Some(1200.0),
                largest_contentful_paint: Some(2500.0),
                speed_index: Some(1800.0),
                time_to_interactive: Some(3200.0),
                total_blocking_time: Some(150.0),
                cumulative_layout_shift: Some(0.05),
            }),
        };

        let lh = LighthouseData::from_audit_scores("page-1", &scores);

        assert_eq!(lh.page_id, "page-1");
        assert_eq!(lh.performance_score, Some(85.0));
        assert_eq!(lh.accessibility_score, Some(92.0));
        assert_eq!(lh.best_practices_score, Some(78.0));
        assert_eq!(lh.seo_score, Some(95.0));
        assert_eq!(lh.first_contentful_paint_ms, Some(1200.0));
        assert_eq!(lh.largest_contentful_paint_ms, Some(2500.0));
        assert_eq!(lh.total_blocking_time_ms, Some(150.0));
        assert_eq!(lh.cumulative_layout_shift, Some(0.05));
        assert!(lh.raw_json.is_some());
    }

    #[test]
    fn from_audit_scores_handles_none_metrics() {
        let scores = AuditScores::default();
        let lh = LighthouseData::from_audit_scores("page-2", &scores);

        assert!(lh.performance_score.is_none());
        assert!(lh.first_contentful_paint_ms.is_none());
    }
}

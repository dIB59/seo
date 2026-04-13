use serde::{Deserialize, Serialize};
use specta::Type;

use crate::contexts::analysis::{
    AnalysisProgress, Job, JobSettings, JobStatus, LinkType,
};

#[derive(Debug, serde::Deserialize, serde::Serialize, specta::Type)]
pub struct AnalysisSettingsRequest {
    pub max_pages: i64,
    pub include_subdomains: bool,
    pub check_images: bool,
    pub mobile_analysis: bool,
    pub lighthouse_analysis: bool,
    pub delay_between_requests: i64,
}

pub(crate) trait SettingsExt {
    fn requested_page_count(&self) -> usize;
}

impl SettingsExt for Option<AnalysisSettingsRequest> {
    fn requested_page_count(&self) -> usize {
        self.as_ref()
            .unwrap_or(&AnalysisSettingsRequest::default())
            .max_pages as usize
    }
}

impl Default for AnalysisSettingsRequest {
    fn default() -> Self {
        Self {
            max_pages: 100,
            include_subdomains: false,
            check_images: true,
            mobile_analysis: false,
            lighthouse_analysis: false,
            delay_between_requests: 50,
        }
    }
}

impl From<AnalysisSettingsRequest> for JobSettings {
    fn from(req: AnalysisSettingsRequest) -> Self {
        Self {
            max_pages: req.max_pages,
            include_subdomains: req.include_subdomains,
            check_images: req.check_images,
            mobile_analysis: req.mobile_analysis,
            lighthouse_analysis: req.lighthouse_analysis,
            delay_between_requests: req.delay_between_requests,
        }
    }
}

#[derive(Debug, serde::Serialize, Type)]
pub struct AnalysisJobResponse {
    pub job_id: String,
    pub url: String,
    pub status: JobStatus,
}

#[derive(Debug, serde::Serialize, Type)]
pub struct PaginatedJobsResponse {
    pub items: Vec<AnalysisProgress>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HeadingElement {
    pub tag: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImageElement {
    pub src: String,
    pub alt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LinkDetail {
    #[serde(rename = "href", alias = "url")]
    pub url: String,
    pub text: String,
    pub link_type: LinkType,
    pub is_broken: bool,
    pub status_code: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct PageAnalysisData {
    pub analysis_id: String,
    pub url: String,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub canonical_url: Option<String>,
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
    pub images: Vec<ImageElement>,
    pub detailed_links: Vec<LinkDetail>,
    pub headings: Vec<HeadingElement>,
    pub extracted_data: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct SeoIssue {
    pub page_id: String,
    pub severity: crate::contexts::analysis::IssueSeverity,
    pub title: String,
    pub description: String,
    pub page_url: String,
    pub element: Option<String>,
    pub recommendation: String,
    pub line_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AnalysisSummary {
    pub analysis_id: String,
    pub seo_score: i64,
    pub avg_load_time: f64,
    pub total_words: i64,
    pub total_issues: i64,
}

impl AnalysisSummary {
    pub(super) fn compute(job: &Job, pages: &[PageAnalysisData]) -> Self {
        let (total_load, load_count) = pages
            .iter()
            .filter(|p| p.load_time > 0.0)
            .fold((0.0f64, 0usize), |(sum, cnt), p| {
                (sum + p.load_time, cnt + 1)
            });

        let lh_scores: Vec<f64> = pages.iter().filter_map(|p| p.lighthouse_seo).collect();
        let seo_score = if lh_scores.is_empty() {
            job.calculate_seo_score()
        } else {
            (lh_scores.iter().sum::<f64>() / lh_scores.len() as f64).round() as i64
        };

        Self {
            analysis_id: job.id.as_str().to_string(),
            seo_score,
            avg_load_time: total_load / load_count.max(1) as f64,
            total_words: pages.iter().map(|p| p.word_count).sum(),
            total_issues: job.summary.total_issues(),
        }
    }
}

#[derive(Debug, Serialize, Type)]
pub struct AnalysisResults {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub progress: f64,
    pub total_pages: i64,
    pub analyzed_pages: i64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub sitemap_found: bool,
    pub robots_txt_found: bool,
    pub ssl_certificate: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Type)]
pub struct CompleteAnalysisResponse {
    pub analysis: AnalysisResults,
    pub pages: Vec<PageAnalysisData>,
    pub issues: Vec<SeoIssue>,
    pub summary: AnalysisSummary,
}

// ── From impls for domain → DTO ─────────────────────────────────────

impl From<crate::contexts::analysis::Link> for LinkDetail {
    fn from(link: crate::contexts::analysis::Link) -> Self {
        Self {
            url: link.target_url,
            text: link.link_text.unwrap_or_default(),
            link_type: link.link_type,
            is_broken: link.status_code.is_some_and(|c| c >= 400),
            status_code: link.status_code,
        }
    }
}

impl From<crate::contexts::analysis::Heading> for HeadingElement {
    fn from(h: crate::contexts::analysis::Heading) -> Self {
        Self {
            tag: format!("h{}", h.level),
            text: h.text,
        }
    }
}

impl From<crate::contexts::analysis::Image> for ImageElement {
    fn from(img: crate::contexts::analysis::Image) -> Self {
        Self {
            src: img.src,
            alt: img.alt,
        }
    }
}

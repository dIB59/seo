use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use url::Url;

use crate::{
    contexts::{
        permissions::PermissionRequest, AnalysisProgress, Job, JobSettings, JobStatus,
        LighthouseData, Page,
    },
    error::CommandError,
    lifecycle::app_state::AppState,
};
use addon_macros::addon_guard;

#[derive(Debug, serde::Deserialize, serde::Serialize, specta::Type)]
pub struct AnalysisSettingsRequest {
    pub max_pages: i64,
    pub include_subdomains: bool,
    pub check_images: bool,
    pub mobile_analysis: bool,
    pub lighthouse_analysis: bool,
    pub delay_between_requests: i64,
}

trait SettingsExt {
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
    pub link_type: crate::contexts::LinkType,
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
    /// Extracted data from custom extractors (key-value pairs)
    pub extracted_data: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct SeoIssue {
    pub page_id: String,
    pub severity: crate::contexts::IssueSeverity,
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
    fn compute(job: &Job, pages: &[PageAnalysisData]) -> Self {
        let (total_load, load_count) = pages.iter().fold((0.0f64, 0usize), |(sum, cnt), p| {
            if p.load_time > 0.0 {
                (sum + p.load_time, cnt + 1)
            } else {
                (sum, cnt)
            }
        });

        Self {
            analysis_id: job.id.clone(),
            seo_score: job.calculate_seo_score(),
            avg_load_time: total_load / load_count.max(1) as f64,
            total_words: pages.iter().map(|p| p.word_count).sum(),
            total_issues: job.summary.total_issues,
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

impl From<crate::contexts::Link> for LinkDetail {
    fn from(link: crate::contexts::Link) -> Self {
        Self {
            url: link.target_url,
            text: link.link_text.unwrap_or_default(),
            link_type: link.link_type,
            is_broken: link.status_code.is_some_and(|c| c >= 400),
            status_code: link.status_code,
        }
    }
}

impl From<crate::contexts::Heading> for HeadingElement {
    fn from(h: crate::contexts::Heading) -> Self {
        Self {
            tag: format!("h{}", h.level),
            text: h.text,
        }
    }
}

impl From<crate::contexts::Image> for ImageElement {
    fn from(img: crate::contexts::Image) -> Self {
        Self {
            src: img.src,
            alt: img.alt,
        }
    }
}

fn count_links_by_type(links: &[LinkDetail]) -> (i64, i64) {
    links.iter().fold((0, 0), |(internal, external), link| {
        if link.link_type == crate::contexts::LinkType::Internal {
            (internal + 1, external)
        } else {
            (internal, external + 1)
        }
    })
}

fn count_images_without_alt(images: &[ImageElement]) -> i64 {
    images
        .iter()
        .filter(|img| img.alt.as_deref().unwrap_or("").is_empty())
        .count() as i64
}

impl CompleteAnalysisResponse {
    fn assemble_page(
        page: Page,
        lh_data: Option<&LighthouseData>,
        detailed_links: Vec<LinkDetail>,
        headings: Vec<HeadingElement>,
        images: Vec<ImageElement>,
        extracted_data: std::collections::HashMap<String, serde_json::Value>,
    ) -> PageAnalysisData {
        let load_time = page.load_time_ms.unwrap_or(0) as f64 / 1000.0;
        let mobile_friendly = lh_data.is_some_and(|lh| lh.is_mobile_friendly())
            || page.is_mobile_friendly_heuristic();
        let has_structured_data =
            page.has_structured_data || lh_data.is_some_and(|lh| lh.has_structured_data());
        let (lighthouse_seo_audits, lighthouse_performance_metrics) =
            lh_data.map(|lh| lh.interpret_raw()).unwrap_or((None, None));
        let (internal_links, external_links) = count_links_by_type(&detailed_links);

        PageAnalysisData {
            analysis_id: page.job_id,
            url: page.url,
            title: page.title,
            meta_description: page.meta_description,
            meta_keywords: None,
            canonical_url: page.canonical_url,
            word_count: page.word_count.unwrap_or(0),
            image_count: images.len() as i64,
            images_without_alt: count_images_without_alt(&images),
            internal_links,
            external_links,
            load_time,
            status_code: page.status_code,
            content_size: page.response_size_bytes.unwrap_or(0),
            mobile_friendly,
            has_structured_data,
            lighthouse_performance: lh_data.and_then(|lh| lh.performance_score),
            lighthouse_accessibility: lh_data.and_then(|lh| lh.accessibility_score),
            lighthouse_best_practices: lh_data.and_then(|lh| lh.best_practices_score),
            lighthouse_seo: lh_data.and_then(|lh| lh.seo_score),
            lighthouse_seo_audits,
            lighthouse_performance_metrics,
            images,
            detailed_links,
            headings,
            extracted_data,
        }
    }
}

impl From<crate::contexts::CompleteJobResult> for CompleteAnalysisResponse {
    fn from(result: crate::contexts::CompleteJobResult) -> Self {
        let job = result.job;
        let pages = result.pages;
        let issues = result.issues;
        let links = result.links;
        let lighthouse = result.lighthouse;
        let headings = result.headings;
        let images = result.images;

        let page_url_by_id: HashMap<String, String> = pages
            .iter()
            .map(|p| (p.id.clone(), p.url.clone()))
            .collect();

        let mut links_by_page: HashMap<String, Vec<LinkDetail>> = HashMap::new();
        let mut headings_by_page: HashMap<String, Vec<HeadingElement>> = HashMap::new();
        let mut images_by_page: HashMap<String, Vec<ImageElement>> = HashMap::new();

        for link in links {
            let page_id = link.source_page_id.clone();
            links_by_page
                .entry(page_id)
                .or_default()
                .push(link.into());
        }

        for heading in headings {
            let page_id = heading.page_id.clone();
            headings_by_page
                .entry(page_id)
                .or_default()
                .push(heading.into());
        }

        for image in images {
            let page_id = image.page_id.clone();
            images_by_page
                .entry(page_id)
                .or_default()
                .push(image.into());
        }

        let lighthouse_by_page: HashMap<String, LighthouseData> = lighthouse
            .into_iter()
            .map(|l| (l.page_id.clone(), l))
            .collect();

        let assembled_pages: Vec<PageAnalysisData> = pages
            .into_iter()
            .map(|p| {
                let page_id = p.id.clone();
                let page_extracted_data = p.extracted_data.clone();
                Self::assemble_page(
                    p,
                    lighthouse_by_page.get(&page_id),
                    links_by_page.remove(&page_id).unwrap_or_default(),
                    headings_by_page.remove(&page_id).unwrap_or_default(),
                    images_by_page.remove(&page_id).unwrap_or_default(),
                    page_extracted_data,
                )
            })
            .collect();

        let assembled_issues: Vec<SeoIssue> = issues
            .into_iter()
            .map(|issue| {
                let page_id = issue.page_id.clone().unwrap_or_default();
                SeoIssue {
                    page_id: page_id.clone(),
                    severity: issue.severity,
                    title: issue.issue_type,
                    description: issue.message,
                    page_url: page_url_by_id.get(&page_id).cloned().unwrap_or_default(),
                    element: issue.details.clone(),
                    recommendation: issue.details.unwrap_or_default(),
                    line_number: None,
                }
            })
            .collect();

        let analysis = AnalysisResults {
            id: job.id.clone(),
            url: job.url.clone(),
            status: job.status.clone(),
            progress: job.progress,
            total_pages: job.summary.total_pages,
            analyzed_pages: job.summary.pages_crawled,
            started_at: Some(job.created_at.to_rfc3339()),
            completed_at: job.completed_at.map(|d| d.to_rfc3339()),
            sitemap_found: job.sitemap_found,
            robots_txt_found: job.robots_txt_found,
            ssl_certificate: job.url.starts_with("https"),
            created_at: job.created_at.to_rfc3339(),
        };

        CompleteAnalysisResponse {
            summary: AnalysisSummary::compute(&job, &assembled_pages),
            analysis,
            pages: assembled_pages,
            issues: assembled_issues,
        }
    }
}

const DANGEROUS_URL_CHARS: &[char] = &['&', ';', '|', '$', '>', '<', '`', '\\', '"', '\''];

fn validate_url(url: &str) -> Result<Url> {
    let parsed = Url::parse(url).with_context(|| format!("Invalid URL format: {}", url))?;

    if url.chars().any(|c| DANGEROUS_URL_CHARS.contains(&c)) {
        anyhow::bail!("URL contains potentially dangerous characters");
    }

    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("Only http and https protocols are supported");
    }

    Ok(parsed)
}

#[tauri::command]
#[addon_guard(PermissionRequest::AnalyzePages(settings.requested_page_count()))]
#[specta::specta]
pub async fn start_analysis(
    url: String,
    settings: Option<AnalysisSettingsRequest>,
    #[provider] app_state: State<'_, AppState>,
) -> Result<AnalysisJobResponse, CommandError> {
    tracing::info!("Starting analysis: {}", url);
    tracing::info!("Settings: {:?}", settings);
    let parsed_url = validate_url(&url).map_err(CommandError::from)?;

    let analysis_settings: JobSettings = settings.unwrap_or_default().into();

    let job_id = app_state
        .analysis_context
        .create_job(parsed_url.as_str(), &analysis_settings)
        .await
        .map_err(CommandError::from)?;

    app_state.analysis_context.notify_new_job().await;
    tracing::debug!("Notified job processor of new job: {}", job_id);

    Ok(AnalysisJobResponse {
        job_id,
        url,
        status: JobStatus::Pending,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_analysis_defaults() -> Result<AnalysisSettingsRequest, CommandError> {
    Ok(AnalysisSettingsRequest::default())
}

#[tauri::command]
#[specta::specta]
pub async fn get_free_tier_defaults() -> Result<AnalysisSettingsRequest, CommandError> {
    let policy = crate::contexts::permissions::Policy::default();
    let defaults = AnalysisSettingsRequest {
        max_pages: policy.max_pages as i64,
        include_subdomains: policy.check(crate::contexts::permissions::PermissionRequest::UseFeature(
            crate::contexts::permissions::Feature::LinkAnalysis,
        )),
        ..Default::default()
    };

    Ok(defaults)
}

#[tauri::command]
#[specta::specta]
pub async fn get_analysis_progress(
    job_id: String,
    app_state: State<'_, AppState>,
) -> Result<AnalysisProgress, CommandError> {
    tracing::info!("Getting analysis progress for job: {}", job_id);

    app_state
        .analysis_context
        .get_progress(&job_id)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn get_all_jobs(
    limit: Option<i64>,
    offset: Option<i64>,
    app_state: State<'_, AppState>,
) -> Result<Vec<AnalysisProgress>, CommandError> {
    tracing::info!("Fetching jobs (limit={:?}, offset={:?})", limit, offset);

    let jobs = if let (Some(l), Some(o)) = (limit, offset) {
        app_state.analysis_context.get_paginated_jobs(l, o).await
    } else {
        app_state.analysis_context.get_all_jobs().await
    }
    .map_err(CommandError::from)?;

    Ok(jobs.into_iter().map(|j| j.into()).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn get_paginated_jobs(
    limit: i64,
    offset: i64,
    url_filter: Option<String>,
    status_filter: Option<String>,
    app_state: State<'_, AppState>,
) -> Result<PaginatedJobsResponse, CommandError> {
    tracing::info!(
        "Fetching paginated jobs (limit={}, offset={}, url_filter={:?}, status_filter={:?})",
        limit,
        offset,
        url_filter,
        status_filter
    );

    let (jobs, total) = app_state
        .analysis_context
        .get_paginated_jobs_with_total(limit, offset, url_filter, status_filter)
        .await
        .map_err(CommandError::from)?;

    Ok(PaginatedJobsResponse {
        items: jobs.into_iter().map(|j| j.into()).collect(),
        total,
    })
}

#[tauri::command]
#[specta::specta]
#[addon_guard(PermissionRequest::UseFeature(crate::contexts::permissions::Feature::LinkAnalysis))]
pub async fn cancel_analysis(
    job_id: String,
    #[provider] state: State<'_, AppState>,
) -> Result<(), CommandError> {
    tracing::trace!("Cancelling analysis job: {}", job_id);
    state
        .analysis_context
        .cancel_job(&job_id)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn get_result(
    job_id: String,
    app_state: State<'_, AppState>,
) -> Result<CompleteAnalysisResponse, CommandError> {
    tracing::trace!("Getting result for job: {}", job_id);

    app_state
        .analysis_context
        .get_complete_result(&job_id)
        .await
        .map_err(CommandError::from)
        .map(|r| r.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use specta_typescript::Typescript;

    #[test]
    fn export_bindings() {
        let mut builder = tauri_specta::Builder::<tauri::Wry>::new();
        builder = builder.commands(crate::commands::register_commands());
        builder
            .export(
                Typescript::default()
                    .formatter(specta_typescript::formatter::prettier)
                    .bigint(specta_typescript::BigIntExportBehavior::Number),
                "../src/bindings.ts",
            )
            .expect("Failed to export typescript bindings");
    }

    use crate::contexts::{JobSettings, LighthouseData, LinkType, NewLink, Page};
    use crate::repository::*;
    use crate::test_utils::fixtures;
    use chrono::Utc;

    #[tokio::test]
    async fn test_mobile_detection_and_structured_data_from_lighthouse() {
        let pool = fixtures::setup_test_db().await;

        let job_repo = sqlite_job_repo(pool.clone());
        let page_repo = sqlite_page_repo(pool.clone());

        let job_id = job_repo
            .create("https://example.com", &JobSettings::default())
            .await
            .unwrap();

        let page = Page {
            id: "".to_string(),
            job_id: job_id.clone(),
            url: "https://example.com/page-1".to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: None,
            title: Some("Page 1".to_string()),
            meta_description: None,
            canonical_url: None,
            robots_meta: None,
            word_count: Some(100),
            load_time_ms: Some(4000),
            response_size_bytes: Some(1024),
            has_viewport: false,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        };

        let page_id = page_repo.insert(&page).await.unwrap();

        let raw = r#"{"seo_audits":{"viewport":{"passed":true}},"structured_data":{}}"#.to_string();

        let lh = LighthouseData {
            page_id: page_id.clone(),
            performance_score: None,
            accessibility_score: None,
            best_practices_score: None,
            seo_score: None,
            first_contentful_paint_ms: None,
            largest_contentful_paint_ms: None,
            total_blocking_time_ms: None,
            cumulative_layout_shift: None,
            speed_index: None,
            time_to_interactive_ms: None,
            raw_json: Some(raw),
        };

        page_repo.insert_lighthouse(&lh).await.unwrap();

        let results_repo = sqlite_results_repo(pool.clone());
        let result: CompleteAnalysisResponse = results_repo
            .get_complete_result(&job_id)
            .await
            .unwrap()
            .into();

        assert_eq!(result.pages.len(), 1);
        let page = &result.pages[0];

        assert!(
            page.mobile_friendly,
            "expected mobile_friendly=true from Lighthouse viewport"
        );
        assert!(
            page.has_structured_data,
            "expected structured data detected from Lighthouse raw JSON"
        );
    }

    #[tokio::test]
    async fn test_mobile_detection_fallback_to_load_time() {
        let pool = fixtures::setup_test_db().await;

        let job_repo = sqlite_job_repo(pool.clone());
        let page_repo = sqlite_page_repo(pool.clone());

        let job_id = job_repo
            .create("https://example.com", &JobSettings::default())
            .await
            .unwrap();

        let page = Page {
            id: "".to_string(),
            job_id: job_id.clone(),
            url: "https://example.com/fast-page".to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: None,
            title: Some("Fast Page".to_string()),
            meta_description: None,
            canonical_url: None,
            robots_meta: None,
            word_count: Some(200),
            load_time_ms: Some(1000),
            response_size_bytes: Some(512),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        };

        page_repo.insert(&page).await.unwrap();

        let results_repo = sqlite_results_repo(pool.clone());
        let result: CompleteAnalysisResponse = results_repo
            .get_complete_result(&job_id)
            .await
            .unwrap()
            .into();

        assert_eq!(result.pages.len(), 1);
        let page = &result.pages[0];

        assert!(
            page.mobile_friendly,
            "expected mobile_friendly=true from load time heuristic"
        );
    }

    #[tokio::test]
    async fn test_link_classification_fallback_when_target_unparsable() {
        let pool = fixtures::setup_test_db().await;

        let job_repo = sqlite_job_repo(pool.clone());
        let page_repo = sqlite_page_repo(pool.clone());
        let link_repo = sqlite_link_repo(pool.clone());

        let job_id = job_repo
            .create("https://example.com", &JobSettings::default())
            .await
            .unwrap();

        let page = Page {
            id: "".to_string(),
            job_id: job_id.clone(),
            url: "https://example.com/page-a".to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: None,
            title: Some("A".to_string()),
            meta_description: None,
            canonical_url: None,
            robots_meta: None,
            word_count: Some(10),
            load_time_ms: Some(500),
            response_size_bytes: Some(256),
            has_viewport: false,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        };

        let page_id = page_repo.insert(&page).await.unwrap();

        let links = vec![
            NewLink {
                job_id: job_id.clone(),
                source_page_id: page_id.clone(),
                target_url: "".to_string(),
                link_text: Some("void link".to_string()),
                link_type: LinkType::Internal,
                status_code: None,
            },
            NewLink {
                job_id: job_id.clone(),
                source_page_id: page_id.clone(),
                target_url: "javascript:external:void(0)".to_string(),
                link_text: Some("void link external".to_string()),
                link_type: LinkType::External,
                status_code: None,
            },
        ];

        link_repo.insert_batch(&links).await.unwrap();

        let results_repo = sqlite_results_repo(pool.clone());
        let result: CompleteAnalysisResponse = results_repo
            .get_complete_result(&job_id)
            .await
            .unwrap()
            .into();

        assert_eq!(result.pages.len(), 1);
        let page = &result.pages[0];

        assert_eq!(page.detailed_links.len(), 2);

        let void_link = page
            .detailed_links
            .iter()
            .find(|l| l.text == "void link")
            .expect("expected void link");
        let void_link_external = page
            .detailed_links
            .iter()
            .find(|l| l.text == "void link external")
            .expect("expected void link external");

        assert_eq!(void_link.url, "", "expected empty target url");
        assert_eq!(
            void_link.link_type,
            LinkType::Internal,
            "empty target should be treated as internal when link_type is Internal"
        );

        assert_eq!(
            void_link_external.link_type,
            LinkType::External,
            "javascript:external:void(0) should be treated as external when link_type is External"
        );
    }
}

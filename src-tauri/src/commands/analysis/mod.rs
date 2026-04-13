mod assembly;
mod dto;

pub use dto::*;

use anyhow::{Context, Result};
use tauri::State;
use url::Url;

use crate::{
    contexts::{
        analysis::{AnalysisProgress, JobSettings, JobStatus},
        permissions::{Feature, PermissionRequest, Policy},
    },
    error::CommandError,
    lifecycle::app_state::AppState,
};
use addon_macros::addon_guard;

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
        job_id: job_id.into_string(),
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
    let policy = Policy::default();
    let defaults = AnalysisSettingsRequest {
        max_pages: policy.max_pages as i64,
        include_subdomains: policy.check(PermissionRequest::UseFeature(Feature::LinkAnalysis)),
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
    tracing::debug!(
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
#[addon_guard(PermissionRequest::UseFeature(Feature::LinkAnalysis))]
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

    use crate::contexts::analysis::{Depth, JobSettings, LighthouseData, LinkType, NewLink, Page};
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
            depth: Depth::root(),
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
            depth: Depth::root(),
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
            depth: Depth::root(),
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

    #[tokio::test]
    async fn summary_seo_score_uses_lighthouse_when_available() {
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
            url: "https://example.com/".to_string(),
            depth: Depth::root(),
            status_code: Some(200),
            content_type: None,
            title: Some("Home".to_string()),
            meta_description: None,
            canonical_url: None,
            robots_meta: None,
            word_count: Some(100),
            load_time_ms: Some(500),
            response_size_bytes: Some(1024),
            has_viewport: true,
            has_structured_data: false,
            crawled_at: Utc::now(),
            extracted_data: std::collections::HashMap::new(),
        };

        let page_id = page_repo.insert(&page).await.unwrap();

        let lh = LighthouseData {
            page_id: page_id.clone(),
            performance_score: None,
            accessibility_score: None,
            best_practices_score: None,
            seo_score: Some(89.0),
            first_contentful_paint_ms: None,
            largest_contentful_paint_ms: None,
            total_blocking_time_ms: None,
            cumulative_layout_shift: None,
            speed_index: None,
            time_to_interactive_ms: None,
            raw_json: None,
        };
        page_repo.insert_lighthouse(&lh).await.unwrap();

        let results_repo = sqlite_results_repo(pool.clone());
        let result: CompleteAnalysisResponse = results_repo
            .get_complete_result(&job_id)
            .await
            .unwrap()
            .into();

        assert_eq!(
            result.summary.seo_score, 89,
            "summary.seo_score should equal the Lighthouse SEO score (89), not the issue-deduction score"
        );
    }

    #[tokio::test]
    async fn summary_seo_score_falls_back_to_issue_formula_without_lighthouse() {
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
            url: "https://example.com/".to_string(),
            depth: Depth::root(),
            status_code: Some(200),
            content_type: None,
            title: Some("Home".to_string()),
            meta_description: None,
            canonical_url: None,
            robots_meta: None,
            word_count: Some(100),
            load_time_ms: Some(500),
            response_size_bytes: Some(1024),
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

        assert_eq!(
            result.summary.seo_score, 100,
            "without Lighthouse data and no issues, seo_score should be 100 (issue-formula fallback)"
        );
    }
}

use crate::domain::models::{
    IssueSeverity, JobSettings, LighthouseData, NewHeading, NewImage, NewIssue, Page,
};
use crate::extractor::page_extractor::{ExtractedHeading, ExtractedImage, PageExtractor};
use crate::repository::sqlite::{IssueRepository, PageRepository};
use crate::service::auditor::{Auditor, DeepAuditor, LightAuditor};
use anyhow::Result;
use scraper::Html;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct AnalyzerService {
    page_db: PageRepository,
    issue_db: IssueRepository,
    light_auditor: Arc<LightAuditor>,
    deep_auditor: Arc<DeepAuditor>,
}

#[derive(Debug, Clone)]
pub struct PageEdge {
    pub from_page_id: String,
    pub to_url: String,
    pub status_code: i32,
    pub link_text: Option<String>,
}

impl PageEdge {
    pub fn new(
        from_page_id: &str,
        to_url: &str,
        status_code: i32,
        link_text: Option<String>,
    ) -> Self {
        Self {
            from_page_id: from_page_id.to_string(),
            to_url: to_url.to_string(),
            status_code,
            link_text,
        }
    }
}

pub struct PageResult {
    pub issues: Vec<NewIssue>,
    pub edges: Vec<PageEdge>,
}

impl AnalyzerService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            page_db: PageRepository::new(pool.clone()),
            issue_db: IssueRepository::new(pool),
            light_auditor: Arc::new(LightAuditor::new()),
            deep_auditor: Arc::new(DeepAuditor::new()),
        }
    }

    pub fn select_auditor(&self, settings: &JobSettings) -> Arc<dyn Auditor + Send + Sync> {
        if settings.lighthouse_analysis {
            if self.deep_auditor.is_available() {
                self.deep_auditor.clone()
            } else {
                log::warn!("[JOB] Deep auditor unavailable, falling back to light auditor");
                self.light_auditor.clone()
            }
        } else {
            self.light_auditor.clone()
        }
    }

    pub async fn analyze_page(
        &self,
        url: &str,
        job_id: &str,
        depth: i64,
        auditor: &Arc<dyn Auditor + Send + Sync>,
    ) -> Result<(PageResult, Vec<String>)> {
        // Fetch and analyze page using the auditor
        let audit_result = auditor.analyze(url).await?;

        // Parse HTML and extract all data BEFORE any awaits
        let (page, extracted_issues, new_urls, edges, headings, images) = {
            let html = Html::parse_document(&audit_result.html);

            // Extract basic page data
            let title = PageExtractor::extract_title(&html);
            let meta_description = PageExtractor::extract_meta_description(&html);
            let canonical_url = PageExtractor::extract_canonical(&html);
            let word_count = PageExtractor::extract_word_count(&html);

            // Extract links
            let (internal_links, _external_links, all_links) =
                PageExtractor::extract_links(&html, url);

            // Extract headings and images
            let headings: Vec<ExtractedHeading> = PageExtractor::extract_headings(&html);
            let images: Vec<ExtractedImage> = PageExtractor::extract_images(&html, url);

            // Create Page
            let page = Page {
                id: uuid::Uuid::new_v4().to_string(),
                job_id: job_id.to_string(),
                url: url.to_string(),
                depth,
                status_code: Some(audit_result.status_code as i64),
                content_type: None,
                title: title.clone(),
                meta_description: meta_description.clone(),
                canonical_url,
                robots_meta: None,
                word_count: Some(word_count),
                load_time_ms: Some(audit_result.load_time_ms as i64),
                response_size_bytes: Some(audit_result.content_size as i64),
                crawled_at: chrono::Utc::now(),
            };

            // Generate SEO issues
            let mut issues: Vec<(String, String, String, IssueSeverity)> = Vec::new();

            if title.is_none() || title.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                issues.push((
                    "Missing Title".to_string(),
                    "Page has no title tag".to_string(),
                    "Add a descriptive title tag".to_string(),
                    IssueSeverity::Critical,
                ));
            }

            if meta_description.is_none()
                || meta_description
                    .as_ref()
                    .map(|d| d.is_empty())
                    .unwrap_or(true)
            {
                issues.push((
                    "Missing Meta Description".to_string(),
                    "Page has no meta description".to_string(),
                    "Add a meta description".to_string(),
                    IssueSeverity::Warning,
                ));
            }

            if audit_result.status_code >= 400 {
                issues.push((
                    "HTTP Error".to_string(),
                    format!("Page returned status code {}", audit_result.status_code),
                    "Fix the HTTP error".to_string(),
                    IssueSeverity::Critical,
                ));
            }

            // Build edges for link tracking (include link text as tuple - page_id is not available yet)
            let edges: Vec<(String, i32, Option<String>)> = all_links
                .into_iter()
                .map(|link| {
                    (
                        link.href,
                        if link.is_internal { 200i32 } else { 0i32 },
                        link.text,
                    )
                })
                .collect();

            (page, issues, internal_links, edges, headings, images)
        };

        // Insert page
        let page_id = self.page_db.insert(&page).await?;

        // Store Lighthouse data (scores + audits + perf metrics)
        let scores = &audit_result.scores;
        let raw_json = serde_json::json!({
            "seo_audits": scores.seo_details.clone(),
            "performance_metrics": scores.performance_metrics.clone(),
        });
        let raw_json = serde_json::to_string(&raw_json).ok();

        let normalize_score = |score: Option<crate::service::auditor::Score>| -> Option<f64> {
            score.map(|s| s.percent())
        };

        let lighthouse = LighthouseData {
            page_id: page_id.clone(),
            performance_score: normalize_score(scores.performance),
            accessibility_score: normalize_score(scores.accessibility),
            best_practices_score: normalize_score(scores.best_practices),
            seo_score: normalize_score(scores.seo),
            // ... map other fields
            first_contentful_paint_ms: audit_result
                .scores
                .performance_metrics
                .as_ref()
                .and_then(|m| m.first_contentful_paint),
            largest_contentful_paint_ms: audit_result
                .scores
                .performance_metrics
                .as_ref()
                .and_then(|m| m.largest_contentful_paint),
            total_blocking_time_ms: audit_result
                .scores
                .performance_metrics
                .as_ref()
                .and_then(|m| m.total_blocking_time),
            cumulative_layout_shift: audit_result
                .scores
                .performance_metrics
                .as_ref()
                .and_then(|m| m.cumulative_layout_shift),
            speed_index: audit_result
                .scores
                .performance_metrics
                .as_ref()
                .and_then(|m| m.speed_index),
            time_to_interactive_ms: audit_result
                .scores
                .performance_metrics
                .as_ref()
                .and_then(|m| m.time_to_interactive),
            raw_json,
        };

        if let Err(e) = self.page_db.insert_lighthouse(&lighthouse).await {
            log::warn!("Failed to store Lighthouse data for {}: {}", url, e);
        }

        // Store headings and images
        let heading_rows: Vec<NewHeading> = headings
            .into_iter()
            .map(|h| NewHeading {
                page_id: page_id.clone(),
                level: h.level,
                text: h.text,
                position: h.position,
            })
            .collect();

        if let Err(e) = self.page_db.replace_headings(&page_id, &heading_rows).await {
            log::warn!("Failed to store headings for {}: {}", url, e);
        }

        let image_rows: Vec<NewImage> = images
            .into_iter()
            .map(|img| NewImage {
                page_id: page_id.clone(),
                src: img.src,
                alt: img.alt,
                width: img.width,
                height: img.height,
                loading: img.loading,
                is_decorative: img.is_decorative,
            })
            .collect();

        if let Err(e) = self.page_db.replace_images(&page_id, &image_rows).await {
            log::warn!("Failed to store images for {}: {}", url, e);
        }

        // Convert and insert issues with the actual page_id
        let issues: Vec<NewIssue> = extracted_issues
            .into_iter()
            .map(|(title, description, recommendation, severity)| NewIssue {
                job_id: job_id.to_string(),
                page_id: Some(page_id.clone()),
                issue_type: title,
                severity,
                message: description,
                details: Some(recommendation),
            })
            .collect();

        if !issues.is_empty() {
            self.issue_db.insert_batch(&issues).await?;
        }

        // Build final edges with page_id
        let final_edges: Vec<PageEdge> = edges
            .into_iter()
            .map(|(href, status_code, text)| PageEdge::new(&page_id, &href, status_code, text))
            .collect();

        Ok((
            PageResult {
                issues,
                edges: final_edges,
            },
            new_urls,
        ))
    }
}

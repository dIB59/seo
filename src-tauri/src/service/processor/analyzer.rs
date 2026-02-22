use crate::domain::{JobSettings, LighthouseData, NewHeading, NewImage, NewIssue, NewLink, Page};
use crate::extractor::page_extractor::{ExtractedHeading, ExtractedImage, PageExtractor};
use crate::repository::{IssueRepository as IssueRepoTrait, PageRepository as PageRepoTrait};
use crate::service::auditor::{Auditor, DeepAuditor, LightAuditor};
use crate::service::spider::SpiderAgent;
use anyhow::Result;
use scraper::Html;
use std::sync::Arc;

pub struct AnalyzerService {
    page_db: Arc<dyn PageRepoTrait>,
    issue_db: Arc<dyn IssueRepoTrait>,
    light_auditor: Arc<LightAuditor>,
    deep_auditor: Arc<DeepAuditor>,
}

// No longer leaking PageEdge, we use NewLink from domain models
pub struct PageResult {
    pub issues: Vec<NewIssue>,
    pub links: Vec<NewLink>,
}

impl AnalyzerService {
    pub fn new(
        page_db: Arc<dyn PageRepoTrait>,
        issue_db: Arc<dyn IssueRepoTrait>,
        deep_spider: Arc<dyn SpiderAgent>,
    ) -> Self {
        Self {
            page_db,
            issue_db,
            light_auditor: Arc::new(LightAuditor::new(deep_spider.clone())),
            deep_auditor: Arc::new(DeepAuditor::new(deep_spider.clone())),
        }
    }

    pub fn select_auditor(&self, settings: &JobSettings) -> Arc<dyn Auditor + Send + Sync> {
        if settings.lighthouse_analysis {
            if self.deep_auditor.is_available() {
                self.deep_auditor.clone()
            } else {
                tracing::warn!("[JOB] Deep auditor unavailable, falling back to light auditor");
                self.light_auditor.clone()
            }
        } else {
            self.light_auditor.clone()
        }
    }

    pub fn deep_auditor(&self) -> Arc<DeepAuditor> {
        self.deep_auditor.clone()
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.light_auditor.shutdown().await?;
        self.deep_auditor.shutdown().await?;
        Ok(())
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
        let final_url = audit_result.url.clone();

        if final_url != url {
            tracing::info!(
                "[ANALYZER] Target redirected during analysis: {} -> {}",
                url,
                final_url
            );
        }

        // Parse HTML and extract all data BEFORE any awaits
        let (page, new_urls, edges, headings, images) = {
            let html = Html::parse_document(&audit_result.html);

            // Extract basic page data
            let title = PageExtractor::extract_title(&html);
            let meta_description = PageExtractor::extract_meta_description(&html);
            let canonical_url = PageExtractor::extract_canonical(&html);
            let word_count = PageExtractor::extract_word_count(&html);

            // Extract SEO flags
            let has_viewport = PageExtractor::extract_has_viewport(&html);
            let has_structured_data = PageExtractor::extract_has_structured_data(&html);

            // Extract links using final url as base
            let (internal_links, _external_links, all_links) =
                PageExtractor::extract_links(&html, &final_url);

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
                has_viewport,
                has_structured_data,
                crawled_at: chrono::Utc::now(),
            };

            // Build edges for link tracking (include link text as tuple - page_id is not available yet)
            let edges: Vec<(String, i32, Option<String>)> = all_links
                .into_iter()
                .map(|link| {
                    (
                        link.href,
                        if matches!(
                            link.link_type,
                            crate::domain::LinkType::Internal | crate::domain::LinkType::Subdomain
                        ) {
                            200i32
                        } else {
                            0i32
                        },
                        link.text,
                    )
                })
                .collect();

            (page, internal_links, edges, headings, images)
        };

        // Insert page
        let page_id = self.page_db.insert(&page).await?;

        // Generate and insert issues using the rich domain model
        let issues = page.audit();
        if !issues.is_empty() {
            self.issue_db.insert_batch(&issues).await?;
        }

        // Build Lighthouse data using the domain factory
        let lighthouse = LighthouseData::from_audit_scores(&page_id, &audit_result.scores);

        if let Err(e) = self.page_db.insert_lighthouse(&lighthouse).await {
            tracing::warn!("Failed to store Lighthouse data for {}: {}", url, e);
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
            tracing::warn!("Failed to store headings for {}: {}", url, e);
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
            tracing::warn!("Failed to store images for {}: {}", url, e);
        }

        // Build final links with page_id and job_id
        // We Use NewLink::create to handle internal/external logic
        let analysis_links: Vec<NewLink> = edges
            .into_iter()
            .map(|(href, status_code, text)| {
                NewLink::create(
                    job_id,
                    &page_id,
                    &href,
                    text,
                    Some(status_code as i64),
                    &final_url,
                )
            })
            .collect();

        Ok((
            PageResult {
                issues,
                links: analysis_links,
            },
            new_urls,
        ))
    }
}

use crate::checker::{CheckContext, CheckerRegistry};
use crate::checker::custom::CustomCheckAdapter;
use crate::contexts::extension::CustomCheck;
use crate::contexts::analysis::{
    JobSettings, LighthouseData, LinkType, NewHeading, NewImage, NewIssue, NewLink, Page,
};
use crate::extractor::data_extractor::ExtractorRegistry;
use crate::extractor::page_extractor::{ExtractedHeading, ExtractedImage, PageExtractor};
use crate::repository::{IssueRepository as IssueRepoTrait, PageRepository as PageRepoTrait};
use crate::service::auditor::{Auditor, AuditResult, DeepAuditor, LightAuditor};
use crate::service::spider::SpiderAgent;
use anyhow::Result;
use scraper::Html;
use std::sync::Arc;

pub struct AnalyzerService {
    page_db: Arc<dyn PageRepoTrait>,
    issue_db: Arc<dyn IssueRepoTrait>,
    light_auditor: Arc<LightAuditor>,
    deep_auditor: Arc<DeepAuditor>,
    checker_registry: Arc<CheckerRegistry>,
    extractor_registry: Arc<ExtractorRegistry>,
}

pub struct PageResult {
    pub issues: Vec<NewIssue>,
    pub links: Vec<NewLink>,
}

struct ExtractedLinkEdge {
    href: String,
    initial_status: i32,
    anchor_text: Option<String>,
}

struct ExtractedPageData {
    page: Page,
    internal_urls: Vec<String>,
    link_edges: Vec<ExtractedLinkEdge>,
    headings: Vec<ExtractedHeading>,
    images: Vec<ExtractedImage>,
    final_url: String,
}

/// Data to persist for a page after extraction.
struct PagePersistData<'a> {
    page_id: &'a str,
    url: &'a str,
    issues: &'a [NewIssue],
    lighthouse: &'a LighthouseData,
    headings: &'a [NewHeading],
    images: &'a [NewImage],
}

fn extract_page_data(
    html: &str,
    url: &str,
    job_id: &str,
    depth: crate::contexts::analysis::Depth,
    audit_result: &AuditResult,
    extracted_data: std::collections::HashMap<String, serde_json::Value>,
) -> ExtractedPageData {
    let parsed_html = Html::parse_document(html);

    let title = PageExtractor::extract_title(&parsed_html);
    let meta_description = PageExtractor::extract_meta_description(&parsed_html);
    let canonical_url = PageExtractor::extract_canonical(&parsed_html);
    let word_count = PageExtractor::extract_word_count(&parsed_html);
    let has_viewport = PageExtractor::extract_has_viewport(&parsed_html);
    let has_structured_data = PageExtractor::extract_has_structured_data(&parsed_html);

    let (internal_urls, _external_urls, all_links) =
        PageExtractor::extract_links(&parsed_html, &audit_result.url);

    let headings = PageExtractor::extract_headings(&parsed_html);
    let images = PageExtractor::extract_images(&parsed_html, url);

    let page = Page {
        id: uuid::Uuid::new_v4().to_string(),
        job_id: job_id.to_string(),
        url: url.to_string(),
        depth,
        status_code: Some(audit_result.status_code as i64),
        content_type: None,
        title,
        meta_description,
        canonical_url,
        robots_meta: None,
        word_count: Some(word_count),
        load_time_ms: Some(audit_result.load_time_ms as i64),
        response_size_bytes: Some(audit_result.content_size as i64),
        has_viewport,
        has_structured_data,
        crawled_at: chrono::Utc::now(),
        extracted_data,
    };

    let link_edges: Vec<ExtractedLinkEdge> = all_links
        .into_iter()
        .map(|link| {
            ExtractedLinkEdge {
                href: link.href,
                initial_status: if matches!(link.link_type, LinkType::Internal | LinkType::Subdomain)
                {
                    200i32
                } else {
                    0i32
                },
                anchor_text: link.text,
            }
        })
        .collect();

    ExtractedPageData {
        page,
        internal_urls,
        link_edges,
        headings,
        images,
        final_url: audit_result.url.clone(),
    }
}

async fn persist_page_data(
    page_db: &dyn PageRepoTrait,
    issue_db: &dyn IssueRepoTrait,
    data: &PagePersistData<'_>,
) -> Result<()> {
    if !data.issues.is_empty() {
        issue_db.insert_batch(data.issues).await?;
    }

    if let Err(e) = page_db.insert_lighthouse(data.lighthouse).await {
        tracing::warn!("Failed to store Lighthouse data for {}: {}", data.url, e);
    }

    if let Err(e) = page_db.replace_headings(data.page_id, data.headings).await {
        tracing::warn!("Failed to store headings for {}: {}", data.url, e);
    }

    if let Err(e) = page_db.replace_images(data.page_id, data.images).await {
        tracing::warn!("Failed to store images for {}: {}", data.url, e);
    }

    Ok(())
}

impl AnalyzerService {
    pub fn new(
        page_db: Arc<dyn PageRepoTrait>,
        issue_db: Arc<dyn IssueRepoTrait>,
        deep_spider: Arc<dyn SpiderAgent>,
        extractor_registry: Arc<ExtractorRegistry>,
    ) -> Self {
        Self::with_custom_checks(page_db, issue_db, deep_spider, extractor_registry, vec![])
    }

    /// Build an analyzer with user-defined custom checks loaded from
    /// the extension repository. Each `CustomCheck` is wrapped in a
    /// [`CustomCheckAdapter`] and registered alongside the built-ins,
    /// so they fire during `analyze_page` the same way as any built-in
    /// check — with `{tag.X}` substitution in the issue message.
    pub fn with_custom_checks(
        page_db: Arc<dyn PageRepoTrait>,
        issue_db: Arc<dyn IssueRepoTrait>,
        deep_spider: Arc<dyn SpiderAgent>,
        extractor_registry: Arc<ExtractorRegistry>,
        custom_checks: Vec<CustomCheck>,
    ) -> Self {
        let mut checker_registry = CheckerRegistry::with_defaults();
        for check in custom_checks {
            if check.enabled {
                checker_registry.register(Box::new(CustomCheckAdapter::new(check)));
            }
        }
        Self {
            page_db,
            issue_db,
            light_auditor: Arc::new(LightAuditor::new(deep_spider.clone())),
            deep_auditor: Arc::new(DeepAuditor::new(deep_spider.clone())),
            checker_registry: Arc::new(checker_registry),
            extractor_registry,
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

    /// Analyze a page using cached HTML from the discovery phase.
    /// Identical to `analyze_page` but skips the HTTP fetch.
    pub async fn analyze_page_cached(
        &self,
        url: &str,
        job_id: &str,
        depth: crate::contexts::analysis::Depth,
        auditor: &Arc<dyn Auditor + Send + Sync>,
        cached: crate::service::auditor::CachedHtml,
    ) -> Result<(PageResult, Vec<String>)> {
        let audit_result = auditor.analyze_from_cache(url, cached).await?;
        self.process_audit_result(url, job_id, depth, audit_result).await
    }

    pub async fn analyze_page(
        &self,
        url: &str,
        job_id: &str,
        depth: crate::contexts::analysis::Depth,
        auditor: &Arc<dyn Auditor + Send + Sync>,
    ) -> Result<(PageResult, Vec<String>)> {
        let audit_result = auditor.analyze(url).await?;
        self.process_audit_result(url, job_id, depth, audit_result).await
    }

    /// Shared processing for both cached and non-cached paths.
    async fn process_audit_result(
        &self,
        url: &str,
        job_id: &str,
        depth: crate::contexts::analysis::Depth,
        audit_result: crate::service::auditor::AuditResult,
    ) -> Result<(PageResult, Vec<String>)> {

        if audit_result.url != url {
            tracing::info!(
                "[ANALYZER] Target redirected during analysis: {} -> {}",
                url,
                audit_result.url
            );
        }

        let custom_data = self.extractor_registry.run(&audit_result.html);
        let extracted = extract_page_data(&audit_result.html, url, job_id, depth, &audit_result, custom_data);

        let page_id = self.page_db.insert(&extracted.page).await?;

        let check_ctx = CheckContext::new(
            &extracted.page,
            &audit_result.scores.seo_details,
            job_id,
            &page_id,
        );
        let issues = self.checker_registry.run(&check_ctx);
        let lighthouse = LighthouseData::from_audit_scores(&page_id, &audit_result.scores);

        let heading_rows: Vec<NewHeading> = extracted
            .headings
            .into_iter()
            .map(|h| NewHeading {
                page_id: page_id.clone(),
                level: h.level,
                text: h.text,
                position: h.position,
            })
            .collect();

        let image_rows: Vec<NewImage> = extracted
            .images
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

        persist_page_data(
            &*self.page_db,
            &*self.issue_db,
            &PagePersistData {
                page_id: &page_id,
                url,
                issues: &issues,
                lighthouse: &lighthouse,
                headings: &heading_rows,
                images: &image_rows,
            },
        )
        .await?;

        let analysis_links: Vec<NewLink> = extracted
            .link_edges
            .into_iter()
            .map(|edge| {
                NewLink::create(
                    job_id,
                    &page_id,
                    &edge.href,
                    edge.anchor_text,
                    Some(edge.initial_status as i64),
                    &extracted.final_url,
                )
            })
            .collect();

        Ok((
            PageResult {
                issues,
                links: analysis_links,
            },
            extracted.internal_urls,
        ))
    }
}


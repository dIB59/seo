use crate::contexts::{JobSettings, LighthouseData, NewHeading, NewImage, NewIssue, NewLink, Page};
use crate::extension::{EvaluationContext, ExtensionRegistry};
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
    extension_registry: Option<Arc<ExtensionRegistry>>,
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
    depth: i64,
    audit_result: &AuditResult,
) -> ExtractedPageData {
    let parsed_html = Html::parse_document(html);
    let final_url = audit_result.url.clone();

    let title = PageExtractor::extract_title(&parsed_html);
    let meta_description = PageExtractor::extract_meta_description(&parsed_html);
    let canonical_url = PageExtractor::extract_canonical(&parsed_html);
    let word_count = PageExtractor::extract_word_count(&parsed_html);
    let has_viewport = PageExtractor::extract_has_viewport(&parsed_html);
    let has_structured_data = PageExtractor::extract_has_structured_data(&parsed_html);

    let (internal_urls, _external_urls, all_links) =
        PageExtractor::extract_links(&parsed_html, &final_url);

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
    };

    let link_edges: Vec<ExtractedLinkEdge> = all_links
        .into_iter()
        .map(|link| {
            use crate::contexts::LinkType;
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
        final_url,
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
    ) -> Self {
        Self {
            page_db,
            issue_db,
            light_auditor: Arc::new(LightAuditor::new(deep_spider.clone())),
            deep_auditor: Arc::new(DeepAuditor::new(deep_spider.clone())),
            extension_registry: None,
        }
    }

    /// Create an AnalyzerService with an extension registry for dynamic rules
    pub fn with_extensions(
        page_db: Arc<dyn PageRepoTrait>,
        issue_db: Arc<dyn IssueRepoTrait>,
        deep_spider: Arc<dyn SpiderAgent>,
        extension_registry: Arc<ExtensionRegistry>,
    ) -> Self {
        Self {
            page_db,
            issue_db,
            light_auditor: Arc::new(LightAuditor::new(deep_spider.clone())),
            deep_auditor: Arc::new(DeepAuditor::new(deep_spider.clone())),
            extension_registry: Some(extension_registry),
        }
    }

    /// Set or update the extension registry
    pub fn set_extension_registry(&mut self, registry: Arc<ExtensionRegistry>) {
        self.extension_registry = Some(registry);
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
        let audit_result = auditor.analyze(url).await?;

        if audit_result.url != url {
            tracing::info!(
                "[ANALYZER] Target redirected during analysis: {} -> {}",
                url,
                audit_result.url
            );
        }

        let extracted = extract_page_data(&audit_result.html, url, job_id, depth, &audit_result);

        let page_id = self.page_db.insert(&extracted.page).await?;

        // Generate issues using extension registry if available, otherwise fall back to built-in audit
        let issues = if let Some(registry) = &self.extension_registry {
            let context = EvaluationContext::new()
                .with_html(audit_result.html.clone());
            registry.evaluate_rules(&extracted.page, &context)
        } else {
            // Fallback to built-in audit for backward compatibility
            extracted.page.audit()
        };
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

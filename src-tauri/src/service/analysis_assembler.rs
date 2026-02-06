//! AnalysisAssembler - orchestrates repository calls and builds frontend-ready responses.

use std::collections::HashMap;

use anyhow::Result;
use sqlx::SqlitePool;

use crate::domain::models::{
    AnalysisResults, AnalysisSummary, CompleteAnalysisResult, HeadingElement, ImageElement,
    LighthouseData, LinkDetail, LinkType, PageAnalysisData, SeoIssue,
};
use url::Url;

// Heuristic thresholds and defaults for assembly decisions
const MOBILE_FRIENDLY_LOAD_TIME_MS: i64 = 2000; // Pages with load time <= 2s considered mobile friendly (heuristic)
const DEFAULT_SITEMAP_FOUND: bool = false; // Unknown if sitemap exists; keep explicit default
const DEFAULT_ROBOTS_TXT_FOUND: bool = false; // Unknown if robots.txt found; explicit default

use crate::repository::sqlite::ResultsRepository;

pub struct AnalysisAssembler {
    repo: ResultsRepository,
}

impl AnalysisAssembler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { repo: ResultsRepository::new(pool) }
    }

    /// Build a CompleteAnalysisResult for the given job id.
    pub async fn assemble(&self, job_id: &str) -> Result<CompleteAnalysisResult> {
        // Reuse repository getters for fetching raw data
        let job = self.repo.get_job(job_id).await?;
        let pages = self.repo.get_pages(job_id).await?;
        let issues = self.repo.get_issues(job_id).await?;
        let links = self.repo.get_links(job_id).await?;
        let lighthouse = self.repo.get_lighthouse(job_id).await?;
        let headings = self.repo.get_headings(job_id).await?;
        let images = self.repo.get_images(job_id).await?;

        // Map pages and build page-level structures
        let page_url_by_id: std::collections::HashMap<String, String> = pages
            .iter()
            .map(|p| (p.id.clone(), p.url.clone()))
            .collect();

        let mut detailed_links_by_page: HashMap<String, Vec<LinkDetail>> =
            std::collections::HashMap::new();
        for link in &links {
            let entry = detailed_links_by_page
                .entry(link.source_page_id.clone())
                .or_default();

            let source_url = page_url_by_id.get(&link.source_page_id);
            let is_external = is_external_by_url(source_url, &link.target_url, &link.link_type);

            entry.push(LinkDetail {
                url: link.target_url.clone(),
                text: link.link_text.clone().unwrap_or_default(),
                is_external,
                is_broken: link.status_code.map(|c| c >= 400).unwrap_or(false),
                status_code: link.status_code,
            });
        }

        let headings_by_page: HashMap<String, Vec<HeadingElement>> = headings
            .into_iter()
            .fold(std::collections::HashMap::new(), |mut acc, heading| {
                let tag = format!("h{}", heading.level);
                acc.entry(heading.page_id)
                    .or_insert_with(Vec::new)
                    .push(HeadingElement { tag, text: heading.text });
                acc
            });

        let images_by_page: HashMap<String, Vec<ImageElement>> = images
            .into_iter()
            .fold(std::collections::HashMap::new(), |mut acc, image| {
                acc.entry(image.page_id)
                    .or_insert_with(Vec::new)
                    .push(ImageElement { src: image.src, alt: image.alt });
                acc
            });

        let lighthouse_by_page: HashMap<String, LighthouseData> = lighthouse
            .into_iter()
            .map(|l| (l.page_id.clone(), l))
            .collect();

        let pages: Vec<PageAnalysisData> = pages
            .into_iter()
            .map(|p| {
                let page_id = p.id.clone();
                // Pre-compute derived values and build `PageAnalysisData` once at the end to avoid
                // mutating a partially-constructed struct and to make the logic more explicit.
                let load_time = p.load_time_ms.unwrap_or(0) as f64 / 1000.0;
                let status_code = p.status_code;
                let content_size = p.response_size_bytes.unwrap_or(0);

                // Lighthouse-derived values
                let mut lighthouse_performance = None;
                let mut lighthouse_accessibility = None;
                let mut lighthouse_best_practices = None;
                let mut lighthouse_seo = None;
                let mut lighthouse_seo_audits = None;
                let mut lighthouse_performance_metrics = None;
                let mut mobile_friendly = false;
                let mut has_structured_data = false;

                if let Some(lh) = lighthouse_by_page.get(&page_id) {
                    lighthouse_performance = lh.performance_score;
                    lighthouse_accessibility = lh.accessibility_score;
                    lighthouse_best_practices = lh.best_practices_score;
                    lighthouse_seo = lh.seo_score;

                    if let Some(raw) = lh.raw_json.as_deref() {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
                            lighthouse_seo_audits = value.get("seo_audits").cloned();
                            lighthouse_performance_metrics = value.get("performance_metrics").cloned();

                            if let Some(viewport) = value.get("seo_audits").and_then(|s| s.get("viewport")) {
                                if let Some(passed) = viewport.get("passed").and_then(|p| p.as_bool()) {
                                    mobile_friendly = passed;
                                }
                            }

                            if value.get("structured_data").is_some() {
                                has_structured_data = true;
                            }
                        }
                    }
                }

                // Links
                let (detailed_links, links_vec, internal_links, external_links) = if let Some(links) = detailed_links_by_page.get(&page_id) {
                    let detailed = links.clone();
                    let links_v = detailed.iter().map(|l| l.url.clone()).collect();
                    let internal = detailed.iter().filter(|l| !l.is_external).count() as i64;
                    let external = detailed.iter().filter(|l| l.is_external).count() as i64;
                    (detailed, links_v, internal, external)
                } else {
                    (vec![], vec![], 0i64, 0i64)
                };

                // Headings
                let (h1_count, h2_count, h3_count, headings_vec) = if let Some(headings) = headings_by_page.get(&page_id) {
                    let h1 = headings.iter().filter(|h| h.tag == "h1").count() as i64;
                    let h2 = headings.iter().filter(|h| h.tag == "h2").count() as i64;
                    let h3 = headings.iter().filter(|h| h.tag == "h3").count() as i64;
                    (h1, h2, h3, headings.clone())
                } else {
                    (0i64, 0i64, 0i64, vec![])
                };

                // Images
                let (image_count, images_without_alt, images_vec) = if let Some(images) = images_by_page.get(&page_id) {
                    let img_count = images.len() as i64;
                    let without_alt = images
                        .iter()
                        .filter(|img| img.alt.as_deref().unwrap_or("").is_empty())
                        .count() as i64;
                    (img_count, without_alt, images.clone())
                } else {
                    (0i64, 0i64, vec![])
                };

                // Fallback mobile-friendly heuristic
                if !mobile_friendly {
                    mobile_friendly = load_time <= (MOBILE_FRIENDLY_LOAD_TIME_MS as f64) / 1000.0;
                }

                PageAnalysisData {
                    analysis_id: p.job_id,
                    url: p.url,
                    title: p.title,
                    meta_description: p.meta_description,
                    meta_keywords: None,
                    canonical_url: p.canonical_url,
                    h1_count,
                    h2_count,
                    h3_count,
                    word_count: p.word_count.unwrap_or(0),
                    image_count,
                    images_without_alt,
                    internal_links,
                    external_links,
                    load_time,
                    status_code,
                    content_size,
                    mobile_friendly,
                    has_structured_data,

                    lighthouse_performance,
                    lighthouse_accessibility,
                    lighthouse_best_practices,
                    lighthouse_seo,
                    lighthouse_seo_audits,
                    lighthouse_performance_metrics,
                    links: links_vec,
                    headings: headings_vec,
                    images: images_vec,
                    detailed_links,
                }
            })
            .collect();

        let issues: Vec<SeoIssue> = issues
            .into_iter()
            .map(|issue| {
                let severity = issue.severity.clone();
                let page_id = issue.page_id.clone().unwrap_or_default();
                let page_url = page_url_by_id.get(&page_id).cloned().unwrap_or_default();

                SeoIssue {
                    page_id,
                    severity,
                    title: issue.issue_type,
                    description: issue.message,
                    page_url,
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
            started_at: Some(job.created_at),
            completed_at: job.completed_at,
            sitemap_found: DEFAULT_SITEMAP_FOUND,
            robots_txt_found: DEFAULT_ROBOTS_TXT_FOUND,
            ssl_certificate: job.url.starts_with("https"),
            created_at: job.created_at,
        };



        fn is_external_by_url(source_url: Option<&String>, target_url: &str, link_type: &LinkType) -> bool {
            let source_url = match source_url {
                Some(url) => url,
                None => return !matches!(link_type, LinkType::Internal),
            };

            let source = Url::parse(source_url).ok();
            let target = Url::parse(target_url).ok();

            if let (Some(source), Some(target)) = (source, target) {
                let same_host = source.host_str() == target.host_str();
                let same_port = source.port() == target.port();
                return !(same_host && same_port);
            }

            !matches!(link_type, LinkType::Internal)
        }

        fn calculate_seo_score(job: &crate::domain::models::Job) -> i64 {
            let total = job.summary.total_issues;
            let critical = job.summary.critical_issues;
            let warning = job.summary.warning_issues;

            if total == 0 {
                return 100;
            }

            let deductions = (critical * 10) + (warning * 5) + (total - critical - warning);
            let score = 100 - deductions;

            score.clamp(0, 100)
        }

        // Compute summary metrics from page data
        let (total_load, load_count) = pages
            .iter()
            .fold((0.0f64, 0usize), |(sum, cnt), p| {
                if p.load_time > 0.0 {
                    (sum + p.load_time, cnt + 1)
                } else {
                    (sum, cnt)
                }
            });

        let avg_load_time = if load_count > 0 { total_load / load_count as f64 } else { 0.0 };

        let summary = AnalysisSummary {
            analysis_id: job.id.clone(),
            seo_score: calculate_seo_score(&job),
            avg_load_time,
            total_words: pages.iter().map(|p| p.word_count).sum(),
            total_issues: job.summary.total_issues,
        };

        Ok(CompleteAnalysisResult {
            analysis,
            pages,
            issues,
            summary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::repository::sqlite::{JobRepository, PageRepository, LinkRepository};
    use crate::domain::models::{Page, LighthouseData, NewLink, LinkType, JobSettings};
    use crate::test_utils::fixtures;

    #[tokio::test]
    async fn test_mobile_detection_and_structured_data_from_lighthouse() {
        let pool = fixtures::setup_test_db().await;

        let job_repo = JobRepository::new(pool.clone());
        let page_repo = PageRepository::new(pool.clone());

        // Create job
        let job_id = job_repo.create("https://example.com", &JobSettings::default()).await.unwrap();

        // Insert a page with large load time (4s) so fallback would be false
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
            crawled_at: Utc::now(),
        };

        let page_id = page_repo.insert(&page).await.unwrap();

        // Insert lighthouse raw json that indicates viewport passed and structured data present
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

        let assembler = AnalysisAssembler::new(pool.clone());
        let result = assembler.assemble(&job_id).await.unwrap();

        assert_eq!(result.pages.len(), 1);
        let page = &result.pages[0];

        // Lighthouse viewport passed should override slow load time
        assert!(page.mobile_friendly, "expected mobile_friendly=true from Lighthouse viewport");
        assert!(page.has_structured_data, "expected structured data detected from Lighthouse raw JSON");
    }

    #[tokio::test]
    async fn test_mobile_detection_fallback_to_load_time() {
        let pool = fixtures::setup_test_db().await;

        let job_repo = JobRepository::new(pool.clone());
        let page_repo = PageRepository::new(pool.clone());

        let job_id = job_repo.create("https://example.com", &JobSettings::default()).await.unwrap();

        // Insert a page with short load time (1s) and no lighthouse data
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
            crawled_at: Utc::now(),
        };

        page_repo.insert(&page).await.unwrap();

        let assembler = AnalysisAssembler::new(pool.clone());
        let result = assembler.assemble(&job_id).await.unwrap();

        assert_eq!(result.pages.len(), 1);
        let page = &result.pages[0];

        // No Lighthouse viewport present; fallback to load_time <= 2s
        assert!(page.mobile_friendly, "expected mobile_friendly=true from load time heuristic");
    }

    #[tokio::test]
    async fn test_link_classification_fallback_when_target_unparsable() {
        let pool = fixtures::setup_test_db().await;

        let job_repo = JobRepository::new(pool.clone());
        let page_repo = PageRepository::new(pool.clone());
        let link_repo = LinkRepository::new(pool.clone());

        let job_id = job_repo.create("https://example.com", &JobSettings::default()).await.unwrap();

        // Insert source page
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
            crawled_at: Utc::now(),
        };

        let page_id = page_repo.insert(&page).await.unwrap();

        // Insert links with unparsable / special target URLs
        let links = vec![
            NewLink {
                job_id: job_id.clone(),
                source_page_id: page_id.clone(),
                target_page_id: None,
                // Empty string is not a valid/parsable URL -> will trigger fallback to link_type
                target_url: "".to_string(),
                link_text: Some("void link".to_string()),
                link_type: LinkType::Internal,
                is_followed: true,
                status_code: None,
            },
            NewLink {
                job_id: job_id.clone(),
                source_page_id: page_id.clone(),
                target_page_id: None,
                // `javascript:` is parseable by Url::parse -> treated as external if hosts differ
                target_url: "javascript:external:void(0)".to_string(),
                link_text: Some("void link external".to_string()),
                link_type: LinkType::External,
                is_followed: true,
                status_code: None,
            },
        ];

        link_repo.insert_batch(&links).await.unwrap();

        let assembler = AnalysisAssembler::new(pool.clone());
        let result = assembler.assemble(&job_id).await.unwrap();

        assert_eq!(result.pages.len(), 1);
        let page = &result.pages[0];

        // Should have two detailed links
        assert_eq!(page.detailed_links.len(), 2);

        // Find links by link text so we don't depend on insertion order
        let void_link = page.detailed_links.iter().find(|l| l.text == "void link").expect("expected void link");
        let void_link_external = page.detailed_links.iter().find(|l| l.text == "void link external").expect("expected void link external");

        // First link: internal link_type and unparsable target (empty string) -> fallback to link_type -> is_external = false
        assert_eq!(void_link.url, "", "expected empty target url");
        assert_eq!(void_link.is_external, false, "empty target should be treated as internal when link_type is Internal");

        // Second link: external link_type and unparsable target -> is_external = true
        assert_eq!(void_link_external.is_external, true, "javascript:external:void(0) should be treated as external when link_type is External");
    }
}

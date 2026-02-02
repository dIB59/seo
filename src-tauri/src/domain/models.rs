//! Rich domain entities - behavior lives WITH data
//!
//! This module contains the core domain models for the SEO analyzer.
//! Models are designed to be:
//! - **Type-safe**: Using newtypes for IDs to prevent mixing them up
//! - **Rich**: Business logic lives with the data it operates on
//! - **Serializable**: Ready for API responses and database storage

use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::service::job_processor::PageEdge;

// ============================================================================
// TYPE-SAFE ID WRAPPERS
// ============================================================================

/// Wrapper for job identifiers (database auto-increment ID).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobId(pub i64);

impl From<i64> for JobId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Wrapper for analysis result identifiers (UUID string).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnalysisId(pub String);

impl From<String> for AnalysisId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for AnalysisId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl AsRef<str> for AnalysisId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AnalysisId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Wrapper for page identifiers (UUID string).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageId(pub String);

impl From<String> for PageId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl AsRef<str> for PageId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// ENUMS
// ============================================================================

/// Status of a resource check (robots.txt, sitemap.xml, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceStatus {
    /// Resource found with content
    Found(String),
    /// Resource exists but access denied
    Unauthorized(String),
    /// Resource not found (404)
    NotFound,
}

impl ResourceStatus {
    pub fn exists(&self) -> bool {
        matches!(
            self,
            ResourceStatus::Found(_) | ResourceStatus::Unauthorized(_)
        )
    }
}

/// Unified status for jobs and analysis results.
/// 
/// Represents the lifecycle of an SEO analysis:
/// ```text
/// Queued → Discovering → Processing → Completed
///                    ↘      ↓       ↗
///                      → Failed
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Initial state, waiting to be processed
    Queued,
    /// Discovering pages to analyze
    Discovering,
    /// Actively analyzing pages
    Processing,
    /// Successfully completed
    Completed,
    /// Failed or cancelled
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Discovering => "discovering",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
    
    /// Check if this is a terminal state (no more transitions expected)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
    
    /// Check if the job is actively running
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Discovering | Self::Processing)
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for JobStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "discovering" => Ok(Self::Discovering),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" | "error" => Ok(Self::Failed), // "error" for legacy compatibility
            "analyzing" => Ok(Self::Processing),    // legacy mapping
            _ => Err(()),
        }
    }
}

/// Issue severity level for SEO problems.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueType {
    /// Must fix - significantly impacts SEO
    Critical,
    /// Should fix - moderate impact on SEO  
    Warning,
    /// Nice to have - minor optimization opportunity
    Suggestion,
}

impl IssueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueType::Critical => "critical",
            IssueType::Warning => "warning",
            IssueType::Suggestion => "suggestion",
        }
    }
}

// ====== Simple Entities (no behavior needed) ======

#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalysisSummary {
    pub analysis_id: String,
    pub seo_score: i64,
    pub avg_load_time: f64,
    pub total_words: i64,
    pub total_issues: i64,
}

#[derive(Debug, Clone)]
pub struct AnalysisSettings {
    pub id: i64,
    pub max_pages: i64,
    pub include_external_links: bool,
    pub check_images: bool,
    pub mobile_analysis: bool,
    pub lighthouse_analysis: bool,
    pub delay_between_requests: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AnalysisJob {
    pub id: i64,
    pub url: String,
    pub settings_id: i64,
    pub created_at: DateTime<Utc>,
    pub status: JobStatus,
    pub result_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct AnalysisProgress {
    pub job_id: String,
    pub url: String,
    pub job_status: String,
    pub result_id: Option<String>,
    pub progress: Option<f64>,
    pub analyzed_pages: Option<i64>,
    pub total_pages: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct CompleteAnalysisResult {
    pub analysis: AnalysisResults,
    pub pages: Vec<PageAnalysisData>,
    pub issues: Vec<SeoIssue>,
    pub summary: AnalysisSummary,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalysisResults {
    pub id: String,
    pub url: String,
    pub status: JobStatus,
    pub progress: f64,
    pub total_pages: i64,
    pub analyzed_pages: i64,
    pub started_at: Option<chrono::DateTime<Utc>>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
    pub sitemap_found: bool,
    pub robots_txt_found: bool,
    pub ssl_certificate: bool,
    pub created_at: DateTime<Utc>,
}

// ====== Detailed Page Elements ======

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeadingElement {
    pub tag: String,
    pub text: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageElement {
    pub src: String,
    pub alt: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinkElement {
    pub href: String,
    pub text: String,
    pub is_internal: bool,
    pub status_code: Option<u16>,
}

// ============================================================================
// PAGE ANALYSIS DATA WITH BUILDER
// ============================================================================

/// Builder for constructing PageAnalysisData with a fluent API.
/// 
/// This extracts common data from HTML once and allows customization of
/// runtime-specific fields like load_time and lighthouse scores.
/// 
/// # Example
/// ```ignore
/// let (page, issues) = PageAnalysisData::builder(url, &document)
///     .load_time(1.5)
///     .status_code(200)
///     .content_size(5000)
///     .lighthouse_scores(Some(scores))
///     .build();
/// ```
pub struct PageAnalysisDataBuilder {
    url: String,
    // Extracted from HTML (computed once)
    title: Option<String>,
    meta_description: Option<String>,
    meta_keywords: Option<String>,
    canonical_url: Option<String>,
    h1_count: i64,
    h2_count: i64,
    h3_count: i64,
    word_count: i64,
    image_count: i64,
    images_without_alt: i64,
    internal_links: i64,
    external_links: i64,
    has_structured_data: bool,
    headings: Vec<HeadingElement>,
    images: Vec<ImageElement>,
    detailed_links: Vec<LinkElement>,
    // Runtime fields (set via builder)
    load_time: f64,
    status_code: Option<i64>,
    content_size: i64,
    lighthouse_scores: Option<crate::service::LighthouseScores>,
}

impl PageAnalysisDataBuilder {
    /// Create a new builder by extracting data from the parsed HTML document.
    fn new(url: String, document: &Html) -> Self {
        let parsed_url = Url::parse(&url).ok();
        let (h1, h2, h3) = PageAnalysisData::count_headings(document);
        let (img_count, img_no_alt) = PageAnalysisData::analyze_images(document);
        let (internal, external) = parsed_url
            .as_ref()
            .map(|u| PageAnalysisData::count_links(document, u))
            .unwrap_or((0, 0));
        
        Self {
            title: PageAnalysisData::extract_title(document),
            meta_description: PageAnalysisData::extract_meta(document, "description"),
            meta_keywords: PageAnalysisData::extract_meta(document, "keywords"),
            canonical_url: PageAnalysisData::extract_canonical(document),
            h1_count: h1,
            h2_count: h2,
            h3_count: h3,
            word_count: PageAnalysisData::count_words(document),
            image_count: img_count,
            images_without_alt: img_no_alt,
            internal_links: internal,
            external_links: external,
            has_structured_data: PageAnalysisData::check_structured_data(document),
            headings: PageAnalysisData::extract_headings(document),
            images: PageAnalysisData::extract_images_detailed(document),
            detailed_links: parsed_url
                .as_ref()
                .map(|u| PageAnalysisData::extract_detailed_links(document, u))
                .unwrap_or_default(),
            url,
            // Defaults for runtime fields
            load_time: 0.0,
            status_code: None,
            content_size: 0,
            lighthouse_scores: None,
        }
    }
    
    pub fn load_time(mut self, time: f64) -> Self {
        self.load_time = time;
        self
    }
    
    pub fn status_code(mut self, code: i64) -> Self {
        self.status_code = Some(code);
        self
    }
    
    pub fn content_size(mut self, size: i64) -> Self {
        self.content_size = size;
        self
    }
    
    pub fn lighthouse_scores(mut self, scores: Option<crate::service::LighthouseScores>) -> Self {
        self.lighthouse_scores = scores;
        self
    }
    
    /// Build the PageAnalysisData and generate SEO issues.
    pub fn build(self) -> (PageAnalysisData, Vec<SeoIssue>) {
        // Convert Lighthouse scores from 0.0-1.0 to 0-100 for UI display
        let to_percentage = |score: Option<f64>| score.map(|s| (s * 100.0).round());
        let scores = self.lighthouse_scores.unwrap_or_default();
        
        let page = PageAnalysisData {
            analysis_id: String::new(),
            url: self.url,
            title: self.title,
            meta_description: self.meta_description,
            meta_keywords: self.meta_keywords,
            canonical_url: self.canonical_url,
            h1_count: self.h1_count,
            h2_count: self.h2_count,
            h3_count: self.h3_count,
            word_count: self.word_count,
            image_count: self.image_count,
            images_without_alt: self.images_without_alt,
            internal_links: self.internal_links,
            external_links: self.external_links,
            load_time: self.load_time,
            status_code: self.status_code,
            content_size: self.content_size,
            mobile_friendly: true,
            has_structured_data: self.has_structured_data,
            lighthouse_performance: to_percentage(scores.performance),
            lighthouse_accessibility: to_percentage(scores.accessibility),
            lighthouse_best_practices: to_percentage(scores.best_practices),
            lighthouse_seo: to_percentage(scores.seo),
            lighthouse_seo_audits: Some(scores.seo_audits),
            lighthouse_performance_metrics: scores.performance_metrics,
            links: Vec::new(),
            headings: self.headings,
            images: self.images,
            detailed_links: self.detailed_links,
        };
        
        let issues = page.generate_issues();
        (page, issues)
    }
}

/// Data extracted from analyzing a single web page.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PageAnalysisData {
    pub analysis_id: String,
    pub url: String,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub canonical_url: Option<String>,
    pub h1_count: i64,
    pub h2_count: i64,
    pub h3_count: i64,
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
    // Detailed Lighthouse audit breakdown (JSON serialized)
    pub lighthouse_seo_audits: Option<crate::service::SeoAuditDetails>,
    pub lighthouse_performance_metrics: Option<crate::service::PerformanceMetrics>,
    pub links: Vec<PageEdge>,
    // detailed data for report view
    pub headings: Vec<HeadingElement>,
    pub images: Vec<ImageElement>,
    pub detailed_links: Vec<LinkElement>,
}

impl PageAnalysisData {
    /// Create a builder for PageAnalysisData from parsed HTML.
    pub fn builder(url: String, document: &Html) -> PageAnalysisDataBuilder {
        PageAnalysisDataBuilder::new(url, document)
    }

    /// Rich factory: parses HTML and performs initial analysis (no lighthouse scores).
    pub fn build_from_parsed(
        url: String,
        document: Html,
        load_time: f64,
        status_code: i64,
        content_size: i64,
    ) -> (Self, Vec<SeoIssue>) {
        Self::builder(url, &document)
            .load_time(load_time)
            .status_code(status_code)
            .content_size(content_size)
            .build()
    }

    /// Rich factory with lighthouse scores: parses HTML and performs initial analysis.
    pub fn build_from_parsed_with_lighthouse(
        url: String,
        document: Html,
        load_time: f64,
        status_code: i64,
        content_size: i64,
        lighthouse_scores: Option<crate::service::LighthouseScores>,
    ) -> (Self, Vec<SeoIssue>) {
        Self::builder(url, &document)
            .load_time(load_time)
            .status_code(status_code)
            .content_size(content_size)
            .lighthouse_scores(lighthouse_scores)
            .build()
    }

    pub const ISSUE_MISSING_TITLE: &'static str = "Missing Title Tag";
    pub const ISSUE_TITLE_TOO_SHORT: &'static str = "Title Too Short";
    pub const ISSUE_TITLE_TOO_LONG: &'static str = "Title Too Long";
    pub const ISSUE_MISSING_DESC: &'static str = "Missing Meta Description";
    pub const ISSUE_MISSING_H1: &'static str = "Missing H1 Tag";
    pub const ISSUE_MULTIPLE_H1: &'static str = "Multiple H1 Tags";
    pub const ISSUE_THIN_CONTENT: &'static str = "Thin Content";
    pub const ISSUE_IMG_MISSING_ALT: &'static str = "Images Missing Alt Text";
    pub const ISSUE_SLOW_LOAD: &'static str = "Slow Page Load";

    /// Validates the page and generates SEO issues using the builder pattern.
    pub fn generate_issues(&self) -> Vec<SeoIssue> {
        let mut issues = Vec::new();
        let page_id = uuid::Uuid::new_v4().to_string();

        // Title checks
        match &self.title {
            None => {
                issues.push(
                    SeoIssue::critical(Self::ISSUE_MISSING_TITLE)
                        .page_id(&page_id)
                        .description("Page has no title tag")
                        .page_url(&self.url)
                        .element("title")
                        .recommendation("Add a unique, descriptive title tag (50-60 characters)")
                        .build()
                );
            }
            Some(title) if title.len() < 5 => {
                issues.push(
                    SeoIssue::warning(Self::ISSUE_TITLE_TOO_SHORT)
                        .page_id(&page_id)
                        .description(format!("Title is only {} characters", title.len()))
                        .page_url(&self.url)
                        .element("title")
                        .recommendation("Expand title to 50-60 characters with main keyword")
                        .build()
                );
            }
            Some(title) if title.len() > 60 => {
                issues.push(
                    SeoIssue::suggestion(Self::ISSUE_TITLE_TOO_LONG)
                        .page_id(&page_id)
                        .description(format!("Title is {} characters", title.len()))
                        .page_url(&self.url)
                        .element("title")
                        .recommendation("Shorten title to display fully in search results")
                        .build()
                );
            }
            _ => {}
        }

        // Missing meta description
        if self.meta_description.is_none() {
            issues.push(
                SeoIssue::warning(Self::ISSUE_MISSING_DESC)
                    .page_id(&page_id)
                    .description("Page has no meta description")
                    .page_url(&self.url)
                    .element("meta[name=description]")
                    .recommendation("Add a compelling meta description (150-160 characters)")
                    .build()
            );
        }

        // H1 checks
        match self.h1_count {
            0 => {
                issues.push(
                    SeoIssue::critical(Self::ISSUE_MISSING_H1)
                        .page_id(&page_id)
                        .description("Page has no H1 heading")
                        .page_url(&self.url)
                        .element("h1")
                        .recommendation("Add one H1 tag with main keyword near the top")
                        .build()
                );
            }
            n if n > 1 => {
                issues.push(
                    SeoIssue::warning(Self::ISSUE_MULTIPLE_H1)
                        .page_id(&page_id)
                        .description(format!("Page has {} H1 tags", n))
                        .page_url(&self.url)
                        .element("h1")
                        .recommendation("Use only one H1 tag per page")
                        .build()
                );
            }
            _ => {}
        }

        // Thin content
        if self.word_count < 300 {
            issues.push(
                SeoIssue::warning(Self::ISSUE_THIN_CONTENT)
                    .page_id(&page_id)
                    .description(format!("Page only has {} words", self.word_count))
                    .page_url(&self.url)
                    .recommendation("Add more comprehensive content (aim for 500+ words)")
                    .build()
            );
        }

        // Images without alt text
        if self.images_without_alt > 0 {
            issues.push(
                SeoIssue::warning(Self::ISSUE_IMG_MISSING_ALT)
                    .page_id(&page_id)
                    .description(format!(
                        "{} of {} images lack alt attribute",
                        self.images_without_alt, self.image_count
                    ))
                    .page_url(&self.url)
                    .element("img")
                    .recommendation("Add descriptive alt text for accessibility and SEO")
                    .build()
            );
        }

        // Slow page load
        if self.load_time > 3.0 {
            issues.push(
                SeoIssue::warning(Self::ISSUE_SLOW_LOAD)
                    .page_id(&page_id)
                    .description(format!("Page loads in {:.2} seconds", self.load_time))
                    .page_url(&self.url)
                    .recommendation("Optimize images, enable caching, reduce server response time")
                    .build()
            );
        }

        issues
    }

    /// Helper for testing: Creates a valid 'good' page instance to minimize boilerplate in tests
    #[cfg(test)]
    pub fn default_test_instance() -> Self {
        Self {
            analysis_id: "test".into(),
            url: "https://example.com".into(),
            title: Some("Valid Page Title Length".into()),
            meta_description: Some("Valid description".into()),
            meta_keywords: None,
            canonical_url: None,
            h1_count: 1,
            h2_count: 1,
            h3_count: 1,
            word_count: 500, // Good length
            image_count: 1,
            images_without_alt: 0, // Good
            internal_links: 5,
            external_links: 2,
            load_time: 0.5, // Fast
            status_code: Some(200),
            content_size: 1024,
            mobile_friendly: true,
            has_structured_data: true,
            lighthouse_performance: None,
            lighthouse_accessibility: None,
            lighthouse_best_practices: None,
            lighthouse_seo: None,
            lighthouse_seo_audits: None,
            lighthouse_performance_metrics: None,
            links: Vec::new(),
            headings: Vec::new(), // Added for consistency with build_from_parsed
            images: Vec::new(),   // Added for consistency with build_from_parsed
            detailed_links: Vec::new(), // Added for consistency with build_from_parsed
        }
    }

    // ====== PRIVATE parsing helpers ======

    fn extract_title(document: &Html) -> Option<String> {
        Selector::parse("title").ok().and_then(|sel| {
            document
                .select(&sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
        })
    }

    fn extract_meta(document: &Html, name: &str) -> Option<String> {
        Selector::parse(&format!(r#"meta[name="{}"]"#, name))
            .ok()
            .and_then(|sel| {
                document
                    .select(&sel)
                    .next()
                    .and_then(|el| el.value().attr("content").map(|s| s.to_string()))
            })
    }

    fn extract_canonical(document: &Html) -> Option<String> {
        Selector::parse(r#"link[rel="canonical"]"#)
            .ok()
            .and_then(|sel| {
                document
                    .select(&sel)
                    .next()
                    .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
            })
    }

    fn count_headings(document: &Html) -> (i64, i64, i64) {
        let h1 = Selector::parse("h1").unwrap();
        let h2 = Selector::parse("h2").unwrap();
        let h3 = Selector::parse("h3").unwrap();

        (
            document.select(&h1).count() as i64,
            document.select(&h2).count() as i64,
            document.select(&h3).count() as i64,
        )
    }

    fn count_words(document: &Html) -> i64 {
        Selector::parse("body")
            .ok()
            .and_then(|sel| {
                document
                    .select(&sel)
                    .next()
                    .map(|body| body.text().collect::<String>().split_whitespace().count() as i64)
            })
            .unwrap_or(0)
    }

    fn analyze_images(document: &Html) -> (i64, i64) {
        let img_selector = Selector::parse("img").unwrap();
        let mut count = 0;
        let mut missing_alt = 0;

        for img in document.select(&img_selector) {
            count += 1;
            if img.value().attr("alt").is_none() {
                missing_alt += 1;
            }
        }
        (count, missing_alt)
    }

    fn count_links(document: &Html, base_url: &Url) -> (i64, i64) {
        let a_selector = Selector::parse("a[href]").unwrap();
        let mut internal = 0;
        let mut external = 0;

        for link in document.select(&a_selector) {
            if let Some(href) = link.value().attr("href") {
                if let Ok(url) = base_url.join(href) {
                    if url.host_str() == base_url.host_str() {
                        internal += 1;
                    } else {
                        external += 1;
                    }
                }
            }
        }
        (internal, external)
    }

    fn extract_headings(document: &Html) -> Vec<HeadingElement> {
        let mut results = Vec::new();
        for level in 1..=6 {
            let tag = format!("h{}", level);
            let selector = Selector::parse(&tag).unwrap();
            for element in document.select(&selector) {
                results.push(HeadingElement {
                    tag: tag.clone(),
                    text: element
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .trim()
                        .to_string(),
                });
            }
        }
        results
    }

    fn extract_images_detailed(document: &Html) -> Vec<ImageElement> {
        let img_selector = Selector::parse("img").unwrap();
        let mut results = Vec::new();

        for img in document.select(&img_selector) {
            if let Some(src) = img.value().attr("src") {
                results.push(ImageElement {
                    src: src.to_string(),
                    alt: img.value().attr("alt").map(|s| s.to_string()),
                });
            }
        }
        results
    }

    fn extract_detailed_links(document: &Html, base_url: &Url) -> Vec<LinkElement> {
        let a_selector = Selector::parse("a[href]").unwrap();
        let mut results = Vec::new();

        for link in document.select(&a_selector) {
            if let Some(href) = link.value().attr("href") {
                let is_internal = if let Ok(url) = base_url.join(href) {
                    url.host_str() == base_url.host_str()
                } else {
                    false
                };

                results.push(LinkElement {
                    href: href.to_string(),
                    text: link.text().collect::<Vec<_>>().join(" ").trim().to_string(),
                    is_internal,
                    status_code: None, // This will be populated later if checked
                });
            }
        }
        results
    }

    fn check_structured_data(document: &Html) -> bool {
        Selector::parse(r#"script[type="application/ld+json"]"#)
            .ok()
            .and_then(|sel| document.select(&sel).next())
            .is_some()
    }
}

// ============================================================================
// SEO ISSUE WITH BUILDER
// ============================================================================

/// An SEO issue found during page analysis.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SeoIssue {
    pub page_id: String,
    pub issue_type: IssueType,
    pub title: String,
    pub description: String,
    pub page_url: String,
    pub element: Option<String>,
    pub line_number: Option<i64>,
    pub recommendation: String,
}

impl SeoIssue {
    /// Start building a new SEO issue.
    pub fn builder(issue_type: IssueType, title: impl Into<String>) -> SeoIssueBuilder {
        SeoIssueBuilder::new(issue_type, title)
    }
    
    /// Create a critical issue (convenience method).
    pub fn critical(title: impl Into<String>) -> SeoIssueBuilder {
        SeoIssueBuilder::new(IssueType::Critical, title)
    }
    
    /// Create a warning issue (convenience method).
    pub fn warning(title: impl Into<String>) -> SeoIssueBuilder {
        SeoIssueBuilder::new(IssueType::Warning, title)
    }
    
    /// Create a suggestion issue (convenience method).
    pub fn suggestion(title: impl Into<String>) -> SeoIssueBuilder {
        SeoIssueBuilder::new(IssueType::Suggestion, title)
    }
}

/// Builder for creating SeoIssue instances with a fluent API.
/// 
/// # Example
/// ```ignore
/// let issue = SeoIssue::critical("Missing Title Tag")
///     .description("Page has no title tag")
///     .page_url("https://example.com")
///     .element("title")
///     .recommendation("Add a unique, descriptive title tag")
///     .build();
/// ```
#[derive(Debug)]
pub struct SeoIssueBuilder {
    page_id: String,
    issue_type: IssueType,
    title: String,
    description: String,
    page_url: String,
    element: Option<String>,
    line_number: Option<i64>,
    recommendation: String,
}

impl SeoIssueBuilder {
    fn new(issue_type: IssueType, title: impl Into<String>) -> Self {
        Self {
            page_id: uuid::Uuid::new_v4().to_string(),
            issue_type,
            title: title.into(),
            description: String::new(),
            page_url: String::new(),
            element: None,
            line_number: None,
            recommendation: String::new(),
        }
    }
    
    pub fn page_id(mut self, id: impl Into<String>) -> Self {
        self.page_id = id.into();
        self
    }
    
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
    
    pub fn page_url(mut self, url: impl Into<String>) -> Self {
        self.page_url = url.into();
        self
    }
    
    pub fn element(mut self, el: impl Into<String>) -> Self {
        self.element = Some(el.into());
        self
    }
    
    pub fn line_number(mut self, line: i64) -> Self {
        self.line_number = Some(line);
        self
    }
    
    pub fn recommendation(mut self, rec: impl Into<String>) -> Self {
        self.recommendation = rec.into();
        self
    }
    
    pub fn build(self) -> SeoIssue {
        SeoIssue {
            page_id: self.page_id,
            issue_type: self.issue_type,
            title: self.title,
            description: self.description,
            page_url: self.page_url,
            element: self.element,
            line_number: self.line_number,
            recommendation: self.recommendation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_status_exists() {
        assert!(ResourceStatus::Found("url".into()).exists());
        assert!(ResourceStatus::Unauthorized("url".into()).exists());
        assert!(!ResourceStatus::NotFound.exists());
    }

    #[test]
    fn test_generate_issues_missing_title() {
        let mut page = PageAnalysisData::default_test_instance();
        page.title = None; // Inject failure

        let issues = page.generate_issues();
        assert!(issues
            .iter()
            .any(|i| i.title == PageAnalysisData::ISSUE_MISSING_TITLE));
    }

    #[test]
    fn test_generate_issues_short_content() {
        let mut page = PageAnalysisData::default_test_instance();
        page.word_count = 50; // Inject failure

        let issues = page.generate_issues();
        assert!(issues
            .iter()
            .any(|i| i.title == PageAnalysisData::ISSUE_THIN_CONTENT));
    }

    #[test]
    fn test_generate_multiple_issues() {
        // A page with multiple problems: no title, thin content, slow load
        let mut page = PageAnalysisData::default_test_instance();
        page.title = None;
        page.word_count = 50;
        page.load_time = 5.0;

        let issues = page.generate_issues();

        assert!(issues
            .iter()
            .any(|i| i.title == PageAnalysisData::ISSUE_MISSING_TITLE));
        assert!(issues
            .iter()
            .any(|i| i.title == PageAnalysisData::ISSUE_THIN_CONTENT));
        assert!(issues
            .iter()
            .any(|i| i.title == PageAnalysisData::ISSUE_SLOW_LOAD));
        assert_eq!(issues.len(), 3);
    }

    #[test]
    fn test_build_from_parsed() {
        let html = r#"
            <html>
                <head>
                    <title>Test Page</title>
                    <meta name="description" content="A test page description.">
                </head>
                <body>
                    <h1>Hello</h1>
                    <img src="test.jpg" alt="test">
                    <img src="missing.jpg">
                    <a href="/link">Link</a>
                    <a href="https://external.com">External</a>
                </body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let (page, issues) = PageAnalysisData::build_from_parsed(
            "https://example.com".into(),
            document,
            0.5,
            200,
            1000,
        );

        assert_eq!(page.h1_count, 1);
        assert_eq!(page.images_without_alt, 1); // img2 missing alt
        assert_eq!(page.internal_links, 1);
        assert_eq!(page.external_links, 1);

        // Issues should be generated too
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title == "Images Missing Alt Text"));
    }

    #[test]
    fn test_heading_extraction() {
        let html = r#"
            <html>
                <body>
                    <h1>Main Title</h1>
                    <h2>Subtitle 1</h2>
                    <div>
                        <h3>Nested Subtitle</h3>
                    </div>
                    <h4>Fourth Level</h4>
                    <h5>Fifth Level</h5>
                    <h6>Sixth Level</h6>
                </body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let headings = PageAnalysisData::extract_headings(&document);

        assert_eq!(headings.len(), 6);
        assert_eq!(headings[0].tag, "h1");
        assert_eq!(headings[0].text, "Main Title");
        assert_eq!(headings[1].tag, "h2");
        assert_eq!(headings[2].tag, "h3");
        assert_eq!(headings[5].tag, "h6");
        assert_eq!(headings[5].text, "Sixth Level");
    }

    // ===== Edge case tests for issue generation =====

    #[test]
    fn test_no_issues_for_good_page() {
        let page = PageAnalysisData::default_test_instance();
        let issues = page.generate_issues();

        assert!(
            issues.is_empty(),
            "A well-configured page should generate no issues, got: {:?}",
            issues.iter().map(|i| &i.title).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_title_exactly_at_short_boundary() {
        let mut page = PageAnalysisData::default_test_instance();
        page.title = Some("1234".to_string()); // 4 chars - should trigger "too short"

        let issues = page.generate_issues();
        assert!(
            issues
                .iter()
                .any(|i| i.title == PageAnalysisData::ISSUE_TITLE_TOO_SHORT),
            "Title with 4 chars should be too short"
        );
    }

    #[test]
    fn test_title_exactly_at_short_threshold() {
        let mut page = PageAnalysisData::default_test_instance();
        page.title = Some("12345".to_string()); // 5 chars - should NOT trigger

        let issues = page.generate_issues();
        assert!(
            !issues
                .iter()
                .any(|i| i.title == PageAnalysisData::ISSUE_TITLE_TOO_SHORT),
            "Title with exactly 5 chars should not be flagged as too short"
        );
    }

    #[test]
    fn test_title_too_long() {
        let mut page = PageAnalysisData::default_test_instance();
        page.title = Some("A".repeat(61)); // 61 chars - should trigger

        let issues = page.generate_issues();
        assert!(
            issues
                .iter()
                .any(|i| i.title == PageAnalysisData::ISSUE_TITLE_TOO_LONG),
            "Title with 61 chars should be too long"
        );
    }

    #[test]
    fn test_slow_load_boundary() {
        let mut page = PageAnalysisData::default_test_instance();
        page.load_time = 3.01; // Just above 3.0

        let issues = page.generate_issues();
        assert!(
            issues
                .iter()
                .any(|i| i.title == PageAnalysisData::ISSUE_SLOW_LOAD),
            "Load time > 3.0s should trigger slow load issue"
        );
    }

    #[test]
    fn test_multiple_h1_warning() {
        let mut page = PageAnalysisData::default_test_instance();
        page.h1_count = 3;

        let issues = page.generate_issues();
        assert!(
            issues
                .iter()
                .any(|i| i.title == PageAnalysisData::ISSUE_MULTIPLE_H1),
            "Multiple H1 tags should generate warning"
        );
    }
}

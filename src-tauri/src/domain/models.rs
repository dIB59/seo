//! Rich domain entities - behavior lives WITH data

use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde::Serialize;
use url::Url;

use crate::service::job_processor::PageEdge;

// ====== Enums ======

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceStatus {
    Found(String),
    Unauthorized(String),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Queued => "queued",
            JobStatus::Processing => "processing",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
        }
    }
}

//TODO:
//Remove AnalysisStatus
//and use Job status instead
//merge both of them

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum AnalysisStatus {
    Analyzing,
    Completed,
    Error,
    Paused,
}

impl AnalysisStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnalysisStatus::Analyzing => "analyzing",
            AnalysisStatus::Completed => "completed",
            AnalysisStatus::Error => "error",
            AnalysisStatus::Paused => "paused",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum IssueType {
    Critical,
    Warning,
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
    pub job_id: i64,
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

// ====== Rich Entity: PageAnalysisData ======

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
    pub links: Vec<PageEdge>,
}

impl PageAnalysisData {
    /// Rich factory: parses HTML and performs initial analysis
    pub fn build_from_parsed(
        url: String,
        document: Html,
        load_time: f64,
        status_code: i64,
        content_size: i64,
    ) -> (Self, Vec<SeoIssue>) {
        let page = Self {
            analysis_id: String::new(),
            url: url.clone(),
            title: Self::extract_title(&document),
            meta_description: Self::extract_meta(&document, "description"),
            meta_keywords: Self::extract_meta(&document, "keywords"),
            canonical_url: Self::extract_canonical(&document),
            h1_count: Self::count_headings(&document).0,
            h2_count: Self::count_headings(&document).1,
            h3_count: Self::count_headings(&document).2,
            word_count: Self::count_words(&document),
            image_count: Self::analyze_images(&document).0,
            images_without_alt: Self::analyze_images(&document).1,
            internal_links: Self::count_links(&document, &Url::parse(&url).unwrap()).0,
            external_links: Self::count_links(&document, &Url::parse(&url).unwrap()).1,
            load_time,
            status_code: Some(status_code),
            content_size,
            mobile_friendly: true,
            has_structured_data: Self::check_structured_data(&document),
            lighthouse_performance: None,
            lighthouse_accessibility: None,
            lighthouse_best_practices: None,
            lighthouse_seo: None,
            links: Vec::new(),
        };

        let issues = page.generate_issues(); // your other rules
        (page, issues)
    }

    /// Rich behavior: validates itself and generates SEO issues
    pub fn generate_issues(&self) -> Vec<SeoIssue> {
        let mut issues = Vec::new();
        let page_id = uuid::Uuid::new_v4().to_string();

        // Missing title
        if self.title.is_none() {
            issues.push(SeoIssue {
                page_id: page_id.clone(),
                issue_type: IssueType::Critical,
                title: "Missing Title Tag".to_string(),
                description: "Page has no title tag".to_string(),
                page_url: self.url.clone(),
                element: Some("title".to_string()),
                line_number: None,
                recommendation: "Add a unique, descriptive title tag (50-60 characters)"
                    .to_string(),
            });
        } else if let Some(title) = &self.title {
            if title.len() < 5 {
                issues.push(SeoIssue {
                    page_id: page_id.clone(),
                    issue_type: IssueType::Warning,
                    title: "Title Too Short".to_string(),
                    description: format!("Title is only {} characters", title.len()),
                    page_url: self.url.clone(),
                    element: Some("title".to_string()),
                    line_number: None,
                    recommendation: "Expand title to 50-60 characters with main keyword"
                        .to_string(),
                });
            } else if title.len() > 60 {
                issues.push(SeoIssue {
                    page_id: page_id.clone(),
                    issue_type: IssueType::Suggestion,
                    title: "Title Too Long".to_string(),
                    description: format!("Title is {} characters", title.len()),
                    page_url: self.url.clone(),
                    element: Some("title".to_string()),
                    line_number: None,
                    recommendation: "Shorten title to display fully in search results".to_string(),
                });
            }
        }

        // Missing meta description
        if self.meta_description.is_none() {
            issues.push(SeoIssue {
                page_id: page_id.clone(),
                issue_type: IssueType::Warning,
                title: "Missing Meta Description".to_string(),
                description: "Page has no meta description".to_string(),
                page_url: self.url.clone(),
                element: Some("meta[name=description]".to_string()),
                line_number: None,
                recommendation: "Add a compelling meta description (150-160 characters)"
                    .to_string(),
            });
        }

        // Missing H1
        if self.h1_count == 0 {
            issues.push(SeoIssue {
                page_id: page_id.clone(),
                issue_type: IssueType::Critical,
                title: "Missing H1 Tag".to_string(),
                description: "Page has no H1 heading".to_string(),
                page_url: self.url.clone(),
                element: Some("h1".to_string()),
                line_number: None,
                recommendation: "Add one H1 tag with main keyword near the top".to_string(),
            });
        } else if self.h1_count > 1 {
            issues.push(SeoIssue {
                page_id: page_id.clone(),
                issue_type: IssueType::Warning,
                title: "Multiple H1 Tags".to_string(),
                description: format!("Page has {} H1 tags", self.h1_count),
                page_url: self.url.clone(),
                element: Some("h1".to_string()),
                line_number: None,
                recommendation: "Use only one H1 tag per page".to_string(),
            });
        }

        // Thin content
        if self.word_count < 300 {
            issues.push(SeoIssue {
                page_id: page_id.clone(),
                issue_type: IssueType::Warning,
                title: "Thin Content".to_string(),
                description: format!("Page only has {} words", self.word_count),
                page_url: self.url.clone(),
                element: None,
                line_number: None,
                recommendation: "Add more comprehensive content (aim for 500+ words)".to_string(),
            });
        }

        // Images without alt text
        if self.images_without_alt > 0 {
            issues.push(SeoIssue {
                page_id: page_id.clone(),
                issue_type: IssueType::Warning,
                title: "Images Missing Alt Text".to_string(),
                description: format!(
                    "{} of {} images lack alt attribute",
                    self.images_without_alt, self.image_count
                ),
                page_url: self.url.clone(),
                element: Some("img".to_string()),
                line_number: None,
                recommendation: "Add descriptive alt text for accessibility and SEO".to_string(),
            });
        }

        // Slow page load
        if self.load_time > 3.0 {
            issues.push(SeoIssue {
                page_id: page_id.clone(),
                issue_type: IssueType::Warning,
                title: "Slow Page Load".to_string(),
                description: format!("Page loads in {:.2} seconds", self.load_time),
                page_url: self.url.clone(),
                element: None,
                line_number: None,
                recommendation: "Optimize images, enable caching, reduce server response time"
                    .to_string(),
            });
        }

        issues
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

    fn check_structured_data(document: &Html) -> bool {
        Selector::parse(r#"script[type="application/ld+json"]"#)
            .ok()
            .and_then(|sel| document.select(&sel).next())
            .is_some()
    }
}

// ====== Simple Entity: SeoIssue ======

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
        let page = PageAnalysisData {
            analysis_id: "1".into(),
            url: "http://example.com".into(),
            title: None, // Missing
            meta_description: Some("desc".into()),
            meta_keywords: None,
            canonical_url: None,
            h1_count: 1,
            h2_count: 0,
            h3_count: 0,
            word_count: 500,
            image_count: 1,
            images_without_alt: 0,
            internal_links: 0,
            external_links: 0,
            load_time: 0.5,
            status_code: Some(200),
            content_size: 1000,
            mobile_friendly: true,
            has_structured_data: true,
            lighthouse_performance: None,
            lighthouse_accessibility: None,
            lighthouse_best_practices: None,
            lighthouse_seo: None,
            links: vec![],
        };

        let issues = page.generate_issues();
        assert!(issues.iter().any(|i| i.title == "Missing Title Tag"));
    }

    #[test]
    fn test_generate_issues_short_content() {
         let page = PageAnalysisData {
            analysis_id: "1".into(),
            url: "http://example.com".into(),
            title: Some("A Very Good Title For SEO Purposes".into()),
            meta_description: Some("desc".into()),
            meta_keywords: None,
            canonical_url: None,
            h1_count: 1,
            h2_count: 0,
            h3_count: 0,
            word_count: 50, // Too short
            image_count: 1,
            images_without_alt: 0,
            internal_links: 0,
            external_links: 0,
            load_time: 0.5,
            status_code: Some(200),
            content_size: 1000,
            mobile_friendly: true,
            has_structured_data: true,
            lighthouse_performance: None,
            lighthouse_accessibility: None,
            lighthouse_best_practices: None,
            lighthouse_seo: None,
            links: vec![],
        };

        let issues = page.generate_issues();
        assert!(issues.iter().any(|i| i.title == "Thin Content"));
    }

    #[test]
    fn test_generate_multiple_issues() {
        // A page with multiple problems: no title, thin content, slow load
        let page = PageAnalysisData {
            analysis_id: "1".into(),
            url: "http://example.com/bad".into(),
            title: None, // Critical: Missing Title
            meta_description: Some("desc".into()),
            meta_keywords: None,
            canonical_url: None,
            h1_count: 1,
            h2_count: 0,
            h3_count: 0,
            word_count: 50, // Warning: Thin content
            image_count: 1,
            images_without_alt: 0,
            internal_links: 0,
            external_links: 0,
            load_time: 5.0, // Warning: Slow load
            status_code: Some(200),
            content_size: 1000,
            mobile_friendly: true,
            has_structured_data: true,
            lighthouse_performance: None,
            lighthouse_accessibility: None,
            lighthouse_best_practices: None,
            lighthouse_seo: None,
            links: vec![],
        };

        let issues = page.generate_issues();
        
        assert!(issues.iter().any(|i| i.title == "Missing Title Tag"));
        assert!(issues.iter().any(|i| i.title == "Thin Content"));
        assert!(issues.iter().any(|i| i.title == "Slow Page Load"));
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
                <h1>Main Heading</h1>
                <h2>Subheading</h2>
                <p>some content here with enough words to be counted.</p>
                <img src="img1.jpg" alt="Description">
                <img src="img2.jpg"> <!-- Missing alt -->
                <a href="/internal">Internal</a>
                <a href="https://external.com">External</a>
            </body>
        </html>
        "#;
        let document = Html::parse_document(html);
        let url = "https://example.com/page".to_string();
        
        let (page, issues) = PageAnalysisData::build_from_parsed(url.clone(), document, 0.5, 200, 500);

        assert_eq!(page.title, Some("Test Page".to_string()));
        assert_eq!(page.meta_description, Some("A test page description.".to_string()));
        assert_eq!(page.h1_count, 1);
        assert_eq!(page.h2_count, 1);
        assert_eq!(page.image_count, 2);
        assert_eq!(page.images_without_alt, 1); // img2 missing alt
        assert_eq!(page.internal_links, 1);
        assert_eq!(page.external_links, 1);
        
        // Issues should be generated too
        assert!(!issues.is_empty()); 
        assert!(issues.iter().any(|i| i.title == "Images Missing Alt Text"));
    }
}

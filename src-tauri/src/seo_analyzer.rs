pub(crate) use crate::analysis::AnalysisSettings;
use anyhow::Context;
use anyhow::Result;
use scraper::Html;
use scraper::Selector;
use std::time::Duration;
use url::Url;

#[derive(Debug)]
struct PageAnalysisData {
    id: String,
    url: String,
    title: Option<String>,
    meta_description: Option<String>,
    meta_keywords: Option<String>,
    canonical_url: Option<String>,
    h1_count: i32,
    h2_count: i32,
    h3_count: i32,
    word_count: i32,
    image_count: i32,
    images_without_alt: i32,
    internal_links: i32,
    external_links: i32,
    load_time: f64,
    status_code: Option<i32>,
    content_size: i32,
    mobile_friendly: bool,
    has_structured_data: bool,
    lighthouse_performance: Option<f64>,
    lighthouse_accessibility: Option<f64>,
    lighthouse_best_practices: Option<f64>,
    lighthouse_seo: Option<f64>,
}

#[derive(Debug)]
struct SeoIssue {
    page_id: String,
    issue_type: String,
    title: String,
    description: String,
    page_url: String,
    element: Option<String>,
    line_number: Option<i32>,
    recommendation: String,
}

struct SeoAnalyzer {
    settings: AnalysisSettings,
}

impl SeoAnalyzer {
    fn new(settings: AnalysisSettings) -> Self {
        Self { settings }
    }

    /// Analyze a page and ALWAYS return SEO data, even for failed pages
    /// Only returns Err for operational failures (network, parsing, etc.)
    async fn analyze_page(&self, url: &Url) -> Result<(PageAnalysisData, Vec<SeoIssue>)> {
        let start = std::time::Instant::now();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let response = match client.get(url.as_str()).send().await {
            Ok(resp) => resp,
            Err(e) => {
                // Even on network error, we create a page record with error status
                let page = PageAnalysisData {
                    id: String::new(),
                    url: url.to_string(),
                    title: None,
                    meta_description: None,
                    meta_keywords: None,
                    canonical_url: None,
                    h1_count: 0,
                    h2_count: 0,
                    h3_count: 0,
                    word_count: 0,
                    image_count: 0,
                    images_without_alt: 0,
                    internal_links: 0,
                    external_links: 0,
                    load_time: 0.0,
                    status_code: None,
                    content_size: 0,
                    mobile_friendly: false,
                    has_structured_data: false,
                    lighthouse_performance: None,
                    lighthouse_accessibility: None,
                    lighthouse_best_practices: None,
                    lighthouse_seo: None,
                };

                let issue = SeoIssue {
                    page_id: String::new(),
                    issue_type: "critical".to_string(),
                    title: "Page Unreachable".to_string(),
                    description: format!("Failed to fetch page: {}", e),
                    page_url: url.to_string(),
                    element: None,
                    line_number: None,
                    recommendation: "Check server connectivity and SSL certificate".to_string(),
                };

                return Ok((page, vec![issue]));
            }
        };

        let status_code = response.status();
        let content_size = response.content_length().unwrap_or(0) as i32;
        let body = response
            .text()
            .await
            .with_context(|| "Failed to read response body")?;
        let load_time = start.elapsed().as_secs_f64();

        // If non-200 status, still analyze but mark as issue
        if !status_code.is_success() {
            let page = PageAnalysisData {
                id: String::new(),
                url: url.to_string(),
                title: None,
                meta_description: None,
                meta_keywords: None,
                canonical_url: None,
                h1_count: 0,
                h2_count: 0,
                h3_count: 0,
                word_count: 0,
                image_count: 0,
                images_without_alt: 0,
                internal_links: 0,
                external_links: 0,
                load_time,
                status_code: Some(status_code.as_u16() as i32),
                content_size,
                mobile_friendly: false,
                has_structured_data: false,
                lighthouse_performance: None,
                lighthouse_accessibility: None,
                lighthouse_best_practices: None,
                lighthouse_seo: None,
            };

            let issue = SeoIssue {
                page_id: String::new(),
                issue_type: "critical".to_string(),
                title: format!("HTTP {} Error", status_code),
                description: format!("Page returned status code {}", status_code),
                page_url: url.to_string(),
                element: None,
                line_number: None,
                recommendation: "Fix the server error or remove broken links".to_string(),
            };

            return Ok((page, vec![issue]));
        }

        // Parse HTML and extract SEO data
        let document = Html::parse_document(&body);

        let page = PageAnalysisData {
            id: String::new(),
            url: url.to_string(),
            title: self.extract_title(&document),
            meta_description: self.extract_meta(&document, "description"),
            meta_keywords: self.extract_meta(&document, "keywords"),
            canonical_url: self.extract_canonical(&document),
            h1_count: self.count_headings(&document).0,
            h2_count: self.count_headings(&document).1,
            h3_count: self.count_headings(&document).2,
            word_count: self.count_words(&document),
            image_count: self.analyze_images(&document).0,
            images_without_alt: self.analyze_images(&document).1,
            internal_links: self.count_links(&document, url).0,
            external_links: self.count_links(&document, url).1,
            load_time,
            status_code: Some(status_code.as_u16() as i32),
            content_size,
            mobile_friendly: true, // Simplified - could use mobile viewport testing
            has_structured_data: self.check_structured_data(&document),
            lighthouse_performance: None,
            lighthouse_accessibility: None,
            lighthouse_best_practices: None,
            lighthouse_seo: None,
        };

        let issues = self.detect_seo_issues(&page, &document, url);

        Ok((page, issues))
    }

    fn detect_seo_issues(
        &self,
        page: &PageAnalysisData,
        document: &Html,
        url: &Url,
    ) -> Vec<SeoIssue> {
        let mut issues = Vec::new();

        // Title analysis
        match &page.title {
            None => issues.push(self.create_issue(
                "critical",
                "Missing Title Tag",
                "Page has no title tag",
                url,
                Some("title"),
                "Add a unique, descriptive title tag (50-60 characters)",
            )),
            Some(title) if title.len() < 5 => issues.push(self.create_issue(
                "warning",
                "Title Too Short",
                &format!("Title is only {} characters (min 30)", title.len()),
                url,
                Some("title"),
                "Expand title to 50-60 characters with main keyword",
            )),
            Some(title) if title.len() > 60 => issues.push(self.create_issue(
                "suggestion",
                "Title Too Long",
                &format!("Title is {} characters (max 60)", title.len()),
                url,
                Some("title"),
                "Shorten title to display fully in search results",
            )),
            _ => {}
        }

        // Meta description
        match &page.meta_description {
            None => issues.push(self.create_issue(
                "warning",
                "Missing Meta Description",
                "Page has no meta description",
                url,
                Some("meta[name=description]"),
                "Add a compelling meta description (150-160 characters)",
            )),
            Some(desc) if desc.len() < 50 => issues.push(self.create_issue(
                "warning",
                "Meta Description Too Short",
                &format!("Meta description is only {} characters", desc.len()),
                url,
                Some("meta[name=description]"),
                "Expand to 150-160 characters with call-to-action",
            )),
            Some(desc) if desc.len() > 160 => issues.push(self.create_issue(
                "suggestion",
                "Meta Description Too Long",
                &format!("Meta description is {} characters", desc.len()),
                url,
                Some("meta[name=description]"),
                "Shorten to prevent truncation in search results",
            )),
            _ => {}
        }

        // Headings
        if page.h1_count == 0 {
            issues.push(self.create_issue(
                "critical",
                "Missing H1 Tag",
                "Page has no H1 heading",
                url,
                Some("h1"),
                "Add one H1 tag with main keyword near the top",
            ));
        } else if page.h1_count > 1 {
            issues.push(self.create_issue(
                "warning",
                "Multiple H1 Tags",
                &format!("Page has {} H1 tags", page.h1_count),
                url,
                Some("h1"),
                "Use only one H1 tag per page for proper semantic structure",
            ));
        }

        if page.h2_count == 0 {
            issues.push(self.create_issue(
                "suggestion",
                "No H2 Headings",
                "Page has no H2 subheadings",
                url,
                Some("h2"),
                "Add H2 headings to improve content structure and readability",
            ));
        }

        // Content length
        if page.word_count < 300 {
            issues.push(self.create_issue(
                "warning",
                "Thin Content",
                &format!("Page only has {} words", page.word_count),
                url,
                None,
                "Add more comprehensive content (aim for 500+ words)",
            ));
        }

        // Images
        if page.images_without_alt > 0 {
            issues.push(self.create_issue(
                "warning",
                "Images Missing Alt Text",
                &format!(
                    "{} of {} images lack alt attribute",
                    page.images_without_alt, page.image_count
                ),
                url,
                Some("img"),
                "Add descriptive alt text for accessibility and SEO",
            ));
        }

        // Links
        if page.internal_links == 0 {
            issues.push(self.create_issue(
                "suggestion",
                "No Internal Links",
                "Page has no internal links",
                url,
                None,
                "Add relevant internal links to improve navigation",
            ));
        }

        // Canonical
        if page.canonical_url.is_none() {
            issues.push(self.create_issue(
                "suggestion",
                "Missing Canonical Tag",
                "Page has no canonical URL specified",
                url,
                Some("link[rel=canonical]"),
                "Add canonical tag to prevent duplicate content issues",
            ));
        }

        // Performance
        if page.load_time > 3.0 {
            issues.push(self.create_issue(
                "warning",
                "Slow Page Load",
                &format!("Page loads in {:.2} seconds", page.load_time),
                url,
                None,
                "Optimize images, enable caching, reduce server response time",
            ));
        }

        issues
    }

    fn create_issue(
        &self,
        issue_type: &str,
        title: &str,
        description: &str,
        url: &Url,
        element: Option<&str>,
        recommendation: &str,
    ) -> SeoIssue {
        SeoIssue {
            page_id: String::new(), // Filled by caller
            issue_type: issue_type.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            page_url: url.to_string(),
            element: element.map(|s| s.to_string()),
            line_number: None, // Could be added with deeper HTML parsing
            recommendation: recommendation.to_string(),
        }
    }

    // All your helper methods remain the same
    fn extract_title(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse("title").ok()?;
        document
            .select(&selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
    }

    fn extract_meta(&self, document: &Html, name: &str) -> Option<String> {
        let selector = Selector::parse(&format!(r#"meta[name="{}"]"#, name)).ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string())
    }

    fn extract_canonical(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse(r#"link[rel="canonical"]"#).ok()?;
        document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string())
    }

    fn count_headings(&self, document: &Html) -> (i32, i32, i32) {
        let h1 = Selector::parse("h1").unwrap();
        let h2 = Selector::parse("h2").unwrap();
        let h3 = Selector::parse("h3").unwrap();

        (
            document.select(&h1).count() as i32,
            document.select(&h2).count() as i32,
            document.select(&h3).count() as i32,
        )
    }

    fn count_words(&self, document: &Html) -> i32 {
        let body_selector = Selector::parse("body").unwrap();
        document
            .select(&body_selector)
            .next()
            .map(|body| {
                let text = body.text().collect::<String>();
                text.split_whitespace().count() as i32
            })
            .unwrap_or(0)
    }

    fn analyze_images(&self, document: &Html) -> (i32, i32) {
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

    fn count_links(&self, document: &Html, base_url: &Url) -> (i32, i32) {
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

    fn check_structured_data(&self, document: &Html) -> bool {
        Selector::parse(r#"script[type="application/ld+json"]"#)
            .ok()
            .and_then(|sel| document.select(&sel).next())
            .is_some()
    }
}

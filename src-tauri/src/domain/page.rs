use crate::domain::{
    HeadingElement, ImageElement, IssueSeverity, LighthouseData, LinkDetail, NewIssue,
    PageAnalysisData,
};
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Pages with load time ≤ 2s considered mobile-friendly (speed heuristic fallback).
const SPEED_HEURISTIC_LOAD_TIME_MS: i64 = 2000;

/// A crawled page with SEO data.
/// Maps to the `pages` table in the new schema.
#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub id: String,
    pub job_id: String,
    pub url: String,
    pub depth: i64,
    pub status_code: Option<i64>,
    pub content_type: Option<String>,

    // Core SEO fields
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub canonical_url: Option<String>,
    pub robots_meta: Option<String>,

    // Content metrics
    pub word_count: Option<i64>,
    pub load_time_ms: Option<i64>,
    pub response_size_bytes: Option<i64>,

    pub crawled_at: DateTime<Utc>,
}

impl Page {
    /// Speed-based mobile-friendly heuristic (fallback when no Lighthouse viewport data).
    pub fn is_mobile_friendly_heuristic(&self) -> bool {
        self.load_time_ms.unwrap_or(0) <= SPEED_HEURISTIC_LOAD_TIME_MS
    }

    /// Assemble a frontend-ready `PageAnalysisData` from this page's data.
    ///
    /// This absorbs the `assemble_single_page` logic that previously lived in
    /// `AnalysisAssembler`, keeping page-level data interpretation in the domain.
    pub fn to_analysis_data(
        self,
        lh_data: Option<&LighthouseData>,
        detailed_links: Vec<LinkDetail>,
        headings: Vec<HeadingElement>,
        images: Vec<ImageElement>,
    ) -> PageAnalysisData {
        let load_time = self.load_time_ms.unwrap_or(0) as f64 / 1000.0;

        // Lighthouse-derived booleans
        let mut mobile_friendly = false;
        let mut has_structured_data = false;
        let mut lighthouse_seo_audits = None;
        let mut lighthouse_performance_metrics = None;

        if let Some(lh) = lh_data {
            mobile_friendly = lh.is_mobile_friendly();
            has_structured_data = lh.has_structured_data();
            let (seo, perf) = lh.interpret_raw();
            lighthouse_seo_audits = seo;
            lighthouse_performance_metrics = perf;
        }

        // Fallback: speed heuristic
        if !mobile_friendly {
            mobile_friendly = self.is_mobile_friendly_heuristic();
        }

        // Link stats
        let internal_links = detailed_links.iter().filter(|l| !l.is_external).count() as i64;
        let external_links = detailed_links.iter().filter(|l| l.is_external).count() as i64;
        let links_vec = detailed_links.iter().map(|l| l.url.clone()).collect();

        // Heading stats
        let h1_count = headings.iter().filter(|h| h.tag == "h1").count() as i64;
        let h2_count = headings.iter().filter(|h| h.tag == "h2").count() as i64;
        let h3_count = headings.iter().filter(|h| h.tag == "h3").count() as i64;

        // Image stats
        let images_without_alt = images
            .iter()
            .filter(|img| img.alt.as_deref().unwrap_or("").is_empty())
            .count() as i64;

        PageAnalysisData {
            analysis_id: self.job_id,
            url: self.url,
            title: self.title,
            meta_description: self.meta_description,
            meta_keywords: None,
            canonical_url: self.canonical_url,
            h1_count,
            h2_count,
            h3_count,
            word_count: self.word_count.unwrap_or(0),
            image_count: images.len() as i64,
            images_without_alt,
            internal_links,
            external_links,
            load_time,
            status_code: self.status_code,
            content_size: self.response_size_bytes.unwrap_or(0),
            mobile_friendly,
            has_structured_data,
            lighthouse_performance: lh_data.and_then(|lh| lh.performance_score),
            lighthouse_accessibility: lh_data.and_then(|lh| lh.accessibility_score),
            lighthouse_best_practices: lh_data.and_then(|lh| lh.best_practices_score),
            lighthouse_seo: lh_data.and_then(|lh| lh.seo_score),
            lighthouse_seo_audits,
            lighthouse_performance_metrics,
            links: links_vec,
            headings,
            images,
            detailed_links,
        }
    }

    /// Perform a basic SEO audit on the page and generate a list of issues.
    pub fn audit(&self) -> Vec<NewIssue> {
        let mut issues = Vec::new();

        // 1. Title checks
        if self.title.is_none() || self.title.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
            issues.push(NewIssue {
                job_id: self.job_id.clone(),
                page_id: Some(self.id.clone()),
                issue_type: "Missing Title".to_string(),
                severity: IssueSeverity::Critical,
                message: "Page has no title tag".to_string(),
                details: Some("Add a descriptive title tag".to_string()),
            });
        }

        // 2. Meta description checks
        if self.meta_description.is_none()
            || self
                .meta_description
                .as_ref()
                .map(|d| d.is_empty())
                .unwrap_or(true)
        {
            issues.push(NewIssue {
                job_id: self.job_id.clone(),
                page_id: Some(self.id.clone()),
                issue_type: "Missing Meta Description".to_string(),
                severity: IssueSeverity::Warning,
                message: "Page has no meta description".to_string(),
                details: Some("Add a meta description".to_string()),
            });
        }

        // 3. HTTP Status Check
        if let Some(status) = self.status_code {
            if status >= 400 {
                issues.push(NewIssue {
                    job_id: self.job_id.clone(),
                    page_id: Some(self.id.clone()),
                    issue_type: "HTTP Error".to_string(),
                    severity: IssueSeverity::Critical,
                    message: format!("Page returned status code {}", status),
                    details: Some("Fix the HTTP error".to_string()),
                });
            }
        }

        issues
    }
}

/// Lightweight page info for listings.
#[derive(Debug, Clone, Serialize)]
pub struct PageInfo {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<i64>,
    pub load_time_ms: Option<i64>,
    pub issue_count: i64,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::IssueSeverity;

    fn make_page(overrides: impl FnOnce(&mut Page)) -> Page {
        let mut page = Page {
            id: "1".to_string(),
            job_id: "job1".to_string(),
            url: "https://example.com".to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: None,
            title: Some("Title".to_string()),
            meta_description: Some("desc".to_string()),
            canonical_url: None,
            robots_meta: None,
            word_count: Some(100),
            load_time_ms: Some(1000),
            response_size_bytes: Some(512),
            crawled_at: Utc::now(),
        };
        overrides(&mut page);
        page
    }

    #[test]
    fn test_audit_missing_title() {
        let page = make_page(|p| {
            p.title = None;
        });
        let issues = page.audit();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, "Missing Title");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_audit_http_error() {
        let page = make_page(|p| {
            p.status_code = Some(404);
        });
        let issues = page.audit();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, "HTTP Error");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_mobile_friendly_heuristic_fast_page() {
        let page = make_page(|p| p.load_time_ms = Some(1500));
        assert!(page.is_mobile_friendly_heuristic());
    }

    #[test]
    fn test_mobile_friendly_heuristic_slow_page() {
        let page = make_page(|p| p.load_time_ms = Some(3000));
        assert!(!page.is_mobile_friendly_heuristic());
    }

    #[test]
    fn test_mobile_friendly_heuristic_exactly_threshold() {
        let page = make_page(|p| p.load_time_ms = Some(2000));
        assert!(page.is_mobile_friendly_heuristic());
    }

    #[test]
    fn test_to_analysis_data_counts_headings_and_images() {
        let page = make_page(|p| {
            p.word_count = Some(500);
            p.load_time_ms = Some(1500);
            p.response_size_bytes = Some(1024);
        });

        let headings = vec![
            HeadingElement {
                tag: "h1".into(),
                text: "Main".into(),
            },
            HeadingElement {
                tag: "h2".into(),
                text: "Sub".into(),
            },
            HeadingElement {
                tag: "h2".into(),
                text: "Sub2".into(),
            },
            HeadingElement {
                tag: "h3".into(),
                text: "Detail".into(),
            },
        ];

        let images = vec![
            ImageElement {
                src: "a.png".into(),
                alt: Some("Alt".into()),
            },
            ImageElement {
                src: "b.png".into(),
                alt: None,
            },
            ImageElement {
                src: "c.png".into(),
                alt: Some("".into()),
            },
        ];

        let links = vec![
            LinkDetail {
                url: "/page2".into(),
                text: "P2".into(),
                is_external: false,
                is_broken: false,
                status_code: Some(200),
            },
            LinkDetail {
                url: "https://ext.com".into(),
                text: "Ext".into(),
                is_external: true,
                is_broken: false,
                status_code: Some(200),
            },
        ];

        let result = page.to_analysis_data(None, links, headings, images);

        assert_eq!(result.h1_count, 1);
        assert_eq!(result.h2_count, 2);
        assert_eq!(result.h3_count, 1);
        assert_eq!(result.image_count, 3);
        assert_eq!(result.images_without_alt, 2); // None and empty string
        assert_eq!(result.internal_links, 1);
        assert_eq!(result.external_links, 1);
        assert_eq!(result.word_count, 500);
        assert_eq!(result.content_size, 1024);
        assert!(result.mobile_friendly); // 1.5s < 2s threshold
    }
}

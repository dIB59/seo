use super::{AuditResult, AuditScores, Auditor, CheckResult, Score, SeoAuditDetails};
use crate::service::spider::SpiderAgent;

use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};
use std::sync::{Arc, OnceLock};
use url::Url;

/// Returns a cached `&'static Selector` for a literal CSS selector. The selector is
/// parsed once on first use and reused thereafter — cheaper than re-parsing per call,
/// and removes the per-check `static OnceLock` boilerplate.
macro_rules! selector {
    ($css:literal) => {{
        static S: OnceLock<Selector> = OnceLock::new();
        S.get_or_init(|| Selector::parse($css).expect("invalid CSS selector"))
    }};
}

/// Builds a `CheckResult` for a "length must fall within `[min, max]`" check.
/// Used by both title and meta-description checks, which previously duplicated
/// the same 3-arm `if/else` and four `CheckResult` constructors each.
fn length_bounded_check(
    value: Option<String>,
    label: &str,
    min: usize,
    max: usize,
) -> CheckResult {
    match value {
        Some(v) if !v.is_empty() => {
            let len = v.len();
            let (passed, score, desc) = if len < min {
                (
                    false,
                    Score::from(0.5),
                    format!("{} too short ({} chars, recommend {}-{})", label, len, min, max),
                )
            } else if len > max {
                (
                    false,
                    Score::from(0.7),
                    format!("{} too long ({} chars, recommend {}-{})", label, len, min, max),
                )
            } else {
                (
                    true,
                    Score::from(1.0),
                    format!("{} length is good ({} chars)", label, len),
                )
            };
            CheckResult {
                passed,
                value: Some(v),
                score,
                description: Some(desc),
            }
        }
        _ => CheckResult {
            passed: false,
            value: None,
            score: Score::from(0.0),
            description: Some(format!("Missing {}", label.to_lowercase())),
        },
    }
}

/// Returns the value of `attr` on the first element matching `sel`, trimmed.
fn first_attr(document: &Html, sel: &Selector, attr: &str) -> Option<String> {
    document
        .select(sel)
        .next()
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.trim().to_string())
}

pub struct LightAuditor {
    spider: Arc<dyn SpiderAgent>,
}

impl LightAuditor {
    pub fn new(spider: Arc<dyn SpiderAgent>) -> Self {
        Self { spider }
    }

    fn analyze_html(&self, html: &str, url: &Url) -> (AuditScores, SeoAuditDetails) {
        let document = Html::parse_document(html);
        let details = self.extract_seo_details(&document, url);

        let seo_score = details.calculate_score();

        let scores = AuditScores {
            performance: None, // Light audit doesn't measure performance
            accessibility: None,
            best_practices: None,
            seo: Some(seo_score),
            seo_details: details.clone(),
            performance_metrics: None,
        };

        (scores, details)
    }

    fn extract_seo_details(&self, document: &Html, url: &Url) -> SeoAuditDetails {
        SeoAuditDetails {
            document_title: self.check_title(document),
            meta_description: self.check_meta_description(document),
            viewport: self.check_viewport(document),
            canonical: self.check_canonical(document, url),
            hreflang: self.check_hreflang(document),
            crawlable_anchors: self.check_crawlable_anchors(document),
            link_text: self.check_link_text(document),
            image_alt: self.check_image_alt(document),
            http_status_code: CheckResult {
                passed: true,
                score: Score::from(1.0),
                ..Default::default()
            },
            is_crawlable: self.check_is_crawlable(document),
        }
    }

    fn check_title(&self, document: &Html) -> CheckResult {
        let title = document
            .select(selector!("title"))
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string());
        let mut result = length_bounded_check(title, "Title", 30, 60);
        if result.value.is_none() {
            result.description = Some("Missing document title".to_string());
        }
        result
    }

    fn check_meta_description(&self, document: &Html) -> CheckResult {
        let description = first_attr(document, selector!("meta[name='description']"), "content");
        let mut result = length_bounded_check(description, "Description", 70, 160);
        if result.value.is_none() {
            result.description = Some("Missing meta description".to_string());
        }
        result
    }

    fn check_viewport(&self, document: &Html) -> CheckResult {
        let viewport = first_attr(document, selector!("meta[name='viewport']"), "content");

        match viewport {
            Some(v) if v.contains("width=device-width") => CheckResult {
                passed: true,
                value: Some(v),
                score: Score::from(1.0),
                description: Some("Viewport is properly configured".to_string()),
            },
            Some(v) => CheckResult {
                passed: false,
                value: Some(v),
                score: Score::from(0.5),
                description: Some("Viewport missing width=device-width".to_string()),
            },
            None => CheckResult {
                passed: false,
                value: None,
                score: Score::from(0.0),
                description: Some("Missing viewport meta tag".to_string()),
            },
        }
    }

    fn check_canonical(&self, document: &Html, page_url: &Url) -> CheckResult {
        let canonical = first_attr(document, selector!("link[rel='canonical']"), "href");

        match canonical {
            Some(c) if !c.is_empty() => {
                // Check if canonical matches current URL
                let matches = c == page_url.as_str()
                    || page_url
                        .join(&c)
                        .map(|u| u.as_str() == page_url.as_str())
                        .unwrap_or(false);
                CheckResult {
                    passed: true,
                    value: Some(c),
                    score: Score::from(1.0),
                    description: Some(if matches {
                        "Canonical URL matches page URL".to_string()
                    } else {
                        "Canonical URL points to different page".to_string()
                    }),
                }
            }
            _ => CheckResult {
                passed: false,
                value: None,
                score: Score::from(0.0),
                description: Some("Missing canonical URL".to_string()),
            },
        }
    }

    fn check_hreflang(&self, document: &Html) -> CheckResult {
        let count = document.select(selector!("link[rel='alternate'][hreflang]")).count();

        if count > 0 {
            CheckResult {
                passed: true,
                value: Some(format!("{} hreflang tags", count)),
                score: Score::from(1.0),
                description: Some(format!(
                    "Found {} hreflang tags for internationalization",
                    count
                )),
            }
        } else {
            // Hreflang is optional - not having it isn't necessarily bad
            CheckResult {
                passed: true,
                value: None,
                score: Score::from(1.0),
                description: Some(
                    "No hreflang tags (optional for single-language sites)".to_string(),
                ),
            }
        }
    }

    fn check_crawlable_anchors(&self, document: &Html) -> CheckResult {
        let mut total = 0;
        let mut uncrawlable = 0;

        for anchor in document.select(selector!("a[href]")) {
            total += 1;
            let href = anchor.value().attr("href").unwrap_or("");

            // Check for uncrawlable patterns
            if href.starts_with("javascript:")
                || href.starts_with("#") && href.len() == 1
                || href.is_empty()
            {
                uncrawlable += 1;
            }
        }

        if total == 0 {
            return CheckResult {
                passed: true,
                value: Some("0 links".to_string()),
                score: Score::from(1.0),
                description: Some("No links found on page".to_string()),
            };
        }

        let crawlable_pct = ((total - uncrawlable) as f64 / total as f64) * 100.0;
        let passed = uncrawlable == 0;
        let score = if passed {
            Score::from(1.0)
        } else {
            Score::from(crawlable_pct / 100.0)
        };

        CheckResult {
            passed,
            value: Some(format!("{}/{} crawlable", total - uncrawlable, total)),
            score,
            description: Some(if uncrawlable > 0 {
                format!(
                    "{} links are not crawlable (javascript: or empty href)",
                    uncrawlable
                )
            } else {
                "All links are crawlable".to_string()
            }),
        }
    }

    fn check_link_text(&self, document: &Html) -> CheckResult {
        let mut total = 0;
        let mut poor_text = 0;
        let bad_texts = ["click here", "read more", "learn more", "here", "link"];
        let img_selector = selector!("img");

        for anchor in document.select(selector!("a[href]")) {
            total += 1;

            // Primary: visible text inside the anchor
            let mut text = anchor.text().collect::<String>().trim().to_lowercase();

            // Fallbacks: aria-label or title attribute
            if text.is_empty() {
                if let Some(attr) = anchor
                    .value()
                    .attr("aria-label")
                    .or_else(|| anchor.value().attr("title"))
                {
                    text = attr.trim().to_lowercase();
                }
            }

            // Fallback: use alt text from first child img if present
            if text.is_empty() {
                for img in anchor.select(img_selector) {
                    if let Some(alt) = img.value().attr("alt") {
                        if !alt.trim().is_empty() {
                            text = alt.trim().to_lowercase();
                            break;
                        }
                    }
                }
            }

            // Normalize text: remove punctuation/symbols and collapse whitespace
            let normalized = text
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c.is_whitespace() {
                        c
                    } else {
                        ' '
                    }
                })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();

            if normalized.is_empty() || bad_texts.iter().any(|b| normalized.contains(b)) {
                poor_text += 1;
            }
        }

        if total == 0 {
            return CheckResult {
                passed: true,
                score: Score::from(1.0),
                value: None,
                description: Some("No links found".to_string()),
            };
        }

        let good_pct = ((total - poor_text) as f64 / total as f64) * 100.0;
        let passed = poor_text == 0;

        CheckResult {
            passed,
            value: Some(format!("{}/{} with good text", total - poor_text, total)),
            score: Score::from(good_pct / 100.0),
            description: Some(if poor_text > 0 {
                format!("{} links have generic/empty text", poor_text)
            } else {
                "All links have descriptive text".to_string()
            }),
        }
    }

    fn check_image_alt(&self, document: &Html) -> CheckResult {
        let mut total = 0;
        let mut missing_alt = 0;

        for img in document.select(selector!("img")) {
            total += 1;
            let alt = img.value().attr("alt");
            if alt.map_or(true, |a| a.trim().is_empty()) {
                missing_alt += 1;
            }
        }

        if total == 0 {
            return CheckResult {
                passed: true,
                score: Score::from(1.0),
                value: Some("0 images".to_string()),
                description: Some("No images found on page".to_string()),
            };
        }

        let with_alt = total - missing_alt;
        let score = Score::from(with_alt as f64 / total as f64);
        let passed = missing_alt == 0;

        CheckResult {
            passed,
            value: Some(format!("{}/{} with alt", with_alt, total)),
            score,
            description: Some(if missing_alt > 0 {
                format!("{} images missing alt attribute", missing_alt)
            } else {
                "All images have alt attributes".to_string()
            }),
        }
    }

    fn check_is_crawlable(&self, document: &Html) -> CheckResult {
        let robots = first_attr(document, selector!("meta[name='robots']"), "content")
            .map(|s| s.to_lowercase());

        match robots {
            Some(r) if r.contains("noindex") => CheckResult {
                passed: false,
                value: Some(r),
                score: Score::from(0.0),
                description: Some("Page has noindex directive".to_string()),
            },
            Some(r) => CheckResult {
                passed: true,
                value: Some(r),
                score: Score::from(1.0),
                description: Some("Page is crawlable".to_string()),
            },
            None => CheckResult {
                passed: true,
                value: None,
                score: Score::from(1.0),
                description: Some("No robots meta tag (page is crawlable by default)".to_string()),
            },
        }
    }
}

#[async_trait]
impl Auditor for LightAuditor {
    async fn analyze(&self, url: &str) -> Result<AuditResult> {
        tracing::info!("[LIGHT] Starting analysis: {}", url);
        let start_time = std::time::Instant::now();

        // Fetch the page
        let response = self.spider.get(url).await?;

        let final_url_str = response.url.clone();
        let final_url = Url::parse(&final_url_str).unwrap_or_else(|_| Url::parse(url).unwrap());

        let status_code = response.status;
        let html = response.body;

        let load_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let content_size = html.len();

        tracing::debug!(
            "[LIGHT] Fetched {} bytes in {:.2}ms",
            content_size,
            load_time_ms
        );

        // Analyze HTML
        let (mut scores, _details) = self.analyze_html(&html, &final_url);

        // Update HTTP status check based on actual status
        if status_code >= 400 {
            scores.seo_details.http_status_code = CheckResult {
                passed: false,
                value: Some(status_code.to_string()),
                score: Score::from(0.0),
                description: Some(format!("HTTP error status: {}", status_code)),
            };
            // Recalculate SEO score
            scores.seo = Some(scores.seo_details.calculate_score());
        }

        tracing::info!(
            "[LIGHT] Complete - status: {}, size: {} bytes, load: {:.2}ms, seo: {:.1}%",
            status_code,
            content_size,
            load_time_ms,
            scores.seo.map(|s| s.percent()).unwrap_or(0.0)
        );

        Ok(AuditResult {
            url: final_url_str,
            html,
            status_code,
            load_time_ms,
            content_size,
            scores,
        })
    }

    fn name(&self) -> &'static str {
        "Light (HTTP)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::spider::{ClientType, Spider};

    #[test]
    fn test_check_title() {
        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let auditor = LightAuditor::new(spider);

        // Good title (30-60 chars)
        let html = "<html><head><title>This Is a Good Title for SEO Testing Purposes</title></head></html>";
        let doc = Html::parse_document(html);
        let result = auditor.check_title(&doc);
        assert!(result.passed, "Title should pass: {:?}", result);
        assert_eq!(result.score, crate::service::auditor::Score::from(1.0));

        // Too short (<30 chars)
        let html = "<html><head><title>Short</title></head></html>";
        let doc = Html::parse_document(html);
        let result = auditor.check_title(&doc);
        assert!(!result.passed);
        assert_eq!(result.score, crate::service::auditor::Score::from(0.5));

        // Missing
        let html = "<html><head></head></html>";
        let doc = Html::parse_document(html);
        let result = auditor.check_title(&doc);
        assert!(!result.passed);
        assert_eq!(result.score, crate::service::auditor::Score::from(0.0));
    }

    #[test]
    fn test_check_image_alt() {
        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let auditor = LightAuditor::new(spider);

        // All images have alt
        let html = r#"<html><body><img src="a.jpg" alt="desc"><img src="b.jpg" alt="other"></body></html>"#;
        let doc = Html::parse_document(html);
        let result = auditor.check_image_alt(&doc);
        assert!(result.passed);
        assert_eq!(result.score, crate::service::auditor::Score::from(1.0));

        // One missing alt
        let html = r#"<html><body><img src="a.jpg" alt="desc"><img src="b.jpg"></body></html>"#;
        let doc = Html::parse_document(html);
        let result = auditor.check_image_alt(&doc);
        assert!(!result.passed);
        assert_eq!(result.score, crate::service::auditor::Score::from(0.5));
    }

    #[test]
    fn test_seo_score_calculation() {
        let details = SeoAuditDetails {
            document_title: CheckResult {
                passed: true,
                score: crate::service::auditor::Score::from(1.0),
                ..Default::default()
            },
            meta_description: CheckResult {
                passed: true,
                score: crate::service::auditor::Score::from(1.0),
                ..Default::default()
            },
            viewport: CheckResult {
                passed: true,
                score: crate::service::auditor::Score::from(1.0),
                ..Default::default()
            },
            canonical: CheckResult {
                passed: false,
                score: crate::service::auditor::Score::from(0.0),
                ..Default::default()
            },
            hreflang: CheckResult {
                passed: true,
                score: crate::service::auditor::Score::from(1.0),
                ..Default::default()
            },
            crawlable_anchors: CheckResult {
                passed: true,
                score: crate::service::auditor::Score::from(1.0),
                ..Default::default()
            },
            link_text: CheckResult {
                passed: true,
                score: crate::service::auditor::Score::from(1.0),
                ..Default::default()
            },
            image_alt: CheckResult {
                passed: true,
                score: crate::service::auditor::Score::from(1.0),
                ..Default::default()
            },
            http_status_code: CheckResult {
                passed: true,
                score: crate::service::auditor::Score::from(1.0),
                ..Default::default()
            },
            is_crawlable: CheckResult {
                passed: true,
                score: crate::service::auditor::Score::from(1.0),
                ..Default::default()
            },
        };

        let score = details.calculate_score();
        // 8/9 checks pass = ~0.889
        assert!(score.raw() > 0.8 && score.raw() < 0.95);
    }

    #[test]
    fn test_check_link_text_various() {
        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let auditor = LightAuditor::new(spider);

        // Good text
        let html = r#"<html><body><a href=\"/a\">Read this article</a></body></html>"#;
        let doc = Html::parse_document(html);
        let result = auditor.check_link_text(&doc);
        assert!(result.passed);
        assert_eq!(result.score, crate::service::auditor::Score::from(1.0));

        // Real-world example with classes (user-provided)
        let html = r#"<html><body><a class=\"px-6 py-3 border border-foreground/20 rounded-full font-medium hover:bg-foreground/5 transition-colors\" href=\"/leetcode\">LeetCode Progress</a></body></html>"#;
        let doc = Html::parse_document(html);
        let result = auditor.check_link_text(&doc);
        assert!(result.passed);
        assert_eq!(result.score, crate::service::auditor::Score::from(1.0));

        // Generic text with arrow/icon - should be flagged
        let html = r#"<html><body><a href=\"/a\">Read more →</a></body></html>"#;
        let doc = Html::parse_document(html);
        let result = auditor.check_link_text(&doc);
        assert!(!result.passed);
        assert_eq!(result.score, crate::service::auditor::Score::from(0.0));

        // Anchor with image alt text - should be considered descriptive
        let html = r#"<html><body><a href="a"><img src=\"a.jpg\" alt=\"Product image\"></a></body></html>"#;
        let doc = Html::parse_document(html);
        let result = auditor.check_link_text(&doc);
        assert!(result.passed);
        assert_eq!(result.score, crate::service::auditor::Score::from(1.0));

        // aria-label fallback
        let html =
            r#"<html><body><a href=\"/a\" aria-label=\"Download PDF\"><svg/></a></body></html>"#;
        let doc = Html::parse_document(html);
        let result = auditor.check_link_text(&doc);
        assert!(result.passed);
        assert_eq!(result.score, crate::service::auditor::Score::from(1.0));
    }
}

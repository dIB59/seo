//! Light Auditor - Fast HTTP-based SEO analysis with custom scoring.
//!
//! Performs quick SEO analysis using direct HTTP fetching and HTML parsing.
//! Much faster than Lighthouse (~1-2s vs ~5-10s) but less comprehensive.

use super::{AuditResult, AuditScores, Auditor, CheckResult, SeoAuditDetails, Score};
use crate::service::http::{create_client, ClientType};
use anyhow::Result;
use async_trait::async_trait;
use rquest::Client;
use scraper::{Html, Selector};
use std::sync::OnceLock;
use url::Url;

/// Light auditor using direct HTTP fetching.
///
/// Provides fast SEO analysis by:
/// - Direct HTTP request (no Chrome overhead)
/// - HTML parsing for SEO elements
/// - Custom scoring based on best practices
///
/// Trade-off: ~1-2 seconds per page, no JS rendering.
pub struct LightAuditor {
    client: Client,
}

impl LightAuditor {
    pub fn new() -> Self {
        Self {
            client: create_client(ClientType::HeavyEmulation)
                .expect("Failed to create HTTP client"),
        }
    }

    /// Analyze HTML and compute SEO scores.
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
            robots_txt: CheckResult::default(), // Checked separately
            crawlable_anchors: self.check_crawlable_anchors(document),
            link_text: self.check_link_text(document),
            image_alt: self.check_image_alt(document),
            http_status_code: CheckResult { passed: true, score: Score::from(1.0), ..Default::default() },
            is_crawlable: self.check_is_crawlable(document),
        }
    }

    fn check_title(&self, document: &Html) -> CheckResult {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("title").unwrap());

        let title = document
            .select(selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string());

        match title {
            Some(t) if !t.is_empty() => {
                let len = t.len();
                let (passed, score, desc) = if len < 30 {
                    (false, Score::from(0.5), format!("Title too short ({} chars, recommend 30-60)", len))
                } else if len > 60 {
                    (false, Score::from(0.7), format!("Title too long ({} chars, recommend 30-60)", len))
                } else {
                    (true, Score::from(1.0), format!("Title length is good ({} chars)", len))
                };
                CheckResult {
                    passed,
                    value: Some(t),
                    score,
                    description: Some(desc),
                }
            }
            _ => CheckResult {
                passed: false,
                value: None,
                score: Score::from(0.0),
                description: Some("Missing document title".to_string()),
            },
        }
    }

    fn check_meta_description(&self, document: &Html) -> CheckResult {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("meta[name='description']").unwrap()
        });

        let description = document
            .select(selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string());

        match description {
            Some(d) if !d.is_empty() => {
                let len = d.len();
                let (passed, score, desc) = if len < 70 {
                    (false, Score::from(0.5), format!("Description too short ({} chars, recommend 70-160)", len))
                } else if len > 160 {
                    (false, Score::from(0.7), format!("Description too long ({} chars, recommend 70-160)", len))
                } else {
                    (true, Score::from(1.0), format!("Description length is good ({} chars)", len))
                };
                CheckResult {
                    passed,
                    value: Some(d),
                    score,
                    description: Some(desc),
                }
            }
            _ => CheckResult {
                passed: false,
                value: None,
                score: Score::from(0.0),
                description: Some("Missing meta description".to_string()),
            },
        }
    }

    fn check_viewport(&self, document: &Html) -> CheckResult {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("meta[name='viewport']").unwrap()
        });

        let viewport = document
            .select(selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string());

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
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("link[rel='canonical']").unwrap()
        });

        let canonical = document
            .select(selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string());

        match canonical {
            Some(c) if !c.is_empty() => {
                // Check if canonical matches current URL
                let matches = c == page_url.as_str() 
                    || page_url.join(&c).map(|u| u.as_str() == page_url.as_str()).unwrap_or(false);
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
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("link[rel='alternate'][hreflang]").unwrap()
        });

        let count = document.select(selector).count();

        if count > 0 {
            CheckResult {
                passed: true,
                value: Some(format!("{} hreflang tags", count)),
                score: Score::from(1.0),
                description: Some(format!("Found {} hreflang tags for internationalization", count)),
            }
        } else {
            // Hreflang is optional - not having it isn't necessarily bad
            CheckResult {
                passed: true,
                value: None,
                score: Score::from(1.0),
                description: Some("No hreflang tags (optional for single-language sites)".to_string()),
            }
        }
    }

    fn check_crawlable_anchors(&self, document: &Html) -> CheckResult {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("a[href]").unwrap());

        let mut total = 0;
        let mut uncrawlable = 0;

        for anchor in document.select(selector) {
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
        let score = if passed { Score::from(1.0) } else { Score::from(crawlable_pct / 100.0) };

        CheckResult {
            passed,
            value: Some(format!("{}/{} crawlable", total - uncrawlable, total)),
            score,
            description: Some(if uncrawlable > 0 {
                format!("{} links are not crawlable (javascript: or empty href)", uncrawlable)
            } else {
                "All links are crawlable".to_string()
            }),
        }
    }

    fn check_link_text(&self, document: &Html) -> CheckResult {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("a[href]").unwrap());

        let mut total = 0;
        let mut poor_text = 0;
        let bad_texts = ["click here", "read more", "learn more", "here", "link"];

        for anchor in document.select(selector) {
            total += 1;
            let text = anchor.text().collect::<String>().trim().to_lowercase();
            
            if text.is_empty() || bad_texts.contains(&text.as_str()) {
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
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("img").unwrap());

        let mut total = 0;
        let mut missing_alt = 0;

        for img in document.select(selector) {
            total += 1;
            let alt = img.value().attr("alt");
            if alt.is_none() || alt.map(|a| a.trim().is_empty()).unwrap_or(true) {
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
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| {
            Selector::parse("meta[name='robots']").unwrap()
        });

        let robots = document
            .select(selector)
            .next()
            .and_then(|el| el.value().attr("content"))
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

impl Default for LightAuditor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Auditor for LightAuditor {
    async fn analyze(&self, url: &str) -> Result<AuditResult> {
        log::info!("[LIGHT] Starting analysis: {}", url);
        let start_time = std::time::Instant::now();

        let parsed_url = Url::parse(url)?;

        // Fetch the page
        let response = self.client
            .get(url)
            .send()
            .await?;

        let status_code = response.status().as_u16();
        let content_length = response.content_length();
        let html = response.text().await?;
        
        let load_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let content_size = content_length.unwrap_or(html.len() as u64) as usize;

        log::debug!("[LIGHT] Fetched {} bytes in {:.2}ms", content_size, load_time_ms);

        // Analyze HTML
        let (mut scores, _details) = self.analyze_html(&html, &parsed_url);

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

        log::info!(
            "[LIGHT] Complete - status: {}, size: {} bytes, load: {:.2}ms, seo: {:.1}%",
            status_code, content_size, load_time_ms,
            scores.seo.map(|s| s.percent()).unwrap_or(0.0)
        );

        Ok(AuditResult {
            url: url.to_string(),
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

    #[test]
    fn test_check_title() {
        let auditor = LightAuditor::new();
        
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
        let auditor = LightAuditor::new();

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
            document_title: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
            meta_description: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
            viewport: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
            canonical: CheckResult { passed: false, score: crate::service::auditor::Score::from(0.0), ..Default::default() },
            hreflang: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
            robots_txt: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
            crawlable_anchors: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
            link_text: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
            image_alt: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
            http_status_code: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
            is_crawlable: CheckResult { passed: true, score: crate::service::auditor::Score::from(1.0), ..Default::default() },
        };

        let score = details.calculate_score();
        // 8/9 checks pass = ~0.889
        assert!(score.raw() > 0.8 && score.raw() < 0.95);
    }
}

use super::checks::{
    CanonicalCheck, CrawlableAnchorsCheck, HreflangCheck, ImageAltCheck, IsCrawlableCheck,
    LinkTextCheck, MetaDescriptionCheck, PageContext, SeoCheck, TitleCheck, ViewportCheck,
};
use super::{AuditResult, AuditScores, Auditor, CheckResult, Score, SeoAuditDetails};
use crate::service::spider::SpiderAgent;

use anyhow::Result;
use async_trait::async_trait;
use scraper::Html;
use std::sync::Arc;
use url::Url;

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
        // Two of the nine SEO checks have been migrated to the new
        // `SeoCheck` trait in `super::checks`. The remaining seven still
        // live as private methods on this auditor and will be migrated
        // one-per-commit. See `service::auditor::checks` for the trait
        // contract and the migration plan.
        // All nine checks now run through the SeoCheck trait. Each rule
        // lives as its own struct in `super::checks` with isolated unit
        // tests. Adding a new built-in check is a new file, not a new
        // method on this 600-line struct.
        let ctx = PageContext { document, url };
        SeoAuditDetails {
            document_title: TitleCheck.evaluate(&ctx),
            meta_description: MetaDescriptionCheck.evaluate(&ctx),
            viewport: ViewportCheck.evaluate(&ctx),
            canonical: CanonicalCheck.evaluate(&ctx),
            hreflang: HreflangCheck.evaluate(&ctx),
            crawlable_anchors: CrawlableAnchorsCheck.evaluate(&ctx),
            link_text: LinkTextCheck.evaluate(&ctx),
            image_alt: ImageAltCheck.evaluate(&ctx),
            http_status_code: CheckResult {
                passed: true,
                score: Score::from(1.0),
                ..Default::default()
            },
            is_crawlable: IsCrawlableCheck.evaluate(&ctx),
        }
    }

}

#[async_trait]
impl Auditor for LightAuditor {
    async fn analyze_from_cache(&self, url: &str, cached: super::CachedHtml) -> Result<AuditResult> {
        tracing::info!("[LIGHT] Using cached HTML for: {} ({} bytes)", url, cached.html.len());

        let final_url_str = cached.final_url;
        let final_url = match Url::parse(&final_url_str) {
            Ok(u) => u,
            Err(_) => Url::parse(url)
                .map_err(|e| anyhow::anyhow!("invalid analysis URL '{url}': {e}"))?,
        };

        let status_code = cached.status_code;
        let html = cached.html;
        let load_time_ms = cached.load_time_ms;
        let content_size = html.len();

        let (mut scores, _details) = self.analyze_html(&html, &final_url);

        if status_code >= 400 {
            scores.seo_details.http_status_code = CheckResult {
                passed: false,
                value: Some(status_code.to_string()),
                score: Score::from(0.0),
                description: Some(format!("HTTP error status: {}", status_code)),
            };
            scores.seo = Some(scores.seo_details.calculate_score());
        }

        tracing::info!(
            "[LIGHT] Cached analysis complete - status: {}, size: {} bytes, load: {:.2}ms, seo: {:.1}%",
            status_code, content_size, load_time_ms,
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

    async fn analyze(&self, url: &str) -> Result<AuditResult> {
        tracing::info!("[LIGHT] Starting analysis: {}", url);
        let start_time = std::time::Instant::now();

        // Fetch the page
        let response = self.spider.get(url).await?;

        let final_url_str = response.url.clone();
        // Prefer the spider's final (post-redirect) URL; fall back to the
        // original input on parse failure. The original `unwrap()` here
        // would panic if `url` was malformed — which the spider has
        // already proven possible by returning a body for it. Map both
        // through `?` so a doubly-bad URL surfaces as a typed error
        // instead of a thread crash.
        let final_url = match Url::parse(&final_url_str) {
            Ok(u) => u,
            Err(_) => Url::parse(url)
                .map_err(|e| anyhow::anyhow!("invalid analysis URL '{url}': {e}"))?,
        };

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
    fn extract_seo_details_delegates_to_title_check() {
        // The fine-grained TitleCheck behavior is covered by dedicated unit
        // tests in `super::checks`. This smoke test pins the wiring: that
        // `extract_seo_details` actually plumbs the trait result into
        // `SeoAuditDetails::document_title`.
        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let auditor = LightAuditor::new(spider);
        let html = "<html><head><title>This Is a Good Title for SEO Testing Purposes</title></head></html>";
        let doc = Html::parse_document(html);
        let url = url::Url::parse("https://example.com/").unwrap();
        let details = auditor.extract_seo_details(&doc, &url);
        assert!(details.document_title.passed);
        assert_eq!(
            details.document_title.score,
            crate::service::auditor::Score::from(1.0)
        );
    }

    // The fine-grained behavior of every check is now covered by dedicated
    // unit tests in `super::checks`. The smoke tests in this module pin the
    // wiring: that `extract_seo_details` plumbs each trait result into the
    // matching `SeoAuditDetails` field.

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
    fn extract_seo_details_runs_all_nine_checks() {
        // Smoke test for the orchestration: every field on
        // `SeoAuditDetails` should be populated by its corresponding
        // `SeoCheck` impl. A blank document means most checks fail; we
        // only assert that the orchestration ran without panicking and
        // populated each slot.
        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let auditor = LightAuditor::new(spider);
        let doc = Html::parse_document("<html><head></head><body></body></html>");
        let url = url::Url::parse("https://example.com/").unwrap();
        let details = auditor.extract_seo_details(&doc, &url);

        // Each check runs and produces a description.
        assert!(details.document_title.description.is_some());
        assert!(details.meta_description.description.is_some());
        assert!(details.viewport.description.is_some());
        assert!(details.canonical.description.is_some());
        assert!(details.hreflang.description.is_some());
        assert!(details.crawlable_anchors.description.is_some());
        assert!(details.link_text.description.is_some());
        assert!(details.image_alt.description.is_some());
        assert!(details.is_crawlable.description.is_some());
    }
}

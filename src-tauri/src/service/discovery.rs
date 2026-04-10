use anyhow::Result;
use scraper::Html;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::contexts::analysis::ResourceStatus;
use crate::service::spider::SpiderAgent;
use std::sync::Arc;

#[cfg(test)]
use crate::service::spider::{ClientType, Spider};

/// Page data captured during discovery. Stored in the `page_queue` DB
/// table so the analysis phase can skip re-fetching the same URL.
#[derive(Debug, Clone)]
pub struct DiscoveredPage {
    pub url: String,
    /// Final URL after any redirects.
    pub final_url: String,
    pub html: String,
    pub status_code: u16,
    pub load_time_ms: f64,
}

/// Result of a site-level resource check (robots.txt presence,
/// sitemap presence, HTTPS).
///
/// Fields are private to enforce that any `SiteResources` value flows
/// through the `new` constructor — that's where the (potential)
/// validation lives. Today there's no validation, but the
/// encapsulation gives us a single place to add it later without a
/// cascade through every read site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteResources {
    robots_txt: bool,
    sitemap: bool,
    ssl: bool,
}

impl SiteResources {
    pub fn new(robots_txt: bool, sitemap: bool, ssl: bool) -> Self {
        Self { robots_txt, sitemap, ssl }
    }

    pub fn robots_txt(&self) -> bool {
        self.robots_txt
    }

    pub fn sitemap(&self) -> bool {
        self.sitemap
    }

    pub fn ssl(&self) -> bool {
        self.ssl
    }

    /// Whether all three site resources are present. Useful for the
    /// "fully optimized" UI badge.
    pub fn all_present(&self) -> bool {
        self.robots_txt && self.sitemap && self.ssl
    }
}

pub struct PageDiscovery {
    spider: Arc<dyn SpiderAgent>,
}

impl PageDiscovery {
    pub fn new(spider: Arc<dyn SpiderAgent>) -> Self {
        Self { spider }
    }

    pub async fn discover(
        &self,
        start_url_str: &str,
        max_pages: i64,
        delay_ms: i64,
        include_subdomains: bool,
        cancel_token: &CancellationToken,
        on_discovered: impl Fn(usize) + Send + Sync,
    ) -> Result<Vec<DiscoveredPage>> {
        let start_url = Url::parse(start_url_str)?;
        tracing::info!("[DISCOVERY] Starting page discovery from: {}", start_url);
        tracing::debug!(
            "[DISCOVERY] Max pages: {}, Delay: {}ms",
            max_pages,
            delay_ms
        );

        let mut visited: HashSet<Url> = HashSet::new();
        let mut discovered_pages: Vec<DiscoveredPage> = Vec::new();
        let mut to_visit = vec![start_url.clone()];

        let base_host = start_url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid host"))?;
        let base_port = start_url.port();
        tracing::debug!(
            "[DISCOVERY] Base host: {}, port: {:?}",
            base_host,
            base_port
        );

        while let Some(url) = to_visit.pop() {
            if cancel_token.is_cancelled() {
                tracing::warn!(
                    "[DISCOVERY] Discovery cancelled by user at {} pages",
                    visited.len()
                );
                return Ok(discovered_pages);
            }
            if visited.contains(&url) {
                tracing::trace!("[DISCOVERY] Skipping already visited: {}", url);
                continue;
            }

            if visited.len() >= max_pages as usize {
                tracing::info!("[DISCOVERY] Reached max pages limit: {}", max_pages);
                break;
            }

            visited.insert(url.clone());
            tracing::info!(
                "[DISCOVERY] Discovered page {}/{}: {}",
                visited.len(),
                max_pages,
                url
            );
            on_discovered(visited.len());

            if delay_ms > 0 {
                tracing::trace!("[DISCOVERY] Waiting {}ms before next request", delay_ms);
            }
            sleep(Duration::from_millis(delay_ms as u64)).await;

            tracing::trace!("[DISCOVERY] Fetching page: {}", url);
            let fetch_start = std::time::Instant::now();
            let Ok(response) = self.spider.get(url.as_str()).await else {
                tracing::debug!("[DISCOVERY] Failed to fetch: {}", url);
                continue;
            };

            let load_time_ms = fetch_start.elapsed().as_secs_f64() * 1000.0;
            let body = response.body;
            let status_code = response.status as u16;
            let final_url = response.url.clone();
            tracing::trace!("[DISCOVERY] Received {} bytes from {}", body.len(), url);

            // Cache the fetched data so analysis can skip re-fetching
            discovered_pages.push(DiscoveredPage {
                url: url.to_string(),
                final_url: final_url.clone(),
                html: body.clone(),
                status_code,
                load_time_ms,
            });

            let links: Vec<Url> = Self::extract_links(&body, &url)
                .into_iter()
                .filter_map(|s| Url::parse(&s).ok())
                .collect();

            tracing::debug!("[DISCOVERY] Found {} links on {}", links.len(), url);

            let mut new_links_count = 0;
            for link in links {
                // Both URLs are already parsed — use the URL-aware
                // classifier so we don't re-parse them.
                let link_type =
                    crate::contexts::link::NewLink::classify_urls(&link, &start_url);

                let should_follow = link_type.should_follow(include_subdomains);

                if should_follow && !visited.contains(&link) && !to_visit.contains(&link) {
                    to_visit.push(link);
                    new_links_count += 1;
                }
            }
            tracing::trace!(
                "[DISCOVERY] Queued {} new internal links (queue size: {})",
                new_links_count,
                to_visit.len()
            );
        }

        tracing::info!(
            "[DISCOVERY] Discovery complete - found {} pages",
            discovered_pages.len()
        );
        Ok(discovered_pages)
    }

    pub fn extract_links(html: &str, base_url: &Url) -> Vec<String> {
        Html::parse_document(html)
            .select(cached_selector!("a[href]"))
            .filter_map(|a| a.value().attr("href"))
            .filter(|raw| !raw.starts_with('#'))
            .filter_map(|raw| base_url.join(raw).ok())
            .map(|mut u| {
                u.set_fragment(None);
                u.to_string()
            })
            .collect()
    }
}

pub struct ResourceChecker {
    spider: Arc<dyn SpiderAgent>,
}

impl ResourceChecker {
    pub fn new(spider: Arc<dyn SpiderAgent>) -> Self {
        Self { spider }
    }

    pub async fn check_robots_txt(&self, base_url_str: &str) -> Result<ResourceStatus> {
        tracing::debug!("[RESOURCE] Checking robots.txt for {}", base_url_str);
        self.check_resource(base_url_str, "robots.txt").await
    }

    pub async fn check_sitemap_xml(&self, base_url_str: &str) -> Result<ResourceStatus> {
        tracing::debug!("[RESOURCE] Checking sitemap.xml for {}", base_url_str);
        self.check_resource(base_url_str, "sitemap.xml").await
    }

    pub fn check_ssl_certificate(&self, url_str: &str) -> bool {
        let has_ssl = url_str.starts_with("https");
        tracing::debug!("[RESOURCE] SSL check for {}: {}", url_str, has_ssl);
        has_ssl
    }

    async fn check_resource(&self, base_url_str: &str, path: &str) -> Result<ResourceStatus> {
        let base_url = Url::parse(base_url_str)?;
        let resource_url = base_url.join(path)?;
        tracing::trace!("[RESOURCE] Fetching: {}", resource_url);
        let response = self.spider.get(resource_url.as_str()).await?;

        let status = match response.status {
            200 => {
                tracing::debug!("[RESOURCE] Found: {}", resource_url);
                ResourceStatus::Found(resource_url.to_string())
            }
            401 | 403 => {
                tracing::debug!("[RESOURCE] Unauthorized: {}", resource_url);
                ResourceStatus::Unauthorized(resource_url.to_string())
            }
            404 => {
                tracing::debug!("[RESOURCE] Not found: {}", resource_url);
                ResourceStatus::NotFound
            }
            status => {
                tracing::debug!(
                    "[RESOURCE] Unexpected status {} for: {}",
                    status,
                    resource_url
                );
                ResourceStatus::NotFound
            }
        };

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links() {
        let base_url = Url::parse("https://example.com").unwrap();
        let html = r##"
            <html>
                <body>
                    <a href="/relative">Relative</a>
                    <a href="https://other.com/absolute">Absolute</a>
                    <a href="#fragment">Fragment Only</a>
                    <a href="/page#section">Page with Fragment</a>
                    <a>No Href</a>
                </body>
            </html>
        "##;
        let links = PageDiscovery::extract_links(html, &base_url);

        assert_eq!(links.len(), 3);
        assert!(links.contains(&"https://example.com/relative".to_string()));
        assert!(links.contains(&"https://other.com/absolute".to_string()));
        assert!(links.contains(&"https://example.com/page".to_string()));
        assert!(!links.iter().any(|l| l.contains("#")));
    }

    #[test]
    fn extract_links_strips_fragment_from_resolved_url() {
        // Pinning that #section is dropped (the deduplication relies on
        // the hash-stripped form, otherwise /page#a and /page#b would
        // both appear as separate URLs).
        let base = Url::parse("https://example.com/").unwrap();
        let html = r##"<html><body>
            <a href="/x#one">One</a>
            <a href="/x#two">Two</a>
            <a href="/x">Plain</a>
        </body></html>"##;
        let links = PageDiscovery::extract_links(html, &base);
        // All three resolve to the same canonical /x URL.
        assert_eq!(links.iter().filter(|l| l == &"https://example.com/x").count(), 3);
    }

    #[test]
    fn extract_links_keeps_query_string() {
        let base = Url::parse("https://example.com/").unwrap();
        let html = r##"<html><body>
            <a href="/search?q=rust">A</a>
            <a href="/search?q=rust&page=2">B</a>
        </body></html>"##;
        let links = PageDiscovery::extract_links(html, &base);
        assert!(links.contains(&"https://example.com/search?q=rust".to_string()));
        assert!(links.contains(&"https://example.com/search?q=rust&page=2".to_string()));
    }

    #[test]
    fn extract_links_handles_protocol_relative_urls() {
        // `//cdn.example.com/script.js` should resolve against the
        // base scheme.
        let base = Url::parse("https://example.com/").unwrap();
        let html = r##"<html><body><a href="//cdn.example.com/lib">CDN</a></body></html>"##;
        let links = PageDiscovery::extract_links(html, &base);
        assert!(links.contains(&"https://cdn.example.com/lib".to_string()));
    }

    #[test]
    fn extract_links_skips_unparseable_hrefs_silently() {
        // url::Url::join is very forgiving — most "bad" hrefs still
        // resolve. javascript: schemes resolve to a Url with the
        // javascript scheme, so they DO end up in the output. The
        // discovery service relies on the downstream classifier to
        // reject those, not extract_links itself. Pin that today.
        let base = Url::parse("https://example.com/").unwrap();
        let html = r##"<html><body>
            <a href="javascript:void(0)">JS</a>
            <a href="mailto:hi@example.com">Mail</a>
            <a href="tel:+15551234">Phone</a>
        </body></html>"##;
        let links = PageDiscovery::extract_links(html, &base);
        // None of these are crawlable HTTP URLs, but extract_links is
        // permissive — it just resolves whatever url::Url accepts.
        // The behaviour is documented here so a future filter pass
        // is an explicit decision, not an accident.
        for link in &links {
            // No assertion on count — just smoke test that it doesn't
            // panic and yields valid URL strings.
            let _ = Url::parse(link).expect("each emitted link parses");
        }
    }

    #[test]
    fn extract_links_handles_empty_document() {
        let base = Url::parse("https://example.com/").unwrap();
        let links = PageDiscovery::extract_links("<html></html>", &base);
        assert!(links.is_empty());
    }

    #[test]
    fn extract_links_handles_no_anchors() {
        let base = Url::parse("https://example.com/").unwrap();
        let html = "<html><body><p>No links here</p></body></html>";
        let links = PageDiscovery::extract_links(html, &base);
        assert!(links.is_empty());
    }

    // ── SiteResources ────────────────────────────────────────────────────

    #[test]
    fn site_resources_constructor_assigns_fields() {
        let r = SiteResources::new(true, false, true);
        assert!(r.robots_txt());
        assert!(!r.sitemap());
        assert!(r.ssl());
    }

    #[test]
    fn site_resources_all_present_requires_every_field_true() {
        assert!(SiteResources::new(true, true, true).all_present());
        assert!(!SiteResources::new(false, true, true).all_present());
        assert!(!SiteResources::new(true, false, true).all_present());
        assert!(!SiteResources::new(true, true, false).all_present());
        assert!(!SiteResources::new(false, false, false).all_present());
    }

    #[test]
    fn site_resources_equality_ignores_construction_path() {
        // Two SiteResources with the same field values are equal
        // regardless of how they were constructed.
        let a = SiteResources::new(true, true, false);
        let b = SiteResources::new(true, true, false);
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn test_check_robots_txt_found() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/robots.txt")
            .with_status(200)
            .with_body("User-agent: *\nDisallow:")
            .create_async()
            .await;

        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let checker = ResourceChecker::new(spider);
        let base_url = Url::parse(&server.url()).unwrap();

        let status = checker.check_robots_txt(base_url.as_str()).await.unwrap();
        assert!(status.exists());
    }

    #[tokio::test]
    async fn test_check_ssl_certificate() {
        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let checker = ResourceChecker::new(spider);
        assert!(checker.check_ssl_certificate("https://google.com"));
        assert!(!checker.check_ssl_certificate("http://google.com"));
    }
}

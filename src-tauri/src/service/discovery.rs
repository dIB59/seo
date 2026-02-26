use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::contexts::ResourceStatus;
use crate::service::spider::SpiderAgent;
use std::sync::Arc;

#[cfg(test)]
use crate::service::spider::{ClientType, Spider};

#[derive(Debug, Clone)]
pub struct SiteResources {
    pub robots_txt: bool,
    pub sitemap: bool,
    pub ssl: bool,
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
    ) -> Result<Vec<String>> {
        let start_url = Url::parse(start_url_str)?;
        tracing::info!("[DISCOVERY] Starting page discovery from: {}", start_url);
        tracing::debug!(
            "[DISCOVERY] Max pages: {}, Delay: {}ms",
            max_pages,
            delay_ms
        );

        let mut visited: HashSet<Url> = HashSet::new();
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
                return Ok(visited.into_iter().map(|u: Url| u.to_string()).collect());
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
            let Ok(response) = self.spider.get(url.as_str()).await else {
                tracing::debug!("[DISCOVERY] Failed to fetch: {}", url);
                continue;
            };

            let body = response.body;
            tracing::trace!("[DISCOVERY] Received {} bytes from {}", body.len(), url);

            let links: Vec<Url> = Self::extract_links(&body, &url)
                .into_iter()
                .filter_map(|s| Url::parse(&s).ok())
                .collect();

            tracing::debug!("[DISCOVERY] Found {} links on {}", links.len(), url);

            let mut new_links_count = 0;
            for link in links {
                let link_type =
                    crate::contexts::link::NewLink::classify(link.as_str(), start_url_str);

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
            visited.len()
        );
        Ok(visited.into_iter().map(|u: Url| u.to_string()).collect())
    }

    pub fn extract_links(html: &str, base_url: &Url) -> Vec<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("a[href]").unwrap());

        Html::parse_document(html)
            .select(selector)
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

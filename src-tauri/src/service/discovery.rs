//! Page discovery and resource checking services

use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

use crate::domain::models::ResourceStatus;

use crate::service::http::{create_client, ClientType};
use rquest::Client;

pub struct PageDiscovery {
    client: Client,
}

impl PageDiscovery {
    pub fn new() -> Self {
        Self {
            client: create_client(ClientType::HeavyEmulation)
                .expect("Failed to create heavy HTTP client"),
        }
    }

    /// ONLY handles HTTP crawling - NO business logic
    pub async fn discover(
        &self,
        start_url: Url,
        max_pages: i64,
        delay_ms: i64,
        cancel_flag: &AtomicBool,
        on_discovered: impl Fn(usize) + Send + Sync,
    ) -> Result<Vec<Url>> {
        log::info!("[DISCOVERY] Starting page discovery from: {}", start_url);
        log::debug!("[DISCOVERY] Max pages: {}, Delay: {}ms", max_pages, delay_ms);
        
        let mut visited = HashSet::new();
        let mut to_visit = vec![start_url.clone()];

        let base_host = start_url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid host"))?;
        let base_port = start_url.port();
        log::debug!("[DISCOVERY] Base host: {}, port: {:?}", base_host, base_port);

        while let Some(url) = to_visit.pop() {
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                log::warn!("[DISCOVERY] Discovery cancelled by user at {} pages", visited.len());
                return Ok(visited.into_iter().collect());
            }
            if visited.contains(&url) {
                log::trace!("[DISCOVERY] Skipping already visited: {}", url);
                continue;
            }

            if visited.len() >= max_pages as usize {
                log::info!("[DISCOVERY] Reached max pages limit: {}", max_pages);
                break;
            }

            visited.insert(url.clone());
            log::info!("[DISCOVERY] Discovered page {}/{}: {}", visited.len(), max_pages, url);
            on_discovered(visited.len());

            if delay_ms > 0 {
                log::trace!("[DISCOVERY] Waiting {}ms before next request", delay_ms);
            }
            sleep(Duration::from_millis(delay_ms as u64)).await;

            log::trace!("[DISCOVERY] Fetching page: {}", url);
            let Ok(response) = self.client.get(url.as_str()).send().await else {
                log::debug!("[DISCOVERY] Failed to fetch: {}", url);
                continue;
            };

            let Ok(body) = response.text().await else {
                log::debug!("[DISCOVERY] Failed to read response body for: {}", url);
                continue;
            };
            log::trace!("[DISCOVERY] Received {} bytes from {}", body.len(), url);

            let document = Html::parse_document(&body);
            let links: Vec<Url> = self
                .extract_links(&document, &url)?
                .into_iter()
                .map(|mut u| {
                    u.set_fragment(None);
                    u
                })
                .collect();
            
            log::debug!("[DISCOVERY] Found {} links on {}", links.len(), url);

            let mut new_links_count = 0;
            for link in links {
                if link.host_str() == Some(base_host)
                    && link.port() == base_port
                    && !visited.contains(&link)
                    && !to_visit.contains(&link)
                {
                    to_visit.push(link);
                    new_links_count += 1;
                }
            }
            log::trace!("[DISCOVERY] Queued {} new internal links (queue size: {})", new_links_count, to_visit.len());
        }

        log::info!("[DISCOVERY] Discovery complete - found {} pages", visited.len());
        Ok(visited.into_iter().collect())
    }

    fn extract_links(&self, document: &Html, base_url: &Url) -> Result<Vec<Url>> {
        let selector = Selector::parse("a[href]").unwrap();
        let mut links = Vec::new();

        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(url) = base_url.join(href) {
                    links.push(url);
                }
            }
        }

        Ok(links)
    }
}

pub struct ResourceChecker {
    client: Client,
}

impl ResourceChecker {
    pub fn new() -> Self {
        Self {
            client: create_client(ClientType::HeavyEmulation)
                .expect("Failed to create heavy HTTP client"),
        }
    }

    /// Check robots.txt exists
    pub async fn check_robots_txt(&self, base_url: Url) -> Result<ResourceStatus> {
        log::debug!("[RESOURCE] Checking robots.txt for {}", base_url);
        self.check_resource(base_url, "robots.txt").await
    }

    /// Check sitemap.xml exists
    pub async fn check_sitemap_xml(&self, base_url: Url) -> Result<ResourceStatus> {
        log::debug!("[RESOURCE] Checking sitemap.xml for {}", base_url);
        self.check_resource(base_url, "sitemap.xml").await
    }

    /// Check SSL certificate (HTTPS)
    pub fn check_ssl_certificate(&self, url: &Url) -> bool {
        let has_ssl = url.scheme() == "https";
        log::debug!("[RESOURCE] SSL check for {}: {}", url, has_ssl);
        has_ssl
    }

    async fn check_resource(&self, base_url: Url, path: &str) -> Result<ResourceStatus> {
        let resource_url = base_url.join(path)?;
        log::trace!("[RESOURCE] Fetching: {}", resource_url);
        let response = self.client.get(resource_url.clone()).send().await?;

        let status = match response.status() {
            rquest::StatusCode::OK => {
                log::debug!("[RESOURCE] Found: {}", resource_url);
                ResourceStatus::Found(resource_url.to_string())
            }
            rquest::StatusCode::UNAUTHORIZED | rquest::StatusCode::FORBIDDEN => {
                log::debug!("[RESOURCE] Unauthorized: {}", resource_url);
                ResourceStatus::Unauthorized(resource_url.to_string())
            }
            rquest::StatusCode::NOT_FOUND => {
                log::debug!("[RESOURCE] Not found: {}", resource_url);
                ResourceStatus::NotFound
            }
            status => {
                log::debug!("[RESOURCE] Unexpected status {} for: {}", status, resource_url);
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
        let discovery = PageDiscovery::new();
        let base_url = Url::parse("https://example.com").unwrap();
        let html = r##"
            <html>
                <body>
                    <a href="/relative">Relative</a>
                    <a href="https://other.com/absolute">Absolute</a>
                    <a href="#fragment">Fragment</a>
                    <a>No Href</a>
                </body>
            </html>
        "##;
        let document = Html::parse_document(html);
        let links = discovery.extract_links(&document, &base_url).unwrap();

        // Should return 3 links (relative resolved, absolute, and fragment one resolved)
        assert_eq!(links.len(), 3);

        assert!(links.contains(&Url::parse("https://example.com/relative").unwrap()));
        assert!(links.contains(&Url::parse("https://other.com/absolute").unwrap()));
        assert!(links.contains(&Url::parse("https://example.com/#fragment").unwrap()));
    }

    #[test]
    fn test_check_ssl_certificate() {
        let checker = ResourceChecker::new();

        let https_url = Url::parse("https://secure.com").unwrap();
        assert!(
            checker.check_ssl_certificate(&https_url),
            "HTTPS should be detected as SSL"
        );

        let http_url = Url::parse("http://insecure.com").unwrap();
        assert!(
            !checker.check_ssl_certificate(&http_url),
            "HTTP should not be detected as SSL"
        );
    }

    // ===== Integration tests for ResourceChecker using mock server =====

    #[tokio::test]
    async fn test_check_robots_txt_found() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/robots.txt")
            .with_status(200)
            .with_body("User-agent: *\nDisallow:")
            .create_async()
            .await;

        let checker = ResourceChecker::new();
        let base_url = Url::parse(&server.url()).unwrap();

        let status = checker.check_robots_txt(base_url).await.unwrap();
        assert!(status.exists(), "robots.txt should be detected as found");
        assert!(matches!(status, ResourceStatus::Found(_)));
    }

    #[tokio::test]
    async fn test_check_robots_txt_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/robots.txt")
            .with_status(404)
            .create_async()
            .await;

        let checker = ResourceChecker::new();
        let base_url = Url::parse(&server.url()).unwrap();

        let status = checker.check_robots_txt(base_url).await.unwrap();
        assert!(
            !status.exists(),
            "robots.txt should be detected as not found"
        );
        assert!(matches!(status, ResourceStatus::NotFound));
    }

    #[tokio::test]
    async fn test_check_sitemap_xml_found() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/sitemap.xml")
            .with_status(200)
            .with_body("<urlset></urlset>")
            .create_async()
            .await;

        let checker = ResourceChecker::new();
        let base_url = Url::parse(&server.url()).unwrap();

        let status = checker.check_sitemap_xml(base_url).await.unwrap();
        assert!(status.exists(), "sitemap.xml should be detected as found");
    }

    #[tokio::test]
    async fn test_check_resource_unauthorized() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/robots.txt")
            .with_status(401)
            .create_async()
            .await;

        let checker = ResourceChecker::new();
        let base_url = Url::parse(&server.url()).unwrap();

        let status = checker.check_robots_txt(base_url).await.unwrap();
        // Unauthorized still means the resource "exists" (it's just protected)
        assert!(status.exists(), "Unauthorized should still count as exists");
        assert!(matches!(status, ResourceStatus::Unauthorized(_)));
    }
}

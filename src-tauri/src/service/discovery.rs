//! Page discovery and resource checking services

use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

use crate::domain::models::ResourceStatus;

pub struct PageDiscovery {
    client: reqwest::Client,
}

impl PageDiscovery {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// ONLY handles HTTP crawling - NO business logic
    pub async fn discover(
        &self,
        start_url: Url,
        max_pages: i64,
        delay_ms: i64,
        cancel_flag: &AtomicBool,
    ) -> Result<Vec<Url>> {
        let mut visited = HashSet::new();
        let mut to_visit = vec![start_url.clone()];

        let base_host = start_url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid host"))?;
        let base_port = start_url.port();

        while let Some(url) = to_visit.pop() {
            if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
                log::info!("Page discovery cancelled for {}", start_url);
                return Ok(visited.into_iter().collect());
            }
            if visited.contains(&url) {
                continue;
            }

            if visited.len() >= max_pages as usize {
                break;
            }

            visited.insert(url.clone());
            sleep(Duration::from_millis(delay_ms as u64)).await;

            let Ok(response) = self.client.get(url.as_str()).send().await else {
                continue;
            };

            let Ok(body) = response.text().await else {
                continue;
            };

            let document = Html::parse_document(&body);
            let links: Vec<Url> = self
                .extract_links(&document, &url)?
                .into_iter()
                .map(|mut u| {
                    u.set_fragment(None);
                    u
                })
                .collect();

            for link in links {
                if link.host_str() == Some(base_host)
                    && link.port() == base_port
                    && !visited.contains(&link)
                    && !to_visit.contains(&link)
                {
                    to_visit.push(link);
                }
            }
        }

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
    client: reqwest::Client,
}

impl ResourceChecker {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Check robots.txt exists
    pub async fn check_robots_txt(&self, base_url: Url) -> Result<ResourceStatus> {
        self.check_resource(base_url, "robots.txt").await
    }

    /// Check sitemap.xml exists
    pub async fn check_sitemap_xml(&self, base_url: Url) -> Result<ResourceStatus> {
        self.check_resource(base_url, "sitemap.xml").await
    }

    /// Check SSL certificate (HTTPS)
    pub fn check_ssl_certificate(&self, url: &Url) -> bool {
        url.scheme() == "https"
    }

    async fn check_resource(&self, base_url: Url, path: &str) -> Result<ResourceStatus> {
        let resource_url = base_url.join(path)?;
        let response = self.client.get(resource_url.clone()).send().await?;

        let status = match response.status() {
            reqwest::StatusCode::OK => ResourceStatus::Found(resource_url.to_string()),
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                ResourceStatus::Unauthorized(resource_url.to_string())
            }
            reqwest::StatusCode::NOT_FOUND => ResourceStatus::NotFound,
            _ => ResourceStatus::NotFound,
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
        assert!(checker.check_ssl_certificate(&https_url));

        let http_url = Url::parse("http://insecure.com").unwrap();
        assert!(!checker.check_ssl_certificate(&http_url));
    }
}

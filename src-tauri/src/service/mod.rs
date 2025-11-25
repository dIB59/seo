//! Thin services - only handle HTTP/IO, delegate to domain

use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::HashSet;
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
    ) -> Result<Vec<Url>> {
        let mut visited = HashSet::new();
        let mut to_visit = vec![start_url.clone()];
        
        let base_host = start_url.host_str().ok_or_else(|| anyhow::anyhow!("Invalid host"))?;
        let base_port = start_url.port();
        
        while let Some(url) = to_visit.pop() {
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
            let links = self.extract_links(&document, &url)?;
            
            for link in links {
                if link.host_str() == Some(base_host) && link.port() == base_port
                    && !visited.contains(&link) && !to_visit.contains(&link) {
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
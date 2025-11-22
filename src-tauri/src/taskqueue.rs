use anyhow::{anyhow, Context, Error, Result};
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use std::{collections::HashSet, fs::create_dir};
use tauri::http::response;
use url::Url;

use crate::extractor::sitemap::SITE_MAP_PATH;

const ROBOTS_TXT_PATH: &str = "robots.txt";

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceStatus {
    Found(String),        // Resource exists and is accessible
    Unauthorized(String), // Resource exists but requires authentication (401/403)
    NotFound,             // Resource doesn't exist (404)
}

impl ResourceStatus {
    /// Check if resource was successfully found
    pub fn is_accessible(&self) -> bool {
        matches!(self, ResourceStatus::Found(_))
    }

    /// Check if resource exists (even if unauthorized)
    pub fn exists(&self) -> bool {
        matches!(
            self,
            ResourceStatus::Found(_) | ResourceStatus::Unauthorized(_)
        )
    }

    /// Get the resource URL if it's found or unauthorized
    pub fn url(&self) -> Option<&str> {
        match self {
            ResourceStatus::Found(url) | ResourceStatus::Unauthorized(url) => Some(url),
            _ => None,
        }
    }
}

pub async fn check_resource(base_url: Url, path: &str) -> Result<ResourceStatus> {
    let client = Client::new();

    let resource_url = base_url
        .join(path)
        .with_context(|| format!("Failed to construct URL from {} and {}", base_url, path))?;

    let response = client
        .get(resource_url.clone())
        .send()
        .await
        .with_context(|| format!("Failed to request {}", resource_url))?;

    let status = match response.status() {
        StatusCode::OK => ResourceStatus::Found(resource_url.to_string()),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            ResourceStatus::Unauthorized(resource_url.to_string())
        }
        StatusCode::NOT_FOUND => ResourceStatus::NotFound,
        other => {
            anyhow::bail!(
                "Unexpected HTTP status {} when checking {}",
                other,
                resource_url
            );
        }
    };

    Ok(status)
}

async fn check_robots_txt(base_url: Url) -> Result<ResourceStatus> {
    check_resource(base_url, ROBOTS_TXT_PATH).await
}

async fn check_sitemap_xml(base_url: Url) -> Result<ResourceStatus> {
    check_resource(base_url, SITE_MAP_PATH).await
}

/// Find all pages for a specific website
async fn find_all_pages_without_site_map(start_url: Url) -> Result<HashSet<Url>, Error> {
    let mut visited = HashSet::new();
    let mut to_visit = vec![start_url.clone()];
    let client = Client::new();

    // Get the base host (domain or IP) and port to stay within the same site
    let base_host = start_url
        .host_str()
        .ok_or_else(|| anyhow!("Invalid host: Start URL has no host component"))?;
    let base_port = start_url.port();

    while let Some(url) = to_visit.pop() {
        if visited.contains(&url) {
            continue;
        }

        log::info!("Visiting: {}", url);
        visited.insert(url.clone());

        // Fetch the page
        let response = client
            .get(url.as_str())
            .send()
            .await
            .context("Error while sending page request")
            .map_err(Error::from)?;

        if let Ok(body) = response.text().await {
            // Parse HTML and find all links
            let document = Html::parse_document(&body);
            let selector = Selector::parse("a[href]").expect("Unable to parse a[href]");

            for element in document.select(&selector) {
                let Some(href) = element.value().attr("href") else {
                    continue;
                };

                let Ok(link_url) = url.join(href) else {
                    continue;
                };

                // Only follow links from the same host and port
                if link_url.host_str() != Some(base_host) || link_url.port() != base_port {
                    continue;
                }

                // Remove fragments (#section)
                let mut clean_url = link_url.clone();
                clean_url.set_fragment(None);

                if visited.contains(&clean_url) {
                    continue;
                }

                to_visit.push(clean_url);
            }
        }
    }

    Ok(visited)
}

fn execute(url: Url) {}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_find_all_pages_single_page() {
        let mut server = Server::new_async().await;

        let _m = server
            .mock("GET", "/")
            .with_status(200)
            .with_body("<html><body><h1>Test</h1></body></html>")
            .create_async()
            .await;

        let url = Url::parse(&server.url()).unwrap();
        let pages = find_all_pages_without_site_map(url.clone()).await.unwrap();

        assert_eq!(pages.len(), 1);
        assert!(pages.contains(&url));
    }

    #[tokio::test]
    async fn test_find_all_pages_with_links() {
        let mut server = Server::new_async().await;

        println!("{}", &server.url());

        let _m1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(
                r#"<html><body>
                <a href="/page1">Page 1</a>
                <a href="/page2">Page 2</a>
            </body></html>"#,
            )
            .create_async()
            .await;

        let _m2 = server
            .mock("GET", "/page1")
            .with_status(200)
            .with_body("<html><body><h1>Page 1</h1></body></html>")
            .create_async()
            .await;

        let _m3 = server
            .mock("GET", "/page2")
            .with_status(200)
            .with_body("<html><body><h1>Page 2</h1></body></html>")
            .create_async()
            .await;

        let url = Url::parse(&server.url()).unwrap();
        let pages = find_all_pages_without_site_map(url.clone()).await.unwrap();
        assert_eq!(pages.len(), 3);
        assert!(pages.contains(&url));
        assert!(pages.contains(&url.join("/page1").unwrap()));
        assert!(pages.contains(&url.join("/page2").unwrap()));
    }

    #[tokio::test]
    async fn test_find_all_pages_nested_links() {
        let mut server = Server::new_async().await;

        let _m1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(r#"<html><body><a href="/about">About</a></body></html>"#)
            .create_async()
            .await;

        let _m2 = server
            .mock("GET", "/about")
            .with_status(200)
            .with_body(r#"<html><body><a href="/contact">Contact</a></body></html>"#)
            .create_async()
            .await;

        let _m3 = server
            .mock("GET", "/contact")
            .with_status(200)
            .with_body("<html><body><h1>Contact</h1></body></html>")
            .create_async()
            .await;

        let url = Url::parse(&server.url()).unwrap();
        let pages = find_all_pages_without_site_map(url.clone()).await.unwrap();

        assert_eq!(pages.len(), 3);
    }

    #[tokio::test]
    async fn test_find_all_pages_ignores_external_links() {
        let mut server = Server::new_async().await;

        let _m = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(
                r#"<html><body>
                <a href="/internal">Internal</a>
                <a href="https://external.com/page">External</a>
            </body></html>"#,
            )
            .create_async()
            .await;

        let _m2 = server
            .mock("GET", "/internal")
            .with_status(200)
            .with_body("<html><body><h1>Internal</h1></body></html>")
            .create_async()
            .await;

        let url = Url::parse(&server.url()).unwrap();
        let pages = find_all_pages_without_site_map(url.clone()).await.unwrap();

        assert_eq!(pages.len(), 2);
        assert!(!pages.iter().any(|u| u.domain() == Some("external.com")));
    }

    #[tokio::test]
    async fn test_find_all_pages_handles_duplicates() {
        let mut server = Server::new_async().await;

        let _m = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(
                r#"<html><body>
                <a href="/page">Page</a>
                <a href="/page">Page Again</a>
                <a href="/page#section">Page with fragment</a>
            </body></html>"#,
            )
            .create_async()
            .await;

        let _m2 = server
            .mock("GET", "/page")
            .with_status(200)
            .with_body("<html><body><h1>Page</h1></body></html>")
            .create_async()
            .await;

        let url = Url::parse(&server.url()).unwrap();
        let pages = find_all_pages_without_site_map(url.clone()).await.unwrap();

        // Should only visit /page once (fragments removed)
        assert_eq!(pages.len(), 2);
    }

    #[tokio::test]
    async fn test_find_all_pages_handles_circular_references() {
        let mut server = Server::new_async().await;

        let _m1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(r#"<html><body><a href="/page1">Page 1</a></body></html>"#)
            .create_async()
            .await;

        let _m2 = server
            .mock("GET", "/page1")
            .with_status(200)
            .with_body(r#"<html><body><a href="/">Home</a></body></html>"#)
            .create_async()
            .await;

        let url = Url::parse(&server.url()).unwrap();
        let pages = find_all_pages_without_site_map(url.clone()).await.unwrap();

        // Should handle circular references without infinite loop
        assert_eq!(pages.len(), 2);
    }

    #[tokio::test]
    async fn test_find_all_pages_handles_404() {
        let mut server = Server::new_async().await;

        let _m1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(
                r#"<html><body>
                <a href="/exists">Exists</a>
                <a href="/not-found">Not Found</a>
            </body></html>"#,
            )
            .create_async()
            .await;

        let _m2 = server
            .mock("GET", "/exists")
            .with_status(200)
            .with_body("<html><body><h1>Exists</h1></body></html>")
            .create_async()
            .await;

        let _m3 = server
            .mock("GET", "/not-found")
            .with_status(404)
            .create_async()
            .await;

        let url = Url::parse(&server.url()).unwrap();
        let pages = find_all_pages_without_site_map(url.clone()).await.unwrap();

        // Should still visit the 404 page (it's part of the site)
        assert!(pages.len() >= 3);
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_all_pages_real_domain() {
        let url = Url::parse("https://google.com").unwrap();
        let pages = find_all_pages_without_site_map(url.clone()).await.unwrap();

        assert!(pages.len() == 318);
        println!("{}", pages.len());
        assert!(pages.contains(&url));

        // All pages should be from example.com domain
        for page in &pages {
            assert_eq!(page.domain(), Some("google.com"));
        }
    }
}

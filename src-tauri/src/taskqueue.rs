use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

/// Find all pages for a specific website
pub async fn find_all_pages(start_url: Url) -> Result<HashSet<Url>, Box<dyn std::error::Error>> {
    let mut visited = HashSet::new();
    let mut to_visit = vec![start_url.clone()];
    let client = Client::new();

    // Get the base host (domain or IP) and port to stay within the same site
    let base_host = start_url.host_str().ok_or("Invalid host")?;
    let base_port = start_url.port();

    while let Some(url) = to_visit.pop() {
        // Skip if already visited
        if visited.contains(&url) {
            continue;
        }

        println!("Visiting: {}", url);
        visited.insert(url.clone());

        // Fetch the page
        match client.get(url.as_str()).send().await {
            Ok(response) => {
                if let Ok(body) = response.text().await {
                    // Parse HTML and find all links
                    let document = Html::parse_document(&body);
                    let selector = Selector::parse("a[href]").unwrap();

                    for element in document.select(&selector) {
                        if let Some(href) = element.value().attr("href") {
                            // Parse the URL (handle relative URLs)
                            if let Ok(link_url) = url.join(href) {
                                // Only follow links from the same host and port
                                if link_url.host_str() == Some(base_host)
                                    && link_url.port() == base_port
                                {
                                    // Remove fragments (#section)
                                    let mut clean_url = link_url.clone();
                                    clean_url.set_fragment(None);

                                    if !visited.contains(&clean_url) {
                                        to_visit.push(clean_url);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", url, e);
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
        let pages = find_all_pages(url.clone()).await.unwrap();

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
        let pages = find_all_pages(url.clone()).await.unwrap();
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
        let pages = find_all_pages(url.clone()).await.unwrap();

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
        let pages = find_all_pages(url.clone()).await.unwrap();

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
        let pages = find_all_pages(url.clone()).await.unwrap();

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
        let pages = find_all_pages(url.clone()).await.unwrap();

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
        let pages = find_all_pages(url.clone()).await.unwrap();

        // Should still visit the 404 page (it's part of the site)
        assert!(pages.len() >= 2);
    }

    #[tokio::test]
    async fn test_find_all_pages_real_domain() {
        let url = Url::parse("https://example.com").unwrap();
        let pages = find_all_pages(url.clone()).await.unwrap();

        // example.com should at least have the homepage
        assert!(pages.len() == 1);
        assert!(pages.contains(&url));

        // All pages should be from example.com domain
        for page in &pages {
            assert_eq!(page.domain(), Some("example.com"));
        }
    }
}

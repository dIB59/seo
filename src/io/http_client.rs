use dioxus::prelude::server_fn::request::reqwest::Client;

pub async fn fetch_html_from_url(url: &str) -> Option<String> {
    println!("Fetching HTML from URL: {}", url);
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RustReqwestBot/1.0; +https://example.com/bot)")
        .build()
        .unwrap();    
    let res = client.get(url).send().await;
    let res = match res {
        Ok(response) => {
            if response.status().is_success() {
                response.text().await.ok()
            } else {
                println!("Failed to fetch HTML: {}", response.status());
                None
            }
        }
        Err(e) => {
            println!("Error fetching HTML: {}", e);
            None
        }
    };
    res
}

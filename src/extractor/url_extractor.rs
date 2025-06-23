use scraper::{Html, Selector};

pub fn extract_url_from_html(html: &str) -> Vec<(String)> {
    println!("Extracting URLs from HTML...");
    let document = Html::parse_document(html);
    let mut results = vec![];

    let selector = Selector::parse("a").unwrap();
    for element in document.select(&selector) {
        let href = element.value().attr("href").unwrap_or("");
        if !href.is_empty() {
            results.push(href.to_string());
        }
    }
    results
}
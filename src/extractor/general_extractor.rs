use scraper::{Html, Selector};

pub fn extract_meta_titles_from_html(html: &str) -> String {
    println!("Extracting meta titles from HTML...");
    let document = Html::parse_document(html);
    let selector = Selector::parse("title").unwrap();
    let mut results = String::new();
    for element in document.select(&selector) {
        let content = element.text().collect::<Vec<_>>().join(" ").to_string();
        if !content.is_empty() {
            results += &content;
        }
    }
    results
    
}

pub fn extract_meta_description_from_html(html: &str) -> String {
    println!("Extracting meta discription from HTML...");
    let document = Html::parse_document(html);
    let selector = Selector::parse("meta[name=\"description\"]").unwrap();
    let mut results = String::new();
    for element in document.select(&selector) {
        let content = element.value().attr("content").unwrap_or("");
        if !content.is_empty() {
            results = content.to_string();
        }
    }
    results
}
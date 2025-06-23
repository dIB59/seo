use scraper::{Html, Selector};

/// Pure function: given HTML, extract headings.
pub fn extract_headings_from_html(html: &str) -> Vec<(String, String)> {
    println!("Extracting headings from HTML...");
    let document = Html::parse_document(html);
    let mut results = vec![];

    for level in 1..=6 {
        let selector = Selector::parse(&format!("h{}", level)).unwrap();
        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
            if !text.is_empty() {
                results.push((format!("h{}", level), text));
            }
        }
    }
    results.clone().iter().for_each(|(tag, text)| {
        println!("{}: {}", tag, text);
    });
    results
}
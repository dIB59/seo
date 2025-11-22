use anyhow::{Context, Error, Result};
use quick_xml::events::Event;
use reqwest::Client;
use url::Url;

pub const SITE_MAP_PATH: &str = "sitemap.xml";

#[derive(Debug, Clone)]
pub enum SitemapFormat {
    Xml,
    PlainText,
}

impl SitemapFormat {
    fn detect(text: &str) -> Self {
        match text.contains("<loc>") {
            true => SitemapFormat::Xml,
            false => SitemapFormat::PlainText,
        }
    }

    fn extract_urls(&self, text: &str) -> Vec<String> {
        match self {
            SitemapFormat::Xml => Self::extract_from_xml(text),
            SitemapFormat::PlainText => Self::extract_from_plain_text(text),
        }
    }

    fn extract_from_xml(text: &str) -> Vec<String> {
        let mut reader = quick_xml::Reader::from_str(text);
        let mut urls = Vec::new();
        let mut buf = Vec::new();
        let mut in_loc_tag = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"loc" => {
                    in_loc_tag = true;
                }
                Ok(Event::Text(e)) if in_loc_tag => {
                    match e.decode() {
                        Ok(txt) => urls.push(txt.to_string()),
                        Err(e) => {
                            log::warn!("Invalid URL text: {:?}", reader.buffer_position());
                            log::warn!("{}", e);
                        }
                    }
                    in_loc_tag = false;
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
        urls
    }
    fn extract_from_plain_text(text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter_map(|token| Url::parse(token).ok())
            .map(|url| url.to_string())
            .collect()
    }
}

pub async fn extract_sitemap_urls(start_url: Url) -> Result<Vec<String>, Error> {
    let client = Client::new();
    let site_map = start_url.join(SITE_MAP_PATH).expect("Unable join URL");
    let result = client
        .get(site_map)
        .send()
        .await
        .context("Unable to send request for sitemap")?;

    let text = result.text().await.context("Unable to get site map text")?;

    extract_url_from_sitemap(&text)
}

fn extract_url_from_sitemap(text: &str) -> Result<Vec<String>, Error> {
    let format: SitemapFormat = SitemapFormat::detect(text);
    println!("{:?}", format);
    let urls = format.extract_urls(text);
    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_xml_format() {
        let text = r#"<loc>https://example.com</loc>"#;
        let format = SitemapFormat::detect(text);
        assert!(matches!(format, SitemapFormat::Xml));
    }

    #[test]
    fn test_detect_plain_text_format() {
        let text = "https://example.com\nhttps://test.com";
        let format = SitemapFormat::detect(text);
        assert!(matches!(format, SitemapFormat::PlainText));
    }

    #[test]
    fn test_extract_plain_text_urls() {
        let text = r#"https://www.google.com/intl/am/gmail/about/
https://www.google.com/intl/am/gmail/about/for-work/
https://www.google.com/intl/am/gmail/about/policy/"#;

        let urls = extract_url_from_sitemap(text).unwrap();
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "https://www.google.com/intl/am/gmail/about/");
    }

    #[test]
    fn test_extract_xml_sitemap() {
        let text = r#"
<sitemapindex>
<sitemap>
<loc>https://www.google.com/gmail/sitemap.xml</loc>
</sitemap>
<sitemap>
<loc>https://www.google.com/forms/sitemaps.xml</loc>
</sitemap>
</sitemapindex>"#;

        let urls = extract_url_from_sitemap(text).unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://www.google.com/gmail/sitemap.xml");
        assert_eq!(urls[1], "https://www.google.com/forms/sitemaps.xml");
    }

    #[test]
    fn test_empty_input() {
        let text = "";
        let urls = extract_url_from_sitemap(text).unwrap();
        assert_eq!(urls.len(), 0);
    }

    #[test]
    fn test_mixed_content() {
        let text = r#"Some text https://example.com more text
        <loc>https://test.com</loc> invalid stuff"#;

        let urls = extract_url_from_sitemap(text).unwrap();
        println!("{}", urls.get(0).expect("MISSING"));
        assert_eq!(urls.len(), 1);
        assert!(urls.contains(&"https://test.com".to_string()));
    }
}

use crate::service::spider::SpiderAgent;
use anyhow::{Context, Error, Result};
use quick_xml::events::Event;
use std::sync::Arc;
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
                            tracing::warn!("Invalid URL text: {:?}", reader.buffer_position());
                            tracing::warn!("{}", e);
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

pub async fn extract_sitemap_urls(
    start_url: Url,
    spider: Arc<dyn SpiderAgent>,
) -> Result<Vec<String>, Error> {
    // `sitemap.xml` is a constant relative path, so the join is in
    // practice infallible — but propagating instead of panicking is
    // cheap and means a malformed `start_url` surfaces as a typed error
    // rather than killing the discovery worker.
    let site_map = start_url
        .join(SITE_MAP_PATH)
        .with_context(|| format!("invalid sitemap URL from base {start_url}"))?;
    let response = spider
        .get(site_map.as_str())
        .await
        .context("Unable to send request for sitemap")?;

    let text = response.body;

    extract_url_from_sitemap(&text)
}

fn extract_url_from_sitemap(text: &str) -> Result<Vec<String>, Error> {
    let format: SitemapFormat = SitemapFormat::detect(text);
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
        println!("{}", urls.first().expect("MISSING"));
        assert_eq!(urls.len(), 1);
        assert!(urls.contains(&"https://test.com".to_string()));
    }

    #[test]
    fn extract_urlset_format_with_xmlns() {
        // Real-world urlset sitemap with the standard sitemap.org
        // namespace declaration. Pinning that the namespace doesn't
        // throw the parser off.
        let text = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>https://example.com/page1</loc>
        <lastmod>2024-01-01</lastmod>
        <changefreq>weekly</changefreq>
    </url>
    <url>
        <loc>https://example.com/page2</loc>
    </url>
</urlset>"#;
        let urls = extract_url_from_sitemap(text).unwrap();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com/page1".to_string()));
        assert!(urls.contains(&"https://example.com/page2".to_string()));
    }

    #[test]
    fn extract_xml_ignores_non_loc_text_content() {
        // The parser should only emit text that's inside a <loc> tag,
        // not text floating between tags.
        let text = r#"<urlset>
            <url>
                <loc>https://example.com/keep-me</loc>
                <lastmod>this is not a url</lastmod>
                <priority>0.8</priority>
            </url>
        </urlset>"#;
        let urls = extract_url_from_sitemap(text).unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/keep-me");
    }

    #[test]
    fn extract_xml_format_handles_multiple_locs() {
        // Many entries — pin that the in_loc_tag flag resets correctly
        // between urls so each <loc> is captured exactly once.
        let text = r#"<urlset>
            <url><loc>https://a.test</loc></url>
            <url><loc>https://b.test</loc></url>
            <url><loc>https://c.test</loc></url>
            <url><loc>https://d.test</loc></url>
        </urlset>"#;
        let urls = extract_url_from_sitemap(text).unwrap();
        assert_eq!(urls.len(), 4);
    }

    #[test]
    fn extract_plain_text_skips_invalid_urls() {
        // The plain-text branch uses Url::parse to filter out
        // non-URL tokens. Pin that contract.
        let text = "https://valid.com not-a-url 123 https://other.com";
        let urls = extract_url_from_sitemap(text).unwrap();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://valid.com/".to_string()));
        assert!(urls.contains(&"https://other.com/".to_string()));
    }

    #[test]
    fn extract_plain_text_handles_various_whitespace() {
        // Newlines, tabs, multiple spaces — split_whitespace handles
        // them all. Pin so a future tokenizer change is deliberate.
        let text = "https://a.com\nhttps://b.com\thttps://c.com   https://d.com";
        let urls = extract_url_from_sitemap(text).unwrap();
        assert_eq!(urls.len(), 4);
    }

    #[test]
    fn detect_format_chooses_xml_when_loc_tag_present() {
        // Pin the heuristic — even one <loc> tag is enough to switch
        // to XML mode. The plain-text branch would otherwise try to
        // parse <loc> as a URL token and fail.
        let text = "junk <loc>https://x.test</loc> junk";
        assert!(matches!(SitemapFormat::detect(text), SitemapFormat::Xml));
    }

    #[test]
    fn detect_format_falls_back_to_plain_text_for_url_only_content() {
        let text = "https://example.com/sitemap.xml";
        assert!(matches!(
            SitemapFormat::detect(text),
            SitemapFormat::PlainText
        ));
    }
}

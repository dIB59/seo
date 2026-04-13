use scraper::Html;
use url::Url;

use crate::contexts::analysis::LinkType;

#[derive(Debug, Clone)]
pub struct ExtractedHeading {
    pub level: i64,
    pub text: String,
    pub position: i64,
}

#[derive(Debug, Clone)]
pub struct ExtractedImage {
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub loading: Option<String>,
    pub is_decorative: bool,
}

#[derive(Debug, Clone)]
pub struct ExtractedLink {
    pub href: String,
    pub link_type: LinkType,
    pub text: Option<String>,
}

pub type LinkLists = (Vec<String>, Vec<String>, Vec<ExtractedLink>);

pub struct PageExtractor;

impl PageExtractor {
    pub fn extract_title(html: &Html) -> Option<String> {
        html.select(cached_selector!("title"))
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn extract_meta_description(html: &Html) -> Option<String> {
        html.select(cached_selector!("meta[name='description']"))
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn extract_canonical(html: &Html) -> Option<String> {
        html.select(cached_selector!("link[rel='canonical']"))
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn extract_word_count(html: &Html) -> i64 {
        html.select(cached_selector!("body"))
            .next()
            .map(|body| body.text().collect::<String>().split_whitespace().count() as i64)
            .unwrap_or(0)
    }

    pub fn extract_headings(html: &Html) -> Vec<ExtractedHeading> {
        html.select(cached_selector!("h1, h2, h3, h4, h5, h6"))
            .enumerate()
            .filter_map(|(idx, element)| {
                let tag = element.value().name();
                let level = tag.trim_start_matches('h').parse::<i64>().ok()?;
                let text = element.text().collect::<String>().trim().to_string();
                if text.is_empty() {
                    return None;
                }

                Some(ExtractedHeading {
                    level,
                    text,
                    position: idx as i64,
                })
            })
            .collect()
    }

    pub fn extract_images(html: &Html, base_url: &str) -> Vec<ExtractedImage> {
        let base = Url::parse(base_url).ok();

        html.select(cached_selector!("img[src]"))
            .filter_map(|element| {
                let src = element.value().attr("src")?.trim().to_string();
                if src.is_empty() {
                    return None;
                }

                let resolved_src = if let Some(ref base) = base {
                    base.join(&src).map(|u| u.to_string()).unwrap_or(src)
                } else {
                    src
                };

                let alt = element.value().attr("alt").map(|s| s.trim().to_string());
                let width = element
                    .value()
                    .attr("width")
                    .and_then(|w| w.parse::<i64>().ok());
                let height = element
                    .value()
                    .attr("height")
                    .and_then(|h| h.parse::<i64>().ok());
                let loading = element.value().attr("loading").map(|s| s.to_string());
                let is_decorative = alt.as_deref().map(|a| a.is_empty()).unwrap_or(false)
                    || element.value().attr("role") == Some("presentation")
                    || element.value().attr("aria-hidden") == Some("true");

                Some(ExtractedImage {
                    src: resolved_src,
                    alt,
                    width,
                    height,
                    loading,
                    is_decorative,
                })
            })
            .collect()
    }

    pub fn extract_links(html: &Html, base_url: &str) -> LinkLists {
        let selector = cached_selector!("a[href]");
        let img_selector = cached_selector!("img");
        let base = Url::parse(base_url).ok();

        let mut internal = Vec::new();
        let mut external = Vec::new();
        let mut all = Vec::new();

        for element in html.select(selector) {
            if let Some(href) = element.value().attr("href") {
                let href = href.trim();

                if href.is_empty()
                    || href.starts_with('#')
                    || href.starts_with("javascript:")
                    || href.starts_with("mailto:")
                    || href.starts_with("tel:")
                {
                    continue;
                }

                // Determine visible/accessible text for the anchor (fallbacks: aria-label/title/img alt)
                let mut text = element.text().collect::<String>().trim().to_string();
                if text.is_empty() {
                    if let Some(attr) = element
                        .value()
                        .attr("aria-label")
                        .or_else(|| element.value().attr("title"))
                    {
                        text = attr.trim().to_string();
                    }
                }
                if text.is_empty() {
                    for img in element.select(img_selector) {
                        if let Some(alt) = img.value().attr("alt") {
                            if !alt.trim().is_empty() {
                                text = alt.trim().to_string();
                                break;
                            }
                        }
                    }
                }

                let link_text = if text.is_empty() { None } else { Some(text) };

                // Resolve as a parsed Url and keep that value end-to-end:
                // - classify_urls(target, base) takes &Url directly so we
                //   skip 2 redundant parses per link (target re-parse +
                //   base re-parse).
                // - Stringify only once at the end for the output struct.
                let resolved_url: Option<Url> = base
                    .as_ref()
                    .and_then(|b| b.join(href).ok());

                let (link_type, resolved) = match (resolved_url.as_ref(), base.as_ref()) {
                    (Some(target), Some(base_url_parsed)) => (
                        crate::contexts::link::NewLink::classify_urls(target, base_url_parsed),
                        target.to_string(),
                    ),
                    _ => (
                        crate::contexts::link::NewLink::classify(href, base_url),
                        href.to_string(),
                    ),
                };

                all.push(ExtractedLink {
                    href: resolved.clone(),
                    link_type: link_type.clone(),
                    text: link_text,
                });

                match link_type {
                    LinkType::Internal => internal.push(resolved),
                    _ => external.push(resolved),
                }
            }
        }

        (internal, external, all)
    }
    pub fn extract_has_viewport(html: &Html) -> bool {
        html.select(cached_selector!("meta[name='viewport']"))
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|content| content.contains("width=device-width"))
            .unwrap_or(false)
    }

    pub fn extract_has_structured_data(html: &Html) -> bool {
        html.select(cached_selector!("script[type='application/ld+json']"))
            .next()
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_has_viewport() {
        let html_content = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>Test Page</title>
            </head>
            <body></body>
            </html>
        "#;
        let html = Html::parse_document(html_content);
        assert!(PageExtractor::extract_has_viewport(&html));
    }

    #[test]
    fn test_extract_no_viewport() {
        let html_content = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Test Page</title>
            </head>
            <body></body>
            </html>
        "#;
        let html = Html::parse_document(html_content);
        assert!(!PageExtractor::extract_has_viewport(&html));
    }

    #[test]
    fn test_extract_has_structured_data() {
        let html_content = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@context": "https://schema.org",
                    "@type": "Organization",
                    "url": "http://www.example.com",
                    "name": "Unlimited Ball Bearings Corp."
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;
        let html = Html::parse_document(html_content);
        assert!(PageExtractor::extract_has_structured_data(&html));
    }

    #[test]
    fn test_extract_no_structured_data() {
        let html_content = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Test Page</title>
            </head>
            <body>
                <script>console.log('hello');</script>
            </body>
            </html>
        "#;
        let html = Html::parse_document(html_content);
        assert!(!PageExtractor::extract_has_structured_data(&html));
    }

    // ── extract_links ────────────────────────────────────────────────────
    //
    // These tests exist to pin the behavior of the per-link hot loop in
    // `extract_links`, which is the canonical link-resolution path for
    // every analyzed page. The recent perf optimization (iteration 31)
    // changed the link resolution to keep the parsed Url end-to-end
    // instead of stringifying-then-reparsing — these tests guard
    // against regressions in that path.

    fn parse_links(html: &str, base: &str) -> (Vec<String>, Vec<String>, Vec<ExtractedLink>) {
        let doc = Html::parse_document(html);
        PageExtractor::extract_links(&doc, base)
    }

    #[test]
    fn extract_links_classifies_internal_and_external() {
        let html = r##"<html><body>
            <a href="/about">About</a>
            <a href="https://other.com/x">External</a>
        </body></html>"##;
        let (internal, external, all) = parse_links(html, "https://example.com");
        assert_eq!(internal.len(), 1);
        assert_eq!(external.len(), 1);
        assert_eq!(all.len(), 2);
        assert!(internal[0].contains("/about"));
        assert!(external[0].contains("other.com"));
    }

    #[test]
    fn extract_links_skips_javascript_mailto_tel_and_hash() {
        let html = r##"<html><body>
            <a href="javascript:void(0)">JS</a>
            <a href="mailto:hi@example.com">Mail</a>
            <a href="tel:+15551234">Phone</a>
            <a href="#section">Hash</a>
            <a href="">Empty</a>
            <a href="/real">Real</a>
        </body></html>"##;
        let (_, _, all) = parse_links(html, "https://example.com");
        // Only the /real link survives the upfront filter.
        assert_eq!(all.len(), 1);
        assert!(all[0].href.contains("/real"));
    }

    #[test]
    fn extract_links_resolves_relative_against_base() {
        let html = r##"<html><body>
            <a href="page2">Plain</a>
            <a href="../sibling">Parent</a>
            <a href="?q=1">Query</a>
        </body></html>"##;
        let (_, _, all) = parse_links(html, "https://example.com/dir/");
        assert_eq!(all.len(), 3);
        assert!(all.iter().any(|l| l.href == "https://example.com/dir/page2"));
        assert!(all.iter().any(|l| l.href == "https://example.com/sibling"));
        assert!(all.iter().any(|l| l.href == "https://example.com/dir/?q=1"));
    }

    #[test]
    fn extract_links_uses_visible_text() {
        let html = r##"<html><body>
            <a href="/about">About Us</a>
        </body></html>"##;
        let (_, _, all) = parse_links(html, "https://example.com");
        assert_eq!(all[0].text.as_deref(), Some("About Us"));
    }

    #[test]
    fn extract_links_falls_back_to_aria_label_when_text_is_empty() {
        let html = r##"<html><body>
            <a href="/x" aria-label="Open menu"><svg/></a>
        </body></html>"##;
        let (_, _, all) = parse_links(html, "https://example.com");
        assert_eq!(all[0].text.as_deref(), Some("Open menu"));
    }

    #[test]
    fn extract_links_falls_back_to_title_when_text_and_aria_empty() {
        let html = r##"<html><body>
            <a href="/x" title="Search the site"><svg/></a>
        </body></html>"##;
        let (_, _, all) = parse_links(html, "https://example.com");
        assert_eq!(all[0].text.as_deref(), Some("Search the site"));
    }

    #[test]
    fn extract_links_falls_back_to_child_img_alt() {
        let html = r##"<html><body>
            <a href="/x"><img src="logo.png" alt="Company logo"></a>
        </body></html>"##;
        let (_, _, all) = parse_links(html, "https://example.com");
        assert_eq!(all[0].text.as_deref(), Some("Company logo"));
    }

    #[test]
    fn extract_links_text_is_none_when_all_fallbacks_empty() {
        let html = r##"<html><body>
            <a href="/x"><img src="logo.png"></a>
        </body></html>"##;
        let (_, _, all) = parse_links(html, "https://example.com");
        assert!(all[0].text.is_none());
    }

    #[test]
    fn extract_links_falls_through_when_base_unparseable() {
        // base_url cannot be parsed → resolved is the raw href, classify
        // is called with strings. Pinning the fallback path so the
        // optimized parsed-Url path doesn't accidentally lose it.
        let html = r##"<html><body>
            <a href="https://other.com/x">External</a>
        </body></html>"##;
        let (_, _, all) = parse_links(html, "not a url");
        assert_eq!(all.len(), 1);
        assert!(all[0].href.contains("other.com"));
    }

    #[test]
    fn extract_links_handles_no_anchors() {
        let html = "<html><body><p>no links</p></body></html>";
        let (i, e, a) = parse_links(html, "https://example.com");
        assert!(i.is_empty());
        assert!(e.is_empty());
        assert!(a.is_empty());
    }

    // ── extract_headings ─────────────────────────────────────────────────

    #[test]
    fn extract_headings_picks_up_all_levels() {
        let html = Html::parse_document(
            r##"<html><body>
                <h1>One</h1>
                <h2>Two</h2>
                <h3>Three</h3>
                <h4>Four</h4>
                <h5>Five</h5>
                <h6>Six</h6>
            </body></html>"##,
        );
        let headings = PageExtractor::extract_headings(&html);
        assert_eq!(headings.len(), 6);
        let levels: Vec<i64> = headings.iter().map(|h| h.level).collect();
        assert_eq!(levels, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn extract_headings_assigns_document_order_position() {
        let html = Html::parse_document(
            "<html><body><h1>A</h1><h3>B</h3><h2>C</h2></body></html>",
        );
        let headings = PageExtractor::extract_headings(&html);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].position, 0);
        assert_eq!(headings[1].position, 1);
        assert_eq!(headings[2].position, 2);
        // Levels follow document order, not numerical order.
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[1].level, 3);
        assert_eq!(headings[2].level, 2);
    }

    #[test]
    fn extract_headings_skips_empty_text() {
        let html = Html::parse_document(
            "<html><body><h1></h1><h2>Real</h2><h3>   </h3></body></html>",
        );
        let headings = PageExtractor::extract_headings(&html);
        // Empty and whitespace-only headings are skipped.
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Real");
    }

    #[test]
    fn extract_headings_trims_whitespace() {
        let html = Html::parse_document("<html><body><h1>  spaced  </h1></body></html>");
        let headings = PageExtractor::extract_headings(&html);
        assert_eq!(headings[0].text, "spaced");
    }

    #[test]
    fn extract_headings_returns_empty_for_no_headings() {
        let html = Html::parse_document("<html><body><p>no headings</p></body></html>");
        assert!(PageExtractor::extract_headings(&html).is_empty());
    }

    // ── extract_images ───────────────────────────────────────────────────

    #[test]
    fn extract_images_resolves_relative_src_against_base() {
        let html = Html::parse_document(
            r##"<html><body><img src="logo.png" alt="Logo"></body></html>"##,
        );
        let images = PageExtractor::extract_images(&html, "https://example.com/dir/");
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].src, "https://example.com/dir/logo.png");
        assert_eq!(images[0].alt.as_deref(), Some("Logo"));
    }

    #[test]
    fn extract_images_keeps_absolute_src_unchanged() {
        let html = Html::parse_document(
            r##"<html><body><img src="https://cdn.example.com/x.png" alt="X"></body></html>"##,
        );
        let images = PageExtractor::extract_images(&html, "https://example.com/");
        assert_eq!(images[0].src, "https://cdn.example.com/x.png");
    }

    #[test]
    fn extract_images_skips_empty_src() {
        let html = Html::parse_document(
            r##"<html><body><img src="" alt="empty"><img src="real.png" alt="ok"></body></html>"##,
        );
        let images = PageExtractor::extract_images(&html, "https://example.com/");
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].alt.as_deref(), Some("ok"));
    }

    #[test]
    fn extract_images_classifies_decorative_via_empty_alt() {
        let html = Html::parse_document(
            r##"<html><body><img src="dec.png" alt=""><img src="real.png" alt="real"></body></html>"##,
        );
        let images = PageExtractor::extract_images(&html, "https://example.com/");
        // First has empty alt → decorative.
        let dec = images.iter().find(|i| i.src.contains("dec.png")).unwrap();
        let real = images.iter().find(|i| i.src.contains("real.png")).unwrap();
        assert!(dec.is_decorative);
        assert!(!real.is_decorative);
    }

    #[test]
    fn extract_images_classifies_decorative_via_aria_hidden() {
        let html = Html::parse_document(
            r##"<html><body><img src="x.png" alt="alt-text" aria-hidden="true"></body></html>"##,
        );
        let images = PageExtractor::extract_images(&html, "https://example.com/");
        assert!(images[0].is_decorative);
    }

    #[test]
    fn extract_images_classifies_decorative_via_role_presentation() {
        let html = Html::parse_document(
            r##"<html><body><img src="x.png" alt="alt" role="presentation"></body></html>"##,
        );
        let images = PageExtractor::extract_images(&html, "https://example.com/");
        assert!(images[0].is_decorative);
    }

    #[test]
    fn extract_images_parses_width_height_loading() {
        let html = Html::parse_document(
            r##"<html><body><img src="x.png" alt="x" width="120" height="60" loading="lazy"></body></html>"##,
        );
        let images = PageExtractor::extract_images(&html, "https://example.com/");
        assert_eq!(images[0].width, Some(120));
        assert_eq!(images[0].height, Some(60));
        assert_eq!(images[0].loading.as_deref(), Some("lazy"));
    }

    #[test]
    fn extract_images_handles_missing_optional_attrs() {
        let html = Html::parse_document(r##"<html><body><img src="x.png"></body></html>"##);
        let images = PageExtractor::extract_images(&html, "https://example.com/");
        assert!(images[0].alt.is_none());
        assert!(images[0].width.is_none());
        assert!(images[0].height.is_none());
        assert!(images[0].loading.is_none());
    }
}

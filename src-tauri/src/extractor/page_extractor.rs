use scraper::{Html, Selector};
use std::sync::OnceLock;
use url::Url;

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
    pub is_internal: bool,
    pub text: Option<String>,
}

pub type LinkLists = (Vec<String>, Vec<String>, Vec<ExtractedLink>);

pub struct PageExtractor;

impl PageExtractor {
    pub fn extract_title(html: &Html) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("title").unwrap());
        html.select(selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn extract_meta_description(html: &Html) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector =
            SELECTOR.get_or_init(|| Selector::parse("meta[name='description']").unwrap());
        html.select(selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn extract_canonical(html: &Html) -> Option<String> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("link[rel='canonical']").unwrap());
        html.select(selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn extract_word_count(html: &Html) -> i64 {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("body").unwrap());
        html.select(selector)
            .next()
            .map(|body| body.text().collect::<String>().split_whitespace().count() as i64)
            .unwrap_or(0)
    }

    pub fn extract_headings(html: &Html) -> Vec<ExtractedHeading> {
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("h1, h2, h3, h4, h5, h6").unwrap());

        html.select(selector)
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
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("img[src]").unwrap());
        let base = Url::parse(base_url).ok();

        html.select(selector)
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
        static SELECTOR: OnceLock<Selector> = OnceLock::new();
        let selector = SELECTOR.get_or_init(|| Selector::parse("a[href]").unwrap());

        static IMG_SELECTOR: OnceLock<Selector> = OnceLock::new();
        let img_selector = IMG_SELECTOR.get_or_init(|| Selector::parse("img").unwrap());

        let base = Url::parse(base_url).ok();
        let base_host = base
            .as_ref()
            .and_then(|u| u.host_str())
            .map(|s| s.to_string());
        let base_port = base.as_ref().and_then(|u| u.port());

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

                let resolved = if let Some(ref base) = base {
                    base.join(href)
                        .map(|u| u.to_string())
                        .unwrap_or_else(|_| href.to_string())
                } else {
                    href.to_string()
                };

                let is_internal = if let Ok(link_url) = Url::parse(&resolved) {
                    link_url.host_str().map(|h| h.to_string()) == base_host
                        && link_url.port() == base_port
                } else {
                    false
                };

                all.push(ExtractedLink {
                    href: resolved.clone(),
                    is_internal,
                    text: link_text,
                });

                if is_internal {
                    internal.push(resolved);
                } else {
                    external.push(resolved);
                }
            }
        }

        (internal, external, all)
    }
}

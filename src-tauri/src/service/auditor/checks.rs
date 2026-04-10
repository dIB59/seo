//! Pluggable SEO checks.
//!
//! Each check is an isolated implementor of [`SeoCheck`] that takes a
//! borrowed [`PageContext`] and returns a [`CheckResult`]. This module
//! replaces a small slice of the long flag-checking methods on
//! `LightAuditor` with composable, individually-testable units.
//!
//! ## Migration status
//!
//! - [`TitleCheck`] — extracted from `light::check_title`.
//! - [`MetaDescriptionCheck`] — extracted from `light::check_meta_description`.
//! - The remaining 7 checks (viewport, canonical, hreflang, crawlable
//!   anchors, link text, image alt, robots/is-crawlable) still live as
//!   methods on `LightAuditor`. Future work: migrate one per commit and
//!   delete the corresponding method from `light.rs`.
//!
//! ## Why a trait
//!
//! Each rule used to be a private method on `LightAuditor`, reachable only
//! via the auditor's own `extract_seo_details` orchestration. Promoting
//! them to standalone `SeoCheck` impls means:
//!
//! - Each rule can be unit-tested in isolation against an HTML fixture
//!   without spinning up a `SpiderAgent` or running the full audit.
//! - Custom user-defined checks (see `contexts/extension`) can implement
//!   the same trait and be plugged into the same registry — no separate
//!   evaluation path.
//! - Adding a new built-in rule is a new file, not a new method on a
//!   600-line struct.

use scraper::{Html, Selector};
use url::Url;

use super::types::{CheckResult, Score};

// The `cached_selector!` macro lives at the crate root (`html_selector`)
// so every module that runs CSS selectors shares the same definition and
// caching semantics. It's pulled in by `#[macro_use] pub mod
// html_selector` in `lib.rs` and is available without an explicit import
// anywhere in the crate.

/// Read-only context handed to every check. Borrowed so checks are cheap
/// and the auditor can run all of them off a single parsed `Html`.
pub struct PageContext<'a> {
    pub document: &'a Html,
    pub url: &'a Url,
}

/// A single SEO rule. Implementors must be cheap to construct (typically
/// unit structs) and side-effect free.
pub trait SeoCheck: Send + Sync {
    /// Stable identifier used in logs and the eventual rule registry.
    fn name(&self) -> &'static str;

    /// Run the rule against the page.
    fn evaluate(&self, ctx: &PageContext) -> CheckResult;
}

// ── Shared helpers ───────────────────────────────────────────────────────────

/// `length_bounded_check` for "value must fall within `[min, max]` characters".
/// Public so the orchestrating auditor and individual checks can share it
/// without re-importing the legacy helper from `light.rs`.
pub(crate) fn length_bounded_result(
    value: Option<String>,
    label: &str,
    min: usize,
    max: usize,
) -> CheckResult {
    match value {
        Some(v) if !v.is_empty() => {
            let len = v.len();
            let (passed, score, desc) = if len < min {
                (
                    false,
                    Score::from(0.5),
                    format!("{} too short ({} chars, recommend {}-{})", label, len, min, max),
                )
            } else if len > max {
                (
                    false,
                    Score::from(0.7),
                    format!("{} too long ({} chars, recommend {}-{})", label, len, min, max),
                )
            } else {
                (
                    true,
                    Score::from(1.0),
                    format!("{} length is good ({} chars)", label, len),
                )
            };
            CheckResult {
                passed,
                value: Some(v),
                score,
                description: Some(desc),
            }
        }
        _ => CheckResult {
            passed: false,
            value: None,
            score: Score::from(0.0),
            description: Some(format!("Missing {}", label.to_lowercase())),
        },
    }
}

// ── Built-in checks ──────────────────────────────────────────────────────────

/// `<title>` length check (recommended 30–60 chars).
pub struct TitleCheck;

impl SeoCheck for TitleCheck {
    fn name(&self) -> &'static str {
        "title"
    }

    fn evaluate(&self, ctx: &PageContext) -> CheckResult {
        let title = ctx
            .document
            .select(cached_selector!("title"))
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string());
        let mut result = length_bounded_result(title, "Title", 30, 60);
        if result.value.is_none() {
            result.description = Some("Missing document title".to_string());
        }
        result
    }
}

/// `<meta name="description">` length check (recommended 70–160 chars).
pub struct MetaDescriptionCheck;

impl SeoCheck for MetaDescriptionCheck {
    fn name(&self) -> &'static str {
        "meta_description"
    }

    fn evaluate(&self, ctx: &PageContext) -> CheckResult {
        let description = ctx
            .document
            .select(cached_selector!("meta[name='description']"))
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string());
        let mut result = length_bounded_result(description, "Description", 70, 160);
        if result.value.is_none() {
            result.description = Some("Missing meta description".to_string());
        }
        result
    }
}

/// Reads the first matching selector's `attr` value, trimmed.
///
/// Takes a `&'static Selector` (built via `cached_selector!`) so the
/// selector is parsed once at first use, not on every call.
fn first_attr(doc: &Html, selector: &Selector, attr: &str) -> Option<String> {
    doc.select(selector)
        .next()
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.trim().to_string())
}

/// `<meta name="viewport">` must contain `width=device-width`.
pub struct ViewportCheck;

impl SeoCheck for ViewportCheck {
    fn name(&self) -> &'static str {
        "viewport"
    }

    fn evaluate(&self, ctx: &PageContext) -> CheckResult {
        let viewport = first_attr(
            ctx.document,
            cached_selector!("meta[name='viewport']"),
            "content",
        );
        match viewport {
            Some(v) if v.contains("width=device-width") => CheckResult {
                passed: true,
                value: Some(v),
                score: Score::from(1.0),
                description: Some("Viewport is properly configured".to_string()),
            },
            Some(v) => CheckResult {
                passed: false,
                value: Some(v),
                score: Score::from(0.5),
                description: Some("Viewport missing width=device-width".to_string()),
            },
            None => CheckResult {
                passed: false,
                value: None,
                score: Score::from(0.0),
                description: Some("Missing viewport meta tag".to_string()),
            },
        }
    }
}

/// `<link rel="canonical">` should be present and resolve to the page URL.
pub struct CanonicalCheck;

impl SeoCheck for CanonicalCheck {
    fn name(&self) -> &'static str {
        "canonical"
    }

    fn evaluate(&self, ctx: &PageContext) -> CheckResult {
        let canonical = first_attr(
            ctx.document,
            cached_selector!("link[rel='canonical']"),
            "href",
        );
        match canonical {
            Some(c) if !c.is_empty() => {
                let matches = c == ctx.url.as_str()
                    || ctx
                        .url
                        .join(&c)
                        .map(|u| u.as_str() == ctx.url.as_str())
                        .unwrap_or(false);
                CheckResult {
                    passed: true,
                    value: Some(c),
                    score: Score::from(1.0),
                    description: Some(if matches {
                        "Canonical URL matches page URL".to_string()
                    } else {
                        "Canonical URL points to different page".to_string()
                    }),
                }
            }
            _ => CheckResult {
                passed: false,
                value: None,
                score: Score::from(0.0),
                description: Some("Missing canonical URL".to_string()),
            },
        }
    }
}

/// `length_bounded_check`'s sibling for "good vs bad out of total" ratio
/// checks. Used by anchors / link-text / image-alt rules.
pub(crate) fn ratio_result(
    total: usize,
    bad: usize,
    value_suffix: &str,
    bad_desc: String,
    good_desc: &str,
    empty_value: Option<&str>,
    empty_desc: &str,
) -> CheckResult {
    if total == 0 {
        return CheckResult {
            passed: true,
            value: empty_value.map(str::to_string),
            score: Score::from(1.0),
            description: Some(empty_desc.to_string()),
        };
    }
    let good = total - bad;
    let passed = bad == 0;
    CheckResult {
        passed,
        value: Some(format!("{}/{} {}", good, total, value_suffix)),
        score: Score::from(good as f64 / total as f64),
        description: Some(if passed {
            good_desc.to_string()
        } else {
            bad_desc
        }),
    }
}

/// `<link rel="alternate" hreflang="…">` is optional but encouraged for i18n.
pub struct HreflangCheck;

impl SeoCheck for HreflangCheck {
    fn name(&self) -> &'static str {
        "hreflang"
    }

    fn evaluate(&self, ctx: &PageContext) -> CheckResult {
        let count = ctx
            .document
            .select(cached_selector!("link[rel='alternate'][hreflang]"))
            .count();
        if count > 0 {
            CheckResult {
                passed: true,
                value: Some(format!("{} hreflang tags", count)),
                score: Score::from(1.0),
                description: Some(format!(
                    "Found {} hreflang tags for internationalization",
                    count
                )),
            }
        } else {
            // Hreflang is optional — no tags is not a failure.
            CheckResult {
                passed: true,
                value: None,
                score: Score::from(1.0),
                description: Some(
                    "No hreflang tags (optional for single-language sites)".to_string(),
                ),
            }
        }
    }
}

/// Anchors (`<a href>`) should not be `javascript:`, empty, or bare `#`.
pub struct CrawlableAnchorsCheck;

impl SeoCheck for CrawlableAnchorsCheck {
    fn name(&self) -> &'static str {
        "crawlable_anchors"
    }

    fn evaluate(&self, ctx: &PageContext) -> CheckResult {
        let mut total = 0usize;
        let mut uncrawlable = 0usize;
        for anchor in ctx.document.select(cached_selector!("a[href]")) {
            total += 1;
            let href = anchor.value().attr("href").unwrap_or("");
            if href.starts_with("javascript:")
                || (href.starts_with('#') && href.len() == 1)
                || href.is_empty()
            {
                uncrawlable += 1;
            }
        }
        ratio_result(
            total,
            uncrawlable,
            "crawlable",
            format!(
                "{} links are not crawlable (javascript: or empty href)",
                uncrawlable
            ),
            "All links are crawlable",
            Some("0 links"),
            "No links found on page",
        )
    }
}

/// Anchor text quality: avoid `click here`, empty text, etc. Falls back to
/// aria-label, title, and child `<img alt>` if visible text is empty.
pub struct LinkTextCheck;

impl SeoCheck for LinkTextCheck {
    fn name(&self) -> &'static str {
        "link_text"
    }

    fn evaluate(&self, ctx: &PageContext) -> CheckResult {
        let bad_texts = ["click here", "read more", "learn more", "here", "link"];

        let mut total = 0usize;
        let mut poor = 0usize;
        for anchor in ctx.document.select(cached_selector!("a[href]")) {
            total += 1;

            let mut text = anchor.text().collect::<String>().trim().to_lowercase();
            if text.is_empty() {
                if let Some(attr) = anchor
                    .value()
                    .attr("aria-label")
                    .or_else(|| anchor.value().attr("title"))
                {
                    text = attr.trim().to_lowercase();
                }
            }
            if text.is_empty() {
                for img in anchor.select(cached_selector!("img")) {
                    if let Some(alt) = img.value().attr("alt") {
                        if !alt.trim().is_empty() {
                            text = alt.trim().to_lowercase();
                            break;
                        }
                    }
                }
            }

            let normalized = text
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c.is_whitespace() {
                        c
                    } else {
                        ' '
                    }
                })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();

            if normalized.is_empty() || bad_texts.iter().any(|b| normalized.contains(b)) {
                poor += 1;
            }
        }

        ratio_result(
            total,
            poor,
            "with good text",
            format!("{} links have generic/empty text", poor),
            "All links have descriptive text",
            None,
            "No links found",
        )
    }
}

/// `<img>` elements should have a non-empty `alt` attribute.
pub struct ImageAltCheck;

impl SeoCheck for ImageAltCheck {
    fn name(&self) -> &'static str {
        "image_alt"
    }

    fn evaluate(&self, ctx: &PageContext) -> CheckResult {
        let mut total = 0usize;
        let mut missing = 0usize;
        for img in ctx.document.select(cached_selector!("img")) {
            total += 1;
            let alt = img.value().attr("alt");
            if alt.is_none_or(|a| a.trim().is_empty()) {
                missing += 1;
            }
        }
        ratio_result(
            total,
            missing,
            "with alt",
            format!("{} images missing alt attribute", missing),
            "All images have alt attributes",
            Some("0 images"),
            "No images found on page",
        )
    }
}

/// `<meta name="robots">` must not contain `noindex`.
pub struct IsCrawlableCheck;

impl SeoCheck for IsCrawlableCheck {
    fn name(&self) -> &'static str {
        "is_crawlable"
    }

    fn evaluate(&self, ctx: &PageContext) -> CheckResult {
        let robots = first_attr(
            ctx.document,
            cached_selector!("meta[name='robots']"),
            "content",
        )
        .map(|s| s.to_lowercase());
        match robots {
            Some(r) if r.contains("noindex") => CheckResult {
                passed: false,
                value: Some(r),
                score: Score::from(0.0),
                description: Some("Page has noindex directive".to_string()),
            },
            Some(r) => CheckResult {
                passed: true,
                value: Some(r),
                score: Score::from(1.0),
                description: Some("Page is crawlable".to_string()),
            },
            None => CheckResult {
                passed: true,
                value: None,
                score: Score::from(1.0),
                description: Some(
                    "No robots meta tag (page is crawlable by default)".to_string(),
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(html: &str) -> (Html, Url) {
        let doc = Html::parse_document(html);
        let url = Url::parse("https://example.com/").unwrap();
        (doc, url)
    }

    fn run<C: SeoCheck>(check: &C, html: &str) -> CheckResult {
        let (doc, url) = ctx(html);
        check.evaluate(&PageContext {
            document: &doc,
            url: &url,
        })
    }

    // ── TitleCheck ────────────────────────────────────────────────────────

    #[test]
    fn title_within_bounds_passes_with_full_score() {
        let html = "<html><head><title>A Reasonably Sized Page Title For SEO</title></head></html>";
        let result = run(&TitleCheck, html);
        assert!(result.passed);
        assert_eq!(result.score, Score::from(1.0));
        assert_eq!(
            result.value.as_deref(),
            Some("A Reasonably Sized Page Title For SEO")
        );
    }

    #[test]
    fn title_too_short_fails_with_partial_score() {
        let html = "<html><head><title>Tiny</title></head></html>";
        let result = run(&TitleCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.5));
        assert!(result.description.unwrap().contains("too short"));
    }

    #[test]
    fn title_too_long_fails_with_higher_score() {
        let long = "x".repeat(80);
        let html = format!("<html><head><title>{long}</title></head></html>");
        let result = run(&TitleCheck, &html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.7));
        assert!(result.description.unwrap().contains("too long"));
    }

    #[test]
    fn title_missing_fails_with_zero_score() {
        let html = "<html><head></head></html>";
        let result = run(&TitleCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.0));
        assert!(result.value.is_none());
        assert_eq!(
            result.description.as_deref(),
            Some("Missing document title")
        );
    }

    #[test]
    fn title_empty_tag_fails() {
        let html = "<html><head><title></title></head></html>";
        let result = run(&TitleCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.0));
    }

    #[test]
    fn title_check_has_stable_name() {
        assert_eq!(TitleCheck.name(), "title");
    }

    // ── MetaDescriptionCheck ──────────────────────────────────────────────

    #[test]
    fn meta_description_within_bounds_passes() {
        let desc = "x".repeat(100);
        let html = format!(
            r#"<html><head><meta name="description" content="{desc}"></head></html>"#
        );
        let result = run(&MetaDescriptionCheck, &html);
        assert!(result.passed);
        assert_eq!(result.score, Score::from(1.0));
    }

    #[test]
    fn meta_description_too_short_fails() {
        let html = r#"<html><head><meta name="description" content="short"></head></html>"#;
        let result = run(&MetaDescriptionCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.5));
    }

    #[test]
    fn meta_description_too_long_fails() {
        let long = "x".repeat(200);
        let html = format!(
            r#"<html><head><meta name="description" content="{long}"></head></html>"#
        );
        let result = run(&MetaDescriptionCheck, &html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.7));
    }

    #[test]
    fn meta_description_missing_fails_with_dedicated_message() {
        let html = "<html><head></head></html>";
        let result = run(&MetaDescriptionCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.0));
        assert_eq!(
            result.description.as_deref(),
            Some("Missing meta description")
        );
    }

    #[test]
    fn meta_description_check_has_stable_name() {
        assert_eq!(MetaDescriptionCheck.name(), "meta_description");
    }

    // ── ViewportCheck ─────────────────────────────────────────────────────

    #[test]
    fn viewport_with_device_width_passes() {
        let html = r#"<html><head><meta name="viewport" content="width=device-width, initial-scale=1"></head></html>"#;
        let result = run(&ViewportCheck, html);
        assert!(result.passed);
        assert_eq!(result.score, Score::from(1.0));
    }

    #[test]
    fn viewport_without_device_width_fails_partially() {
        let html = r#"<html><head><meta name="viewport" content="initial-scale=1"></head></html>"#;
        let result = run(&ViewportCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.5));
    }

    #[test]
    fn viewport_missing_fails_completely() {
        let html = "<html><head></head></html>";
        let result = run(&ViewportCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.0));
    }

    // ── CanonicalCheck ────────────────────────────────────────────────────

    #[test]
    fn canonical_matching_page_url_passes() {
        let html = r#"<html><head><link rel="canonical" href="https://example.com/"></head></html>"#;
        let result = run(&CanonicalCheck, html);
        assert!(result.passed);
        assert!(result.description.unwrap().contains("matches"));
    }

    #[test]
    fn canonical_pointing_elsewhere_still_passes_but_notes_mismatch() {
        let html = r#"<html><head><link rel="canonical" href="https://other.com/"></head></html>"#;
        let result = run(&CanonicalCheck, html);
        assert!(result.passed);
        assert!(result.description.unwrap().contains("different"));
    }

    #[test]
    fn canonical_missing_fails() {
        let html = "<html><head></head></html>";
        let result = run(&CanonicalCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.0));
    }

    #[test]
    fn canonical_resolves_relative_href() {
        // The page URL is https://example.com/ — a bare path should
        // resolve to the same URL via Url::join.
        let html = r#"<html><head><link rel="canonical" href="/"></head></html>"#;
        let result = run(&CanonicalCheck, html);
        assert!(result.passed);
        assert!(result.description.unwrap().contains("matches"));
    }

    // ── HreflangCheck ─────────────────────────────────────────────────────

    #[test]
    fn hreflang_present_passes() {
        let html = r#"<html><head>
            <link rel="alternate" hreflang="en" href="https://example.com/en">
            <link rel="alternate" hreflang="fr" href="https://example.com/fr">
        </head></html>"#;
        let result = run(&HreflangCheck, html);
        assert!(result.passed);
        assert_eq!(result.value.as_deref(), Some("2 hreflang tags"));
    }

    #[test]
    fn hreflang_absent_passes_as_optional() {
        let html = "<html><head></head></html>";
        let result = run(&HreflangCheck, html);
        assert!(result.passed);
        assert_eq!(result.score, Score::from(1.0));
        assert!(result.value.is_none());
    }

    // ── CrawlableAnchorsCheck ─────────────────────────────────────────────

    #[test]
    fn all_crawlable_anchors_pass() {
        let html = r#"<html><body>
            <a href="/about">About</a>
            <a href="/contact">Contact</a>
        </body></html>"#;
        let result = run(&CrawlableAnchorsCheck, html);
        assert!(result.passed);
        assert_eq!(result.score, Score::from(1.0));
    }

    #[test]
    fn javascript_and_empty_hrefs_are_uncrawlable() {
        let html = r##"<html><body>
            <a href="/ok">Good</a>
            <a href="javascript:void(0)">Bad</a>
            <a href="">Empty</a>
            <a href="#">Bare hash</a>
        </body></html>"##;
        let result = run(&CrawlableAnchorsCheck, html);
        assert!(!result.passed);
        // 1 good of 4
        assert_eq!(result.score, Score::from(0.25));
    }

    #[test]
    fn no_anchors_passes_with_empty_marker() {
        let html = "<html><body></body></html>";
        let result = run(&CrawlableAnchorsCheck, html);
        assert!(result.passed);
        assert_eq!(result.value.as_deref(), Some("0 links"));
    }

    // ── LinkTextCheck ─────────────────────────────────────────────────────

    #[test]
    fn descriptive_link_text_passes() {
        let html = r#"<html><body>
            <a href="/about">Learn about our company</a>
            <a href="/contact">Get in touch with sales</a>
        </body></html>"#;
        let result = run(&LinkTextCheck, html);
        assert!(result.passed);
    }

    #[test]
    fn click_here_is_poor_link_text() {
        let html = r#"<html><body>
            <a href="/x">click here</a>
            <a href="/y">read more</a>
            <a href="/z">Good descriptive text</a>
        </body></html>"#;
        let result = run(&LinkTextCheck, html);
        assert!(!result.passed);
    }

    #[test]
    fn aria_label_used_when_text_is_empty() {
        let html = r#"<html><body>
            <a href="/x" aria-label="Go to homepage"><span></span></a>
        </body></html>"#;
        let result = run(&LinkTextCheck, html);
        assert!(result.passed);
    }

    #[test]
    fn img_alt_used_when_anchor_text_and_aria_are_empty() {
        let html = r#"<html><body>
            <a href="/x"><img src="logo.png" alt="Company logo"></a>
        </body></html>"#;
        let result = run(&LinkTextCheck, html);
        assert!(result.passed);
    }

    // ── ImageAltCheck ─────────────────────────────────────────────────────

    #[test]
    fn images_with_alt_pass() {
        let html = r#"<html><body>
            <img src="a.jpg" alt="A photo">
            <img src="b.jpg" alt="Another photo">
        </body></html>"#;
        let result = run(&ImageAltCheck, html);
        assert!(result.passed);
        assert_eq!(result.score, Score::from(1.0));
    }

    #[test]
    fn missing_alt_fails() {
        let html = r#"<html><body>
            <img src="a.jpg" alt="ok">
            <img src="b.jpg">
        </body></html>"#;
        let result = run(&ImageAltCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.5));
    }

    #[test]
    fn empty_alt_counts_as_missing() {
        let html = r#"<html><body>
            <img src="a.jpg" alt="   ">
        </body></html>"#;
        let result = run(&ImageAltCheck, html);
        assert!(!result.passed);
    }

    #[test]
    fn no_images_passes_with_empty_marker() {
        let html = "<html><body></body></html>";
        let result = run(&ImageAltCheck, html);
        assert!(result.passed);
        assert_eq!(result.value.as_deref(), Some("0 images"));
    }

    // ── IsCrawlableCheck ──────────────────────────────────────────────────

    #[test]
    fn no_robots_meta_is_crawlable_by_default() {
        let html = "<html><head></head></html>";
        let result = run(&IsCrawlableCheck, html);
        assert!(result.passed);
    }

    #[test]
    fn noindex_robots_fails() {
        let html = r#"<html><head><meta name="robots" content="noindex,nofollow"></head></html>"#;
        let result = run(&IsCrawlableCheck, html);
        assert!(!result.passed);
        assert_eq!(result.score, Score::from(0.0));
    }

    #[test]
    fn index_robots_passes() {
        let html = r#"<html><head><meta name="robots" content="index,follow"></head></html>"#;
        let result = run(&IsCrawlableCheck, html);
        assert!(result.passed);
    }
}

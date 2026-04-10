//! Shared `cached_selector!` macro used by every module that parses HTML
//! with a fixed CSS selector literal. Consolidates the `OnceLock<Selector>`
//! boilerplate previously hand-rolled in 12+ places across `extractor/`,
//! `service/auditor/`, and `service/discovery.rs`.
//!
//! CSS parsing allocates, so re-parsing per call on a 1000-page crawl was
//! ~8k wasted selector parses per audit. Each call site caches its own
//! `OnceLock`, scoped to the call site, so separate selectors don't share
//! state.

/// Returns a cached `&'static Selector` for a literal CSS selector.
/// The selector is parsed once on first use and reused thereafter.
/// Panics (at first use) if the literal is not a valid CSS selector —
/// which would be a programming error caught on first test run.
#[macro_export]
macro_rules! cached_selector {
    ($css:literal) => {{
        static S: ::std::sync::OnceLock<::scraper::Selector> = ::std::sync::OnceLock::new();
        S.get_or_init(|| ::scraper::Selector::parse($css).expect("invalid CSS selector"))
    }};
}

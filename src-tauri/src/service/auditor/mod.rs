pub mod checks;
mod deep;
mod light;
mod types;

pub use checks::{
    CanonicalCheck, CrawlableAnchorsCheck, HreflangCheck, ImageAltCheck, IsCrawlableCheck,
    LinkTextCheck, MetaDescriptionCheck, PageContext, SeoCheck, TitleCheck, ViewportCheck,
};
pub use deep::DeepAuditor;
pub use light::LightAuditor;
pub use types::*;

use anyhow::Result;
use async_trait::async_trait;

/// Pre-fetched page data from the discovery phase. Passed to
/// `analyze_from_cache` so the auditor can skip the HTTP fetch.
#[derive(Debug, Clone)]
pub struct CachedHtml {
    pub html: String,
    pub final_url: String,
    pub status_code: u16,
    pub load_time_ms: f64,
}

#[async_trait]
pub trait Auditor: Send + Sync {
    async fn analyze(&self, url: &str) -> Result<AuditResult>;

    /// Analyze using pre-fetched HTML from the discovery cache.
    /// Default implementation ignores the cache and re-fetches.
    /// `LightAuditor` overrides this to skip the HTTP request.
    async fn analyze_from_cache(&self, url: &str, _cached: CachedHtml) -> Result<AuditResult> {
        self.analyze(url).await
    }

    fn name(&self) -> &'static str;

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuditMode {
    #[default]
    Light,
    Deep,
}

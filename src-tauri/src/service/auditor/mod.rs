//! Audit strategies for SEO analysis.
//!
//! This module provides two audit strategies:
//! - **Light**: Fast CDP-based auditing using Chrome DevTools Protocol
//! - **Deep**: Comprehensive Lighthouse-based auditing (slower but more detailed)
//!
//! Both implement the `Auditor` trait for consistent usage.

mod deep;
mod light;
mod types;

pub use deep::DeepAuditor;
pub use light::LightAuditor;
pub use types::*;

use anyhow::Result;
use async_trait::async_trait;

/// Strategy trait for URL auditing.
/// 
/// Implementations provide different trade-offs between speed and depth:
/// - `LightAuditor`: ~1-2s per page, basic SEO checks
/// - `DeepAuditor`: ~5-10s per page, full Lighthouse analysis
#[async_trait]
pub trait Auditor: Send + Sync {
    /// Analyze a single URL and return audit results.
    async fn analyze(&self, url: &str) -> Result<AuditResult>;
    
    /// Analyze multiple URLs sequentially.
    /// Default implementation calls `analyze` for each URL.
    async fn analyze_urls(&self, urls: &[String]) -> Vec<Result<AuditResult>> {
        let mut results = Vec::with_capacity(urls.len());
        for url in urls {
            results.push(self.analyze(url).await);
        }
        results
    }
    
    /// Human-readable name for this auditor.
    fn name(&self) -> &'static str;
    
    /// Shutdown and cleanup resources.
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

/// Audit mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuditMode {
    /// Fast CDP-based audit (~1-2s per page)
    #[default]
    Light,
    /// Full Lighthouse audit (~5-10s per page)
    Deep,
}

impl AuditMode {
    pub fn from_deep_enabled(deep: bool) -> Self {
        if deep {
            Self::Deep
        } else {
            Self::Light
        }
    }
}

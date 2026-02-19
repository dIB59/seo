mod deep;
mod light;
mod types;

pub use deep::DeepAuditor;
pub use light::LightAuditor;
pub use types::*;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Auditor: Send + Sync {
    async fn analyze(&self, url: &str) -> Result<AuditResult>;

    async fn analyze_urls(&self, urls: &[String]) -> Vec<Result<AuditResult>> {
        let mut results = Vec::with_capacity(urls.len());
        for url in urls {
            results.push(self.analyze(url).await);
        }
        results
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

use crate::domain::models::JobSettings;
use crate::service::discovery::{PageDiscovery, ResourceChecker};
use crate::service::processor::reporter::{ProgressEmitter, ProgressEvent};
use anyhow::{Context, Result};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use url::Url;

pub struct Crawler {
    discovery: PageDiscovery,
    resource_checker: ResourceChecker,
}

pub struct CrawlContext {
    pub job_id: String,
    pub settings: JobSettings,
    pub start_url: Url,
    pub cancel_flag: Arc<AtomicBool>,
}

impl Crawler {
    pub fn new() -> Self {
        Self {
            discovery: PageDiscovery::new(),
            resource_checker: ResourceChecker::new(),
        }
    }

    pub async fn check_resources(&self, url: &Url) -> Result<SiteResources> {
        let robots_txt = self
            .resource_checker
            .check_robots_txt(url.clone())
            .await
            .is_ok();
        let sitemap = self
            .resource_checker
            .check_sitemap_xml(url.clone())
            .await
            .is_ok();

        Ok(SiteResources {
            robots_txt,
            sitemap,
            ssl: url.scheme() == "https",
        })
    }

    /// Discover pages using unified progress emission
    pub async fn discover_pages(
        &self,
        context: &CrawlContext,
        progress_emitter: Arc<dyn ProgressEmitter>, // ← Single trait object
    ) -> Result<Vec<Url>> {
        let job_id = context.job_id.clone();
        let max_pages = context.settings.max_pages as usize;

        // Clone emitter for the callback (Arc is cheap)
        let emitter = progress_emitter.clone();
        let job_id_clone = job_id.clone();

        let mut discovered = self
            .discovery
            .discover(
                context.start_url.clone(),
                context.settings.max_pages,
                context.settings.delay_between_requests,
                &context.cancel_flag,
                move |count| {
                    // Emit discovery progress as a typed event
                    emitter.emit(ProgressEvent::Discovery {
                        job_id: job_id_clone.clone(),
                        count,
                        total_pages: max_pages,
                    });
                },
            )
            .await
            .context("Page discovery failed")?;

        if discovered.is_empty() {
            tracing::warn!("[JOB] Discovery returned no pages, falling back to start URL");
            discovered.push(context.start_url.clone());
        }

        Ok(discovered)
    }
}

/// Site-level resource check results.
#[allow(dead_code)]
pub struct SiteResources {
    pub robots_txt: bool,
    pub sitemap: bool,
    pub ssl: bool,
}
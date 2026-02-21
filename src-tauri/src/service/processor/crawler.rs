use crate::domain::JobSettings;
use crate::service::discovery::{PageDiscovery, ResourceChecker, SiteResources};
use crate::service::processor::reporter::{ProgressEmitter, ProgressEvent};
use crate::service::spider::SpiderAgent;
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
    pub start_url: String,
    pub cancel_flag: Arc<AtomicBool>,
}

impl Crawler {
    pub fn new(spider: Arc<dyn SpiderAgent>) -> Self {
        Self {
            discovery: PageDiscovery::new(spider.clone()),
            resource_checker: ResourceChecker::new(spider),
        }
    }

    pub async fn check_resources(&self, url_str: &str) -> Result<SiteResources> {
        let url = Url::parse(url_str)?;
        let robots_txt = self
            .resource_checker
            .check_robots_txt(url.as_str())
            .await
            .is_ok();
        let sitemap = self
            .resource_checker
            .check_sitemap_xml(url.as_str())
            .await
            .is_ok();

        Ok(SiteResources {
            robots_txt,
            sitemap,
            ssl: url.scheme() == "https",
        })
    }

    pub async fn discover_pages(
        &self,
        context: &CrawlContext,
        progress_emitter: Arc<dyn ProgressEmitter>, // ← Single trait object
    ) -> Result<Vec<String>> {
        let job_id = context.job_id.clone();
        let max_pages = context.settings.max_pages as usize;

        // Clone emitter for the callback (Arc is cheap)
        let emitter = progress_emitter.clone();
        let job_id_clone = job_id.clone();

        let mut discovered = self
            .discovery
            .discover(
                &context.start_url,
                context.settings.max_pages,
                context.settings.delay_between_requests,
                &context.cancel_flag,
                move |count| {
                    tracing::trace!("Discovery progress: {}", count);
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
            discovered.push(context.start_url.to_string());
        }

        Ok(discovered)
    }
}

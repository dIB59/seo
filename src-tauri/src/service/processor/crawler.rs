use crate::contexts::analysis::JobSettings;
use crate::service::discovery::{DiscoveredPage, PageDiscovery, ResourceChecker, SiteResources};
use crate::service::processor::reporter::{ProgressEmitter, ProgressEvent};
use crate::service::spider::SpiderAgent;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use url::Url;

pub struct Crawler {
    discovery: PageDiscovery,
    resource_checker: ResourceChecker,
}

pub struct CrawlContext {
    pub job_id: String,
    pub settings: JobSettings,
    pub start_url: String,
    pub cancel_token: CancellationToken,
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

        // Both checks silently treat errors as "resource absent", but we
        // now log the error so a misconfigured spider or a transient
        // network blip doesn't vanish into a negative signal. The
        // previous code dropped the error with `.unwrap_or(false)`.
        async fn present<E: std::fmt::Display>(
            what: &str,
            fut: impl std::future::Future<
                Output = std::result::Result<crate::contexts::analysis::ResourceStatus, E>,
            >,
        ) -> bool {
            match fut.await {
                Ok(status) => status.exists(),
                Err(e) => {
                    tracing::debug!("resource check for {what} failed: {e}");
                    false
                }
            }
        }

        let robots_txt = present("robots.txt", self.resource_checker.check_robots_txt(url.as_str())).await;
        let sitemap = present("sitemap.xml", self.resource_checker.check_sitemap_xml(url.as_str())).await;

        Ok(SiteResources::new(
            robots_txt,
            sitemap,
            url.scheme() == "https",
        ))
    }

    pub async fn discover_pages(
        &self,
        context: &CrawlContext,
        progress_emitter: Arc<dyn ProgressEmitter>,
    ) -> Result<Vec<DiscoveredPage>> {
        let job_id = context.job_id.clone();
        let max_pages = context.settings.max_pages as usize;

        let emitter = progress_emitter.clone();
        let job_id_clone = job_id.clone();

        let mut discovered = self
            .discovery
            .discover(
                &context.start_url,
                context.settings.max_pages,
                context.settings.delay_between_requests,
                context.settings.include_subdomains,
                &context.cancel_token,
                move |count| {
                    tracing::trace!("Discovery progress: {}", count);
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
            discovered.push(DiscoveredPage {
                url: context.start_url.clone(),
                final_url: context.start_url.clone(),
                html: String::new(),
                status_code: 0,
                load_time_ms: 0.0,
            });
        }

        Ok(discovered)
    }
}

use crate::{
    contexts::{
        ai::{AiService, AiServiceFactory},
        analysis::{AnalysisService, AnalysisServiceFactory},
        licensing::{LicenseTier, LicensingAgent, PermissionRequest, Policy},
    },
    extractor::data_extractor::{ExtractorConfig, ExtractorRegistry},
    extractor::data_extractor::selector::SelectorExtractor,
    repository::{
        sqlite_ai_repo, sqlite_extension_repo, sqlite_issue_repo, sqlite_job_repo,
        sqlite_link_repo, sqlite_page_queue_repo, sqlite_page_repo, sqlite_results_repo,
        sqlite_settings_repo, ExtensionRepository,
    },
    service::{
        JobProcessor, ProgressReporter,
        licensing::{LicensingService, MockLicensingService},
        processor::{AnalyzerService, Crawler, reporter::ProgressEmitter},
        spider::{ClientType, Spider, SpiderAgent},
    },
};
use std::sync::{Arc, RwLock};
use tauri::AppHandle;

/// Complete dependency graph for the application.
pub struct AppState {
    pub standard_spider: Arc<dyn SpiderAgent>,
    pub heavy_spider: Arc<dyn SpiderAgent>,

    /// Kept alive via AppState; not exposed to commands directly.
    pub job_processor: Arc<JobProcessor>,

    pub permissions: RwLock<Policy>,
    pub licensing_context: Arc<dyn LicensingAgent>,
    pub analysis_context: AnalysisService,
    pub ai_context: AiService,
    pub extension_repo: Arc<dyn ExtensionRepository>,
}

impl AppState {
    pub(crate) async fn new(app_handle: AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = crate::db::init_db(&app_handle).await?;

        let standard_spider = Spider::new_agent(ClientType::Standard)?;
        let heavy_spider = Spider::new_agent(ClientType::HeavyEmulation)?;

        let job_repo = sqlite_job_repo(pool.clone());
        let link_repo = sqlite_link_repo(pool.clone());
        let pages_repo = sqlite_page_repo(pool.clone());
        let issues_repo = sqlite_issue_repo(pool.clone());
        let results_repo = sqlite_results_repo(pool.clone());
        let settings_repo = sqlite_settings_repo(pool.clone());
        let ai_repo = sqlite_ai_repo(pool.clone());
        let page_queue_repo = sqlite_page_queue_repo(pool.clone());
        let extension_repo = sqlite_extension_repo(pool.clone());
        let progress_reporter: Arc<dyn ProgressEmitter> =
            Arc::new(ProgressReporter::new(app_handle.clone()));

        // Build extractor registry from persisted custom extractors
        let extractor_registry = {
            let mut registry = ExtractorRegistry::new();
            match extension_repo.list_enabled_extractors().await {
                Ok(extractors) => {
                    for ext in extractors {
                        let config = ExtractorConfig {
                            key: ext.key,
                            selector: ext.selector,
                            attribute: ext.attribute,
                            multiple: ext.multiple,
                        };
                        registry.register(Box::new(SelectorExtractor::new(config)));
                    }
                    tracing::info!("[INIT] Loaded {} custom extractor(s)", registry.len());
                }
                Err(e) => {
                    tracing::warn!("[INIT] Failed to load custom extractors: {}", e);
                }
            }
            Arc::new(registry)
        };

        let analyzer = AnalyzerService::new(
            pages_repo,
            issues_repo,
            heavy_spider.clone(),
            extractor_registry,
        );
        let crawler = Crawler::new(heavy_spider.clone());

        let job_processor = Arc::new(JobProcessor::new(
            job_repo.clone(),
            link_repo,
            page_queue_repo.clone(),
            analyzer,
            crawler,
            progress_reporter.clone(),
        ));

        let proc_clone = job_processor.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = proc_clone.run().await {
                tracing::error!("Job processor crashed: {}", e);
            }
        });

        let da_clone = job_processor.analyzer().deep_auditor();
        tauri::async_runtime::spawn(async move {
            match da_clone.start_persistent().await {
                Ok(_) => tracing::info!("DeepAuditor persistent mode started"),
                Err(e) => tracing::warn!(
                    "DeepAuditor persistent mode failed (falling back to one-shot): {}",
                    e
                ),
            }
        });

        let licensing_context: Arc<dyn LicensingAgent> =
            match cfg!(debug_assertions) || cfg!(feature = "mock-licensing") {
                true => Arc::new(MockLicensingService::new(settings_repo.clone())),
                false => Arc::new(LicensingService::new(settings_repo.clone())?),
            };
        let initial_status = licensing_context.load_license().await?;
        tracing::info!("License status on startup: {:?}", initial_status);

        let analysis_context = AnalysisServiceFactory::with_processor(
            job_repo.clone(),
            results_repo.clone(),
            job_processor.clone(),
        );

        let ai_context = AiServiceFactory::from_repositories(
            ai_repo.clone(),
            settings_repo.clone(),
        );

        Ok(AppState {
            standard_spider,
            heavy_spider,
            job_processor,
            permissions: RwLock::new(Policy::from_status(initial_status)),
            licensing_context,
            analysis_context,
            ai_context,
            extension_repo,
        })
    }

    pub fn update_from_status(&self, status: crate::contexts::licensing::LicenseStatus) {
        if let Ok(mut p) = self.permissions.write() {
            *p = Policy::from_status(status);
        }
    }
}

impl addon_macros::AddonCheck<PermissionRequest> for AppState {
    fn check(&self, requirement: PermissionRequest) -> bool {
        self.permissions
            .read()
            .map(|p| p.check(requirement))
            .unwrap_or(false)
    }
}
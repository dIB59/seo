use crate::{
    contexts::{
        ai::{AiService, AiServiceFactory}, analysis::{AnalysisService, AnalysisServiceFactory}, extension::ExtensionRegistry, licensing::{LicenseTier, LicensingAgent, PermissionRequest, Policy}
    }, repository::{
        ExtensionRepositoryTrait, sqlite_ai_repo, sqlite_issue_repo, sqlite_job_repo, sqlite_link_repo, sqlite_page_queue_repo, sqlite_page_repo, sqlite_results_repo, sqlite_settings_repo
    }, service::{
        JobProcessor, ProgressReporter, licensing::{LicensingService, MockLicensingService}, processor::{AnalyzerService, Crawler, reporter::ProgressEmitter}, spider::{ClientType, Spider, SpiderAgent}
    }
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

    /// Extension registry for dynamic SEO rules and data extractors
    pub extension_repository: Arc<dyn ExtensionRepositoryTrait>,
    pub extension_registry: Arc<ExtensionRegistry>,
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
        let progress_reporter: Arc<dyn ProgressEmitter> =
            Arc::new(ProgressReporter::new(app_handle.clone()));

        // Load extension registry from database before creating analyzer
        let extension_registry = Arc::new(
            ExtensionRegistry::load_from_database(&pool)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to load extensions, using empty registry: {}", e);
                    ExtensionRegistry::new()
                }),
        );

        let analyzer = AnalyzerService::with_extensions(
            pages_repo,
            issues_repo,
            heavy_spider.clone(),
            extension_registry.clone(),
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
                false => Arc::new(LicensingService::new(
                    settings_repo.clone(),
                    standard_spider.clone(),
                )?),
            };
        let initial_tier = licensing_context.load_license().await?;
        if initial_tier != LicenseTier::Free {
            tracing::info!("License verified: {:?}", initial_tier);
        }

        let analysis_context = AnalysisServiceFactory::with_processor(
            job_repo.clone(),
            results_repo.clone(),
            job_processor.clone(),
        );

        let ai_context = AiServiceFactory::from_repositories(
            ai_repo.clone(),
            settings_repo.clone(),
        );

        let extension_repository = crate::repository::sqlite_extension_repo(pool.clone());

        Ok(AppState {
            standard_spider,
            heavy_spider,
            job_processor,
            permissions: RwLock::new(Policy::new(initial_tier)),
            licensing_context,
            analysis_context,
            ai_context,
            extension_repository,
            extension_registry
        })
    }

    pub fn update_from_tier(&self, tier: LicenseTier) {
        if let Ok(mut p) = self.permissions.write() {
            p.update_from_tier(tier);
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
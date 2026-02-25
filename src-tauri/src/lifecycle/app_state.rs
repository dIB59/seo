use crate::{
    contexts::{
        ai::{AiService, AiServiceFactory}, analysis::{AnalysisService, AnalysisServiceFactory}, licensing::{LicenseTier, LicensingAgent, PermissionRequest, Policy}
    },
    repository::{
        sqlite_ai_repo, sqlite_issue_repo, sqlite_job_repo, sqlite_link_repo, sqlite_page_queue_repo,
        sqlite_page_repo, sqlite_results_repo, sqlite_settings_repo,
    },
    service::{
        JobProcessor, ProgressReporter, licensing::{LicensingService, MockLicensingService}, processor::{AnalyzerService, Crawler, reporter::ProgressEmitter}, spider::{ClientType, Spider, SpiderAgent}
    },
};
use std::sync::{Arc, RwLock};
use tauri::AppHandle;

/// Complete dependency graph for the application
pub struct AppState {
    // Spiders for web crawling
    pub standard_spider: Arc<dyn SpiderAgent>,
    pub heavy_spider: Arc<dyn SpiderAgent>,

    // Background services (not exposed to commands, but kept alive via AppState)
    pub job_processor: Arc<JobProcessor>,

    // Licensing and Permissions
    pub permissions: RwLock<Policy>,
    /// Context-based licensing service
    pub licensing_context: Arc<dyn LicensingAgent>,
    /// Context-based analysis service
    pub analysis_context: AnalysisService,
    /// Context-based AI service
    pub ai_context: AiService,
}

impl AppState {
    /// Build entire dependency graph explicitly
    pub(crate) async fn new(app_handle: AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Initialize database
        let pool = crate::db::init_db(&app_handle).await?;

        // 2. Build spiders (foundation for services)
        let standard_spider = Spider::new_agent(ClientType::Standard)?;
        let heavy_spider = Spider::new_agent(ClientType::HeavyEmulation)?;

        // 3. Build repositories
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

        // 4. Build services
        let analyzer = AnalyzerService::new(pages_repo, issues_repo, heavy_spider.clone());
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

        // Pre-warm DeepAuditor
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


        // 5. Licensing Init
        // Use MockLicensingService in development mode or if a specific flag is set
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

        // Create the new context-based analysis service (Strangler Fig pattern)
        let analysis_context = AnalysisServiceFactory::with_processor(
            job_repo.clone(),
            results_repo.clone(),
            job_processor.clone(),
        );

        // Create the new context-based AI service (Strangler Fig pattern)
        let ai_context = AiServiceFactory::from_repositories(
            ai_repo.clone(),
            settings_repo.clone(),
        );

        // 6. Return composed state
        Ok(AppState {
            standard_spider,
            heavy_spider,
            job_processor,
            permissions: RwLock::new(Policy::new(initial_tier)),
            licensing_context,
            analysis_context,
            ai_context,
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

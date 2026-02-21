use crate::{
    domain::permissions::{LicenseTier, Policy},
    repository::{
        sqlite_ai_repo, sqlite_issue_repo, sqlite_job_repo, sqlite_link_repo, sqlite_page_repo,
        sqlite_results_repo, sqlite_settings_repo,
    },
    service::{
        licensing,
        processor::{reporter::ProgressEmitter, AnalyzerService, Crawler},
        spider::{ClientType, Spider, SpiderAgent},
        JobProcessor, ProgressReporter,
    },
};
use std::sync::{Arc, RwLock};
use tauri::AppHandle;

/// Complete dependency graph for the application
pub struct AppState {
    // Repositories exposed to commands
    pub settings_repo: Arc<dyn crate::repository::SettingsRepository>,
    pub ai_repo: Arc<dyn crate::repository::AiRepository>,
    pub job_repo: Arc<dyn crate::repository::JobRepository>,
    pub results_repo: Arc<dyn crate::repository::ResultsRepository>,

    pub standard_spider: Arc<dyn SpiderAgent>,
    pub heavy_spider: Arc<dyn SpiderAgent>,

    // Background services (not exposed to commands, but kept alive via AppState)
    pub job_processor: Arc<JobProcessor>,

    // Licensing and Permissions
    pub permissions: RwLock<Policy>,
    pub licensing_service: Arc<dyn crate::domain::licensing::LicensingAgent>,
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
        let progress_reporter: Arc<dyn ProgressEmitter> =
            Arc::new(ProgressReporter::new(app_handle.clone()));

        // 4. Build services
        let analyzer = AnalyzerService::new(pages_repo, issues_repo, heavy_spider.clone());
        let crawler = Crawler::new(heavy_spider.clone());

        let job_processor = Arc::new(JobProcessor::new(
            job_repo.clone(),
            link_repo,
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
        let licensing_service: Arc<dyn crate::domain::licensing::LicensingAgent> =
            match cfg!(debug_assertions) || cfg!(feature = "mock-licensing") {
                true => Arc::new(licensing::MockLicensingService::new(settings_repo.clone())),
                false => Arc::new(licensing::LicensingService::new(
                    settings_repo.clone(),
                    standard_spider.clone(),
                )?),
            };
        let initial_tier = licensing_service.load_license().await.unwrap_or_default();
        if initial_tier != LicenseTier::Free {
            tracing::info!("License verified: {:?}", initial_tier);
        }

        // 6. Return composed state
        Ok(AppState {
            settings_repo,
            ai_repo,
            job_repo,
            results_repo,
            standard_spider,
            heavy_spider,
            job_processor,
            permissions: RwLock::new(Policy::new(initial_tier)),
            licensing_service,
        })
    }

    pub fn update_from_tier(&self, tier: LicenseTier) {
        if let Ok(mut p) = self.permissions.write() {
            p.update_from_tier(tier);
        }
    }
}

impl addon_macros::AddonCheck<crate::domain::permissions::PermissionRequest> for AppState {
    fn check(&self, requirement: crate::domain::permissions::PermissionRequest) -> bool {
        self.permissions
            .read()
            .map(|p| p.check(requirement))
            .unwrap_or(false)
    }
}

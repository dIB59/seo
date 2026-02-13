use crate::{
    domain::licensing::{LicenseTier, UserPermissions},
    repository::sqlite::{
        AiRepository, IssueRepository, JobRepository, LinkRepository, PageRepository,
        ResultsRepository, SettingsRepository,
    },
    service::{
        licensing_service::LicensingService,
        processor::{reporter::ProgressEmitter, AnalyzerService},
        JobProcessor, ProgressReporter,
    },
};
use std::sync::{Arc, RwLock};
use tauri::AppHandle;

/// Complete dependency graph for the application
pub struct AppState {
    // Repositories exposed to commands
    pub settings_repo: Arc<SettingsRepository>,
    pub ai_repo: Arc<AiRepository>,
    pub job_repo: Arc<JobRepository>,
    pub results_repo: Arc<ResultsRepository>,

    // Services exposed to commands

    // Background services (not exposed to commands, but kept alive via AppState)
    pub job_processor: Arc<JobProcessor>, // underscore = intentionally unused but kept alive

    // Licensing and Permissions
    pub permissions: RwLock<UserPermissions>,
    pub licensing_service: Arc<LicensingService>,
}

impl AppState {
    /// Build entire dependency graph explicitly
    pub(crate) async fn new(app_handle: AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Initialize database
        let pool = crate::db::init_db(&app_handle).await?;

        // 2. Build repositories (bottom layer of dependency graph)
        let job_repo = Arc::new(JobRepository::new(pool.clone()));
        let link_repo = Arc::new(LinkRepository::new(pool.clone()));
        let pages_repo = Arc::new(PageRepository::new(pool.clone()));
        let issues_repo = Arc::new(IssueRepository::new(pool.clone()));
        let results_repo = Arc::new(ResultsRepository::new(pool.clone()));
        let settings_repo = Arc::new(SettingsRepository::new(pool.clone()));
        let ai_repo = Arc::new(AiRepository::new(pool.clone()));
        let progress_reporter: Arc<dyn ProgressEmitter> =
            Arc::new(ProgressReporter::new(app_handle.clone()));

        // 3. Build services (middle layer)
        let analyzer = AnalyzerService::new(pages_repo, issues_repo);
        // 4. Start background tasks
        let deep_auditor = analyzer.deep_auditor();
        let job_processor = Arc::new(JobProcessor::new(
            job_repo.clone(),
            link_repo,
            analyzer,
            progress_reporter.clone(),
        ));
        let proc_clone = job_processor.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = proc_clone.run().await {
                tracing::error!("Job processor crashed: {}", e);
            }
        });

        // Pre-warm DeepAuditor
        let da_clone = deep_auditor.clone();
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
        let licensing_service = Arc::new(LicensingService::new(settings_repo.clone())?);
        let initial_tier = licensing_service.load_license().await.unwrap_or_default();
        if initial_tier != LicenseTier::Free {
            tracing::info!("License verified: {:?}", initial_tier);
        }

        // 6. Return composed state (only expose what commands need)
        Ok(AppState {
            settings_repo,
            ai_repo,
            job_repo,
            results_repo,
            job_processor: job_processor,
            permissions: RwLock::new(UserPermissions::new(initial_tier)),
            licensing_service,
        })
    }

    pub fn update_from_tier(&self, tier: LicenseTier) {
        if let Ok(mut p) = self.permissions.write() {
            p.update_from_tier(tier);
        }
    }
}

impl addon_macros::AddonProvider for AppState {
    fn verify_addon(&self, addon_name: &str) -> bool {
        self.permissions
            .read()
            .map(|p| p.check_addon_str(addon_name))
            .unwrap_or(false)
    }
}

use crate::{
    domain::{
        // licensing::LicensingService, // Removed invalid import
        permissions::{LicenseTier, Policy},
    },
    repository::{
        sqlite_ai_repo, sqlite_issue_repo, sqlite_job_repo, sqlite_link_repo, sqlite_page_repo,
        sqlite_results_repo, sqlite_settings_repo,
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
    pub settings_repo: Arc<dyn crate::repository::SettingsRepository>,
    pub ai_repo: Arc<dyn crate::repository::AiRepository>,
    pub job_repo: Arc<dyn crate::repository::JobRepository>,
    pub results_repo: Arc<dyn crate::repository::ResultsRepository>,

    // Services exposed to commands

    // Background services (not exposed to commands, but kept alive via AppState)
    pub job_processor: Arc<JobProcessor>, // underscore = intentionally unused but kept alive

    // Licensing and Permissions
    pub permissions: RwLock<Policy>,
    pub licensing_service: Arc<LicensingService>,
}

impl AppState {
    /// Build entire dependency graph explicitly
    pub(crate) async fn new(app_handle: AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Initialize database
        let pool = crate::db::init_db(&app_handle).await?;

        // 2. Build repositories (bottom layer of dependency graph)
        let job_repo = sqlite_job_repo(pool.clone());
        let link_repo = sqlite_link_repo(pool.clone());
        let pages_repo = sqlite_page_repo(pool.clone());
        let issues_repo = sqlite_issue_repo(pool.clone());
        let results_repo = sqlite_results_repo(pool.clone());
        let settings_repo = sqlite_settings_repo(pool.clone());
        let ai_repo = sqlite_ai_repo(pool.clone());
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

// lifecycle/app_state.rs
use std::sync::Arc;
use tauri::AppHandle;
use crate::{
    repository::
        sqlite::{
            JobRepository, LinkRepository, PageRepository, IssueRepository,
            SettingsRepository, AiRepository, ResultsRepository,
        }
    ,
    service::{JobProcessor, LighthouseService, processor::AnalyzerService},
};

/// Complete dependency graph for the application
pub struct AppState {
    // Repositories exposed to commands
    pub settings_repo: Arc<SettingsRepository>,
    pub ai_repo: Arc<AiRepository>,
    pub job_repo: Arc<JobRepository>,
    pub results_repo: Arc<ResultsRepository>,
    
    // Services exposed to commands
    pub lighthouse_service: Arc<LighthouseService>,
    
    // Background services (not exposed to commands, but kept alive via AppState)
    _job_processor: Arc<JobProcessor>, // underscore = intentionally unused but kept alive
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
        
        // 3. Build services (middle layer)
        let analyzer = AnalyzerService::new(pages_repo, issues_repo);
        let job_processor = Arc::new(JobProcessor::new(
            job_repo.clone(),
            link_repo,
            analyzer,
            app_handle.clone(),
        ));
        
        // 4. Start background tasks
        let proc_clone = job_processor.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = proc_clone.run().await {
                tracing::error!("Job processor crashed: {}", e);
            }
        });
        
        let lighthouse = Arc::new(LighthouseService::new());
        let lh_clone = lighthouse.clone();
        tauri::async_runtime::spawn(async move {
            match lh_clone.start_persistent().await {
                Ok(_) => tracing::info!("Lighthouse persistent mode started"),
                Err(e) => tracing::warn!(
                    "Lighthouse persistent mode failed (falling back to one-shot): {}",
                    e
                ),
            }
        });
        
        // 5. Return composed state (only expose what commands need)
        Ok(AppState {
            settings_repo,
            ai_repo,
            job_repo,
            results_repo,
            lighthouse_service: lighthouse,
            _job_processor: job_processor, // keeps background task alive
        })
    }
}
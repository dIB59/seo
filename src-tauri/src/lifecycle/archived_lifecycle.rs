//! Application lifecycle management for Tauri.
//!
//! Handles setup and shutdown of long-running services like LighthouseService.

use std::sync::Arc;
use tauri::{AppHandle, Manager, RunEvent};

use crate::db;
use crate::service::{self, JobProcessor, LighthouseService};

/// Wrapper for LighthouseService to use as Tauri managed state
pub struct LighthouseState(pub Arc<LighthouseService>);

/// Wrapper for SettingsRepository to expose to Tauri commands
pub struct SettingsState(pub Arc<dyn crate::repository::SettingsRepository>);

/// Wrapper for AiRepository to expose to Tauri commands
pub struct AiState(pub Arc<dyn crate::repository::AiRepository>);

/// Wrapper for JobRepository to expose to Tauri commands
pub struct JobState(pub Arc<dyn crate::repository::JobRepository>);

/// Wrapper for ResultsRepository to expose to Tauri commands
pub struct ResultsState(pub Arc<dyn crate::repository::ResultsRepository>);

/// Initialize logging with tracing_subscriber.
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("sqlx=warn".parse().unwrap())
                .add_directive("app=debug".parse().unwrap())
                .add_directive("info".parse().unwrap())
        )
        .compact()
        .with_target(false)
        .with_ansi(true)
        .init();
}

/// Setup hook for Tauri app initialization.
/// 
/// Initializes:
/// - Database connection pool
/// - Job processor (background task)
/// - Lighthouse service (persistent mode for fast audits)
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let pool = tauri::async_runtime::block_on(async {
        db::init_db(app.handle())
            .await
            .unwrap_or_else(|e| panic!("failed to init db: {}", e))
    });

    // Start job processor (V2 schema) - construct repos and services and pass via DI
    let job_repo = Arc::new(crate::repository::sqlite::JobRepository::new(pool.clone()));
    let link_repo = Arc::new(crate::repository::sqlite::LinkRepository::new(pool.clone()));
    let page_repo = Arc::new(crate::repository::sqlite::PageRepository::new(pool.clone()));
    let issue_repo = Arc::new(crate::repository::sqlite::IssueRepository::new(pool.clone()));

    // Expose Job and Results repositories as managed state for commands
    app.manage(crate::lifecycle::JobState(job_repo.clone()));
    let results_repo: std::sync::Arc<dyn crate::repository::ResultsRepository> =
        std::sync::Arc::new(crate::repository::sqlite::ResultsRepository::new(pool.clone()));
    app.manage(crate::lifecycle::ResultsState(results_repo.clone()));

    let analyzer = crate::service::processor::AnalyzerService::new(page_repo, issue_repo);

    let processor = Arc::new(JobProcessor::new(
        job_repo.clone(),
        link_repo,
        analyzer,
        app.handle().clone(),
    ));

    let proc_clone = processor.clone();
    tauri::async_runtime::spawn(async move {
        proc_clone.run().await.expect("job-processor died")
    });

    // Settings repository managed state (exposed to commands)
    let settings_repo: std::sync::Arc<dyn crate::repository::SettingsRepository> =
        std::sync::Arc::new(crate::repository::sqlite::SettingsRepository::new(pool.clone()));
    app.manage(crate::lifecycle::SettingsState(settings_repo));

    // AI repository managed state (exposed to commands)
    let ai_repo: std::sync::Arc<dyn crate::repository::AiRepository> =
        std::sync::Arc::new(crate::repository::sqlite::AiRepository::new(pool.clone()));
    app.manage(crate::lifecycle::AiState(ai_repo));

    // Initialize LighthouseService and start persistent mode
    let lighthouse = Arc::new(service::LighthouseService::new());
    let lighthouse_clone = lighthouse.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = lighthouse_clone.start_persistent().await {
            tracing::
warn!("Failed to start Lighthouse persistent mode: {}", e);
            tracing::
info!("Lighthouse will use one-shot mode (slower but still works)");
        } else {
            tracing::
info!("Lighthouse persistent mode started successfully");
        }
    });

    // Register managed state
    // DB pool is no longer exposed as managed state; repositories are registered instead.
    app.manage(processor);
    app.manage(LighthouseState(lighthouse));
    
    Ok(())
}

/// Handle Tauri run events (app lifecycle).
pub fn handle_run_event(app_handle: &AppHandle, event: RunEvent) {
    if let RunEvent::ExitRequested { .. } = event {
        shutdown_services(app_handle);
    }
}

/// Gracefully shutdown all managed services.
fn shutdown_services(app_handle: &AppHandle) {
    // Shutdown LighthouseService
    if let Some(lighthouse) = app_handle.try_state::<LighthouseState>() {
        let lighthouse = lighthouse.0.clone();
        tauri::async_runtime::block_on(async move {
            tracing::
info!("Shutting down Lighthouse service...");
            if let Err(e) = lighthouse.shutdown().await {
                tracing::
error!("Error shutting down Lighthouse: {}", e);
            } else {
                tracing::
info!("Lighthouse service shut down successfully");
            }
        });
    }
}

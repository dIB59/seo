//! Application lifecycle management for Tauri.
//!
//! Handles setup and shutdown of long-running services like LighthouseService.

use std::sync::Arc;
use tauri::{AppHandle, Manager, RunEvent};

use crate::db::{self, DbState};
use crate::service::{self, JobProcessor, LighthouseService};

/// Wrapper for LighthouseService to use as Tauri managed state
pub struct LighthouseState(pub Arc<LighthouseService>);

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

    // Start job processor (V2 schema)
    let processor = Arc::new(JobProcessor::new(
        pool.clone(),
        app.handle().clone(),
    ));
    let proc_clone = processor.clone();
    tauri::async_runtime::spawn(async move {
        proc_clone.run().await.expect("job-processor died")
    });

    // Initialize LighthouseService and start persistent mode
    let lighthouse = Arc::new(service::LighthouseService::new());
    let lighthouse_clone = lighthouse.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = lighthouse_clone.start_persistent().await {
            log::warn!("Failed to start Lighthouse persistent mode: {}", e);
            log::info!("Lighthouse will use one-shot mode (slower but still works)");
        } else {
            log::info!("Lighthouse persistent mode started successfully");
        }
    });

    // Register managed state
    app.manage(DbState(pool));
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
            log::info!("Shutting down Lighthouse service...");
            if let Err(e) = lighthouse.shutdown().await {
                log::error!("Error shutting down Lighthouse: {}", e);
            } else {
                log::info!("Lighthouse service shut down successfully");
            }
        });
    }
}

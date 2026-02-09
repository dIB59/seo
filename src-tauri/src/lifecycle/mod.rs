// lifecycle/mod.rs
use tauri::{App, AppHandle, Manager, RunEvent};

use crate::lifecycle::app_state::AppState;

pub mod app_state;

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

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Build entire dependency graph in ONE place
    let state : AppState = tauri::async_runtime::block_on(async {
        AppState::new(app.handle().clone())
            .await
            .expect("Failed to initialize application state")
    });
    
    // Register SINGLE managed state object
    app.manage(state);
    
    Ok(())
}

// No shutdown handling (as requested)
pub fn handle_run_event(app_handle: &tauri::AppHandle, event: tauri::RunEvent) {
    if let RunEvent::ExitRequested { .. } = event {
        shutdown_services(app_handle);
    }
}

/// Gracefully shutdown all managed services.
fn shutdown_services(app_handle: &AppHandle) {
    // Shutdown LighthouseService
    if let Some(app_state) = app_handle.try_state::<AppState>() {
        let lighthouse = app_state.lighthouse_service.clone();
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
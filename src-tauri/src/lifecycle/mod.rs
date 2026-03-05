use crate::lifecycle::app_state::AppState;
use tauri::{App, AppHandle, Manager, RunEvent};

pub mod app_state;

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::default()
                .add_directive("sqlx=warn".parse().unwrap())
                .add_directive("app=debug".parse().unwrap())
                .add_directive("info".parse().unwrap()),
        )
        .compact()
        .with_target(false)
        .with_ansi(true)
        .init();
}

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Build entire dependency graph in ONE place
    let state: AppState = tauri::async_runtime::block_on(async {
        AppState::new(app.handle().clone())
            .await
            .expect("Failed to initialize application state")
    });

    // Register SINGLE managed state object
    app.manage(state);

    Ok(())
}

pub fn handle_run_event(app_handle: &tauri::AppHandle, event: tauri::RunEvent) {
    if let RunEvent::ExitRequested { .. } | RunEvent::Exit = event {
        shutdown_services(app_handle);
    }
}

/// Gracefully shutdown all managed services.
fn shutdown_services(app_handle: &AppHandle) {
    if let Some(app_state) = app_handle.try_state::<AppState>() {
        let job_processor = app_state.job_processor.clone();
        tauri::async_runtime::block_on(async move {
            tracing::info!("Shutting down Job Processor...");
            if let Err(e) = job_processor.shutdown().await {
                tracing::error!("Error shutting down Job Processor: {}", e);
            } else {
                tracing::info!("Job Processor shut down successfully");
            }
        });
    }
}

use crate::service::DiscoveryProgressEmitter;
use serde::Serialize;
use tauri::Emitter;

pub struct ProgressReporter<R: tauri::Runtime> {
    app_handle: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> Clone for ProgressReporter<R> {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
        }
    }
}

#[derive(Serialize)]
struct ProgressEvent {
    job_id: String,
    progress: f64,
    pages_analyzed: i64,
    status: String,
}

impl<R: tauri::Runtime> ProgressReporter<R> {
    pub fn new(app_handle: tauri::AppHandle<R>) -> Self {
        Self { app_handle }
    }

    pub fn emit_progress(&self, job_id: &str, progress: f64, pages_analyzed: i64) {
        let event = ProgressEvent {
            job_id: job_id.to_string(),
            progress,
            pages_analyzed,
            status: "running".to_string(),
        };

        if let Err(e) = self.app_handle.emit("analysis:progress", &event) {
            log::warn!("Failed to emit progress event: {}", e);
        }
    }
}

impl<R: tauri::Runtime> DiscoveryProgressEmitter for ProgressReporter<R> {
    fn emit_discovery_progress(&self, job_id: &str, count: usize, total_pages: usize) {
        self.app_handle
            .emit_discovery_progress(job_id, count, total_pages);
    }
}

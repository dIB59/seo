use tauri::Emitter;

// src/service/progress.rs
use serde::Serialize;

/// Domain events for job progress reporting
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ProgressEvent {
    /// Analysis phase: pages processed, % complete
    Analysis {
        job_id: String,
        progress: f64,
        pages_analyzed: usize,
        total_pages: usize, // Optional total for more accurate progress
    },
    /// Discovery phase: URLs found during crawl
    Discovery {
        job_id: String,
        count: usize,
        total_pages: usize,
    },
}

pub trait ProgressEmitter: Send + Sync {
    fn emit(&self, event: ProgressEvent);
}

/// Tauri-specific progress reporter (production implementation only)
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

impl<R: tauri::Runtime> ProgressReporter<R> {
    pub fn new(app_handle: tauri::AppHandle<R>) -> Self {
        Self { app_handle }
    }
}

// Single implementation – emit enum variants directly as serialized payloads
impl<R: tauri::Runtime> ProgressEmitter for ProgressReporter<R> {
    fn emit(&self, event: ProgressEvent) {
        // Route to appropriate channel based on event variant
        tracing::trace!("Emitting progress event: {:?}", event);
        let channel = match event {
            ProgressEvent::Analysis { .. } => "analysis:progress",
            ProgressEvent::Discovery { .. } => "discovery:progress",
        };

        // Serialize and emit the enum variant directly – no intermediate struct needed
        if let Err(e) = self.app_handle.emit(channel, &event) {
            tracing::warn!("Failed to emit progress event '{}': {}", channel, e);
        }
    }
}

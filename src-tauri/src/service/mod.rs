pub mod analysis_assembler;
pub mod analyzer_service;
use tauri::Emitter;
pub mod auditor;
pub mod crawler;
pub mod discovery;
pub mod gemini;
pub mod http;
pub mod job_canceler;
pub mod job_processor_v2;
pub mod job_queue;
pub mod lighthouse;
pub mod progress_reporter;

pub use auditor::{AuditMode, AuditResult, AuditScores, Auditor, DeepAuditor, LightAuditor};
pub use discovery::{PageDiscovery, ResourceChecker};
pub use gemini::{generate_gemini_analysis, GeminiRequest};
pub use job_processor_v2::JobProcessor;
// Re-export lighthouse types (used by domain models)
pub use lighthouse::{
    AuditResult as LighthouseAuditResult, LighthouseRequest, LighthouseScores, LighthouseService,
    PageFetchResult, PerformanceMetrics, SeoAuditDetails,
};

/// Trait abstracting discovery progress emission.
pub trait DiscoveryProgressEmitter {
    fn emit_discovery_progress(&self, job_id: &str, count: usize, total_pages: usize);
}

impl<R: tauri::Runtime> DiscoveryProgressEmitter for tauri::AppHandle<R> {
    fn emit_discovery_progress(&self, job_id: &str, count: usize, total_pages: usize) {
        #[derive(serde::Serialize, Clone)]
        struct DiscoveryProgressEvent {
            job_id: String,
            count: usize,
            total_pages: usize,
        }

        if let Err(e) = self.emit(
            "discovery-progress",
            DiscoveryProgressEvent {
                job_id: job_id.to_string(),
                count,
                total_pages,
            },
        ) {
            log::warn!("Failed to emit discovery progress: {}", e);
        }
    }
}

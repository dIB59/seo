pub mod analysis_assembler;
pub mod auditor;
use tauri::Emitter;
pub mod discovery;
pub mod gemini;
pub mod http;
pub mod lighthouse;
pub mod processor;

pub use analysis_assembler::AnalysisAssembler;
pub use auditor::{AuditMode, AuditResult, AuditScores, Auditor, DeepAuditor, LightAuditor};
pub use discovery::{PageDiscovery, ResourceChecker};
pub use gemini::{generate_gemini_analysis, GeminiRequest};
pub use lighthouse::{
    AuditResult as LighthouseAuditResult, LighthouseRequest, LighthouseScores, LighthouseService,
    PageFetchResult, PerformanceMetrics, SeoAuditDetails,
};
pub use processor::{
    AnalyzerService, Crawler, JobCanceler, JobProcessor, JobQueue, ProgressReporter,
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

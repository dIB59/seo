pub mod analysis_assembler;
pub mod auditor;
use tauri::Emitter;
pub mod discovery;
pub mod gemini;
pub mod hardware;
pub mod http;
pub mod licensing_service;
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

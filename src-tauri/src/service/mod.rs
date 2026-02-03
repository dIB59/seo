pub mod auditor;
pub mod discovery;
pub mod gemini;
pub mod http;
pub mod job_processor;
pub mod lighthouse;

pub use auditor::{AuditMode, AuditResult, AuditScores, Auditor, DeepAuditor, LightAuditor};
pub use discovery::{PageDiscovery, ResourceChecker};
pub use gemini::{generate_gemini_analysis, GeminiRequest};
pub use job_processor::JobProcessor;
// Re-export lighthouse types (used by domain models)
pub use lighthouse::{
    AuditResult as LighthouseAuditResult, LighthouseRequest, LighthouseScores, LighthouseService,
    PageFetchResult, PerformanceMetrics, SeoAuditDetails,
};

pub mod discovery;
pub mod gemini;
pub mod http;
pub mod job_processor;
pub mod lighthouse;

pub use discovery::{PageDiscovery, ResourceChecker};
pub use gemini::{generate_gemini_analysis, GeminiRequest};
pub use job_processor::JobProcessor;
pub use lighthouse::{
    AuditResult, LighthouseRequest, LighthouseScores, LighthouseService, PageFetchResult,
    PerformanceMetrics, SeoAuditDetails,
};

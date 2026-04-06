pub mod auditor;
pub mod discovery;
pub mod gemini;
pub mod hardware;
pub mod licensing;
pub mod local_model;
pub mod processor;
pub mod prompt;
pub mod spider;

#[cfg(test)]
mod tests;

pub use auditor::{AuditMode, Auditor, DeepAuditor, LightAuditor};
pub use discovery::{PageDiscovery, ResourceChecker};
pub use gemini::{generate_gemini_analysis, GeminiRequest};
pub use processor::{
    AnalyzerService, Crawler, JobCanceler, JobProcessor, JobQueue, ProgressReporter,
};
pub use prompt::{build_prompt_from_blocks, DEFAULT_PERSONA};
pub use spider::Spider;

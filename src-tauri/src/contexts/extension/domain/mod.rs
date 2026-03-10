// Extension Domain Types
// Core domain types for the extension system

mod audit_check;
mod data_extractor;
mod html_extraction_rule;
pub mod issue_rule;

// Re-export domain types
pub use audit_check::{AuditCheck, AuditContext, CheckResult};
pub use data_extractor::{ExtractedData, PageDataExtractor};
pub use html_extraction_rule::{
    HtmlExtractionConfig, HtmlExtractionRule, ValidationReason, ValidationResult,
};
pub use issue_rule::{EvaluationContext, IssueRule, RuleCondition, RuleType};

// Re-export specific implementations
pub use audit_check::{
    CanonicalCheck, CrawlableAnchorsCheck, HreflangCheck, HttpStatusCodeCheck,
    ImageAltCheck, LinkTextCheck, MetaDescriptionCheck, RobotsMetaCheck, TitleCheck,
    ViewportCheck,
};
pub use data_extractor::{
    CssSelectorExtractor, HrefTagExtractor, KeywordExtractor, OpenGraphExtractor,
    StructuredDataExtractor, TwitterCardExtractor,
};
pub use issue_rule::{LengthRule, PresenceRule, RegexRule, StatusCodeRule, ThresholdRule};

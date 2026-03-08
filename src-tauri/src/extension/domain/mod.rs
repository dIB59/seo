//! Extension Domain Types
//! 
//! Domain types for custom rules and extensions

mod html_extraction_rule;

// Re-exports
pub use html_extraction_rule::{HtmlExtractionConfig, HtmlExtractionRule, ValidationResult, ValidationReason};

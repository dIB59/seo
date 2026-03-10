//! Built-in Extensions
//!
//! This module provides the built-in extensions that ship with the application.
//! These include issue generators, data extractors, and data exporters.

mod rules;
mod extractors;
mod exporters;

pub use rules::*;
pub use extractors::*;
pub use exporters::*;

use std::sync::Arc;

use super::ExtensionRegistry;

/// Register all built-in extensions with the pipeline.
pub fn register_builtins(registry: &ExtensionRegistry) {
    // Register issue generators (rules)
    registry.register_validator(Arc::new(rules::TitlePresenceRule::new()));
    registry.register_validator(Arc::new(rules::TitleLengthRule::new()));
    registry.register_validator(Arc::new(rules::MetaDescriptionPresenceRule::new()));
    registry.register_validator(Arc::new(rules::MetaDescriptionLengthRule::new()));
    registry.register_validator(Arc::new(rules::HttpStatusCodeRule::new()));
    registry.register_validator(Arc::new(rules::WordCountRule::new()));
    registry.register_validator(Arc::new(rules::LoadTimeRule::new()));
    
    // Register data extractors
    registry.register_extractor(Arc::new(extractors::OpenGraphExtractor::new()));
    registry.register_extractor(Arc::new(extractors::TwitterCardExtractor::new()));
    registry.register_extractor(Arc::new(extractors::StructuredDataExtractor::new()));
    registry.register_extractor(Arc::new(extractors::HrefTagsExtractor::new()));
    registry.register_extractor(Arc::new(extractors::KeywordsExtractor::new()));
    
    // Register data exporters
    registry.register_exporter(Arc::new(exporters::JsonExporter::new()));
    
    tracing::info!("Registered built-in extensions");
}

/// Get the count of built-in extensions.
pub fn builtin_counts() -> (usize, usize, usize) {
    (7, 5, 1) // rules, extractors, exporters
}

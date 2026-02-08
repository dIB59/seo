//! Type synchronization module for frontend-backend contract.
//!
//! This module re-exports all types that are shared with the TypeScript frontend.
//! Types in this module derive `ts_rs::TS` for automatic TypeScript type generation.
//!
//! ## Generate TypeScript types
//! Run: `cargo test export_bindings -- --nocapture`
//!
//! Generated types are written to: `../src/lib/bindings/`

// Re-export all shared types
pub use crate::domain::models::{
    // Analysis types
    AnalysisProgress,
    AnalysisResults,
    AnalysisSummary,
    CompleteAnalysisResult,
    CompleteJobResult,

    HeadingElement,
    ImageElement,
    IssueSeverity,

    JobInfo,

    JobSettings,
    // Job types
    JobStatus,
    JobSummary,
    // Lighthouse types
    LighthouseData,

    LinkDetail,

    // Link types
    LinkType,

    Page,
    // Page types
    PageAnalysisData,
    PageInfo,
    // Resource types
    ResourceStatus,
    // Issue types
    SeoIssue,
};

pub use crate::commands::analysis::{AnalysisJobResponse, AnalysisSettingsRequest};

/// Export all TypeScript bindings to the frontend bindings directory.
///
/// Run with: `cargo test export_bindings -- --nocapture`
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use ts_rs::TS;

    #[test]
    fn export_bindings() {
        let bindings_dir = Path::new("../src/lib/bindings");

        // Create bindings directory if it doesn't exist
        fs::create_dir_all(bindings_dir).expect("Failed to create bindings directory");

        // Export all types
        AnalysisProgress::export_all_to(bindings_dir).expect("Failed to export AnalysisProgress");
        AnalysisResults::export_all_to(bindings_dir).expect("Failed to export AnalysisResults");
        AnalysisSummary::export_all_to(bindings_dir).expect("Failed to export AnalysisSummary");
        CompleteAnalysisResult::export_all_to(bindings_dir)
            .expect("Failed to export CompleteAnalysisResult");
        PageAnalysisData::export_all_to(bindings_dir).expect("Failed to export PageAnalysisData");
        SeoIssue::export_all_to(bindings_dir).expect("Failed to export SeoIssue");
        IssueSeverity::export_all_to(bindings_dir).expect("Failed to export IssueSeverity");
        JobStatus::export_all_to(bindings_dir).expect("Failed to export JobStatus");
        JobSettings::export_all_to(bindings_dir).expect("Failed to export JobSettings");
        JobSummary::export_all_to(bindings_dir).expect("Failed to export JobSummary");
        LinkType::export_all_to(bindings_dir).expect("Failed to export LinkType");
        ResourceStatus::export_all_to(bindings_dir).expect("Failed to export ResourceStatus");
        AnalysisJobResponse::export_all_to(bindings_dir)
            .expect("Failed to export AnalysisJobResponse");
        AnalysisSettingsRequest::export_all_to(bindings_dir)
            .expect("Failed to export AnalysisSettingsRequest");
        HeadingElement::export_all_to(bindings_dir).expect("Failed to export HeadingElement");
        ImageElement::export_all_to(bindings_dir).expect("Failed to export ImageElement");
        LinkDetail::export_all_to(bindings_dir).expect("Failed to export LinkDetail");
        LighthouseData::export_all_to(bindings_dir).expect("Failed to export LighthouseData");

        println!(
            "✅ TypeScript bindings exported to: {}",
            bindings_dir.display()
        );
    }
}

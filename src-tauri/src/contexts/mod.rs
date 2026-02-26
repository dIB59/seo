// Bounded Contexts for the SEO Analyzer
// Each context represents a distinct domain with clear boundaries

pub mod analysis;
pub mod licensing;
pub mod ai;

// Re-exports for backwards compatibility - types from analysis context
pub use crate::contexts::analysis::{
    extract_host, extract_root_domain, same_root_domain, AnalysisProgress, AnalysisResult,
    CompleteJobResult, Heading, Image, Issue, IssueBuilder, IssueSeverity, Job, JobFilter, JobId,
    JobInfo, JobSettings, JobStatus, JobSummary, LighthouseData, Link, LinkType, NewHeading,
    NewImage, NewIssue, NewLink, NewPageQueueItem, Page, PageDetails, PageInfo, PageQueueItem,
    PageQueueStatus, ResourceStatus,
};

// Re-exports for backwards compatibility - types from ai context
pub use crate::contexts::ai::{AiInsight, PromptBlock, PromptConfig};

// Re-exports for backwards compatibility - types from licensing context
pub use crate::contexts::licensing::{
    AddonError, EntitlementRequest, EntitlementSet, Feature, LicenseActivationRequest, LicenseData,
    LicenseStatus, LicenseTier, LicenseVerifier, LicensingAgent, PermissionRequest, Policy,
    SignedLicense, TierPolicy, TierVersion,
};

// Submodule-like re-exports for backwards compatibility
pub mod permissions {
    pub use crate::contexts::licensing::{
        Feature, LicenseTier, PermissionRequest, Policy, TierPolicy,
    };
}

pub mod link {
    pub use crate::contexts::analysis::NewLink;
}

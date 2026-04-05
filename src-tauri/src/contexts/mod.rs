// Bounded Contexts for the SEO Analyzer
// Each context represents a distinct domain with clear boundaries

pub mod ai;
pub mod analysis;
pub mod extension;
pub mod licensing;

#[allow(unused_imports)]
pub(crate) use crate::contexts::analysis::{
    extract_host, extract_root_domain, same_root_domain, AnalysisProgress, AnalysisResult,
    CompleteJobResult, Heading, Image, Issue, IssueBuilder, IssueSeverity, Job, JobFilter, JobId,
    JobInfo, JobSettings, JobStatus, JobSummary, LighthouseData, Link, LinkType, NewHeading,
    NewImage, NewIssue, NewLink, NewPageQueueItem, Page, PageDetails, PageInfo, PageQueueItem,
    PageQueueStatus, ResourceStatus,
};
#[allow(unused_imports)]
pub(crate) use crate::contexts::ai::{AiInsight, PromptBlock, PromptConfig};
#[allow(unused_imports)]
pub(crate) use crate::contexts::licensing::{
    AddonError, EntitlementRequest, EntitlementSet, Feature, LicenseData,
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



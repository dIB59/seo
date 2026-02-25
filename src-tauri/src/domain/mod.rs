// Domain Module - Re-exports from Bounded Contexts

pub use crate::contexts::analysis::{
    extract_host, extract_root_domain, same_root_domain, AnalysisProgress, AnalysisResult,
    CompleteJobResult, Heading, Image, Issue, IssueBuilder, IssueSeverity, Job, JobFilter, JobId,
    JobInfo, JobSettings, JobStatus, JobSummary, LighthouseData, Link, LinkType, NewHeading,
    NewImage, NewIssue, NewLink, NewPageQueueItem, Page, PageDetails, PageInfo, PageQueueItem,
    PageQueueStatus, ResourceStatus,
};

pub use crate::contexts::ai::{AiInsight, PromptBlock, PromptConfig};

pub use crate::contexts::licensing::{
    AddonError, EntitlementRequest, EntitlementSet, Feature, LicenseActivationRequest, LicenseData,
    LicenseStatus, LicenseTier, LicenseVerifier, LicensingAgent, PermissionRequest, Policy,
    SignedLicense, TierPolicy, TierVersion,
};

pub mod licensing {
    pub use crate::contexts::licensing::{
        AddonError, EntitlementRequest, EntitlementSet, LicenseActivationRequest, LicenseData,
        LicenseStatus, LicenseTier, LicenseVerifier, LicensingAgent, SignedLicense, TierVersion,
    };
}

pub mod permissions {
    pub use crate::contexts::licensing::{
        Feature, LicenseTier, PermissionRequest, Policy, TierPolicy,
    };
}

pub mod link {
    pub use crate::contexts::analysis::NewLink;
}

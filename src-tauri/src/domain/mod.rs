// Domain Module - Re-exports from Bounded Contexts
// This module provides backward compatibility during the DDD migration.
// All types are now owned by their respective bounded contexts.
//
// Migration Note: Update imports from `crate::domain::*` to `crate::contexts::*`
// This module will be deprecated in a future version.

// Analysis Context - Job and Page management
pub use crate::contexts::analysis::{
    Job, JobId, JobSettings, JobStatus, JobInfo, JobFilter, JobSummary, CompleteJobResult,
};
pub use crate::contexts::analysis::{
    Page, PageInfo, PageDetails, PageQueueItem, PageQueueStatus, NewPageQueueItem,
};
pub use crate::contexts::analysis::{
    Issue, NewIssue, IssueBuilder, IssueSeverity,
};
pub use crate::contexts::analysis::{
    Link, NewLink, LinkType,
};
pub use crate::contexts::analysis::LighthouseData;
pub use crate::contexts::analysis::{
    Heading, Image, NewHeading, NewImage, ResourceStatus,
};
pub use crate::contexts::analysis::{
    AnalysisProgress, AnalysisResult,
};
pub use crate::contexts::analysis::{
    extract_root_domain, extract_host, same_root_domain,
};

// Licensing Context - License and Permission management
pub mod licensing {
    pub use crate::contexts::licensing::{
        LicenseTier, LicenseStatus, LicenseData, SignedLicense,
        LicenseActivationRequest, LicenseVerifier, LicensingAgent,
    };
    pub use crate::contexts::licensing::{
        EntitlementRequest, EntitlementSet,
    };
    pub use crate::contexts::licensing::{AddonError, TierVersion};
}

pub mod permissions {
    pub use crate::contexts::licensing::{
        PermissionRequest, Policy, TierPolicy, Feature,
    };
    pub use crate::contexts::licensing::LicenseTier;
}

// AI Context - AI insights and prompts
pub use crate::contexts::ai::{
    AiInsight, PromptConfig, PromptBlock,
};

// Link submodule for backward compatibility with `crate::domain::link::NewLink`
pub mod link {
    pub use crate::contexts::analysis::NewLink;
}

// Top-level licensing types for backward compatibility
pub use crate::contexts::licensing::{
    LicenseTier, LicenseStatus, LicenseData, SignedLicense,
    LicenseActivationRequest, LicenseVerifier, LicensingAgent,
    EntitlementRequest, EntitlementSet, AddonError, TierVersion,
};

// Top-level permission types for backward compatibility
pub use crate::contexts::licensing::{
    PermissionRequest, Policy, TierPolicy, Feature,
};
// Bounded Contexts for the SEO Analyzer
// Each context represents a distinct domain with clear boundaries

pub mod analysis;
pub mod licensing;
pub mod extension;
pub mod ai;

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
    AddonError, EntitlementRequest, EntitlementSet, Feature, LicenseActivationRequest, LicenseData,
    LicenseStatus, LicenseTier, LicenseVerifier, LicensingAgent, PermissionRequest, Policy,
    SignedLicense, TierPolicy, TierVersion,
};

// IssueRuleInfo is now defined here for backward compatibility with the database layer
use serde::{Deserialize, Serialize};

/// Information about an issue rule for the frontend and database
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct IssueRuleInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub severity: String,
    pub rule_type: String,
    pub target_field: Option<String>,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub regex_pattern: Option<String>,
    pub recommendation: Option<String>,
    pub is_builtin: bool,
    pub is_enabled: bool,
}

// Submodule-like re-exports for backwards compatibility
pub mod permissions {
    pub use crate::contexts::licensing::{
        Feature, LicenseTier, PermissionRequest, Policy, TierPolicy,
    };
}

pub mod link {
    pub use crate::contexts::analysis::NewLink;
}



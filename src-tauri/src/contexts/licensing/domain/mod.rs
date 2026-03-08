// Licensing Context Domain Models
// These are the core domain types for the Licensing bounded context.

mod entitlement;
mod license;
mod tier;

// ============================================================================
// License Types
// ============================================================================

pub use license::{
    LicenseData, SignedLicense, LicenseActivationRequest, LicenseVerifier, 
    LicensingAgent, AddonError,
};

// ============================================================================
// Entitlement Types
// ============================================================================

pub use entitlement::{
    LicenseTier, PermissionRequest, Policy, TierPolicy, Feature,
};

// ============================================================================
// Tier Types
// ============================================================================

pub use tier::TierVersion;

// ============================================================================
// Context-Specific Types
// ============================================================================

/// Status of a license
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseStatus {
    /// No license activated
    Unlicensed,
    /// License is valid and active
    Active,
    /// License has expired
    Expired,
    /// License verification failed
    Invalid,
}

/// Request to check an entitlement
#[derive(Debug, Clone)]
pub struct EntitlementRequest {
    pub permission: PermissionRequest,
}

impl From<PermissionRequest> for EntitlementRequest {
    fn from(permission: PermissionRequest) -> Self {
        Self { permission }
    }
}

/// Set of entitlements for a tier
#[derive(Debug, Clone)]
pub struct EntitlementSet {
    pub tier: LicenseTier,
    pub max_pages_per_job: i64,
    pub max_concurrent_jobs: i64,
    pub lighthouse_enabled: bool,
    pub ai_insights_enabled: bool,
    pub deep_crawling_enabled: bool,
}

impl Default for EntitlementSet {
    fn default() -> Self {
        Self::from_tier(LicenseTier::Free)
    }
}

impl EntitlementSet {
    pub fn from_tier(tier: LicenseTier) -> Self {
        match tier {
            LicenseTier::Free => Self {
                tier,
                max_pages_per_job: 100,
                max_concurrent_jobs: 1,
                lighthouse_enabled: false,
                ai_insights_enabled: false,
                deep_crawling_enabled: false,
            },
            LicenseTier::Premium => Self {
                tier,
                max_pages_per_job: 5000,
                max_concurrent_jobs: 5,
                lighthouse_enabled: true,
                ai_insights_enabled: true,
                deep_crawling_enabled: true,
            },
        }
    }
}

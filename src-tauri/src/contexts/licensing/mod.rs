// Licensing Bounded Context
// Handles license verification, entitlements, and tier management

mod domain;

// Domain types exposed to external contexts
pub use domain::{
    LicenseTier, LicenseStatus, LicenseData, SignedLicense,
    LicenseVerifier, LicensingAgent,
};
pub use domain::{PermissionRequest, Policy, TierPolicy, Feature};
pub use domain::{EntitlementRequest, EntitlementSet};
pub use domain::{AddonError, TierVersion};

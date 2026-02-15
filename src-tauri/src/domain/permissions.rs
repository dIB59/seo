use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents a request to perform an action or use a feature.
/// This is the "intent" of the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum PermissionRequest {
    /// Request to analyze N pages
    AnalyzePages(usize),
    /// Request to use a specific feature
    UseFeature(Feature),
}

/// Represents a static capability/feature of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum Feature {
    LinkAnalysis,
    GraphView,
    ExportReports,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, specta::Type)]
pub enum LicenseTier {
    #[default]
    Free,
    Premium,
}

pub trait TierPolicy {
    fn check(&self, request: PermissionRequest) -> bool;
    fn get_policy(&self) -> Policy;
}

impl TierPolicy for LicenseTier {
    fn check(&self, request: PermissionRequest) -> bool {
        self.get_policy().check(request)
    }

    fn get_policy(&self) -> Policy {
        match self {
            LicenseTier::Free => Policy {
                tier: LicenseTier::Free,
                max_pages: 1, // Strict limit for free users
                enabled_features: HashSet::new(),
            },
            LicenseTier::Premium => Policy {
                tier: LicenseTier::Premium,
                max_pages: 100000, // Unlimited for premium
                enabled_features: HashSet::from([
                    Feature::LinkAnalysis,
                    Feature::GraphView,
                    Feature::ExportReports,
                ]),
            },
        }
    }
}

/// Represents the active set of rules for a user.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Policy {
    pub tier: LicenseTier,
    pub max_pages: usize,
    pub enabled_features: HashSet<Feature>,
}

impl Policy {
    pub fn default() -> Self {
        LicenseTier::Free.get_policy()
    }

    pub fn new(tier: LicenseTier) -> Self {
        tier.get_policy()
    }

    /// Check if a request is allowed by this policy.
    pub fn check(&self, request: PermissionRequest) -> bool {
        match request {
            PermissionRequest::AnalyzePages(count) => count <= self.max_pages,
            PermissionRequest::UseFeature(feature) => self.enabled_features.contains(&feature),
        }
    }

    pub fn update_from_tier(&mut self, tier: LicenseTier) {
        *self = tier.get_policy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_tier_restrictions() {
        let policy = Policy::new(LicenseTier::Free);

        // Check page limits
        assert!(policy.check(PermissionRequest::AnalyzePages(1)));
        assert!(!policy.check(PermissionRequest::AnalyzePages(2)));

        // Check features
        assert!(!policy.check(PermissionRequest::UseFeature(Feature::LinkAnalysis)));
    }

    #[test]
    fn test_premium_tier_capabilities() {
        let policy = Policy::new(LicenseTier::Premium);

        // Check page limits
        assert!(policy.check(PermissionRequest::AnalyzePages(1)));
        assert!(policy.check(PermissionRequest::AnalyzePages(10000)));

        // Check features
        assert!(policy.check(PermissionRequest::UseFeature(Feature::LinkAnalysis)));
        assert!(policy.check(PermissionRequest::UseFeature(Feature::GraphView)));
    }
}

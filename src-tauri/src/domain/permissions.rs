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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, specta::Type,
)]
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

impl Default for Policy {
    fn default() -> Self {
        LicenseTier::Free.get_policy()
    }
}

impl Policy {
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
    fn test_policy_default_is_free() {
        let policy = Policy::default();
        assert_eq!(policy.tier, LicenseTier::Free);
        assert_eq!(policy.max_pages, 1);
        assert!(policy.enabled_features.is_empty());
    }

    #[test]
    fn test_policy_update_from_tier() {
        let mut policy = Policy::new(LicenseTier::Free);
        assert_eq!(policy.tier, LicenseTier::Free);

        policy.update_from_tier(LicenseTier::Premium);
        assert_eq!(policy.tier, LicenseTier::Premium);
        assert_eq!(policy.max_pages, 100000);
        assert!(policy.enabled_features.contains(&Feature::LinkAnalysis));
        assert!(policy.enabled_features.contains(&Feature::GraphView));
        assert!(policy.enabled_features.contains(&Feature::ExportReports));

        policy.update_from_tier(LicenseTier::Free);
        assert_eq!(policy.tier, LicenseTier::Free);
        assert_eq!(policy.max_pages, 1);
        assert!(policy.enabled_features.is_empty());
    }

    #[test]
    fn test_all_features_restricted_on_free() {
        let policy = Policy::new(LicenseTier::Free);
        let features = [
            Feature::LinkAnalysis,
            Feature::GraphView,
            Feature::ExportReports,
        ];

        for feature in features {
            assert!(
                !policy.check(PermissionRequest::UseFeature(feature)),
                "Feature {:?} should be restricted on Free tier",
                feature
            );
        }
    }

    #[test]
    fn test_all_features_allowed_on_premium() {
        let policy = Policy::new(LicenseTier::Premium);
        let features = [
            Feature::LinkAnalysis,
            Feature::GraphView,
            Feature::ExportReports,
        ];

        for feature in features {
            assert!(
                policy.check(PermissionRequest::UseFeature(feature)),
                "Feature {:?} should be allowed on Premium tier",
                feature
            );
        }
    }

    #[test]
    fn test_analyze_pages_boundary() {
        let free_policy = Policy::new(LicenseTier::Free);
        assert!(free_policy.check(PermissionRequest::AnalyzePages(1)));
        assert!(!free_policy.check(PermissionRequest::AnalyzePages(2)));

        let premium_policy = Policy::new(LicenseTier::Premium);
        assert!(premium_policy.check(PermissionRequest::AnalyzePages(100000)));
        assert!(!premium_policy.check(PermissionRequest::AnalyzePages(100001)));
    }
}

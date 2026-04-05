use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum PermissionRequest {
    AnalyzePages(usize),
    UseFeature(Feature),
}

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
                max_pages: 1,
                enabled_features: HashSet::new(),
                updates_expired: false,
            },
            LicenseTier::Premium => Policy {
                tier: LicenseTier::Premium,
                max_pages: 100000,
                enabled_features: HashSet::from([
                    Feature::LinkAnalysis,
                    Feature::GraphView,
                    Feature::ExportReports,
                ]),
                updates_expired: false,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Policy {
    pub tier: LicenseTier,
    pub max_pages: usize,
    pub enabled_features: HashSet<Feature>,
    /// True when the installed build is newer than the license's update window.
    /// The app still works — this flag drives a renewal banner in the UI.
    pub updates_expired: bool,
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

    pub fn check(&self, request: PermissionRequest) -> bool {
        match request {
            PermissionRequest::AnalyzePages(count) => count <= self.max_pages,
            PermissionRequest::UseFeature(feature) => self.enabled_features.contains(&feature),
        }
    }

    pub fn update_from_tier(&mut self, tier: LicenseTier) {
        *self = tier.get_policy();
    }

    pub fn from_status(status: crate::contexts::licensing::domain::license::LicenseStatus) -> Self {
        use crate::contexts::licensing::domain::license::LicenseStatus;
        let (tier, updates_expired) = match status {
            LicenseStatus::Active(t) => (t, false),
            LicenseStatus::UpdatesExpired(t) => (t, true),
        };
        let mut policy = tier.get_policy();
        policy.updates_expired = updates_expired;
        policy
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
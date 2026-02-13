use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Addon {
    LinkAnalysis,
    UnlimitedPages,
    GraphView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LicenseTier {
    #[default]
    Free,
    Premium,
}

pub trait Tier {
    fn available_addons(&self) -> HashSet<Addon>;
}

impl Tier for LicenseTier {
    fn available_addons(&self) -> HashSet<Addon> {
        let mut addons = HashSet::new();
        match self {
            LicenseTier::Free => {
                // Free tier might have limited or no addons
            }
            LicenseTier::Premium => {
                addons.insert(Addon::LinkAnalysis);
                addons.insert(Addon::UnlimitedPages);
                addons.insert(Addon::GraphView);
            }
        }
        addons
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub tier: LicenseTier,
    pub active_addons: HashSet<Addon>,
}

impl UserPermissions {
    pub fn default() -> Self {
        Self {
            tier: LicenseTier::Free,
            active_addons: HashSet::new(),
        }
    }

    pub fn new(tier: LicenseTier) -> Self {
        Self {
            tier,
            active_addons: tier.available_addons(),
        }
    }

    pub fn check_addon(&self, addon: Addon) -> bool {
        self.active_addons.contains(&addon)
    }

    pub fn check_addon_str(&self, addon_name: &str) -> bool {
        self.active_addons
            .iter()
            .any(|a| format!("{:?}", a) == addon_name)
    }

    pub fn update_from_tier(&mut self, tier: LicenseTier) {
        self.tier = tier;
        self.active_addons = tier.available_addons();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum AddonError {
    #[error(
        "Feature {0:?} is locked. Access denied.
        Please activate your license to use this feature."
    )]
    FeatureLocked(Addon),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_tier_no_addons() {
        let perms = UserPermissions::new(LicenseTier::Free);
        assert!(!perms.check_addon(Addon::LinkAnalysis));
        assert!(!perms.check_addon(Addon::UnlimitedPages));
        assert!(!perms.check_addon(Addon::GraphView));
    }

    #[test]
    fn test_premium_tier_all_addons() {
        let perms = UserPermissions::new(LicenseTier::Premium);
        assert!(perms.check_addon(Addon::LinkAnalysis));
        assert!(perms.check_addon(Addon::UnlimitedPages));
        assert!(perms.check_addon(Addon::GraphView));
    }

    #[test]
    fn test_check_addon_str() {
        let perms = UserPermissions::new(LicenseTier::Premium);
        assert!(perms.check_addon_str("LinkAnalysis"));
        assert!(!perms.check_addon_str("NonExistentAddon"));
    }

    #[test]
    fn test_tier_update() {
        let mut perms = UserPermissions::new(LicenseTier::Free);
        assert!(!perms.check_addon(Addon::LinkAnalysis));

        perms.update_from_tier(LicenseTier::Premium);
        assert!(perms.check_addon(Addon::LinkAnalysis));
        assert_eq!(perms.tier, LicenseTier::Premium);
    }
}

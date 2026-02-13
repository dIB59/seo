use addon_macros::{addon_guard, AddonProvider};
use serde::Serialize;

#[derive(Debug, Serialize, thiserror::Error)]
pub enum MockError {
    #[error("Mock error")]
    Any,
}

impl From<crate::domain::licensing::AddonError> for MockError {
    fn from(_: crate::domain::licensing::AddonError) -> Self {
        MockError::Any
    }
}

// Mock state that implements AddonProvider
struct MockState {
    has_addon: bool,
}

impl AddonProvider for MockState {
    fn verify_addon(&self, addon_name: &str) -> bool {
        addon_name == "LinkAnalysis" && self.has_addon
    }
}

// Using a crate-relative path for Addon since the macro generates crate::domain::licensing::Addon
// We need to make sure the test can resolve that.
// For testing purposes, we might need a dummy module structure if we run this as an integration test.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::licensing::Addon;

    #[addon_guard(Addon::LinkAnalysis)]
    async fn guarded_function(#[provider] state: &MockState) -> Result<(), MockError> {
        Ok(())
    }

    #[tokio::test]
    async fn test_macro_allows_when_addon_present() {
        let state = MockState { has_addon: true };
        let result = guarded_function(&state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_macro_blocks_when_addon_missing() {
        let state = MockState { has_addon: false };
        let result = guarded_function(&state).await;
        assert!(result.is_err());
    }
}

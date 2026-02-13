use addon_macros::{addon_guard, AddonProvider};
use app::domain::licensing::Addon;
use serde::Serialize; // Import from the library crate 'app'

#[derive(Debug, Serialize, thiserror::Error)]
pub enum MockError {
    #[error("Mock error")]
    Any,
}

impl From<app::domain::licensing::AddonError> for MockError {
    fn from(_: app::domain::licensing::AddonError) -> Self {
        MockError::Any
    }
}

// Re-export app::domain as crate::domain for the macro
pub mod domain {
    pub use app::domain::*;
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

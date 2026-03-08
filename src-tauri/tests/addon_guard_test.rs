use addon_macros::{addon_guard, AddonCheck};
use app::contexts::licensing::{Feature, PermissionRequest};
use serde::Serialize; // Import from the library crate 'app'

#[derive(Debug, Serialize, thiserror::Error)]
pub enum MockError {
    #[error("Mock error")]
    Any,
}

impl From<app::contexts::licensing::AddonError> for MockError {
    fn from(_: app::contexts::licensing::AddonError) -> Self {
        MockError::Any
    }
}

// Re-export app::contexts as crate::contexts for the macro
pub mod contexts {
    pub use app::contexts::*;
}

// Mock state that implements AddonCheck
struct MockState {
    has_feature: bool,
    max_pages: usize,
}

impl AddonCheck<PermissionRequest> for MockState {
    fn check(&self, requirement: PermissionRequest) -> bool {
        match requirement {
            PermissionRequest::UseFeature(Feature::LinkAnalysis) => self.has_feature,
            PermissionRequest::AnalyzePages(count) => count <= self.max_pages,
            _ => false,
        }
    }
}

#[addon_guard(PermissionRequest::UseFeature(Feature::LinkAnalysis))]
async fn guarded_function(#[provider] state: &MockState) -> Result<(), MockError> {
    Ok(())
}

#[addon_guard(PermissionRequest::AnalyzePages(100))]
async fn limit_guarded_function(#[provider] state: &MockState) -> Result<(), MockError> {
    Ok(())
}

#[tokio::test]
async fn test_macro_allows_when_feature_present() {
    let state = MockState {
        has_feature: true,
        max_pages: 0,
    };
    let result = guarded_function(&state).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_macro_blocks_when_feature_missing() {
    let state = MockState {
        has_feature: false,
        max_pages: 0,
    };
    let result = guarded_function(&state).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_macro_allows_when_limit_sufficient() {
    let state = MockState {
        has_feature: false,
        max_pages: 150,
    };
    let result = limit_guarded_function(&state).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_macro_blocks_when_limit_insufficient() {
    let state = MockState {
        has_feature: false,
        max_pages: 50,
    };
    let result = limit_guarded_function(&state).await;
    assert!(result.is_err());
}

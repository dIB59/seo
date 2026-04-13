use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceStatus {
    #[default]
    NotChecked,
    Found(String),
    NotFound,
    Unauthorized(String),
    Error,
}

impl ResourceStatus {
    /// Returns true if the resource exists (Found or Unauthorized)
    pub fn exists(&self) -> bool {
        matches!(self, Self::Found(_) | Self::Unauthorized(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub id: i64,
    pub page_id: String,
    pub level: i64, // 1-6
    pub text: String,
    pub position: i64,
}

#[derive(Debug, Clone)]
pub struct NewHeading {
    pub page_id: String,
    pub level: i64,
    pub text: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: i64,
    pub page_id: String,
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub loading: Option<String>,
    pub is_decorative: bool,
}

#[derive(Debug, Clone)]
pub struct NewImage {
    pub page_id: String,
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub loading: Option<String>,
    pub is_decorative: bool,
}

#[cfg(test)]
mod tests {
    //! Characterization tests for `ResourceStatus`. The `exists()`
    //! predicate is depended on by the discovery service to decide
    //! whether to surface a sitemap/robots.txt warning, so the
    //! Unauthorized → exists=true edge case must stay pinned.

    use super::*;

    #[test]
    fn default_is_not_checked() {
        assert_eq!(ResourceStatus::default(), ResourceStatus::NotChecked);
    }

    #[test]
    fn exists_returns_true_for_found() {
        let s = ResourceStatus::Found("https://example.com/sitemap.xml".to_string());
        assert!(s.exists());
    }

    #[test]
    fn exists_returns_true_for_unauthorized() {
        // Unauthorized still counts as "exists" — a 401 means the
        // resource is there, the crawler just can't read it. Pinning
        // this so a future refactor doesn't accidentally flip it.
        let s = ResourceStatus::Unauthorized("https://example.com/robots.txt".to_string());
        assert!(s.exists());
    }

    #[test]
    fn exists_returns_false_for_not_checked() {
        assert!(!ResourceStatus::NotChecked.exists());
    }

    #[test]
    fn exists_returns_false_for_not_found() {
        assert!(!ResourceStatus::NotFound.exists());
    }

    #[test]
    fn exists_returns_false_for_error() {
        assert!(!ResourceStatus::Error.exists());
    }

    #[test]
    fn serde_uses_snake_case_tags() {
        // Wire format pinned for the Tauri bindings.
        let json = serde_json::to_string(&ResourceStatus::NotChecked).unwrap();
        assert_eq!(json, "\"not_checked\"");
        let json = serde_json::to_string(&ResourceStatus::NotFound).unwrap();
        assert_eq!(json, "\"not_found\"");
    }

    #[test]
    fn serde_round_trip_for_found_variant_carries_url() {
        let original =
            ResourceStatus::Found("https://example.com/sitemap.xml".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let parsed: ResourceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn serde_round_trip_for_unauthorized_variant_carries_url() {
        let original = ResourceStatus::Unauthorized("https://example.com/robots.txt".into());
        let json = serde_json::to_string(&original).unwrap();
        let parsed: ResourceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }
}
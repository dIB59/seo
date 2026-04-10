//! Strongly-typed identifiers for the analysis bounded context.
//!
//! Each ID is a `#[repr(transparent)]` newtype around `String`. The newtypes
//! cost nothing at runtime but make argument-swap bugs (passing a `PageId`
//! where a `JobId` is expected) compile errors.
//!
//! ## Serde / SQL interop
//!
//! All IDs are `#[serde(transparent)]`, so they (de)serialize as plain JSON
//! strings — the existing wire format does not change. For SQL parameter
//! binding, callers can use [`Self::as_str`].

use serde::{Deserialize, Serialize};

/// Error returned when constructing an ID fails validation.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IdError {
    /// The supplied string was empty.
    #[error("id must not be empty")]
    Empty,
}

macro_rules! define_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            Serialize,
            Deserialize,
            specta::Type,
        )]
        #[serde(transparent)]
        #[specta(transparent)]
        #[repr(transparent)]
        pub struct $name(String);

        impl $name {
            /// Generate a new random ID (UUID v4).
            pub fn generate() -> Self {
                Self(uuid::Uuid::new_v4().to_string())
            }

            /// Construct an ID from an existing string, rejecting empty input.
            pub fn new(s: impl Into<String>) -> Result<Self, IdError> {
                let s = s.into();
                if s.is_empty() {
                    return Err(IdError::Empty);
                }
                Ok(Self(s))
            }

            /// Borrow the underlying string slice — useful for SQL parameter
            /// binding and equality checks.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consume the ID and return the owned inner string.
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        // Deref to `str` enables `&JobId` to coerce to `&str` at call sites
        // (sqlx parameters, format args, log fields). The newtype still
        // rejects cross-type assignments — `let _: JobId = page_id;` is a
        // compile error — so type distinction is preserved. This mirrors the
        // `String → str` and `PathBuf → Path` patterns in std.
        impl std::ops::Deref for $name {
            type Target = str;
            fn deref(&self) -> &str {
                &self.0
            }
        }

        // Infallible conversions are deliberate: SQL row decoding and existing
        // call sites that already hold a known-good string should not have to
        // unwrap a Result. Use `new` when validation matters.
        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

define_id!(JobId, "Unique identifier for an analysis job.");
define_id!(PageId, "Unique identifier for a crawled page.");
define_id!(IssueId, "Unique identifier for a detected SEO issue.");
define_id!(LinkId, "Unique identifier for a link edge between pages.");
define_id!(ResourceId, "Unique identifier for a page resource (heading, image, …).");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_unique_ids() {
        let a = JobId::generate();
        let b = JobId::generate();
        assert_ne!(a, b);
        assert!(!a.as_str().is_empty());
    }

    #[test]
    fn new_rejects_empty_string() {
        assert_eq!(JobId::new(""), Err(IdError::Empty));
        assert_eq!(PageId::new(String::new()), Err(IdError::Empty));
    }

    #[test]
    fn new_accepts_non_empty() {
        let id = JobId::new("abc-123").expect("non-empty should succeed");
        assert_eq!(id.as_str(), "abc-123");
    }

    #[test]
    fn display_matches_inner() {
        let id = PageId::from("page-42");
        assert_eq!(format!("{id}"), "page-42");
    }

    #[test]
    fn as_ref_str_borrows_inner() {
        let id = IssueId::from("issue-1");
        let s: &str = id.as_ref();
        assert_eq!(s, "issue-1");
    }

    #[test]
    fn into_string_returns_owned() {
        let id = LinkId::from("link-9");
        assert_eq!(id.into_string(), String::from("link-9"));
    }

    #[test]
    fn serde_round_trip_is_transparent() {
        let id = JobId::from("job-xyz");
        let json = serde_json::to_string(&id).unwrap();
        // Transparent: serializes as a plain JSON string, not an object.
        assert_eq!(json, "\"job-xyz\"");
        let parsed: JobId = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn distinct_id_types_are_not_interchangeable() {
        // This test exists for documentation; the assertion below is a
        // compile-time guarantee enforced by the type system. Attempting
        // `let _: JobId = PageId::generate();` would fail to compile.
        let job: JobId = JobId::generate();
        let page: PageId = PageId::generate();
        assert_ne!(job.as_str(), page.as_str());
    }

    #[test]
    fn from_owned_and_borrowed_string_agree() {
        let owned = ResourceId::from(String::from("res-1"));
        let borrowed = ResourceId::from("res-1");
        assert_eq!(owned, borrowed);
    }

    #[test]
    fn ids_are_hashable_for_use_in_maps() {
        use std::collections::HashSet;
        let mut set: HashSet<JobId> = HashSet::new();
        set.insert(JobId::from("a"));
        set.insert(JobId::from("a"));
        set.insert(JobId::from("b"));
        assert_eq!(set.len(), 2);
    }
}

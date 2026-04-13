//! Repository-layer error type.
//!
//! Replaces the previous `anyhow::Result` returned by every repository
//! method with a structured enum. Calling code can match on
//! [`RepositoryError::NotFound`] without parsing error message strings,
//! and service-layer errors carry repository errors as a `#[from]` source
//! so `?` continues to work seamlessly across layer boundaries.
//!
//! ## Migration plan
//!
//! This type lives alongside the existing `anyhow::Result` traits today.
//! Repository methods will be migrated to return `Result<T, RepositoryError>`
//! one at a time per commit, starting with the simplest (single-row reads).
//! Until then the type is exposed via `From` impls so service-layer code
//! can already begin to match on specific variants.

use thiserror::Error;

/// Errors returned by the repository layer.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// The underlying SQL connection or query failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// A row that the caller required was not present.
    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },

    /// A row that should have been unique already exists.
    #[error("{entity} already exists: {id}")]
    Conflict { entity: &'static str, id: String },

    /// A row was decoded but its contents could not be parsed into a
    /// valid domain value (e.g. an unknown enum variant in a status
    /// column).
    #[error("decode error in {entity}: {message}")]
    Decode {
        entity: &'static str,
        message: String,
    },
}

impl RepositoryError {
    pub fn not_found(entity: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity,
            id: id.into(),
        }
    }

    pub fn conflict(entity: &'static str, id: impl Into<String>) -> Self {
        Self::Conflict {
            entity,
            id: id.into(),
        }
    }

    pub fn decode(entity: &'static str, message: impl Into<String>) -> Self {
        Self::Decode {
            entity,
            message: message.into(),
        }
    }

    /// Stable error code suitable for logging and frontend display.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Database(_) => "REPO_DATABASE",
            Self::NotFound { .. } => "REPO_NOT_FOUND",
            Self::Conflict { .. } => "REPO_CONFLICT",
            Self::Decode { .. } => "REPO_DECODE",
        }
    }
}

/// Result alias for repository operations.
pub type RepositoryResult<T> = std::result::Result<T, RepositoryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_constructor_carries_entity_and_id() {
        let err = RepositoryError::not_found("job", "abc-123");
        match err {
            RepositoryError::NotFound { entity, id } => {
                assert_eq!(entity, "job");
                assert_eq!(id, "abc-123");
            }
            _ => panic!("expected NotFound"),
        }
    }

    #[test]
    fn conflict_constructor_carries_entity_and_id() {
        let err = RepositoryError::conflict("page", "xyz");
        assert!(matches!(err, RepositoryError::Conflict { .. }));
    }

    #[test]
    fn decode_constructor_carries_message() {
        let err = RepositoryError::decode("job", "unknown status: foo");
        match err {
            RepositoryError::Decode { entity, message } => {
                assert_eq!(entity, "job");
                assert!(message.contains("unknown status"));
            }
            _ => panic!("expected Decode"),
        }
    }

    #[test]
    fn code_is_stable_per_variant() {
        assert_eq!(
            RepositoryError::not_found("j", "1").code(),
            "REPO_NOT_FOUND"
        );
        assert_eq!(RepositoryError::conflict("j", "1").code(), "REPO_CONFLICT");
        assert_eq!(RepositoryError::decode("j", "x").code(), "REPO_DECODE");
    }

    #[test]
    fn display_includes_entity_for_not_found() {
        let err = RepositoryError::not_found("job", "abc");
        let s = format!("{err}");
        assert!(s.contains("job"));
        assert!(s.contains("abc"));
    }

    #[test]
    fn from_sqlx_error_preserves_source() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let err: RepositoryError = sqlx_err.into();
        assert!(matches!(err, RepositoryError::Database(_)));
        assert_eq!(err.code(), "REPO_DATABASE");
    }
}

//! Service-layer error type.
//!
//! Service code orchestrates repositories, external HTTP fetches, and
//! domain operations. `ServiceError` is the union of those failure modes
//! and provides `From` impls so `?` flows naturally from each lower
//! layer.
//!
//! ## Why per-layer errors
//!
//! With a single global `AppError`, callers had no way to distinguish
//! "the row doesn't exist" from "the network call timed out" without
//! string-matching on `Display`. Per-layer types let the command layer
//! decide how to map a domain failure into a frontend-friendly code,
//! and let services match on specific repository failures (`NotFound`)
//! to apply business logic.

use thiserror::Error;

use crate::repository::RepositoryError;

#[derive(Debug, Error)]
pub enum ServiceError {
    /// A repository call failed.
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    /// A pagination/filter argument failed validation before reaching
    /// the repository.
    #[error("invalid query: {0}")]
    InvalidQuery(String),

    /// A required precondition for the operation was violated
    /// (e.g. attempting to cancel a job that has already completed).
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// An external dependency (HTTP fetch, sidecar process, LLM API)
    /// reported failure.
    #[error("external service '{service}' failed: {message}")]
    External {
        service: &'static str,
        message: String,
    },

    /// Operation was cancelled by the user or a shutdown signal.
    #[error("operation cancelled")]
    Cancelled,
}

impl ServiceError {
    pub fn invalid_query(msg: impl Into<String>) -> Self {
        Self::InvalidQuery(msg.into())
    }

    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }

    pub fn external(service: &'static str, msg: impl Into<String>) -> Self {
        Self::External {
            service,
            message: msg.into(),
        }
    }

    /// Stable error code suitable for logging and frontend display.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Repository(inner) => inner.code(),
            Self::InvalidQuery(_) => "SVC_INVALID_QUERY",
            Self::InvalidState(_) => "SVC_INVALID_STATE",
            Self::External { .. } => "SVC_EXTERNAL",
            Self::Cancelled => "SVC_CANCELLED",
        }
    }

    /// Did the underlying repository call report "not found"?
    /// Lets services apply special-case business logic without parsing
    /// `Display` strings.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Repository(RepositoryError::NotFound { .. }))
    }
}

pub type ServiceResult<T> = std::result::Result<T, ServiceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_repository_not_found_is_classified() {
        let svc: ServiceError = RepositoryError::not_found("job", "x").into();
        assert!(svc.is_not_found());
        assert_eq!(svc.code(), "REPO_NOT_FOUND");
    }

    #[test]
    fn from_repository_database_error_is_not_classified_as_not_found() {
        let svc: ServiceError = RepositoryError::Database(sqlx::Error::PoolTimedOut).into();
        assert!(!svc.is_not_found());
        assert_eq!(svc.code(), "REPO_DATABASE");
    }

    #[test]
    fn invalid_query_carries_message() {
        let err = ServiceError::invalid_query("limit must be ≥ 1");
        assert!(format!("{err}").contains("limit"));
        assert_eq!(err.code(), "SVC_INVALID_QUERY");
    }

    #[test]
    fn invalid_state_is_distinct_from_invalid_query() {
        let err = ServiceError::invalid_state("job already completed");
        assert_eq!(err.code(), "SVC_INVALID_STATE");
    }

    #[test]
    fn external_carries_service_name() {
        let err = ServiceError::external("gemini", "rate limited");
        match &err {
            ServiceError::External { service, message } => {
                assert_eq!(*service, "gemini");
                assert!(message.contains("rate"));
            }
            _ => panic!("expected External"),
        }
        assert_eq!(err.code(), "SVC_EXTERNAL");
    }

    #[test]
    fn cancelled_has_distinct_code() {
        assert_eq!(ServiceError::Cancelled.code(), "SVC_CANCELLED");
    }

    #[test]
    fn question_mark_works_through_repository_to_service() {
        // Smoke test for the `?` operator wiring across the layer
        // boundary.
        fn repo_call() -> Result<(), RepositoryError> {
            Err(RepositoryError::not_found("page", "p1"))
        }
        fn service_call() -> Result<(), ServiceError> {
            repo_call()?;
            Ok(())
        }
        let err = service_call().unwrap_err();
        assert!(err.is_not_found());
    }
}

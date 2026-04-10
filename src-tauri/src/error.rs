//! Error types for the SEO analyzer application.
//!
//! This module provides `CommandError`, the serializable wrapper that
//! every Tauri command returns to the frontend. Internal layer errors
//! (`RepositoryError`, `ServiceError`, `PaginationError`) all convert
//! into it via `From` impls that prefix the wire string with a stable
//! `[CODE]` prefix the frontend can branch on.
//!
//! The previous global `AppError` enum was removed once the per-layer
//! error refactor (Phase 3) was complete — every layer now uses its
//! own typed error and the global enum was unused dead code.

use serde::Serialize;
use specta::Type;
use std::fmt;

// ============================================================================
// COMMAND ERROR (FOR TAURI)
// ============================================================================

/// Wrapper for errors returned from Tauri commands.
/// This type is serializable and can be sent to the frontend.
#[derive(Debug, Type, Serialize)]
#[specta(transparent)] // Tells Specta: "In TS, this is just a string"
#[serde(transparent)] // Tells Serde: "In JSON, this is just a string"
pub struct CommandError(String);

impl From<anyhow::Error> for CommandError {
    fn from(error: anyhow::Error) -> Self {
        // {:#} gives you the full error chain (all "caused by" messages)
        Self(format!("{:#}", error))
    }
}

impl From<crate::contexts::licensing::AddonError> for CommandError {
    fn from(error: crate::contexts::licensing::AddonError) -> Self {
        Self(error.to_string())
    }
}

impl From<crate::repository::RepositoryError> for CommandError {
    fn from(error: crate::repository::RepositoryError) -> Self {
        // Prefix with the stable code so the frontend can branch on it
        // even though the wire type is currently a flat string.
        Self(format!("[{}] {}", error.code(), error))
    }
}

impl From<crate::service::ServiceError> for CommandError {
    fn from(error: crate::service::ServiceError) -> Self {
        Self(format!("[{}] {}", error.code(), error))
    }
}

impl From<crate::contexts::analysis::PaginationError> for CommandError {
    fn from(error: crate::contexts::analysis::PaginationError) -> Self {
        Self(format!("[SVC_INVALID_QUERY] {}", error))
    }
}

impl std::error::Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    //! Characterization tests for the `CommandError` `From` impls. The
    //! wire format of `CommandError` ships to the frontend
    //! (`specta(transparent)` flattens it to a plain JSON string), so
    //! the prefix codes for typed errors are frontend-observable
    //! contracts.

    use super::*;
    use crate::contexts::analysis::{Pagination, PaginationError};
    use crate::repository::RepositoryError;
    use crate::service::ServiceError;

    // ── CommandError serialization ───────────────────────────────────────

    #[test]
    fn command_error_serializes_as_flat_string() {
        // specta(transparent) + serde(transparent) → JSON string, no
        // wrapping object. Pinning the wire format the frontend reads.
        let err: CommandError = RepositoryError::not_found("page", "p1").into();
        let json = serde_json::to_string(&err).unwrap();
        // Starts and ends with a quote — flat string, not an object.
        assert!(json.starts_with('"'));
        assert!(json.ends_with('"'));
        assert!(!json.contains('{'));
    }

    #[test]
    fn command_error_display_matches_inner_string() {
        let err: CommandError = RepositoryError::not_found("page", "p1").into();
        let s = format!("{err}");
        assert!(s.starts_with("[REPO_NOT_FOUND]"));
        assert!(s.contains("page"));
        assert!(s.contains("p1"));
    }

    // ── From conversions: typed errors get stable code prefix ───────────

    #[test]
    fn from_repository_not_found_prefixes_with_repo_not_found_code() {
        let repo: RepositoryError = RepositoryError::not_found("job", "abc-123");
        let cmd: CommandError = repo.into();
        let s = format!("{cmd}");
        assert!(s.starts_with("[REPO_NOT_FOUND]"));
        assert!(s.contains("job"));
        assert!(s.contains("abc-123"));
    }

    #[test]
    fn from_repository_database_prefixes_with_repo_database_code() {
        let repo: RepositoryError = RepositoryError::Database(sqlx::Error::PoolTimedOut);
        let cmd: CommandError = repo.into();
        let s = format!("{cmd}");
        assert!(s.starts_with("[REPO_DATABASE]"));
    }

    #[test]
    fn from_repository_conflict_prefixes_with_repo_conflict_code() {
        let cmd: CommandError = RepositoryError::conflict("page", "p1").into();
        let s = format!("{cmd}");
        assert!(s.starts_with("[REPO_CONFLICT]"));
    }

    #[test]
    fn from_repository_decode_prefixes_with_repo_decode_code() {
        let cmd: CommandError = RepositoryError::decode("job", "bad enum").into();
        let s = format!("{cmd}");
        assert!(s.starts_with("[REPO_DECODE]"));
        assert!(s.contains("bad enum"));
    }

    #[test]
    fn from_service_invalid_query_prefixes_with_svc_invalid_query_code() {
        let svc: ServiceError = ServiceError::invalid_query("limit must be ≥ 1");
        let cmd: CommandError = svc.into();
        let s = format!("{cmd}");
        assert!(s.starts_with("[SVC_INVALID_QUERY]"));
    }

    #[test]
    fn from_service_external_prefixes_with_svc_external_code() {
        let svc: ServiceError = ServiceError::external("gemini", "rate limit");
        let cmd: CommandError = svc.into();
        let s = format!("{cmd}");
        assert!(s.starts_with("[SVC_EXTERNAL]"));
    }

    #[test]
    fn from_service_cancelled_prefixes_with_svc_cancelled_code() {
        let cmd: CommandError = ServiceError::Cancelled.into();
        let s = format!("{cmd}");
        assert!(s.starts_with("[SVC_CANCELLED]"));
    }

    #[test]
    fn from_service_repository_propagates_repo_code() {
        // ServiceError wraps RepositoryError; the code() should be the
        // repo code, not a generic SVC_* one.
        let svc: ServiceError = RepositoryError::not_found("page", "p1").into();
        let cmd: CommandError = svc.into();
        let s = format!("{cmd}");
        assert!(s.starts_with("[REPO_NOT_FOUND]"));
    }

    #[test]
    fn from_pagination_error_prefixes_with_svc_invalid_query_code() {
        let pag_err = Pagination::new(0, 0).unwrap_err();
        assert!(matches!(pag_err, PaginationError::LimitTooSmall));
        let cmd: CommandError = pag_err.into();
        let s = format!("{cmd}");
        assert!(s.starts_with("[SVC_INVALID_QUERY]"));
    }

    #[test]
    fn from_anyhow_uses_alternate_format_for_full_chain() {
        // The `{:#}` Display format on anyhow::Error renders the full
        // "outer: inner" chain. Pinning the chain comes through.
        let inner = anyhow::anyhow!("inner cause");
        let outer = inner.context("outer context");
        let cmd: CommandError = outer.into();
        let s = format!("{cmd}");
        assert!(s.contains("outer context"));
        assert!(s.contains("inner cause"));
    }

}

//! Pagination primitives for list queries.
//!
//! Replaces ad-hoc `(limit: i64, offset: i64)` parameter pairs throughout the
//! repository layer. The smart constructor enforces sane bounds — limit in
//! `[1, 500]`, offset `>= 0` — so any `Pagination` value reaching SQL has
//! already been validated.

use serde::{Deserialize, Serialize};

/// Maximum allowed page size. Above this we'd risk loading too many rows
/// into memory at once and slowing down SQLite scans.
pub const MAX_LIMIT: i64 = 500;

/// Errors returned by [`Pagination::new`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PaginationError {
    #[error("limit must be at least 1")]
    LimitTooSmall,
    #[error("limit must be at most {MAX_LIMIT}")]
    LimitTooLarge,
    #[error("offset must be non-negative")]
    NegativeOffset,
}

/// Validated pagination window.
///
/// Construct via [`Pagination::new`]; the inner fields are private so a
/// `Pagination` value is always in-range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
    limit: i64,
    offset: i64,
}

impl Pagination {
    pub fn new(limit: i64, offset: i64) -> Result<Self, PaginationError> {
        if limit < 1 {
            return Err(PaginationError::LimitTooSmall);
        }
        if limit > MAX_LIMIT {
            return Err(PaginationError::LimitTooLarge);
        }
        if offset < 0 {
            return Err(PaginationError::NegativeOffset);
        }
        Ok(Self { limit, offset })
    }

    /// Return a default first-page window of `limit` items.
    pub fn first_page(limit: i64) -> Result<Self, PaginationError> {
        Self::new(limit, 0)
    }

    pub fn limit(self) -> i64 {
        self.limit
    }

    pub fn offset(self) -> i64 {
        self.offset
    }
}

impl Default for Pagination {
    /// First page of 50 items — chosen as a sensible UI default.
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

/// Filter + pagination bundle for the `JobRepository::get_paginated_with_total`
/// query. Bundles the four positional parameters (limit, offset, url filter,
/// status filter) into a single named struct so callers can build it
/// incrementally and so adding a new filter is non-breaking.
#[derive(Debug, Clone, Default)]
pub struct JobPageQuery {
    pagination: Pagination,
    url_contains: Option<String>,
    status: Option<String>,
}

impl JobPageQuery {
    pub fn new(pagination: Pagination) -> Self {
        Self {
            pagination,
            url_contains: None,
            status: None,
        }
    }

    pub fn with_url_filter(mut self, contains: impl Into<String>) -> Self {
        self.url_contains = Some(contains.into());
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn pagination(&self) -> Pagination {
        self.pagination
    }

    pub fn url_contains(&self) -> Option<&str> {
        self.url_contains.as_deref()
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Consume the query, returning owned filter strings (used by repository
    /// SQL builders that need to bind owned values).
    pub fn into_parts(self) -> (Pagination, Option<String>, Option<String>) {
        (self.pagination, self.url_contains, self.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_limit() {
        assert_eq!(Pagination::new(0, 0), Err(PaginationError::LimitTooSmall));
    }

    #[test]
    fn rejects_negative_limit() {
        assert_eq!(Pagination::new(-5, 0), Err(PaginationError::LimitTooSmall));
    }

    #[test]
    fn rejects_oversized_limit() {
        assert_eq!(
            Pagination::new(MAX_LIMIT + 1, 0),
            Err(PaginationError::LimitTooLarge)
        );
    }

    #[test]
    fn accepts_max_limit() {
        let p = Pagination::new(MAX_LIMIT, 0).unwrap();
        assert_eq!(p.limit(), MAX_LIMIT);
    }

    #[test]
    fn rejects_negative_offset() {
        assert_eq!(
            Pagination::new(10, -1),
            Err(PaginationError::NegativeOffset)
        );
    }

    #[test]
    fn accepts_valid_window() {
        let p = Pagination::new(25, 100).unwrap();
        assert_eq!(p.limit(), 25);
        assert_eq!(p.offset(), 100);
    }

    #[test]
    fn first_page_has_zero_offset() {
        let p = Pagination::first_page(20).unwrap();
        assert_eq!(p.offset(), 0);
        assert_eq!(p.limit(), 20);
    }

    #[test]
    fn default_is_first_page_of_fifty() {
        let p = Pagination::default();
        assert_eq!(p.limit(), 50);
        assert_eq!(p.offset(), 0);
    }
}

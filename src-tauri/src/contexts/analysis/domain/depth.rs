//! Crawl depth newtype.
//!
//! Replaces the bare `i64` used for `Page.depth`, `PageQueueItem.depth`,
//! and the `depth` parameter on analyzer/crawler methods. The newtype
//! prevents two classes of bug:
//!
//! 1. **Negative depth**: SQL columns are `INTEGER`, which permits
//!    negatives, but a negative crawl depth is meaningless. The smart
//!    constructor enforces `>= 0`.
//! 2. **Unbounded recursion**: an out-of-control crawl with a malformed
//!    depth field could spiral. [`MAX_DEPTH`] caps it at 50 — far above
//!    any realistic site structure but well below stack-overflow risk.
//!
//! ## Migration plan
//!
//! - **Step 1 (this commit)**: type lives alongside the existing `i64`
//!   fields. New code can opt in via `Depth::new` / `Depth::root`.
//! - **Step 2 (later)**: `PageQueueItem.depth` and `Page.depth` migrate
//!   to `Depth`, with sqlx `From<i64>` interop at the row decoder.
//! - **Step 3 (later)**: analyzer/crawler signatures take `Depth` so
//!   `&job_id, depth` can no longer be swapped at the call site.

use serde::{Deserialize, Serialize};

/// Hard upper bound on crawl depth. Anything deeper is almost certainly
/// a misconfigured loop or an infinitely-paginating site.
pub const MAX_DEPTH: i64 = 50;

/// Errors returned by [`Depth::new`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DepthError {
    #[error("depth must be non-negative")]
    Negative,
    #[error("depth must be at most {MAX_DEPTH}")]
    TooDeep,
}

/// Validated crawl depth.
///
/// Construct via [`Depth::new`] or [`Depth::root`]; the inner field is
/// private so any `Depth` value reaching SQL or the crawler has already
/// been bounds-checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Depth(i64);

impl Depth {
    /// Construct a new `Depth`, rejecting negative or out-of-range values.
    pub fn new(value: i64) -> Result<Self, DepthError> {
        if value < 0 {
            return Err(DepthError::Negative);
        }
        if value > MAX_DEPTH {
            return Err(DepthError::TooDeep);
        }
        Ok(Self(value))
    }

    /// The root URL — depth 0. Cannot fail.
    pub const fn root() -> Self {
        Self(0)
    }

    /// The depth of a child page = parent depth + 1, capped at
    /// [`MAX_DEPTH`]. Saturating semantics so the crawler never panics
    /// on an over-deep site.
    pub fn child(self) -> Self {
        Self((self.0 + 1).min(MAX_DEPTH))
    }

    /// Raw integer for SQL parameter binding. Prefer this over `.0`
    /// to keep call sites self-documenting.
    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for Depth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_is_zero() {
        assert_eq!(Depth::root().as_i64(), 0);
    }

    #[test]
    fn rejects_negative() {
        assert_eq!(Depth::new(-1), Err(DepthError::Negative));
        assert_eq!(Depth::new(-100), Err(DepthError::Negative));
    }

    #[test]
    fn accepts_zero() {
        let d = Depth::new(0).unwrap();
        assert_eq!(d, Depth::root());
    }

    #[test]
    fn accepts_max_depth() {
        let d = Depth::new(MAX_DEPTH).unwrap();
        assert_eq!(d.as_i64(), MAX_DEPTH);
    }

    #[test]
    fn rejects_above_max() {
        assert_eq!(Depth::new(MAX_DEPTH + 1), Err(DepthError::TooDeep));
        assert_eq!(Depth::new(i64::MAX), Err(DepthError::TooDeep));
    }

    #[test]
    fn child_increments() {
        assert_eq!(Depth::root().child().as_i64(), 1);
        assert_eq!(Depth::new(5).unwrap().child().as_i64(), 6);
    }

    #[test]
    fn child_saturates_at_max_depth() {
        let max = Depth::new(MAX_DEPTH).unwrap();
        assert_eq!(max.child(), max, "child of MAX_DEPTH should still be MAX_DEPTH");
    }

    #[test]
    fn ordering_is_total() {
        let a = Depth::new(2).unwrap();
        let b = Depth::new(5).unwrap();
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
    }

    #[test]
    fn serde_round_trip_is_transparent() {
        let d = Depth::new(7).unwrap();
        let json = serde_json::to_string(&d).unwrap();
        // Transparent: serialized as a plain JSON number, not an object.
        assert_eq!(json, "7");
        let parsed: Depth = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, d);
    }

    #[test]
    fn display_matches_inner_integer() {
        let d = Depth::new(3).unwrap();
        assert_eq!(format!("{d}"), "3");
    }

    #[test]
    fn copy_is_implemented_for_cheap_passing() {
        // Compile-time evidence that Depth is Copy — the test would fail
        // to compile otherwise.
        let d = Depth::new(1).unwrap();
        let _a = d;
        let _b = d;
        assert_eq!(d.as_i64(), 1);
    }
}

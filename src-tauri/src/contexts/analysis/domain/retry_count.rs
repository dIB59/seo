//! Retry counter newtype for the page queue.
//!
//! Replaces the bare `i64` `retry_count` field on `PageQueueItem`. The
//! newtype enforces a non-negative invariant and provides a saturating
//! `increment` method so the counter can never overflow or roll back to
//! a misleading state.

use serde::{Deserialize, Serialize};

/// Hard ceiling on retry attempts. Above this we stop retrying — far
/// more than any production crawl needs but well below the boundary
/// where exponential backoff would create absurd delays.
pub const MAX_RETRY_COUNT: i64 = 1_000;

/// Errors returned by [`RetryCount::new`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RetryCountError {
    #[error("retry count must be non-negative")]
    Negative,
    #[error("retry count must be at most {MAX_RETRY_COUNT}")]
    TooHigh,
}

/// Validated retry counter. Always in `[0, MAX_RETRY_COUNT]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RetryCount(i64);

impl RetryCount {
    /// Construct from a raw integer, rejecting out-of-range values.
    pub fn new(value: i64) -> Result<Self, RetryCountError> {
        if value < 0 {
            return Err(RetryCountError::Negative);
        }
        if value > MAX_RETRY_COUNT {
            return Err(RetryCountError::TooHigh);
        }
        Ok(Self(value))
    }

    /// Zero retries — the natural starting state.
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Increment the counter, saturating at [`MAX_RETRY_COUNT`] so the
    /// caller never has to handle overflow.
    pub fn increment(self) -> Self {
        Self((self.0 + 1).min(MAX_RETRY_COUNT))
    }

    /// Whether the counter has room for another retry given a `limit`.
    /// `limit` is the maximum number of retries the caller will tolerate;
    /// returns `true` while `self < limit`.
    pub fn can_retry(self, limit: i64) -> bool {
        self.0 < limit
    }

    /// Raw integer for SQL parameter binding.
    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl Default for RetryCount {
    fn default() -> Self {
        Self::zero()
    }
}

impl std::fmt::Display for RetryCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_default() {
        assert_eq!(RetryCount::default(), RetryCount::zero());
        assert_eq!(RetryCount::zero().as_i64(), 0);
    }

    #[test]
    fn rejects_negative() {
        assert_eq!(RetryCount::new(-1), Err(RetryCountError::Negative));
    }

    #[test]
    fn rejects_above_max() {
        assert_eq!(
            RetryCount::new(MAX_RETRY_COUNT + 1),
            Err(RetryCountError::TooHigh)
        );
    }

    #[test]
    fn accepts_zero_and_max() {
        assert!(RetryCount::new(0).is_ok());
        assert!(RetryCount::new(MAX_RETRY_COUNT).is_ok());
    }

    #[test]
    fn increment_steps_up_by_one() {
        let r = RetryCount::zero().increment();
        assert_eq!(r.as_i64(), 1);
        let r = r.increment().increment();
        assert_eq!(r.as_i64(), 3);
    }

    #[test]
    fn increment_saturates_at_max() {
        let max = RetryCount::new(MAX_RETRY_COUNT).unwrap();
        assert_eq!(max.increment(), max);
    }

    #[test]
    fn can_retry_compares_against_limit() {
        let r = RetryCount::zero();
        assert!(r.can_retry(3));
        let r = r.increment().increment().increment();
        assert!(!r.can_retry(3));
        assert!(r.can_retry(4));
    }

    #[test]
    fn can_retry_zero_limit_always_false() {
        assert!(!RetryCount::zero().can_retry(0));
    }

    #[test]
    fn ordering_is_total() {
        let a = RetryCount::new(2).unwrap();
        let b = RetryCount::new(5).unwrap();
        assert!(a < b);
        assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
    }

    #[test]
    fn serde_round_trip_is_transparent() {
        let r = RetryCount::new(7).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(json, "7");
        let parsed: RetryCount = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, r);
    }

    #[test]
    fn display_matches_inner_integer() {
        assert_eq!(format!("{}", RetryCount::new(4).unwrap()), "4");
    }
}

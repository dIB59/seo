//! Typed value object for license tier versions (major.minor.patch)
//!
//! Serialized as a string (e.g. "1.2.3") for compactness and compatibility.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TierVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl TierVersion {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl std::fmt::Display for TierVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Returned by [`TierVersion::from_str`]. The previous `type Err = String`
/// was untyped (no equality, no PartialEq for tests, no thiserror chain).
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ParseTierVersionError {
    #[error("invalid version: '{0}' (expected `major.minor.patch`)")]
    Shape(String),
    #[error("invalid version component '{0}': {1}")]
    Component(String, std::num::ParseIntError),
}

impl FromStr for TierVersion {
    type Err = ParseTierVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(ParseTierVersionError::Shape(s.to_string()));
        }

        let parse = |i: usize| -> Result<u64, ParseTierVersionError> {
            parts[i]
                .parse::<u64>()
                .map_err(|e| ParseTierVersionError::Component(parts[i].to_string(), e))
        };

        Ok(TierVersion::new(parse(0)?, parse(1)?, parse(2)?))
    }
}

impl Serialize for TierVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TierVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn parse_and_display() {
        let v: TierVersion = "1.2.3".parse().unwrap();
        assert_eq!(v, TierVersion::new(1, 2, 3));
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn cmp_and_ordering() {
        let a = TierVersion::new(1, 0, 0);
        let b = TierVersion::new(1, 2, 0);
        let c = TierVersion::new(2, 0, 0);

        assert!(a < b);
        assert!(b < c);
        assert!(c > a);
        assert_eq!(a, TierVersion::new(1, 0, 0));
    }

    #[test]
    fn serde_roundtrip() {
        let v = TierVersion::new(2, 3, 4);
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, '"'.to_string() + "2.3.4" + '"'.to_string().as_str());
        let v2: TierVersion = serde_json::from_str(&s).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn from_str_rejects_too_few_parts() {
        assert!(TierVersion::from_str("1.2").is_err());
        assert!(TierVersion::from_str("1").is_err());
        assert!(TierVersion::from_str("").is_err());
    }

    #[test]
    fn from_str_rejects_too_many_parts() {
        assert!(TierVersion::from_str("1.2.3.4").is_err());
        assert!(TierVersion::from_str("1.2.3.4.5").is_err());
    }

    #[test]
    fn from_str_rejects_non_numeric_parts() {
        assert!(TierVersion::from_str("a.b.c").is_err());
        assert!(TierVersion::from_str("1.x.3").is_err());
        assert!(TierVersion::from_str("1.2.foo").is_err());
        // No "v" prefix support — pinning we don't accidentally accept
        // common version-string conventions like "v1.0.0".
        assert!(TierVersion::from_str("v1.2.3").is_err());
    }

    #[test]
    fn from_str_rejects_negative_parts() {
        // u64 doesn't accept negatives — pinning the natural failure
        // mode so a future signed migration is a deliberate decision.
        assert!(TierVersion::from_str("-1.2.3").is_err());
        assert!(TierVersion::from_str("1.-2.3").is_err());
    }

    #[test]
    fn from_str_accepts_zero_components() {
        let v: TierVersion = "0.0.0".parse().unwrap();
        assert_eq!(v, TierVersion::new(0, 0, 0));
    }

    #[test]
    fn from_str_accepts_leading_zeros() {
        // Pin behaviour: u64::from_str accepts leading zeros silently.
        // "01" parses as 1, etc.
        let v: TierVersion = "01.02.03".parse().unwrap();
        assert_eq!(v, TierVersion::new(1, 2, 3));
    }

    #[test]
    fn from_str_handles_large_numbers() {
        let v: TierVersion = "1000000.999999.42".parse().unwrap();
        assert_eq!(v.major, 1_000_000);
        assert_eq!(v.minor, 999_999);
        assert_eq!(v.patch, 42);
    }

    #[test]
    fn ordering_compares_major_first_then_minor_then_patch() {
        // Pin lexicographic ordering on (major, minor, patch).
        let a = TierVersion::new(1, 0, 100);
        let b = TierVersion::new(1, 1, 0);
        // minor breaks the tie even though a.patch >> b.patch
        assert!(a < b);

        let c = TierVersion::new(0, 99, 99);
        let d = TierVersion::new(1, 0, 0);
        // major breaks the tie even though c.minor + c.patch >> d's
        assert!(c < d);
    }

    #[test]
    fn ordering_is_total() {
        // Concrete check on Ord — pin that compare returns the
        // expected ordering enum.
        let a = TierVersion::new(2, 0, 0);
        let b = TierVersion::new(2, 0, 0);
        assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
    }

    #[test]
    fn copy_semantics() {
        // Compile-time evidence that TierVersion is Copy.
        let a = TierVersion::new(1, 2, 3);
        let b = a;
        let c = a;
        assert_eq!(b, c);
    }

    #[test]
    fn deserialize_rejects_malformed_string() {
        let result: Result<TierVersion, _> = serde_json::from_str("\"not-a-version\"");
        assert!(result.is_err());
    }

    #[test]
    fn const_new_is_usable_in_const_context() {
        // Pin that `new` is `const` — the licensing module uses
        // TierVersion in const contexts elsewhere.
        const V: TierVersion = TierVersion::new(1, 2, 3);
        assert_eq!(V.major, 1);
    }
}
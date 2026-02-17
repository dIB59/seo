//! Typed value object for license tier versions (major.minor.patch)
//!
//! Serialized as a string (e.g. "1.2.3") for compactness and compatibility.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Value object for tier versions — parsed, comparable and serialized as
/// a `major.minor.patch` string. Implements `PartialEq`/`PartialOrd` so callers
/// can use `==` / `<` / `>` directly.
#[derive(Debug, Clone, Copy)]
pub struct TierVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl TierVersion {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self { major, minor, patch }
    }
}

impl std::fmt::Display for TierVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for TierVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("invalid version: {}", s));
        }

        let major = parts[0].parse::<u64>().map_err(|e| e.to_string())?;
        let minor = parts[1].parse::<u64>().map_err(|e| e.to_string())?;
        let patch = parts[2].parse::<u64>().map_err(|e| e.to_string())?;

        Ok(TierVersion::new(major, minor, patch))
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

impl PartialEq for TierVersion {
    fn eq(&self, other: &Self) -> bool {
        (self.major, self.minor, self.patch) == (other.major, other.minor, other.patch)
    }
}
impl Eq for TierVersion {}

impl PartialOrd for TierVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TierVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
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
}

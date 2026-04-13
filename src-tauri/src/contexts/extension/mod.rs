use serde::{Deserialize, Serialize};

// Reuse the canonical IssueSeverity from the analysis domain.
pub use crate::contexts::analysis::IssueSeverity;

/// Condition operator for a custom check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Missing,
    Present,
    Eq,
    Lt,
    Gt,
    Contains,
    NotContains,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(f, "missing"),
            Self::Present => write!(f, "present"),
            Self::Eq => write!(f, "eq"),
            Self::Lt => write!(f, "lt"),
            Self::Gt => write!(f, "gt"),
            Self::Contains => write!(f, "contains"),
            Self::NotContains => write!(f, "not_contains"),
        }
    }
}

/// Returned by [`Operator::from_str`] when the input doesn't map to a
/// known operator. Carries the offending string so logs / decoder
/// errors surface what was actually seen — same shape as
/// `ParseLinkTypeError`, `ParseIssueSeverityError`, etc.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("invalid operator: '{0}'")]
pub struct ParseOperatorError(pub String);

impl std::str::FromStr for Operator {
    type Err = ParseOperatorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "missing" => Ok(Self::Missing),
            "present" => Ok(Self::Present),
            "eq" => Ok(Self::Eq),
            "lt" => Ok(Self::Lt),
            "gt" => Ok(Self::Gt),
            "contains" => Ok(Self::Contains),
            "not_contains" => Ok(Self::NotContains),
            other => Err(ParseOperatorError(other.to_string())),
        }
    }
}

/// A user-defined check that inspects extracted page data and produces an issue.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CustomCheck {
    pub id: String,
    pub name: String,
    pub severity: IssueSeverity,
    /// The extracted_data key (or built-in page field) to evaluate.
    pub field: String,
    pub operator: Operator,
    /// Threshold value for numeric/text comparisons; unused for `Missing`.
    pub threshold: Option<String>,
    /// Template for the issue message. `{value}` is replaced with the actual field value.
    pub message_template: String,
    pub enabled: bool,
}

/// Parameters for creating or updating a custom check.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CustomCheckParams {
    pub name: String,
    pub severity: IssueSeverity,
    pub field: String,
    pub operator: Operator,
    pub threshold: Option<String>,
    pub message_template: String,
    pub enabled: bool,
}

/// A user-defined CSS-selector extractor that populates `page.extracted_data`.
///
/// `tag` (formerly `key`) is the symbol the consultant references from
/// custom checks, report templates, and AI prompts. A tag defined here
/// becomes reachable as:
///   - `tag:<tag>` in a `CustomCheck` or `ReportPattern` `field`.
///   - `{tag.<tag>}` in a `CustomCheck.message_template` (chunk 3).
///   - `{tag.<tag>}` in report template text and AI prompt blocks (chunk 4).
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CustomExtractor {
    pub id: String,
    pub name: String,
    /// The user-defined tag this extractor publishes into
    /// `page.extracted_data`. Must be unique across all extractors —
    /// the DB column has a `UNIQUE` constraint.
    pub tag: String,
    pub selector: String,
    pub attribute: Option<String>,
    pub multiple: bool,
    pub enabled: bool,
}

/// Parameters for creating or updating a custom extractor.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CustomExtractorParams {
    pub name: String,
    pub tag: String,
    pub selector: String,
    pub attribute: Option<String>,
    pub multiple: bool,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    //! Characterization tests for the extension domain types. The
    //! `Operator` enum's parse path is depended on by the SQLite
    //! `extension_repository::CheckRow::into_domain` decoder, so
    //! breaking the round-trip would surface as a `RepositoryError::Decode`
    //! at runtime — these tests pin the contract.

    use super::*;
    use std::str::FromStr;

    #[test]
    fn operator_display_round_trips_through_from_str() {
        for op in [
            Operator::Missing,
            Operator::Present,
            Operator::Eq,
            Operator::Lt,
            Operator::Gt,
            Operator::Contains,
            Operator::NotContains,
        ] {
            let s = op.to_string();
            let parsed = Operator::from_str(&s).expect("round-trip should succeed");
            assert_eq!(parsed, op, "round trip failed for {op:?}");
        }
    }

    #[test]
    fn operator_from_str_uses_snake_case() {
        // The SQL column stores the snake_case form via Display.
        assert_eq!(Operator::from_str("not_contains").unwrap(), Operator::NotContains);
        assert_eq!(Operator::from_str("missing").unwrap(), Operator::Missing);
    }

    #[test]
    fn operator_from_str_is_case_sensitive() {
        // Migrations / data inserted by hand should never use uppercase;
        // pinning this prevents accidental drift.
        assert!(Operator::from_str("MISSING").is_err());
        assert!(Operator::from_str("NotContains").is_err());
    }

    #[test]
    fn operator_from_str_rejects_unknown_value() {
        let err = Operator::from_str("nonsense").unwrap_err();
        assert!(format!("{err}").contains("nonsense"));
    }

    #[test]
    fn operator_serde_uses_snake_case_too() {
        // The wire format is snake_case via #[serde(rename_all)] — Tauri
        // bindings depend on it.
        let json = serde_json::to_string(&Operator::NotContains).unwrap();
        assert_eq!(json, "\"not_contains\"");
        let parsed: Operator = serde_json::from_str("\"not_contains\"").unwrap();
        assert_eq!(parsed, Operator::NotContains);
    }

    #[test]
    fn custom_check_clone_and_eq_via_serde() {
        // The struct doesn't derive PartialEq directly; round-trip via
        // serde_json gives us a comparable representation that pins the
        // wire shape.
        let check = CustomCheck {
            id: "abc".into(),
            name: "Title length".into(),
            severity: IssueSeverity::Warning,
            field: "title".into(),
            operator: Operator::Lt,
            threshold: Some("30".into()),
            message_template: "Title is too short ({value} chars)".into(),
            enabled: true,
        };
        let json = serde_json::to_value(&check).unwrap();
        assert_eq!(json["id"], "abc");
        assert_eq!(json["operator"], "lt");
        assert_eq!(json["enabled"], true);
        // Threshold deserializes back to Option<String>.
        let parsed: CustomCheck = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.threshold.as_deref(), Some("30"));
    }

    #[test]
    fn custom_extractor_threshold_field_optional() {
        let extractor = CustomExtractor {
            id: "x".into(),
            name: "OG image".into(),
            tag: "og_image".into(),
            selector: "meta[property='og:image']".into(),
            attribute: Some("content".into()),
            multiple: false,
            enabled: true,
        };
        let json = serde_json::to_value(&extractor).unwrap();
        assert_eq!(json["attribute"], "content");
        let no_attr = CustomExtractor {
            attribute: None,
            ..extractor
        };
        let json = serde_json::to_value(&no_attr).unwrap();
        assert!(json["attribute"].is_null());
    }
}

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

impl std::str::FromStr for Operator {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "missing" => Ok(Self::Missing),
            "present" => Ok(Self::Present),
            "eq" => Ok(Self::Eq),
            "lt" => Ok(Self::Lt),
            "gt" => Ok(Self::Gt),
            "contains" => Ok(Self::Contains),
            "not_contains" => Ok(Self::NotContains),
            other => Err(anyhow::anyhow!("Unknown operator: {}", other)),
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

/// A user-defined CSS-selector extractor that populates extracted_data.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CustomExtractor {
    pub id: String,
    pub name: String,
    /// Key written into `page.extracted_data`.
    pub key: String,
    pub selector: String,
    pub attribute: Option<String>,
    pub multiple: bool,
    pub enabled: bool,
}

/// Parameters for creating or updating a custom extractor.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CustomExtractorParams {
    pub name: String,
    pub key: String,
    pub selector: String,
    pub attribute: Option<String>,
    pub multiple: bool,
    pub enabled: bool,
}

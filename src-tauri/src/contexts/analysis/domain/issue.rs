use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

impl IssueSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// Returned by [`IssueSeverity::from_str`] when the input doesn't map to a
/// known severity. Carries the offending string so logs / decoder errors can
/// surface what was actually seen instead of a bare "parse failed".
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("invalid issue severity: '{0}'")]
pub struct ParseIssueSeverityError(pub String);

impl std::str::FromStr for IssueSeverity {
    type Err = ParseIssueSeverityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Self::Critical),
            "warning" => Ok(Self::Warning),
            "info" | "suggestion" => Ok(Self::Info),
            other => Err(ParseIssueSeverityError(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub id: i64,
    pub job_id: String,
    pub page_id: Option<String>,

    pub issue_type: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub details: Option<String>,

    pub created_at: DateTime<Utc>,
}

pub struct IssueBuilder {
    job_id: String,
    page_id: Option<String>,
    issue_type: String,
    severity: IssueSeverity,
    message: String,
    details: Option<String>,
}

impl IssueBuilder {
    pub fn new(
        job_id: String,
        issue_type: String,
        severity: IssueSeverity,
        message: String,
    ) -> Self {
        Self {
            job_id,
            page_id: None,
            issue_type,
            severity,
            message,
            details: None,
        }
    }

    pub fn page_id(mut self, page_id: String) -> Self {
        self.page_id = Some(page_id);
        self
    }

    pub fn details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    pub fn build(self) -> NewIssue {
        NewIssue {
            job_id: self.job_id,
            page_id: self.page_id,
            issue_type: self.issue_type,
            severity: self.severity,
            message: self.message,
            details: self.details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct NewIssue {
    pub job_id: String,
    pub page_id: Option<String>,
    pub issue_type: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub details: Option<String>,
}

#[cfg(test)]
mod tests {
    //! Characterization tests for `IssueSeverity` and `IssueBuilder`.
    //! `IssueSeverity::from_str` is depended on by the SQLite issue
    //! decoder (`map_severity`); breaking the round-trip would surface
    //! at runtime as an unknown severity. The serde wire format is
    //! also frontend-visible.

    use super::*;
    use std::str::FromStr;

    // ── IssueSeverity ────────────────────────────────────────────────────

    #[test]
    fn severity_round_trips_through_str() {
        for s in [IssueSeverity::Critical, IssueSeverity::Warning, IssueSeverity::Info] {
            assert_eq!(IssueSeverity::from_str(s.as_str()).unwrap(), s);
        }
    }

    #[test]
    fn severity_from_str_accepts_suggestion_alias_for_info() {
        // Some legacy data uses "suggestion" — pinning the alias so the
        // decoder doesn't reject those rows.
        assert_eq!(IssueSeverity::from_str("suggestion").unwrap(), IssueSeverity::Info);
    }

    #[test]
    fn severity_from_str_rejects_unknown() {
        assert!(IssueSeverity::from_str("nonsense").is_err());
        assert!(IssueSeverity::from_str("CRITICAL").is_err()); // case sensitive
    }

    #[test]
    fn severity_serde_uses_lowercase() {
        let json = serde_json::to_string(&IssueSeverity::Critical).unwrap();
        assert_eq!(json, "\"critical\"");
        let parsed: IssueSeverity = serde_json::from_str("\"warning\"").unwrap();
        assert_eq!(parsed, IssueSeverity::Warning);
    }

    #[test]
    fn severity_is_copy() {
        // Compile-time evidence: the matches! below would fail to
        // compile if IssueSeverity were not Copy.
        let s = IssueSeverity::Info;
        let _a = s;
        let _b = s;
        assert!(matches!(s, IssueSeverity::Info));
    }

    // ── IssueBuilder ─────────────────────────────────────────────────────

    fn make_builder() -> IssueBuilder {
        IssueBuilder::new(
            "job-1".to_string(),
            "missing_meta".to_string(),
            IssueSeverity::Warning,
            "no meta description".to_string(),
        )
    }

    #[test]
    fn builder_new_initializes_required_fields() {
        let issue = make_builder().build();
        assert_eq!(issue.job_id, "job-1");
        assert_eq!(issue.issue_type, "missing_meta");
        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert_eq!(issue.message, "no meta description");
        assert!(issue.page_id.is_none());
        assert!(issue.details.is_none());
    }

    #[test]
    fn builder_page_id_attaches_optional_field() {
        let issue = make_builder().page_id("page-7".to_string()).build();
        assert_eq!(issue.page_id.as_deref(), Some("page-7"));
    }

    #[test]
    fn builder_details_attaches_optional_field() {
        let issue = make_builder()
            .details("ran for 1500ms".to_string())
            .build();
        assert_eq!(issue.details.as_deref(), Some("ran for 1500ms"));
    }

    #[test]
    fn builder_chains_page_id_and_details() {
        let issue = make_builder()
            .page_id("page-9".to_string())
            .details("found 42 broken links".to_string())
            .build();
        assert_eq!(issue.page_id.as_deref(), Some("page-9"));
        assert_eq!(issue.details.as_deref(), Some("found 42 broken links"));
    }

    #[test]
    fn new_issue_round_trips_through_serde() {
        let issue = NewIssue {
            job_id: "j".into(),
            page_id: Some("p".into()),
            issue_type: "broken_link".into(),
            severity: IssueSeverity::Critical,
            message: "404".into(),
            details: None,
        };
        let json = serde_json::to_value(&issue).unwrap();
        assert_eq!(json["severity"], "critical");
        assert!(json["details"].is_null());
        let parsed: NewIssue = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.job_id, "j");
        assert_eq!(parsed.severity, IssueSeverity::Critical);
    }
}
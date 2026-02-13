use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

/// Severity level for SEO issues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
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

impl std::str::FromStr for IssueSeverity {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Self::Critical),
            "warning" => Ok(Self::Warning),
            "info" | "suggestion" => Ok(Self::Info),
            _ => Err(()),
        }
    }
}

/// An SEO issue found during analysis.
/// Maps to the `issues` table with direct `job_id` FK.
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

/// Builder for creating issues.
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

/// New issue to be inserted (without auto-generated fields).
#[derive(Debug, Clone)]
pub struct NewIssue {
    pub job_id: String,
    pub page_id: Option<String>,
    pub issue_type: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub details: Option<String>,
}

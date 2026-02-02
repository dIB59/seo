//! Repository layer for the redesigned schema (v2)
//!
//! This module provides data access for the new normalized schema.
//! All queries use direct `job_id` foreign keys eliminating expensive JOINs.

mod job_repository_v2;
mod page_repository_v2;
mod issue_repository_v2;
mod link_repository_v2;
mod results_repository_v2;

pub use job_repository_v2::JobRepositoryV2;
pub use page_repository_v2::PageRepositoryV2;
pub use issue_repository_v2::{IssueRepositoryV2, IssueCounts, IssueGroup};
pub use link_repository_v2::{LinkRepositoryV2, LinkCounts, ExternalDomain};
pub use results_repository_v2::ResultsRepositoryV2;

use crate::domain::models_v2::{IssueSeverity, JobStatus, LinkType};

/// Map database string to JobStatus.
pub fn map_job_status_v2(s: &str) -> JobStatus {
    s.parse().unwrap_or(JobStatus::Pending)
}

/// Map database string to IssueSeverity.
pub fn map_severity(s: &str) -> IssueSeverity {
    s.parse().unwrap_or(IssueSeverity::Info)
}

/// Map database string to LinkType.
pub fn map_link_type(s: &str) -> LinkType {
    s.parse().unwrap_or(LinkType::Internal)
}

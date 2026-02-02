//! Repository layer for the redesigned schema.
//!
//! This module provides data access for the normalized schema.
//! All queries use direct `job_id` foreign keys eliminating expensive JOINs.

mod job_repository;
mod page_repository;
mod issue_repository;
mod link_repository;
mod results_repository;

pub use job_repository::JobRepository;
pub use page_repository::PageRepository;
pub use issue_repository::{IssueRepository, IssueCounts, IssueGroup};
pub use link_repository::{LinkRepository, LinkCounts, ExternalDomain};
pub use results_repository::ResultsRepository;

use crate::domain::models::{IssueSeverity, JobStatus, LinkType};

/// Map database string to JobStatus.
pub fn map_job_status(s: &str) -> JobStatus {
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

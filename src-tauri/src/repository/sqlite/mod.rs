//! Repository layer for the redesigned schema.
//!
//! This module provides data access for the normalized schema.
//! All queries use direct `job_id` foreign keys eliminating expensive JOINs.

mod ai_repository;
mod issue_repository;
mod job_repository;
mod link_repository;
mod page_repository;
mod results_repository;
mod settings_repository;

pub use ai_repository::AiRepository;
pub use issue_repository::{IssueCounts, IssueGroup, IssueRepository};
pub use job_repository::JobRepository;
pub use link_repository::{ExternalDomain, LinkCounts, LinkRepository};
pub use page_repository::PageRepository;
pub use results_repository::ResultsRepository;
pub use settings_repository::SettingsRepository;

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

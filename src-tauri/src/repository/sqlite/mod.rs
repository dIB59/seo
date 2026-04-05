mod ai_repository;
mod extension_repository;
mod issue_repository;
mod job_repository;
mod link_repository;
mod page_queue_repository;
mod page_repository;
mod results_repository;
mod settings_repository;

pub use ai_repository::AiRepository;
pub use extension_repository::SqliteExtensionRepository;
pub use issue_repository::{IssueCounts, IssueGroup, IssueRepository};
pub use job_repository::JobRepository;
pub use link_repository::{ExternalDomain, LinkCounts, LinkRepository};
pub use page_queue_repository::PageQueueRepository;
pub use page_repository::PageRepository;
pub use results_repository::ResultsRepository;
pub use settings_repository::SettingsRepository;

use crate::contexts::{IssueSeverity, JobStatus, LinkType};

pub fn map_job_status(s: &str) -> JobStatus {
    s.parse().unwrap_or(JobStatus::Pending)
}

pub fn map_severity(s: &str) -> IssueSeverity {
    s.parse().unwrap_or(IssueSeverity::Info)
}

pub fn map_link_type(s: &str) -> LinkType {
    s.parse().unwrap_or(LinkType::Internal)
}

// Analysis Context Domain Models
// These are the core domain types for the Analysis bounded context.

mod depth;
mod ids;
mod issue;
mod job;
mod job_state;
mod link;
mod lighthouse;
mod page;
mod pagination;
mod progress;
mod resource;
mod retry_count;
mod url_utils;

// ============================================================================
// Identifiers
// ============================================================================

pub use depth::{Depth, DepthError, MAX_DEPTH};
pub use ids::{IdError, IssueId, JobId, LinkId, PageId, ResourceId};
pub use pagination::{JobPageQuery, Pagination, PaginationError, MAX_LIMIT};
pub use retry_count::{RetryCount, RetryCountError, MAX_RETRY_COUNT};

// ============================================================================
// Job Types
// ============================================================================

pub use job::{Job, JobSettings, JobStatus, JobInfo, JobSummary, CompleteJobResult};
pub use job_state::{
    AnyJob, Cancelled, Completed, Discovery, Failed, JobState, Pending, Processing,
};

/// Filter for listing jobs
#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    pub status: Option<JobStatus>,
    pub url_contains: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// Page Types
// ============================================================================

pub use page::{
    NewPageQueueItem, Page, PageInfo, PageQueueItem, PageQueueStatus, ParsePageQueueStatusError,
};

/// Detailed page information for display
#[derive(Debug, Clone, serde::Serialize)]
pub struct PageDetails {
    pub page: Page,
    pub issues: Vec<Issue>,
    pub headings: Vec<Heading>,
    pub images: Vec<Image>,
    pub links: Vec<Link>,
}

// ============================================================================
// Issue Types
// ============================================================================

pub use issue::{Issue, NewIssue, IssueBuilder, IssueSeverity};

// ============================================================================
// Link Types
// ============================================================================

pub use link::{Link, NewLink, LinkType};

// ============================================================================
// Lighthouse Types
// ============================================================================

pub use lighthouse::LighthouseData;

// ============================================================================
// Progress Types
// ============================================================================

pub use progress::AnalysisProgress;

// ============================================================================
// Resource Types (Heading, Image)
// ============================================================================

pub use resource::{ResourceStatus, Heading, Image, NewHeading, NewImage};

// ============================================================================
// URL Utilities
// ============================================================================

pub use url_utils::{extract_root_domain, extract_host, same_root_domain};

// ============================================================================
// Analysis Result
// ============================================================================

/// Complete analysis result
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalysisResult {
    pub job: Job,
    pub pages: Vec<Page>,
    pub issues: Vec<Issue>,
    pub links: Vec<Link>,
    pub lighthouse: Vec<LighthouseData>,
}

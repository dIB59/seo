// Analysis Bounded Context
// Handles SEO analysis jobs, page crawling, and issue detection

mod domain;
mod services;
mod factory;

#[cfg(test)]
mod tests;

// Public API - what external modules can use
pub use services::AnalysisService;
pub use factory::AnalysisServiceFactory;

// Domain types exposed to external contexts
pub use domain::{Depth, DepthError, MAX_DEPTH};
pub use domain::{IdError, IssueId, JobId, LinkId, PageId, ResourceId};
pub use domain::{JobPageQuery, Pagination, PaginationError, MAX_LIMIT};
pub use domain::{RetryCount, RetryCountError, MAX_RETRY_COUNT};
pub use domain::{
    AnyJob, Cancelled, Completed, Discovery, Failed, Job, JobFilter, JobInfo, JobSettings,
    JobState, JobStatus, JobSummary, Pending, Processing, CompleteJobResult,
};
pub use domain::{
    NewPageQueueItem, Page, PageDetails, PageInfo, PageQueueItem, PageQueueStatus,
    ParsePageQueueStatusError,
};
pub use domain::{Issue, NewIssue, IssueBuilder, IssueSeverity};
pub use domain::{Link, NewLink, LinkType};
pub use domain::LighthouseData;
pub use domain::{Heading, Image, NewHeading, NewImage, ResourceStatus};
pub use domain::{AnalysisProgress, AnalysisResult};
pub use domain::{extract_root_domain, extract_host, same_root_domain};

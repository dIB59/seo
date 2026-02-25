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
pub use domain::{
    Job, JobId, JobSettings, JobStatus, JobInfo, JobFilter, JobSummary, CompleteJobResult,
};
pub use domain::{Page, PageInfo, PageDetails, PageQueueItem, PageQueueStatus, NewPageQueueItem};
pub use domain::{Issue, NewIssue, IssueBuilder, IssueSeverity};
pub use domain::{Link, NewLink, LinkType};
pub use domain::LighthouseData;
pub use domain::{Heading, Image, NewHeading, NewImage, ResourceStatus};
pub use domain::{AnalysisProgress, AnalysisResult};
pub use domain::{extract_root_domain, extract_host, same_root_domain};

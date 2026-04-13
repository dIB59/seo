use crate::contexts::{
    ai::AiInsight,
    analysis::{
        CompleteJobResult, Heading, Image, Issue, IssueSeverity, Job, JobInfo, JobPageQuery,
        JobSettings, JobStatus, LighthouseData, Link, NewHeading, NewImage, NewIssue, NewLink,
        NewPageQueueItem, Page, PageInfo, PageQueueItem, PageQueueStatus,
    },
    extension::{CustomCheck, CustomCheckParams, CustomExtractor, CustomExtractorParams},
    report::{ReportPattern, ReportPatternParams, ReportTemplate},
};
use async_trait::async_trait;
use std::sync::Arc;
pub mod error;
pub mod sqlite;

pub use error::{RepositoryError, RepositoryResult};

// Factory functions for SQLite repositories
pub fn sqlite_job_repo(pool: sqlx::SqlitePool) -> Arc<dyn JobRepository> {
    Arc::new(sqlite::JobRepository::new(pool))
}

pub fn sqlite_page_repo(pool: sqlx::SqlitePool) -> Arc<dyn PageRepository> {
    Arc::new(sqlite::PageRepository::new(pool))
}

pub fn sqlite_link_repo(pool: sqlx::SqlitePool) -> Arc<dyn LinkRepository> {
    Arc::new(sqlite::LinkRepository::new(pool))
}

pub fn sqlite_issue_repo(pool: sqlx::SqlitePool) -> Arc<dyn IssueRepository> {
    Arc::new(sqlite::IssueRepository::new(pool))
}

pub fn sqlite_results_repo(pool: sqlx::SqlitePool) -> Arc<dyn ResultsRepository> {
    Arc::new(sqlite::ResultsRepository::new(pool))
}

pub fn sqlite_settings_repo(pool: sqlx::SqlitePool) -> Arc<dyn SettingsRepository> {
    Arc::new(sqlite::SettingsRepository::new(pool))
}

pub fn sqlite_ai_repo(pool: sqlx::SqlitePool) -> Arc<dyn AiRepository> {
    Arc::new(sqlite::AiRepository::new(pool))
}

pub fn sqlite_page_queue_repo(pool: sqlx::SqlitePool) -> Arc<dyn PageQueueRepository> {
    Arc::new(sqlite::PageQueueRepository::new(pool))
}

pub fn sqlite_extension_repo(pool: sqlx::SqlitePool) -> Arc<dyn ExtensionRepository> {
    Arc::new(sqlite::SqliteExtensionRepository::new(pool))
}

pub fn sqlite_report_pattern_repo(pool: sqlx::SqlitePool) -> Arc<dyn ReportPatternRepository> {
    Arc::new(sqlite::SqliteReportPatternRepository::new(pool))
}

pub fn sqlite_report_template_repo(pool: sqlx::SqlitePool) -> Arc<dyn ReportTemplateRepository> {
    Arc::new(sqlite::ReportTemplateRepository::new(pool))
}

pub use sqlite::{ExternalDomain, IssueCounts, IssueGroup, LinkCounts};

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create(&self, url: &str, settings: &JobSettings) -> RepositoryResult<String>;
    async fn get_by_id(&self, id: &str) -> RepositoryResult<Job>;
    async fn get_all(&self) -> RepositoryResult<Vec<JobInfo>>;
    async fn get_paginated(&self, limit: i64, offset: i64) -> RepositoryResult<Vec<JobInfo>>;
    async fn get_paginated_with_total(
        &self,
        query: JobPageQuery,
    ) -> RepositoryResult<(Vec<JobInfo>, i64)>;
    async fn get_pending(&self) -> RepositoryResult<Vec<Job>>;
    async fn get_running_jobs_id(&self) -> RepositoryResult<Vec<String>>;
    async fn update_status(&self, job_id: &str, status: JobStatus) -> RepositoryResult<()>;
    async fn update_progress(&self, id: &str, progress: f64) -> RepositoryResult<()>;
    async fn update_resources(
        &self,
        id: &str,
        sitemap_found: bool,
        robots_txt_found: bool,
    ) -> RepositoryResult<()>;
    async fn set_error(&self, job_id: &str, error: &str) -> RepositoryResult<()>;
    /// Total number of jobs in the database. Migrated to the per-layer
    /// `RepositoryResult` so callers can match on `RepositoryError::Database`
    /// for retry logic instead of stringly-typed `anyhow` matching.
    async fn count(&self) -> RepositoryResult<i64>;
    async fn delete(&self, job_id: &str) -> RepositoryResult<()>;
}

#[async_trait]
pub trait PageRepository: Send + Sync {
    async fn insert(&self, page: &Page) -> RepositoryResult<String>;
    async fn insert_batch(&self, pages: &[Page]) -> RepositoryResult<()>;
    async fn get_by_job_id(&self, job_id: &str) -> RepositoryResult<Vec<Page>>;
    async fn get_info_by_job_id(&self, job_id: &str) -> RepositoryResult<Vec<PageInfo>>;
    async fn get_by_id(&self, page_id: &str) -> RepositoryResult<Page>;
    async fn replace_headings(
        &self,
        page_id: &str,
        headings: &[NewHeading],
    ) -> RepositoryResult<()>;
    async fn replace_images(&self, page_id: &str, images: &[NewImage]) -> RepositoryResult<()>;
    async fn count_by_job_id(&self, job_id: &str) -> RepositoryResult<i64>;
    async fn insert_lighthouse(&self, data: &LighthouseData) -> RepositoryResult<()>;
    async fn get_lighthouse_by_job_id(
        &self,
        job_id: &str,
    ) -> RepositoryResult<Vec<LighthouseData>>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get_setting(&self, key: &str) -> RepositoryResult<Option<String>>;
    async fn set_setting(&self, key: &str, value: &str) -> RepositoryResult<()>;
}

#[async_trait]
pub trait LinkRepository: Send + Sync {
    async fn insert_batch(&self, links: &[NewLink]) -> RepositoryResult<()>;
    async fn get_by_job_id(&self, job_id: &str) -> RepositoryResult<Vec<Link>>;
    async fn get_outgoing(&self, source_page_id: &str) -> RepositoryResult<Vec<Link>>;
    async fn get_incoming(&self, target_page_id: &str) -> RepositoryResult<Vec<Link>>;
    async fn get_broken(&self, job_id: &str) -> RepositoryResult<Vec<Link>>;
    async fn count_by_type(&self, job_id: &str) -> RepositoryResult<LinkCounts>;
    async fn get_external_domains(&self, job_id: &str) -> RepositoryResult<Vec<ExternalDomain>>;
    async fn update_status_codes(&self, updates: &[(i64, i64)]) -> RepositoryResult<()>;
}

#[async_trait]
pub trait IssueRepository: Send + Sync {
    async fn insert_batch(&self, issues: &[NewIssue]) -> RepositoryResult<()>;
    async fn get_by_job_id(&self, job_id: &str) -> RepositoryResult<Vec<Issue>>;
    async fn get_by_page_id(&self, page_id: &str) -> RepositoryResult<Vec<Issue>>;
    async fn get_by_job_and_severity(
        &self,
        job_id: &str,
        severity: IssueSeverity,
    ) -> RepositoryResult<Vec<Issue>>;
    async fn count_by_severity(&self, job_id: &str) -> RepositoryResult<IssueCounts>;
    async fn count_by_job_id(&self, job_id: &str) -> RepositoryResult<i64>;
    async fn get_grouped_by_type(&self, job_id: &str) -> RepositoryResult<Vec<IssueGroup>>;
}

#[async_trait]
pub trait ResultsRepository: Send + Sync {
    async fn get_complete_result(&self, job_id: &str) -> RepositoryResult<CompleteJobResult>;
    async fn get_job(&self, job_id: &str) -> RepositoryResult<Job>;
    async fn get_pages(&self, job_id: &str) -> RepositoryResult<Vec<Page>>;
    async fn get_issues(&self, job_id: &str) -> RepositoryResult<Vec<Issue>>;
    async fn get_links(&self, job_id: &str) -> RepositoryResult<Vec<Link>>;
    async fn get_lighthouse(&self, job_id: &str) -> RepositoryResult<Vec<LighthouseData>>;
    async fn get_headings(&self, job_id: &str) -> RepositoryResult<Vec<Heading>>;
    async fn get_images(&self, job_id: &str) -> RepositoryResult<Vec<Image>>;
    async fn get_ai_insights(&self, job_id: &str) -> RepositoryResult<AiInsight>;
    async fn save_ai_insights(
        &self,
        job_id: &str,
        summary: Option<&str>,
        recommendations: Option<&str>,
        raw_response: Option<&str>,
        model: Option<&str>,
    ) -> RepositoryResult<()>;
}

#[async_trait]
pub trait AiRepository: Send + Sync {
    async fn get_ai_insights(&self, job_id: &str) -> RepositoryResult<Option<String>>;
    async fn save_ai_insights(&self, job_id: &str, insights: &str) -> RepositoryResult<()>;
}

/// Repository for managing the page analysis queue.
/// Enables resumability, concurrent page analysis, and individual page status tracking.
#[async_trait]
pub trait PageQueueRepository: Send + Sync {
    /// Insert a single page into the queue.
    async fn insert(&self, item: &NewPageQueueItem) -> RepositoryResult<String>;

    /// Insert multiple pages into the queue in a single transaction.
    async fn insert_batch(&self, items: &[NewPageQueueItem]) -> RepositoryResult<()>;

    /// Claim the next pending page for a specific job (atomic status update).
    /// Returns None if no pending pages are available.
    async fn claim_next_pending(
        &self,
        job_id: &str,
    ) -> RepositoryResult<Option<PageQueueItem>>;

    /// Claim the next pending page across all jobs (atomic status update).
    /// Returns None if no pending pages are available.
    async fn claim_any_pending(&self) -> RepositoryResult<Option<PageQueueItem>>;

    /// Update the status of a page queue item.
    async fn update_status(&self, id: &str, status: PageQueueStatus) -> RepositoryResult<()>;

    /// Mark a page as failed with an error message.
    async fn mark_failed(&self, id: &str, error: &str) -> RepositoryResult<()>;

    /// Get all queue items for a job.
    async fn get_by_job_id(&self, job_id: &str) -> RepositoryResult<Vec<PageQueueItem>>;

    /// Get queue items by status for a job.
    async fn get_by_job_and_status(
        &self,
        job_id: &str,
        status: PageQueueStatus,
    ) -> RepositoryResult<Vec<PageQueueItem>>;

    /// Count pending pages for a job.
    async fn count_pending(&self, job_id: &str) -> RepositoryResult<i64>;

    /// Count completed pages for a job.
    async fn count_completed(&self, job_id: &str) -> RepositoryResult<i64>;

    /// Count total pages for a job.
    async fn count_total(&self, job_id: &str) -> RepositoryResult<i64>;

    /// Delete all queue items for a job.
    async fn delete_by_job_id(&self, job_id: &str) -> RepositoryResult<()>;

    /// Reset processing pages back to pending (for recovery after crash).
    async fn reset_processing_to_pending(&self, job_id: &str) -> RepositoryResult<i64>;

    /// Check if all pages for a job are complete (no pending or processing).
    async fn is_job_complete(&self, job_id: &str) -> RepositoryResult<bool>;
}

#[async_trait]
pub trait ExtensionRepository: Send + Sync {
    async fn create_check(&self, params: &CustomCheckParams) -> RepositoryResult<CustomCheck>;
    async fn list_checks(&self) -> RepositoryResult<Vec<CustomCheck>>;
    async fn get_check(&self, id: &str) -> RepositoryResult<CustomCheck>;
    async fn update_check(
        &self,
        id: &str,
        params: &CustomCheckParams,
    ) -> RepositoryResult<CustomCheck>;
    async fn delete_check(&self, id: &str) -> RepositoryResult<()>;
    async fn list_enabled_checks(&self) -> RepositoryResult<Vec<CustomCheck>>;

    async fn create_extractor(
        &self,
        params: &CustomExtractorParams,
    ) -> RepositoryResult<CustomExtractor>;
    async fn list_extractors(&self) -> RepositoryResult<Vec<CustomExtractor>>;
    async fn get_extractor(&self, id: &str) -> RepositoryResult<CustomExtractor>;
    async fn update_extractor(
        &self,
        id: &str,
        params: &CustomExtractorParams,
    ) -> RepositoryResult<CustomExtractor>;
    async fn delete_extractor(&self, id: &str) -> RepositoryResult<()>;
    async fn list_enabled_extractors(&self) -> RepositoryResult<Vec<CustomExtractor>>;
}

#[async_trait]
pub trait ReportPatternRepository: Send + Sync {
    async fn list_patterns(&self) -> RepositoryResult<Vec<ReportPattern>>;
    async fn list_enabled_patterns(&self) -> RepositoryResult<Vec<ReportPattern>>;
    async fn get_pattern(&self, id: &str) -> RepositoryResult<ReportPattern>;
    async fn create_pattern(&self, params: &ReportPatternParams) -> RepositoryResult<ReportPattern>;
    async fn update_pattern(
        &self,
        id: &str,
        params: &ReportPatternParams,
    ) -> RepositoryResult<ReportPattern>;
    async fn toggle_pattern(&self, id: &str, enabled: bool) -> RepositoryResult<()>;
    async fn delete_pattern(&self, id: &str) -> RepositoryResult<()>;
}

#[async_trait]
pub trait ReportTemplateRepository: Send + Sync {
    async fn list_templates(&self) -> RepositoryResult<Vec<ReportTemplate>>;
    async fn get_template(&self, id: &str) -> RepositoryResult<ReportTemplate>;
    async fn get_active_template(&self) -> RepositoryResult<Option<ReportTemplate>>;
    async fn create_template(&self, template: &ReportTemplate) -> RepositoryResult<()>;
    async fn update_template(&self, template: &ReportTemplate) -> RepositoryResult<()>;
    async fn set_active_template(&self, id: &str) -> RepositoryResult<()>;
    async fn delete_template(&self, id: &str) -> RepositoryResult<()>;
}

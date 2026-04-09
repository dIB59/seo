use crate::contexts::{
    ai::AiInsight,
    analysis::{
        CompleteJobResult, Heading, Image, Issue, IssueSeverity, Job, JobInfo, JobSettings,
        JobStatus, LighthouseData, Link, NewHeading, NewImage, NewIssue, NewLink,
        NewPageQueueItem, Page, PageInfo, PageQueueItem, PageQueueStatus,
    },
    extension::{CustomCheck, CustomCheckParams, CustomExtractor, CustomExtractorParams},
    report::{ReportPattern, ReportPatternParams},
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
pub mod sqlite;

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

pub use sqlite::{ExternalDomain, IssueCounts, IssueGroup, LinkCounts};

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create(&self, url: &str, settings: &JobSettings) -> Result<String>;
    async fn get_by_id(&self, id: &str) -> Result<Job>;
    async fn get_all(&self) -> Result<Vec<JobInfo>>;
    async fn get_paginated(&self, limit: i64, offset: i64) -> Result<Vec<JobInfo>>;
    async fn get_paginated_with_total(
        &self,
        limit: i64,
        offset: i64,
        url_filter: Option<String>,
        status_filter: Option<String>,
    ) -> Result<(Vec<JobInfo>, i64)>;
    async fn get_pending(&self) -> Result<Vec<Job>>;
    async fn get_running_jobs_id(&self) -> Result<Vec<String>>;
    async fn update_status(&self, job_id: &str, status: JobStatus) -> Result<()>;
    async fn update_progress(&self, id: &str, progress: f64) -> Result<()>;
    async fn update_resources(
        &self,
        id: &str,
        sitemap_found: bool,
        robots_txt_found: bool,
    ) -> Result<()>;
    async fn set_error(&self, job_id: &str, error: &str) -> Result<()>;
    async fn count(&self) -> Result<i64>;
    async fn delete(&self, job_id: &str) -> Result<()>;
}

#[async_trait]
pub trait PageRepository: Send + Sync {
    async fn insert(&self, page: &Page) -> Result<String>;
    async fn insert_batch(&self, pages: &[Page]) -> Result<()>;
    async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<Page>>;
    async fn get_info_by_job_id(&self, job_id: &str) -> Result<Vec<PageInfo>>;
    async fn get_by_id(&self, page_id: &str) -> Result<Page>;
    async fn replace_headings(
        &self,
        page_id: &str,
        headings: &[NewHeading],
    ) -> Result<()>;
    async fn replace_images(&self, page_id: &str, images: &[NewImage]) -> Result<()>;
    async fn count_by_job_id(&self, job_id: &str) -> Result<i64>;
    async fn insert_lighthouse(&self, data: &LighthouseData) -> Result<()>;
    async fn get_lighthouse_by_job_id(&self, job_id: &str) -> Result<Vec<LighthouseData>>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get_setting(&self, key: &str) -> Result<Option<String>>;
    async fn set_setting(&self, key: &str, value: &str) -> Result<()>;
}

#[async_trait]
pub trait LinkRepository: Send + Sync {
    async fn insert_batch(&self, links: &[NewLink]) -> Result<()>;
    async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<Link>>;
    async fn get_outgoing(&self, source_page_id: &str) -> Result<Vec<Link>>;
    async fn get_incoming(&self, target_page_id: &str) -> Result<Vec<Link>>;
    async fn get_broken(&self, job_id: &str) -> Result<Vec<Link>>;
    async fn count_by_type(&self, job_id: &str) -> Result<LinkCounts>;
    async fn get_external_domains(&self, job_id: &str) -> Result<Vec<ExternalDomain>>;
    async fn update_status_codes(&self, updates: &[(i64, i64)]) -> Result<()>;
}

#[async_trait]
pub trait IssueRepository: Send + Sync {
    async fn insert_batch(&self, issues: &[NewIssue]) -> Result<()>;
    async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<Issue>>;
    async fn get_by_page_id(&self, page_id: &str) -> Result<Vec<Issue>>;
    async fn get_by_job_and_severity(
        &self,
        job_id: &str,
        severity: IssueSeverity,
    ) -> Result<Vec<Issue>>;
    async fn count_by_severity(&self, job_id: &str) -> Result<IssueCounts>;
    async fn count_by_job_id(&self, job_id: &str) -> Result<i64>;
    async fn get_grouped_by_type(&self, job_id: &str) -> Result<Vec<IssueGroup>>;
}

#[async_trait]
pub trait ResultsRepository: Send + Sync {
    async fn get_complete_result(&self, job_id: &str) -> Result<CompleteJobResult>;
    async fn get_job(&self, job_id: &str) -> Result<Job>;
    async fn get_pages(&self, job_id: &str) -> Result<Vec<Page>>;
    async fn get_issues(&self, job_id: &str) -> Result<Vec<Issue>>;
    async fn get_links(&self, job_id: &str) -> Result<Vec<Link>>;
    async fn get_lighthouse(&self, job_id: &str) -> Result<Vec<LighthouseData>>;
    async fn get_headings(&self, job_id: &str) -> Result<Vec<Heading>>;
    async fn get_images(&self, job_id: &str) -> Result<Vec<Image>>;
    async fn get_ai_insights(&self, job_id: &str) -> Result<AiInsight>;
    async fn save_ai_insights(
        &self,
        job_id: &str,
        summary: Option<&str>,
        recommendations: Option<&str>,
        raw_response: Option<&str>,
        model: Option<&str>,
    ) -> Result<()>;
}

#[async_trait]
pub trait AiRepository: Send + Sync {
    async fn get_ai_insights(&self, job_id: &str) -> Result<Option<String>>;
    async fn save_ai_insights(&self, job_id: &str, insights: &str) -> Result<()>;
}

/// Repository for managing the page analysis queue.
/// Enables resumability, concurrent page analysis, and individual page status tracking.
#[async_trait]
pub trait PageQueueRepository: Send + Sync {
    /// Insert a single page into the queue.
    async fn insert(&self, item: &NewPageQueueItem) -> Result<String>;

    /// Insert multiple pages into the queue in a single transaction.
    async fn insert_batch(&self, items: &[NewPageQueueItem]) -> Result<()>;

    /// Claim the next pending page for a specific job (atomic status update).
    /// Returns None if no pending pages are available.
    async fn claim_next_pending(&self, job_id: &str) -> Result<Option<PageQueueItem>>;

    /// Claim the next pending page across all jobs (atomic status update).
    /// Returns None if no pending pages are available.
    async fn claim_any_pending(&self) -> Result<Option<PageQueueItem>>;

    /// Update the status of a page queue item.
    async fn update_status(&self, id: &str, status: PageQueueStatus) -> Result<()>;

    /// Mark a page as failed with an error message.
    async fn mark_failed(&self, id: &str, error: &str) -> Result<()>;

    /// Get all queue items for a job.
    async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<PageQueueItem>>;

    /// Get queue items by status for a job.
    async fn get_by_job_and_status(
        &self,
        job_id: &str,
        status: PageQueueStatus,
    ) -> Result<Vec<PageQueueItem>>;

    /// Count pending pages for a job.
    async fn count_pending(&self, job_id: &str) -> Result<i64>;

    /// Count completed pages for a job.
    async fn count_completed(&self, job_id: &str) -> Result<i64>;

    /// Count total pages for a job.
    async fn count_total(&self, job_id: &str) -> Result<i64>;

    /// Delete all queue items for a job.
    async fn delete_by_job_id(&self, job_id: &str) -> Result<()>;

    /// Reset processing pages back to pending (for recovery after crash).
    async fn reset_processing_to_pending(&self, job_id: &str) -> Result<i64>;

    /// Check if all pages for a job are complete (no pending or processing).
    async fn is_job_complete(&self, job_id: &str) -> Result<bool>;
}

#[async_trait]
pub trait ExtensionRepository: Send + Sync {
    async fn create_check(&self, params: &CustomCheckParams) -> Result<CustomCheck>;
    async fn list_checks(&self) -> Result<Vec<CustomCheck>>;
    async fn get_check(&self, id: &str) -> Result<CustomCheck>;
    async fn update_check(&self, id: &str, params: &CustomCheckParams) -> Result<CustomCheck>;
    async fn delete_check(&self, id: &str) -> Result<()>;
    async fn list_enabled_checks(&self) -> Result<Vec<CustomCheck>>;

    async fn create_extractor(&self, params: &CustomExtractorParams) -> Result<CustomExtractor>;
    async fn list_extractors(&self) -> Result<Vec<CustomExtractor>>;
    async fn get_extractor(&self, id: &str) -> Result<CustomExtractor>;
    async fn update_extractor(
        &self,
        id: &str,
        params: &CustomExtractorParams,
    ) -> Result<CustomExtractor>;
    async fn delete_extractor(&self, id: &str) -> Result<()>;
    async fn list_enabled_extractors(&self) -> Result<Vec<CustomExtractor>>;
}

#[async_trait]
pub trait ReportPatternRepository: Send + Sync {
    async fn list_patterns(&self) -> Result<Vec<ReportPattern>>;
    async fn list_enabled_patterns(&self) -> Result<Vec<ReportPattern>>;
    async fn get_pattern(&self, id: &str) -> Result<ReportPattern>;
    async fn create_pattern(&self, params: &ReportPatternParams) -> Result<ReportPattern>;
    async fn update_pattern(&self, id: &str, params: &ReportPatternParams) -> Result<ReportPattern>;
    async fn toggle_pattern(&self, id: &str, enabled: bool) -> Result<()>;
    async fn delete_pattern(&self, id: &str) -> Result<()>;
}

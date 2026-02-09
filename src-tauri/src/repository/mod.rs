use crate::domain::models::{Job, JobInfo, JobSettings, JobStatus};
use anyhow::Result;
use async_trait::async_trait;

pub mod sqlite;

/// Repository trait for Job operations. Use this trait to inject mock or alternate
/// repository implementations for testing or swapping persistence layers.
#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create(&self, url: &str, settings: &JobSettings) -> Result<String>;
    async fn get_by_id(&self, id: &str) -> Result<Job>;
    async fn get_all(&self) -> Result<Vec<JobInfo>>;
    async fn get_pending(&self) -> Result<Vec<Job>>;
    async fn update_status(&self, job_id: &str, status: JobStatus) -> Result<()>;
    async fn update_progress(
        &self,
        id: &str,
        progress: f64,
        current_stage: Option<&str>,
    ) -> Result<()>;
    async fn set_error(&self, job_id: &str, error: &str) -> Result<()>;
    async fn delete(&self, job_id: &str) -> Result<()>;
}

/// Page repository trait - abstract page persistence/read operations.
#[async_trait]
pub trait PageRepository: Send + Sync {
    async fn insert(&self, page: &crate::domain::models::Page) -> Result<String>;
    async fn insert_batch(&self, pages: &[crate::domain::models::Page]) -> Result<()>;
    async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<crate::domain::models::Page>>;
    async fn get_info_by_job_id(
        &self,
        job_id: &str,
    ) -> Result<Vec<crate::domain::models::PageInfo>>;
    async fn get_by_id(&self, page_id: &str) -> Result<crate::domain::models::Page>;
    async fn replace_headings(
        &self,
        page_id: &str,
        headings: &[crate::domain::models::NewHeading],
    ) -> Result<()>;
    async fn replace_images(
        &self,
        page_id: &str,
        images: &[crate::domain::models::NewImage],
    ) -> Result<()>;
    async fn count_by_job_id(&self, job_id: &str) -> Result<i64>;
    async fn insert_lighthouse(&self, data: &crate::domain::models::LighthouseData) -> Result<()>;
    async fn get_lighthouse_by_job_id(
        &self,
        job_id: &str,
    ) -> Result<Vec<crate::domain::models::LighthouseData>>;
}

/// Settings repository trait - key/value configuration store.
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get_setting(&self, key: &str) -> Result<Option<String>>;
    async fn set_setting(&self, key: &str, value: &str) -> Result<()>;
}

/// Link repository trait - abstract link persistence/queries.
#[async_trait]
pub trait LinkRepository: Send + Sync {
    async fn insert_batch(&self, links: &[crate::domain::models::NewLink]) -> Result<()>;
    async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<crate::domain::models::Link>>;
    async fn get_outgoing(&self, source_page_id: &str) -> Result<Vec<crate::domain::models::Link>>;
    async fn get_incoming(&self, target_page_id: &str) -> Result<Vec<crate::domain::models::Link>>;
    async fn get_broken(&self, job_id: &str) -> Result<Vec<crate::domain::models::Link>>;
    async fn count_by_type(&self, job_id: &str) -> Result<crate::repository::sqlite::LinkCounts>;
    async fn get_external_domains(
        &self,
        job_id: &str,
    ) -> Result<Vec<crate::repository::sqlite::ExternalDomain>>;
    async fn update_status_codes(&self, updates: &[(i64, i64)]) -> Result<()>;
}

/// Issue repository trait - abstract issue persistence and queries.
#[async_trait]
pub trait IssueRepository: Send + Sync {
    async fn insert_batch(&self, issues: &[crate::domain::models::NewIssue]) -> Result<()>;
    async fn get_by_job_id(&self, job_id: &str) -> Result<Vec<crate::domain::models::Issue>>;
    async fn get_by_page_id(&self, page_id: &str) -> Result<Vec<crate::domain::models::Issue>>;
    async fn get_by_job_and_severity(
        &self,
        job_id: &str,
        severity: crate::domain::models::IssueSeverity,
    ) -> Result<Vec<crate::domain::models::Issue>>;
    async fn count_by_severity(
        &self,
        job_id: &str,
    ) -> Result<crate::repository::sqlite::IssueCounts>;
    async fn count_by_job_id(&self, job_id: &str) -> Result<i64>;
    async fn get_grouped_by_type(
        &self,
        job_id: &str,
    ) -> Result<Vec<crate::repository::sqlite::IssueGroup>>;
}

/// Results repository trait - high-level getters for assembled results.
#[async_trait]
pub trait ResultsRepository: Send + Sync {
    async fn get_complete_result(
        &self,
        job_id: &str,
    ) -> Result<crate::domain::models::CompleteJobResult>;
    async fn get_job(&self, job_id: &str) -> Result<crate::domain::models::Job>;
    async fn get_pages(&self, job_id: &str) -> Result<Vec<crate::domain::models::Page>>;
    async fn get_issues(&self, job_id: &str) -> Result<Vec<crate::domain::models::Issue>>;
    async fn get_links(&self, job_id: &str) -> Result<Vec<crate::domain::models::Link>>;
    async fn get_lighthouse(
        &self,
        job_id: &str,
    ) -> Result<Vec<crate::domain::models::LighthouseData>>;
    async fn get_headings(&self, job_id: &str) -> Result<Vec<crate::domain::models::Heading>>;
    async fn get_images(&self, job_id: &str) -> Result<Vec<crate::domain::models::Image>>;
    async fn get_ai_insights(&self, job_id: &str) -> Result<crate::domain::models::AiInsight>;
    async fn save_ai_insights(
        &self,
        job_id: &str,
        summary: Option<&str>,
        recommendations: Option<&str>,
        raw_response: Option<&str>,
        model: Option<&str>,
    ) -> Result<()>;
}

/// AI repository trait - simple caching interface.
#[async_trait]
pub trait AiRepository: Send + Sync {
    async fn get_ai_insights(&self, job_id: &str) -> Result<Option<String>>;
    async fn save_ai_insights(&self, job_id: &str, insights: &str) -> Result<()>;
}

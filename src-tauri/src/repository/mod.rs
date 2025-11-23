use async_trait::async_trait;
use anyhow::Result;
use crate::domain::models::*;

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn get_pending_jobs(&self) -> Result<Vec<AnalysisJob>>;
    async fn update_status(&self, job_id: i64, status: &str, result_id: Option<&str>) -> Result<()>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get_by_id(&self, id: i64) -> Result<AnalysisSettings>;
}

#[async_trait]
pub trait ResultsRepository: Send + Sync {
    async fn create(&self, url: &str, sitemap: bool, robots: bool, ssl: bool) -> Result<String>;
    async fn update_progress(&self, id: &str, progress: f64, analyzed: i32, total: i32) -> Result<()>;
    async fn finalize(&self, id: &str, status: &str) -> Result<()>;
}

#[async_trait]
pub trait PageRepository: Send + Sync {
    async fn insert(&self, page: &PageAnalysisData) -> Result<String>;
}

#[async_trait]
pub trait IssuesRepository: Send + Sync {
    async fn insert_batch(&self, issues: &[SeoIssue]) -> Result<()>;
}

#[async_trait]
pub trait SummaryRepository: Send + Sync {
    async fn update_from_issues(&self, analysis_id: &str, issues: &[SeoIssue], total_pages: i32) -> Result<()>;
}
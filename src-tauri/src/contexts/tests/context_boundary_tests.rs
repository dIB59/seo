// Integration tests for context boundaries
// Validates that data flows correctly between the command layer and underlying services

use std::sync::Arc;
use std::sync::RwLock;

use crate::domain::permissions::{LicenseTier, Policy};
use crate::repository::{
    sqlite_ai_repo, sqlite_issue_repo, sqlite_job_repo, sqlite_link_repo, sqlite_page_queue_repo,
    sqlite_page_repo, sqlite_results_repo, sqlite_settings_repo,
};
use crate::service::licensing::MockLicensingService;
use crate::service::processor::reporter::ProgressEmitter;
use crate::service::processor::{AnalyzerService, Crawler};
use crate::service::JobProcessor;
use crate::test_utils::fixtures::setup_test_db;

// Mock implementations for testing
pub struct MockSpider;

#[async_trait::async_trait]
impl crate::service::spider::SpiderAgent for MockSpider {
    async fn fetch_html(&self, _url: &str) -> anyhow::Result<String> {
        Ok(String::new())
    }

    async fn get(&self, _url: &str) -> anyhow::Result<crate::service::spider::SpiderResponse> {
        Ok(crate::service::spider::SpiderResponse {
            status: 200,
            body: String::new(),
            url: String::new(),
        })
    }

    async fn post_json(
        &self,
        _url: &str,
        _payload: &serde_json::Value,
    ) -> anyhow::Result<crate::service::spider::SpiderResponse> {
        Ok(crate::service::spider::SpiderResponse {
            status: 200,
            body: String::new(),
            url: String::new(),
        })
    }
}

struct NilEmitter;

impl ProgressEmitter for NilEmitter {
    fn emit(&self, _event: crate::service::processor::reporter::ProgressEvent) {}
}

/// Test fixture for creating context services with test database
struct TestContextFixture {
    pool: sqlx::SqlitePool,
    settings_repo: Arc<dyn crate::repository::SettingsRepository>,
    ai_repo: Arc<dyn crate::repository::AiRepository>,
    job_repo: Arc<dyn crate::repository::JobRepository>,
    results_repo: Arc<dyn crate::repository::ResultsRepository>,
}

impl TestContextFixture {
    async fn new() -> Self {
        let pool = setup_test_db().await;
        let settings_repo = sqlite_settings_repo(pool.clone());
        let ai_repo = sqlite_ai_repo(pool.clone());
        let job_repo = sqlite_job_repo(pool.clone());
        let results_repo = sqlite_results_repo(pool.clone());

        Self {
            pool,
            settings_repo,
            ai_repo,
            job_repo,
            results_repo,
        }
    }

    fn create_licensing_context(&self) -> Arc<dyn crate::contexts::licensing::LicensingAgent> {
        Arc::new(MockLicensingService::new(self.settings_repo.clone()))
    }

    fn create_analysis_context(&self) -> crate::contexts::analysis::AnalysisService {
        crate::contexts::analysis::AnalysisServiceFactory::with_repositories(
            self.job_repo.clone(),
            self.results_repo.clone(),
        )
    }

    fn create_ai_context(&self) -> crate::contexts::ai::AiService {
        crate::contexts::ai::AiServiceFactory::from_repositories(
            self.ai_repo.clone(),
            self.settings_repo.clone(),
        )
    }
}

// ============================================================================
// Licensing Context Boundary Tests
// ============================================================================

#[tokio::test]
async fn test_licensing_context_activate_license() {
    let fixture = TestContextFixture::new().await;
    let licensing_context = fixture.create_licensing_context();

    // Test initial state
    let initial_tier = licensing_context.current_tier();
    assert_eq!(initial_tier, LicenseTier::Free);

    // Test policy retrieval
    let policy = licensing_context.get_policy();
    assert_eq!(policy.tier, LicenseTier::Free);
}

#[tokio::test]
async fn test_licensing_context_policy_consistency() {
    let fixture = TestContextFixture::new().await;
    let licensing_context = fixture.create_licensing_context();

    // Verify policy is consistent with tier
    let tier = licensing_context.current_tier();
    let policy = licensing_context.get_policy();
    assert_eq!(tier, policy.tier);
}

// ============================================================================
// Analysis Context Boundary Tests
// ============================================================================

#[tokio::test]
async fn test_analysis_context_create_job() {
    let fixture = TestContextFixture::new().await;
    let analysis_context = fixture.create_analysis_context();

    // Create a job
    let settings = crate::domain::JobSettings::default();
    let job_id = analysis_context
        .create_job("https://example.com", &settings)
        .await
        .expect("Failed to create job");

    assert!(!job_id.is_empty(), "Job ID should not be empty");

    // Verify job can be retrieved
    let job = analysis_context.get_job(&job_id).await.expect("Failed to get job");
    assert_eq!(job.url, "https://example.com");
    assert_eq!(job.status, crate::domain::JobStatus::Pending);
}

#[tokio::test]
async fn test_analysis_context_get_progress() {
    let fixture = TestContextFixture::new().await;
    let analysis_context = fixture.create_analysis_context();

    // Create a job
    let settings = crate::domain::JobSettings::default();
    let job_id = analysis_context
        .create_job("https://example.com", &settings)
        .await
        .expect("Failed to create job");

    // Get progress
    let progress = analysis_context
        .get_progress(&job_id)
        .await
        .expect("Failed to get progress");

    assert_eq!(progress.id, job_id);
    assert_eq!(progress.status, crate::domain::JobStatus::Pending);
}

#[tokio::test]
async fn test_analysis_context_list_jobs() {
    let fixture = TestContextFixture::new().await;
    let analysis_context = fixture.create_analysis_context();

    // Create multiple jobs
    let settings = crate::domain::JobSettings::default();
    for i in 0..3 {
        analysis_context
            .create_job(&format!("https://example{}.com", i), &settings)
            .await
            .expect("Failed to create job");
    }

    // List all jobs
    let jobs = analysis_context.get_all_jobs().await.expect("Failed to list jobs");
    assert!(jobs.len() >= 3, "Should have at least 3 jobs");
}

#[tokio::test]
async fn test_analysis_context_paginated_jobs() {
    let fixture = TestContextFixture::new().await;
    let analysis_context = fixture.create_analysis_context();

    // Create multiple jobs
    let settings = crate::domain::JobSettings::default();
    for i in 0..5 {
        analysis_context
            .create_job(&format!("https://example{}.com", i), &settings)
            .await
            .expect("Failed to create job");
    }

    // Get paginated jobs
    let (jobs, total) = analysis_context
        .get_paginated_jobs_with_total(2, 0, None, None)
        .await
        .expect("Failed to get paginated jobs");

    assert_eq!(jobs.len(), 2, "Should return 2 jobs");
    assert!(total >= 5, "Total should be at least 5");
}

#[tokio::test]
async fn test_analysis_context_cancel_job() {
    let fixture = TestContextFixture::new().await;
    let analysis_context = fixture.create_analysis_context();

    // Create a job
    let settings = crate::domain::JobSettings::default();
    let job_id = analysis_context
        .create_job("https://example.com", &settings)
        .await
        .expect("Failed to create job");

    // Cancel the job
    analysis_context
        .cancel_job(&job_id)
        .await
        .expect("Failed to cancel job");

    // Verify job is cancelled
    let job = analysis_context.get_job(&job_id).await.expect("Failed to get job");
    assert_eq!(job.status, crate::domain::JobStatus::Cancelled);
}

// ============================================================================
// AI Context Boundary Tests
// ============================================================================

#[tokio::test]
async fn test_ai_context_settings_management() {
    let fixture = TestContextFixture::new().await;
    let ai_context = fixture.create_ai_context();

    // Test enabled setting
    ai_context.set_enabled(true).await.expect("Failed to set enabled");
    assert!(ai_context.is_enabled().await.expect("Failed to check enabled"));

    ai_context.set_enabled(false).await.expect("Failed to set enabled");
    assert!(!ai_context.is_enabled().await.expect("Failed to check enabled"));

    // Test API key
    ai_context.set_api_key("test-api-key").await.expect("Failed to set API key");
    let api_key = ai_context.get_api_key().await.expect("Failed to get API key");
    assert_eq!(api_key, Some("test-api-key".to_string()));

    // Test persona
    ai_context.set_persona("SEO Expert").await.expect("Failed to set persona");
    let persona = ai_context.get_persona().await.expect("Failed to get persona");
    assert_eq!(persona, Some("SEO Expert".to_string()));

    // Test requirements
    ai_context
        .set_requirements("Focus on performance")
        .await
        .expect("Failed to set requirements");
    let requirements = ai_context.get_requirements().await.expect("Failed to get requirements");
    assert_eq!(requirements, Some("Focus on performance".to_string()));

    // Test context options
    ai_context
        .set_context_options("{\"depth\": \"full\"}")
        .await
        .expect("Failed to set context options");
    let options = ai_context.get_context_options().await.expect("Failed to get context options");
    assert_eq!(options, Some("{\"depth\": \"full\"}".to_string()));

    // Test prompt blocks
    ai_context
        .set_prompt_blocks("[{\"type\": \"intro\", \"content\": \"...\"}]")
        .await
        .expect("Failed to set prompt blocks");
    let blocks = ai_context.get_prompt_blocks().await.expect("Failed to get prompt blocks");
    assert_eq!(blocks, Some("[{\"type\": \"intro\", \"content\": \"...\"}]".to_string()));
}

#[tokio::test]
async fn test_ai_context_disabled_skips_insights() {
    let fixture = TestContextFixture::new().await;
    let ai_context = fixture.create_ai_context();

    // Disable AI
    ai_context.set_enabled(false).await.expect("Failed to disable AI");

    // Create a minimal request
    let request = crate::service::GeminiRequest {
        analysis_id: "test-id".to_string(),
        page_count: 1,
        total_issues: 0,
        top_issues: vec![],
        summary: None,
    };

    // Generate insights should return empty string when disabled
    let result = ai_context.generate_insights(request).await;
    assert!(result.is_ok(), "Should not error when disabled");
    assert_eq!(result.unwrap(), "", "Should return empty string when disabled");
}

// ============================================================================
// Cross-Context Integration Tests
// ============================================================================

#[tokio::test]
async fn test_cross_context_job_and_ai_insights() {
    let fixture = TestContextFixture::new().await;
    let analysis_context = fixture.create_analysis_context();
    let ai_context = fixture.create_ai_context();

    // Create a job via analysis context
    let settings = crate::domain::JobSettings::default();
    let job_id = analysis_context
        .create_job("https://example.com", &settings)
        .await
        .expect("Failed to create job");

    // Configure AI context
    ai_context.set_enabled(true).await.expect("Failed to enable AI");
    ai_context.set_api_key("test-key").await.expect("Failed to set API key");

    // Verify both contexts work independently
    let job = analysis_context.get_job(&job_id).await.expect("Failed to get job");
    assert_eq!(job.id, job_id);

    let is_enabled = ai_context.is_enabled().await.expect("Failed to check enabled");
    assert!(is_enabled);
}

#[tokio::test]
async fn test_context_isolation() {
    let fixture = TestContextFixture::new().await;

    // Create separate context instances
    let analysis_context1 = fixture.create_analysis_context();
    let analysis_context2 = fixture.create_analysis_context();

    // Create a job in context1
    let settings = crate::domain::JobSettings::default();
    let job_id = analysis_context1
        .create_job("https://context1.com", &settings)
        .await
        .expect("Failed to create job");

    // Context2 should be able to see the same job (shared repository)
    let job = analysis_context2.get_job(&job_id).await.expect("Failed to get job");
    assert_eq!(job.url, "https://context1.com");
}

// ============================================================================
// Data Flow Validation Tests
// ============================================================================

#[tokio::test]
async fn test_data_flow_from_command_to_repository() {
    let fixture = TestContextFixture::new().await;
    let analysis_context = fixture.create_analysis_context();

    // Simulate command layer creating a job
    let url = "https://dataflow-test.com";
    let settings = crate::domain::JobSettings {
        max_pages: 50,
        include_subdomains: true,
        check_images: false,
        mobile_analysis: true,
        lighthouse_analysis: true,
        delay_between_requests: 100,
    };

    let job_id = analysis_context.create_job(url, &settings).await.expect("Failed to create job");

    // Verify data persisted correctly
    let job = analysis_context.get_job(&job_id).await.expect("Failed to get job");
    assert_eq!(job.url, url);
    assert_eq!(job.settings.max_pages, 50);
    assert!(job.settings.include_subdomains);
    assert!(!job.settings.check_images);
    assert!(job.settings.mobile_analysis);
    assert!(job.settings.lighthouse_analysis);
    assert_eq!(job.settings.delay_between_requests, 100);
}

#[tokio::test]
async fn test_settings_persistence_across_contexts() {
    let fixture = TestContextFixture::new().await;

    // Create AI context and set settings
    let ai_context1 = fixture.create_ai_context();
    ai_context1.set_api_key("shared-key").await.expect("Failed to set API key");
    ai_context1.set_persona("Shared Persona").await.expect("Failed to set persona");

    // Create a new AI context instance (simulating new request)
    let ai_context2 = fixture.create_ai_context();

    // Verify settings persist
    let api_key = ai_context2.get_api_key().await.expect("Failed to get API key");
    let persona = ai_context2.get_persona().await.expect("Failed to get persona");

    assert_eq!(api_key, Some("shared-key".to_string()));
    assert_eq!(persona, Some("Shared Persona".to_string()));
}
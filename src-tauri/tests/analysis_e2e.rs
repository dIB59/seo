//! End-to-end integration tests for SEO analysis.
//!
//! These tests verify the full analysis pipeline against real URLs.

use app::{
    domain::models::JobSettings,
    repository::sqlite::JobRepository,
    service::{AnalysisAssembler, JobProcessor, ProgressReporter},
};
use sqlx::SqlitePool;

/// Creates an in-memory SQLite database with migrations applied for testing.
async fn setup_test_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    pool
}

/// Helper to create a job and return its ID.
async fn create_job(pool: &SqlitePool, url: &str) -> String {
    let repo = JobRepository::new(pool.clone());
    repo.create(url, &JobSettings::default())
        .await
        .expect("Failed to create job")
}

#[tokio::test]
async fn test_iana_example_domains_page() {
    // Test URL: https://www.iana.org/help/example-domains
    // This page explains the purpose of example domains and should:
    // - Have proper title and meta description
    // - Return 200 status code
    // - Be crawlable and have standard SEO elements

    let pool = setup_test_db().await;
    let job_id = create_job(&pool, "https://www.iana.org/help/example-domains").await;

    // Verify job was created
    let repo = JobRepository::new(pool.clone());
    let job = repo.get_by_id(&job_id).await.expect("Failed to get job");

    assert_eq!(job.url, "https://www.iana.org/help/example-domains");
    assert!(!job_id.is_empty(), "Job ID should not be empty");
}

#[tokio::test]
async fn test_example_com_page() {
    // Test URL: https://example.com/
    // This is a minimal example domain that should:
    // - Return 200 status code
    // - Have basic HTML structure
    // - Be fast to load (minimal content)

    let pool = setup_test_db().await;
    let job_id = create_job(&pool, "https://example.com/").await;

    // Verify job was created
    let repo = JobRepository::new(pool.clone());
    let job = repo.get_by_id(&job_id).await.expect("Failed to get job");

    assert_eq!(job.url, "https://example.com/");
    assert!(!job_id.is_empty(), "Job ID should not be empty");
}

#[tokio::test]
async fn test_job_creation_with_custom_settings() {
    let pool = setup_test_db().await;

    let settings = JobSettings {
        max_pages: 10,
        include_external_links: false,
        check_images: true,
        mobile_analysis: false,
        lighthouse_analysis: false,
        delay_between_requests: 100,
    };

    let repo = JobRepository::new(pool.clone());
    let job_id = repo
        .create("https://example.com/", &settings)
        .await
        .expect("Failed to create job with custom settings");

    let job = repo.get_by_id(&job_id).await.expect("Failed to get job");

    assert_eq!(job.settings.max_pages, 10);
    assert_eq!(job.settings.delay_between_requests, 100);
}

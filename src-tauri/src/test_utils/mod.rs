use sqlx::SqlitePool;

pub async fn set_up_test_db_with_prod_data() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:src/test_utils/test.db")
        .await
        .expect("Failed to connect");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    // Apply performance pragmas (same as production)
    sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await.ok();
    sqlx::query("PRAGMA synchronous = NORMAL").execute(&pool).await.ok();
    sqlx::query("PRAGMA cache_size = -65536").execute(&pool).await.ok();
    sqlx::query("PRAGMA mmap_size = 268435456").execute(&pool).await.ok();
    sqlx::query("PRAGMA busy_timeout = 5000").execute(&pool).await.ok();
    sqlx::query("PRAGMA temp_store = MEMORY").execute(&pool).await.ok();
    
    pool
}

/// Connects to the test database WITHOUT running migrations.
/// Use this when the database is already migrated (e.g., for V2 schema benchmarks)
/// and the migration files reference old schema tables that no longer exist.
pub async fn connect_test_db_no_migrate() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:src/test_utils/test.db")
        .await
        .expect("Failed to connect");
    
    // Apply performance pragmas (same as production)
    sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await.ok();
    sqlx::query("PRAGMA synchronous = NORMAL").execute(&pool).await.ok();
    sqlx::query("PRAGMA cache_size = -65536").execute(&pool).await.ok();
    sqlx::query("PRAGMA mmap_size = 268435456").execute(&pool).await.ok();
    sqlx::query("PRAGMA busy_timeout = 5000").execute(&pool).await.ok();
    sqlx::query("PRAGMA temp_store = MEMORY").execute(&pool).await.ok();
    
    pool
}

/// Connects to the V1 test database (old schema) for comparison benchmarks.
/// This database has the original schema: analysis_jobs, analysis_results,
/// page_analysis, seo_issues, page_edge tables.
pub async fn connect_test_db_v1() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:src/test_utils/test_v1.db")
        .await
        .expect("Failed to connect to V1 test database");
    
    // Apply performance pragmas (same as production)
    sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await.ok();
    sqlx::query("PRAGMA synchronous = NORMAL").execute(&pool).await.ok();
    sqlx::query("PRAGMA cache_size = -65536").execute(&pool).await.ok();
    sqlx::query("PRAGMA mmap_size = 268435456").execute(&pool).await.ok();
    sqlx::query("PRAGMA busy_timeout = 5000").execute(&pool).await.ok();
    sqlx::query("PRAGMA temp_store = MEMORY").execute(&pool).await.ok();
    
    pool
}

/// Creates an in-memory database for benchmark write operations
/// Uses the same pragmas as production for realistic measurements
pub async fn set_up_benchmark_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create benchmark database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    // Apply performance pragmas (same as production, except WAL which doesn't work with :memory:)
    sqlx::query("PRAGMA synchronous = NORMAL").execute(&pool).await.ok();
    sqlx::query("PRAGMA cache_size = -65536").execute(&pool).await.ok();
    sqlx::query("PRAGMA temp_store = MEMORY").execute(&pool).await.ok();
    
    pool
}

/// Benchmark data generators for realistic test data
/// Made public for use in benches/
pub mod generators {
    use crate::domain::models::{IssueType, PageAnalysisData, SeoIssue};
    use crate::service::job_processor::PageEdge;

    /// Generate mock pages for benchmarking write operations
    pub fn generate_mock_pages(count: usize, analysis_id: &str) -> Vec<PageAnalysisData> {
        (0..count)
            .map(|i| PageAnalysisData {
                analysis_id: analysis_id.to_string(),
                url: format!("https://example.com/page-{}", i),
                title: Some(format!("Test Page {} - SEO Optimized Title", i)),
                meta_description: Some(format!(
                    "This is a meta description for page {}. It contains relevant keywords.",
                    i
                )),
                meta_keywords: Some("seo, test, benchmark".to_string()),
                canonical_url: Some(format!("https://example.com/page-{}", i)),
                h1_count: 1,
                h2_count: (i % 5) as i64 + 1,
                h3_count: (i % 3) as i64,
                word_count: 500 + (i % 1000) as i64,
                image_count: (i % 10) as i64,
                images_without_alt: (i % 3) as i64,
                internal_links: (i % 20) as i64 + 5,
                external_links: (i % 5) as i64,
                load_time: 0.5 + (i % 100) as f64 / 100.0,
                status_code: Some(200),
                content_size: 10000 + (i * 100) as i64,
                mobile_friendly: i % 10 != 0,
                has_structured_data: i % 5 == 0,
                lighthouse_performance: Some(80.0 + (i % 20) as f64),
                lighthouse_accessibility: Some(90.0 + (i % 10) as f64),
                lighthouse_best_practices: Some(85.0 + (i % 15) as f64),
                lighthouse_seo: Some(88.0 + (i % 12) as f64),
                lighthouse_seo_audits: None,
                lighthouse_performance_metrics: None,
                links: vec![],
                headings: vec![
                    crate::domain::models::HeadingElement {
                        tag: "h1".to_string(),
                        text: format!("Main Heading {}", i),
                    },
                    crate::domain::models::HeadingElement {
                        tag: "h2".to_string(),
                        text: format!("Subheading {}", i),
                    },
                ],
                images: vec![crate::domain::models::ImageElement {
                    src: format!("/images/img-{}.jpg", i),
                    alt: if i % 3 == 0 {
                        None
                    } else {
                        Some(format!("Image {}", i))
                    },
                }],
                detailed_links: vec![
                    crate::domain::models::LinkElement {
                        href: format!("/page-{}", (i + 1) % count.max(1)),
                        text: format!("Link to page {}", (i + 1) % count.max(1)),
                        is_internal: true,
                        status_code: Some(200),
                    },
                    crate::domain::models::LinkElement {
                        href: "https://external.com".to_string(),
                        text: "External link".to_string(),
                        is_internal: false,
                        status_code: None,
                    },
                ],
            })
            .collect()
    }

    /// Generate mock SEO issues for benchmarking
    pub fn generate_mock_issues(count: usize, page_id: &str, page_url: &str) -> Vec<SeoIssue> {
        let issue_templates = [
            (IssueType::Critical, "Missing Title Tag", "Page has no title tag defined"),
            (IssueType::Critical, "Missing Meta Description", "No meta description found"),
            (IssueType::Warning, "Title Too Long", "Title exceeds 60 characters"),
            (IssueType::Warning, "Multiple H1 Tags", "Page has more than one H1 tag"),
            (IssueType::Warning, "Images Missing Alt Text", "Some images lack alt attributes"),
            (IssueType::Suggestion, "Thin Content", "Page has less than 300 words"),
            (IssueType::Suggestion, "No Schema Markup", "Consider adding structured data"),
            (IssueType::Suggestion, "Slow Page Load", "Page load time exceeds 3 seconds"),
        ];

        (0..count)
            .map(|i| {
                let template = &issue_templates[i % issue_templates.len()];
                SeoIssue {
                    page_id: page_id.to_string(),
                    issue_type: template.0.clone(),
                    title: template.1.to_string(),
                    description: template.2.to_string(),
                    page_url: page_url.to_string(),
                    element: Some(format!("<element-{}>", i)),
                    line_number: Some((i + 1) as i64),
                    recommendation: format!("Fix issue #{} by following best practices", i),
                }
            })
            .collect()
    }

    /// Generate mock page edges for benchmarking link graph operations
    pub fn generate_mock_edges(count: usize, page_ids: &[String]) -> Vec<PageEdge> {
        if page_ids.is_empty() {
            return vec![];
        }

        (0..count)
            .map(|i| PageEdge {
                from_page_id: page_ids[i % page_ids.len()].clone(),
                to_url: format!("https://example.com/linked-page-{}", i),
                status_code: if i % 10 == 0 { 404 } else { 200 },
            })
            .collect()
    }
}

#[cfg(test)]
pub mod fixtures {
    use crate::commands::analysis::AnalysisSettingsRequest;
    use crate::service::gemini::GeminiRequest;
    use sqlx::SqlitePool;
    /// Creates an in-memory SQLite database with migrations applied
    pub async fn setup_test_db() -> SqlitePool {
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

    /// Creates a minimal GeminiRequest for testing
    pub fn minimal_gemini_request() -> GeminiRequest {
        GeminiRequest {
            analysis_id: "test".into(),
            url: "https://example.com".into(),
            seo_score: 50,
            pages_count: 1,
            total_issues: 0,
            critical_issues: 0,
            warning_issues: 0,
            suggestion_issues: 0,
            top_issues: vec![],
            avg_load_time: 1.0,
            total_words: 100,
            ssl_certificate: true,
            sitemap_found: true,
            robots_txt_found: true,
        }
    }

    /// Creates default analysis settings for testing
    pub fn default_settings() -> AnalysisSettingsRequest {
        AnalysisSettingsRequest::default()
    }

    /// Creates settings with a specific max_pages value
    pub fn settings_with_max_pages(max_pages: i64) -> AnalysisSettingsRequest {
        AnalysisSettingsRequest {
            max_pages,
            ..Default::default()
        }
    }
}

/// Helper assertions for tests
#[cfg(test)]
pub mod assertions {
    use crate::domain::models::SeoIssue;

    /// Checks if issues contain a specific issue title
    pub fn has_issue(issues: &[SeoIssue], title: &str) -> bool {
        issues.iter().any(|i| i.title == title)
    }

    /// Counts issues of a specific type
    pub fn count_issues(issues: &[SeoIssue], title: &str) -> usize {
        issues.iter().filter(|i| i.title == title).count()
    }

    /// Asserts that a result contains the expected issue
    #[macro_export]
    macro_rules! assert_has_issue {
        ($issues:expr, $title:expr) => {
            assert!(
                $crate::test_utils::assertions::has_issue($issues, $title),
                "Expected to find issue '{}' but it was not present",
                $title
            );
        };
    }

    /// Asserts that a result does NOT contain the specified issue
    #[macro_export]
    macro_rules! assert_no_issue {
        ($issues:expr, $title:expr) => {
            assert!(
                !$crate::test_utils::assertions::has_issue($issues, $title),
                "Expected NOT to find issue '{}' but it was present",
                $title
            );
        };
    }
}

/// Mock server helpers for integration tests
#[cfg(test)]
pub mod mocks {
    use serde_json::json;

    /// Creates a standard HTML page for testing
    pub fn basic_html_page(title: &str, h1: &str) -> String {
        format!(
            r#"
            <html>
                <head><title>{}</title></head>
                <body>
                    <h1>{}</h1>
                    <p>Some content here.</p>
                </body>
            </html>
            "#,
            title, h1
        )
    }

    /// Creates HTML with an image missing alt text
    pub fn html_with_missing_alt() -> String {
        r#"
        <html>
            <head><title>Test Page</title></head>
            <body>
                <h1>Welcome</h1>
                <img src="logo.png">
            </body>
        </html>
        "#
        .to_string()
    }

    /// Creates a mock Gemini API response body
    pub fn gemini_response(text: &str) -> String {
        json!({
            "candidates": [{
                "content": {
                    "parts": [{ "text": text }]
                }
            }]
        })
        .to_string()
    }

    pub fn discord_html() -> String {
        include_str!("mockdiscord.html").to_string()
    }
}

#[cfg(test)]
mod connection_tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_prod_db_connection_test() {
        let pool = set_up_test_db_with_prod_data().await;
        // Simple query to ensure connection is actually working
        let res: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to execute query");
        assert_eq!(res.0, 1);
    }
}

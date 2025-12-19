#[cfg(test)]
pub mod fixtures {
    use crate::commands::analysis::AnalysisSettingsRequest;
    use crate::domain::models::*;
    use crate::service::gemini::GeminiRequest;
    use sqlx::SqlitePool;
    use std::fs;

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

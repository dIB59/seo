use crate::service::http::{create_client, ClientType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;

use crate::db;

/// The Gemini API endpoint path (without base URL)
pub const GEMINI_API_PATH: &str = "/v1beta/models/gemini-2.0-flash:generateContent";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PromptBlock {
    pub id: String,
    pub r#type: String, // "text" or "variable"
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GeminiRequest {
    pub analysis_id: String, // Added for caching
    pub url: String,
    pub seo_score: i32,
    pub pages_count: i32,
    pub total_issues: i32,
    pub critical_issues: i32,
    pub warning_issues: i32,
    pub suggestion_issues: i32,
    pub top_issues: Vec<String>,
    pub avg_load_time: f64,
    pub total_words: i32,
    pub ssl_certificate: bool,
    pub sitemap_found: bool,
    pub robots_txt_found: bool,
}

/// Generate AI-powered SEO analysis using Google Gemini API
pub async fn generate_gemini_analysis(
    pool: &SqlitePool,
    request: GeminiRequest,
    api_base_url: Option<String>,
) -> Result<String> {
    // 1. Check cache first
    if let Ok(Some(cached_insights)) = db::get_ai_insights(pool, &request.analysis_id).await {
        log::info!(
            "Using cached AI insights for analysis {}",
            request.analysis_id
        );
        return Ok(cached_insights);
    }

    // Get API key from database
    let api_key = match db::get_setting(pool, "gemini_api_key").await? {
        Some(key) if !key.is_empty() => key,
        _ => {
            anyhow::bail!("API_KEY_MISSING: Please configure your Gemini API key");
        }
    };

    // Get persona from database
    let persona = match db::get_setting(pool, "gemini_persona").await? {
        Some(p) if !p.is_empty() => p,
        _ => "You are an expert SEO consultant. Your tone is professional, encouraging, and data-driven.".to_string(),
    };

    // Get prompt blocks from database
    let blocks_json = db::get_setting(pool, "gemini_prompt_blocks")
        .await?
        .unwrap_or_else(|| "[]".to_string());

    let blocks: Vec<PromptBlock> = serde_json::from_str(&blocks_json).unwrap_or_default();

    // Helper closure for variable substitution is replaced by public helper

    // Build the requirements/data part of the prompt by processing blocks
    let mut requirements_parts = Vec::new();
    for block in blocks {
        let processed_content = replace_prompt_vars(&block.content, &request);
        requirements_parts.push(processed_content);
    }

    // If no blocks specificed (e.g. migration failed or empty), fallback to sensible default (legacy behavior support)
    if requirements_parts.is_empty() {
        requirements_parts.push(format!(
            "Website: {}\nSEO Score: {}/100",
            request.url, request.seo_score
        ));
    }

    let requirements_text = requirements_parts.join("\n\n");
    let persona_text = replace_prompt_vars(&persona, &request);

    // Assemble the final prompt
    let prompt = format!(
        "{}\n\nAnalyze the following SEO audit results:\n\n{}",
        persona_text, requirements_text
    );

    // Prepare API request
    let base = api_base_url
        .as_deref()
        .unwrap_or("https://generativelanguage.googleapis.com");
    let api_url = format!("{}{}?key={}", base, GEMINI_API_PATH, api_key);

    let request_body = json!({
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }]
    });

    // Make API request
    let client = create_client(ClientType::Standard)?;
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .body(request_body.to_string())
        .send()
        .await
        .context("Failed to send request to Gemini API")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Gemini API error {}: {}", status, error_text);
    }

    // Parse response
    let response_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse Gemini API response")?;

    // Extract text from response
    let text = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .context("Failed to extract text from Gemini response")?
        .to_string();

    // 2. Save to cache
    if let Err(e) = db::save_ai_insights(pool, &request.analysis_id, &text).await {
        log::error!("Failed to save AI insights to cache: {}", e);
    }

    Ok(text)
}

/// Helper to substitute variables in prompt templates
pub fn replace_prompt_vars(text: &str, request: &GeminiRequest) -> String {
    text.replace("{url}", &request.url)
        .replace("{score}", &request.seo_score.to_string())
        .replace("{pages_count}", &request.pages_count.to_string())
        .replace("{total_issues}", &request.total_issues.to_string())
        .replace("{critical_issues}", &request.critical_issues.to_string())
        .replace("{warning_issues}", &request.warning_issues.to_string())
        .replace(
            "{suggestion_issues}",
            &request.suggestion_issues.to_string(),
        )
        .replace("{top_issues}", &request.top_issues.join("\n"))
        .replace("{avg_load_time}", &format!("{:.2}", request.avg_load_time))
        .replace("{total_words}", &request.total_words.to_string())
        .replace(
            "{ssl_certificate}",
            if request.ssl_certificate { "Yes" } else { "No" },
        )
        .replace(
            "{sitemap_found}",
            if request.sitemap_found { "Yes" } else { "No" },
        )
        .replace(
            "{robots_txt_found}",
            if request.robots_txt_found {
                "Yes"
            } else {
                "No"
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_prompt_vars() {
        let request = GeminiRequest {
            analysis_id: "test".into(),
            url: "https://example.com".into(),
            seo_score: 85,
            pages_count: 5,
            total_issues: 10,
            critical_issues: 2,
            warning_issues: 3,
            suggestion_issues: 5,
            top_issues: vec!["Issue 1".into(), "Issue 2".into()],
            avg_load_time: 1.23,
            total_words: 1000,
            ssl_certificate: true,
            sitemap_found: false,
            robots_txt_found: true,
        };

        let template = "Analyze {url} with score {score}. Top issues:\n{top_issues}";
        let result = replace_prompt_vars(template, &request);

        assert!(result.contains("https://example.com"));
        assert!(result.contains("85"));
        assert!(result.contains("Issue 1"));
        assert!(result.contains("Issue 2"));
    }

    #[tokio::test]
    async fn test_gemini_integration() {
        use crate::test_utils::{fixtures, mocks};

        // 1. Setup DB with API Key
        let pool = fixtures::setup_test_db().await;
        db::set_setting(&pool, "gemini_api_key", "test_key")
            .await
            .unwrap();

        // 2. Mock Gemini API using the same constant as production code
        let mut server = mockito::Server::new_async().await;
        let api_path = format!("{}?key=test_key", GEMINI_API_PATH);
        let mock = server
            .mock("POST", api_path.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mocks::gemini_response("AI Analysis Result"))
            .create_async()
            .await;

        // 3. Make Request using fixture
        let mut request = fixtures::minimal_gemini_request();
        request.analysis_id = "integration_test".into();
        request.url = "https://test.com".into();

        let result = generate_gemini_analysis(&pool, request, Some(server.url()))
            .await
            .unwrap();

        // 4. Verify - use contains() for resilience against minor text changes
        assert!(
            result.contains("AI Analysis"),
            "Expected result to contain 'AI Analysis', got: {}",
            result
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_gemini_missing_api_key_returns_error() {
        use crate::test_utils::fixtures;

        // Setup DB WITHOUT API Key
        let pool = fixtures::setup_test_db().await;
        // Don't set gemini_api_key

        let request = fixtures::minimal_gemini_request();

        let result = generate_gemini_analysis(&pool, request, None).await;

        assert!(result.is_err(), "Should fail when API key is missing");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("API_KEY_MISSING"),
            "Error should mention missing API key: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_gemini_api_error_response() {
        use crate::test_utils::fixtures;

        let pool = fixtures::setup_test_db().await;
        db::set_setting(&pool, "gemini_api_key", "bad_key")
            .await
            .unwrap();

        let mut server = mockito::Server::new_async().await;
        let api_path = format!("{}?key=bad_key", GEMINI_API_PATH);
        let _mock = server
            .mock("POST", api_path.as_str())
            .with_status(401)
            .with_body(r#"{"error": "Invalid API key"}"#)
            .create_async()
            .await;

        let request = fixtures::minimal_gemini_request();

        let result = generate_gemini_analysis(&pool, request, Some(server.url())).await;

        assert!(result.is_err(), "Should fail when API returns error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("401"),
            "Error should contain status code: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_gemini_uses_cache_on_second_request() {
        use crate::test_utils::{fixtures, mocks};

        let pool = fixtures::setup_test_db().await;
        db::set_setting(&pool, "gemini_api_key", "test_key")
            .await
            .unwrap();

        // Create a jobs record to satisfy FK constraint when caching (V2 schema)
        let test_job_id = "cache_test_job";
        sqlx::query(
            "INSERT INTO jobs (id, url, status, created_at, updated_at) 
             VALUES (?, 'https://test.com', 'completed', datetime('now'), datetime('now'))"
        )
        .bind(test_job_id)
        .execute(&pool)
        .await
        .unwrap();

        let mut server = mockito::Server::new_async().await;
        let api_path = format!("{}?key=test_key", GEMINI_API_PATH);
        let mock = server
            .mock("POST", api_path.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mocks::gemini_response("Cached Result"))
            .expect(1) // Should only be called ONCE
            .create_async()
            .await;

        // Use the job_id that was pre-created (now called analysis_id in request for backward compat)
        let mut request = fixtures::minimal_gemini_request();
        request.analysis_id = test_job_id.to_string();

        // First call - should hit API
        let result1 = generate_gemini_analysis(&pool, request.clone(), Some(server.url()))
            .await
            .unwrap();
        assert!(
            result1.contains("Cached"),
            "First call should return API result"
        );

        // Second call with same analysis_id - should use cache
        let result2 = generate_gemini_analysis(&pool, request, Some(server.url()))
            .await
            .unwrap();
        assert!(
            result2.contains("Cached"),
            "Second call should return cached result"
        );

        // Verify API was only called once
        mock.assert_async().await;
    }
}

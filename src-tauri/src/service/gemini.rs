use crate::service::prompt::{build_prompt_from_blocks, load_persona, load_prompt_blocks};
use crate::service::spider::SpiderAgent;
#[cfg(test)]
use crate::service::spider::{ClientType, Spider};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;

pub const GEMINI_API_PATH: &str = "/v1beta/models/gemini-2.0-flash:generateContent";

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
pub struct PromptBlock {
    pub id: String,
    pub r#type: String, // "text" or "variable"
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone, Type)]
pub struct GeminiRequest {
    pub analysis_id: String,
    pub url: String,
    pub seo_score: i32,
    pub pages_count: i32,
    pub total_issues: i32,
    pub critical_issues: i32,
    pub warning_issues: i32,
    pub suggestion_issues: i32,
    /// Top issue titles (legacy — kept for backwards compat with
    /// custom prompt blocks that reference `{top_issues}`).
    pub top_issues: Vec<String>,
    pub avg_load_time: f64,
    pub total_words: i32,
    pub ssl_certificate: bool,
    pub sitemap_found: bool,
    pub robots_txt_found: bool,

    // ── Rich context (new) ──────────────────────────────────────────

    /// Per-issue detail lines: "severity | title | page_url | description".
    /// Top 15 issues sorted by severity. Gives the AI enough signal to
    /// write issue-specific recommendations instead of generic advice.
    #[serde(default)]
    pub issue_details: Vec<String>,

    /// Per-page summary lines: "url | title | status | load_time_ms | issue_count".
    /// Top 10 pages by issue count. Lets the AI identify the worst
    /// offenders and reference specific URLs.
    #[serde(default)]
    pub page_summaries: Vec<String>,

    /// Count of pages missing a meta description.
    #[serde(default)]
    pub missing_meta_count: i32,

    /// Count of pages with load time > 3000ms.
    #[serde(default)]
    pub slow_pages_count: i32,

    /// Count of pages returning HTTP 4xx/5xx.
    #[serde(default)]
    pub error_pages_count: i32,

    /// Site-level aggregated tag values from custom extractors. Each
    /// key is the extractor tag name (e.g. `"og_image"`), each value
    /// is a comma-separated list of distinct extracted values across
    /// all pages (capped at 5). `{tag.og_image}` in a prompt block
    /// resolves against this map.
    ///
    /// `#[serde(default)]` so existing frontend calls that don't
    /// populate this field still deserialize correctly with an empty
    /// map — zero breaking change for the wire format.
    #[serde(default)]
    pub tag_values: std::collections::HashMap<String, String>,
}

pub async fn generate_gemini_analysis(
    ai_repo: std::sync::Arc<dyn crate::repository::AiRepository>,
    settings_repo: std::sync::Arc<dyn crate::repository::SettingsRepository>,
    request: GeminiRequest,
    spider: std::sync::Arc<dyn SpiderAgent>,
    api_base_url: Option<String>,
) -> Result<String> {
    // 1. Check cache first
    if let Ok(Some(cached_insights)) = ai_repo.get_ai_insights(&request.analysis_id).await {
        tracing::info!(
            "Using cached AI insights for analysis {}",
            request.analysis_id
        );
        return Ok(cached_insights);
    }

    // Get API key from database
    let api_key = match settings_repo.get_setting("gemini_api_key").await? {
        Some(key) if !key.is_empty() => key,
        _ => {
            anyhow::bail!("API_KEY_MISSING: Please configure your Gemini API key");
        }
    };

    let persona = load_persona(settings_repo.as_ref()).await?;

    let blocks = load_prompt_blocks(settings_repo.as_ref()).await?;

    let prompt = build_prompt_from_blocks(&persona, &blocks, &request);

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
    let response = spider
        .post_json(&api_url, &request_body)
        .await
        .context("Failed to send request to Gemini API")?;

    if response.status != 200 {
        anyhow::bail!("Gemini API error {}: {}", response.status, response.body);
    }

    // Parse response
    let response_json: serde_json::Value =
        serde_json::from_str(&response.body).context("Failed to parse Gemini API response")?;

    // Extract text from response
    let text = response_json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .context("Failed to extract text from Gemini response")?
        .to_string();

    // 2. Save to cache
    if let Err(e) = ai_repo.save_ai_insights(&request.analysis_id, &text).await {
        tracing::error!("Failed to save AI insights to cache: {}", e);
    }

    Ok(text)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite_settings_repo;
    use crate::service::prompt::replace_prompt_vars;

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
            issue_details: vec![],
            page_summaries: vec![],
            missing_meta_count: 0,
            slow_pages_count: 0,
            error_pages_count: 0,
            tag_values: Default::default(),
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
        let repo = sqlite_settings_repo(pool.clone());
        repo.set_setting("gemini_api_key", "test_key")
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

        let ai_repo = crate::repository::sqlite_ai_repo(pool.clone());
        let settings_repo = crate::repository::sqlite_settings_repo(pool.clone());
        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let result =
            generate_gemini_analysis(ai_repo, settings_repo, request, spider, Some(server.url()))
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

        let ai_repo = crate::repository::sqlite_ai_repo(pool.clone());
        let settings_repo = crate::repository::sqlite_settings_repo(pool.clone());
        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let result = generate_gemini_analysis(ai_repo, settings_repo, request, spider, None).await;

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
        let repo = sqlite_settings_repo(pool.clone());
        repo.set_setting("gemini_api_key", "bad_key").await.unwrap();

        let mut server = mockito::Server::new_async().await;
        let api_path = format!("{}?key=bad_key", GEMINI_API_PATH);
        let _mock = server
            .mock("POST", api_path.as_str())
            .with_status(401)
            .with_body(r#"{"error": "Invalid API key"}"#)
            .create_async()
            .await;

        let request = fixtures::minimal_gemini_request();

        let ai_repo = crate::repository::sqlite_ai_repo(pool.clone());
        let settings_repo = crate::repository::sqlite_settings_repo(pool.clone());
        let spider = Spider::new_agent(ClientType::Standard).unwrap();
        let result =
            generate_gemini_analysis(ai_repo, settings_repo, request, spider, Some(server.url()))
                .await;

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
        let repo = sqlite_settings_repo(pool.clone());
        repo.set_setting("gemini_api_key", "test_key")
            .await
            .unwrap();

        // Create a jobs record to satisfy FK constraint when caching (V2 schema)
        let test_job_id = "cache_test_job";
        sqlx::query(
            "INSERT INTO jobs (id, url, status, created_at, updated_at) 
             VALUES (?, 'https://test.com', 'completed', datetime('now'), datetime('now'))",
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
        let ai_repo = crate::repository::sqlite_ai_repo(pool.clone());
        let settings_repo = crate::repository::sqlite_settings_repo(pool.clone());
        let result1 = generate_gemini_analysis(
            ai_repo.clone(),
            settings_repo.clone(),
            request.clone(),
            Spider::new_agent(ClientType::Standard).unwrap(),
            Some(server.url()),
        )
        .await
        .unwrap();
        assert!(
            result1.contains("Cached"),
            "First call should return API result"
        );

        // Second call with same analysis_id - should use cache
        let result2 = generate_gemini_analysis(
            ai_repo,
            settings_repo,
            request,
            Spider::new_agent(ClientType::Standard).unwrap(),
            Some(server.url()),
        )
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

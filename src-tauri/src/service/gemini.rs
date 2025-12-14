use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;

use crate::db;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PromptBlock {
    pub id: String,
    pub r#type: String, // "text" or "variable"
    pub content: String,
}

#[derive(Serialize, Deserialize)]
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
pub async fn generate_gemini_analysis(pool: &SqlitePool, request: GeminiRequest) -> Result<String> {
    // 1. Check cache first
    if let Ok(Some(cached_insights)) = db::get_ai_insights(pool, &request.analysis_id).await {
        log::info!("Using cached AI insights for analysis {}", request.analysis_id);
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
    let blocks_json = db::get_setting(pool, "gemini_prompt_blocks").await?
        .unwrap_or_else(|| "[]".to_string());
    
    let blocks: Vec<PromptBlock> = serde_json::from_str(&blocks_json)
        .unwrap_or_default();

    // Helper closure for variable substitution
    let replace_vars = |text: &str| -> String {
        text.replace("{url}", &request.url)
            .replace("{score}", &request.seo_score.to_string())
            .replace("{pages_count}", &request.pages_count.to_string())
            .replace("{total_issues}", &request.total_issues.to_string())
            .replace("{critical_issues}", &request.critical_issues.to_string())
            .replace("{warning_issues}", &request.warning_issues.to_string())
            .replace("{suggestion_issues}", &request.suggestion_issues.to_string())
            .replace("{top_issues}", &request.top_issues.join("\n"))
            .replace("{avg_load_time}", &format!("{:.2}", request.avg_load_time))
            .replace("{total_words}", &request.total_words.to_string())
            .replace("{ssl_certificate}", if request.ssl_certificate { "Yes" } else { "No" })
            .replace("{sitemap_found}", if request.sitemap_found { "Yes" } else { "No" })
            .replace("{robots_txt_found}", if request.robots_txt_found { "Yes" } else { "No" })
    };

    // Build the requirements/data part of the prompt by processing blocks
    let mut requirements_parts = Vec::new();
    for block in blocks {
        let processed_content = replace_vars(&block.content);
        requirements_parts.push(processed_content);
    }
    
    // If no blocks specificed (e.g. migration failed or empty), fallback to sensible default (legacy behavior support)
    if requirements_parts.is_empty() {
        requirements_parts.push(format!("Website: {}\nSEO Score: {}/100", request.url, request.seo_score));
    }

    let requirements_text = requirements_parts.join("\n\n");
    let persona_text = replace_vars(&persona);

    // Assemble the final prompt
    let prompt = format!("{}\n\nAnalyze the following SEO audit results:\n\n{}", persona_text, requirements_text);

    // Prepare API request
    let api_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    let request_body = json!({
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }]
    });

    // Make API request
    let client = reqwest::Client::new();
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

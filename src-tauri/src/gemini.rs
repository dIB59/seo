use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;

use crate::db;

#[derive(Serialize, Deserialize)]
pub struct GeminiRequest {
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
    // Get API key from database
    let api_key = match db::get_setting(pool, "gemini_api_key").await? {
        Some(key) if !key.is_empty() => key,
        _ => {
            anyhow::bail!("API_KEY_MISSING: Please configure your Gemini API key");
        }
    };

    // Get system prompt from database, or use default if missing
    let system_prompt_template = match db::get_setting(pool, "gemini_system_prompt").await? {
        Some(prompt) if !prompt.is_empty() => prompt,
        _ => r#"You are an expert SEO consultant. Analyze the following SEO audit results and provide actionable recommendations.

Website: {url}
SEO Score: {score}/100
Pages Analyzed: {pages_count}
Total Issues: {total_issues}
- Critical: {critical_issues}
- Warnings: {warning_issues}
- Suggestions: {suggestion_issues}

Top Issues Found:
{top_issues}

Site Metrics:
- Average Load Time: {avg_load_time}s
- Total Words: {total_words}
- SSL Certificate: {ssl_certificate}
- Sitemap Found: {sitemap_found}
- Robots.txt Found: {robots_txt_found}

Please provide:
1. A brief executive summary of the site's SEO health (2-3 sentences)
2. Top 5 priority actions the site owner should take, ranked by impact
3. Expected outcomes if these recommendations are implemented

Keep your response concise, actionable, and professional. Format for a PDF report."#.to_string(),
    };

    // Replace placeholders in the prompt
    let prompt = system_prompt_template
        .replace("{url}", &request.url)
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
        .replace("{robots_txt_found}", if request.robots_txt_found { "Yes" } else { "No" });

    // Prepare API request
    let api_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
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

    Ok(text)
}

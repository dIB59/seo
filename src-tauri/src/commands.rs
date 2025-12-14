use tauri::{command, State};

use crate::analysis;
use crate::db::{get_setting, set_setting, DbState};
use crate::gemini::{generate_gemini_analysis, GeminiRequest};

#[command]
pub async fn get_gemini_insights(
    db: State<'_, DbState>,
    analysis_id: String,
    url: String,
    seo_score: i32,
    pages_count: i32,
    total_issues: i32,
    critical_issues: i32,
    warning_issues: i32,
    suggestion_issues: i32,
    top_issues: Vec<String>,
    avg_load_time: f64,
    total_words: i32,
    ssl_certificate: bool,
    sitemap_found: bool,
    robots_txt_found: bool,
) -> Result<String, String> {
    // Check if AI is enabled globally
    log::info!("Analysis Id for AI insight: {:?}", analysis_id);
    let enabled = get_setting(&db.0, "gemini_enabled")
        .await
        .map_err(|e| format!("Failed to check AI settings: {}", e))?;

    if let Some(val) = enabled {
        if val == "false" {
            log::info!("AI analysis skipped (disabled by user)");
            return Ok("".to_string());
        }
    }

    let request = GeminiRequest {
        analysis_id,
        url,
        seo_score,
        pages_count,
        total_issues,
        critical_issues,
        warning_issues,
        suggestion_issues,
        top_issues,
        avg_load_time,
        total_words,
        ssl_certificate,
        sitemap_found,
        robots_txt_found,
    };

    generate_gemini_analysis(&db.0, request)
        .await
        .map_err(|e| format!("Failed to generate AI insights: {}", e))
}

#[command]
pub async fn get_gemini_enabled(db: State<'_, DbState>) -> Result<bool, String> {
    let val = get_setting(&db.0, "gemini_enabled")
        .await
        .map_err(|e| format!("Failed to check AI settings: {}", e))?;

    // Default to true if not set
    Ok(val.map(|v| v != "false").unwrap_or(true))
}

#[command]
pub async fn set_gemini_enabled(db: State<'_, DbState>, enabled: bool) -> Result<(), String> {
    set_setting(
        &db.0,
        "gemini_enabled",
        if enabled { "true" } else { "false" },
    )
    .await
    .map_err(|e| format!("Failed to update AI settings: {}", e))
}

#[command]
pub async fn get_gemini_api_key(db: State<'_, DbState>) -> Result<Option<String>, String> {
    get_setting(&db.0, "gemini_api_key")
        .await
        .map_err(|e| format!("Failed to get API key: {}", e))
}

#[command]
pub async fn set_gemini_api_key(db: State<'_, DbState>, api_key: String) -> Result<(), String> {
    set_setting(&db.0, "gemini_api_key", &api_key)
        .await
        .map_err(|e| format!("Failed to set API key: {}", e))
}

#[command]
pub async fn get_gemini_persona(db: State<'_, DbState>) -> Result<Option<String>, String> {
    get_setting(&db.0, "gemini_persona")
        .await
        .map_err(|e| format!("Failed to get persona: {}", e))
}

#[command]
pub async fn set_gemini_persona(db: State<'_, DbState>, persona: String) -> Result<(), String> {
    set_setting(&db.0, "gemini_persona", &persona)
        .await
        .map_err(|e| format!("Failed to set persona: {}", e))
}

#[command]
pub async fn get_gemini_requirements(db: State<'_, DbState>) -> Result<Option<String>, String> {
    get_setting(&db.0, "gemini_requirements")
        .await
        .map_err(|e| format!("Failed to get requirements: {}", e))
}

#[command]
pub async fn set_gemini_requirements(
    db: State<'_, DbState>,
    requirements: String,
) -> Result<(), String> {
    set_setting(&db.0, "gemini_requirements", &requirements)
        .await
        .map_err(|e| format!("Failed to set requirements: {}", e))
}

#[command]
pub async fn get_gemini_context_options(db: State<'_, DbState>) -> Result<Option<String>, String> {
    get_setting(&db.0, "gemini_context_options")
        .await
        .map_err(|e| format!("Failed to get context options: {}", e))
}

#[command]
pub async fn set_gemini_context_options(
    db: State<'_, DbState>,
    options: String,
) -> Result<(), String> {
    set_setting(&db.0, "gemini_context_options", &options)
        .await
        .map_err(|e| format!("Failed to set context options: {}", e))
}

#[command]
pub async fn get_gemini_prompt_blocks(db: State<'_, DbState>) -> Result<Option<String>, String> {
    get_setting(&db.0, "gemini_prompt_blocks")
        .await
        .map_err(|e| format!("Failed to get prompt blocks: {}", e))
}

#[command]
pub async fn set_gemini_prompt_blocks(
    db: State<'_, DbState>,
    blocks: String,
) -> Result<(), String> {
    set_setting(&db.0, "gemini_prompt_blocks", &blocks)
        .await
        .map_err(|e| format!("Failed to set prompt blocks: {}", e))
}

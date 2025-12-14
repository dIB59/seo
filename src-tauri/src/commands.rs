use tauri::{command, State};

use crate::db::{DbState, get_setting, set_setting};
use crate::gemini::{generate_gemini_analysis, GeminiRequest};

#[command]
pub async fn get_gemini_insights(
    db: State<'_, DbState>,
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
    let request = GeminiRequest {
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
pub async fn set_gemini_requirements(db: State<'_, DbState>, requirements: String) -> Result<(), String> {
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
pub async fn set_gemini_context_options(db: State<'_, DbState>, options: String) -> Result<(), String> {
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
pub async fn set_gemini_prompt_blocks(db: State<'_, DbState>, blocks: String) -> Result<(), String> {
    set_setting(&db.0, "gemini_prompt_blocks", &blocks)
        .await
        .map_err(|e| format!("Failed to set prompt blocks: {}", e))
}

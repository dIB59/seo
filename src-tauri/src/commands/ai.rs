use tauri::{command, State};

use crate::{
    lifecycle::app_state::AppState,
    service::{generate_gemini_analysis, GeminiRequest},
};

#[command]
#[specta::specta]
pub async fn get_gemini_insights(
    request: GeminiRequest,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    // Check if AI is enabled globally
    tracing::info!("Analysis Id for AI insight: {:?}", request.analysis_id);
    let settings_repo = app_state.settings_repo.clone();
    let ai_repo = app_state.ai_repo.clone();
    let enabled = settings_repo
        .get_setting("gemini_enabled")
        .await
        .map_err(|e| format!("Failed to check AI settings: {}", e))?;

    if let Some(val) = enabled {
        if val == "false" {
            tracing::info!("AI analysis skipped (disabled by user)");
            return Ok("".to_string());
        }
    }

    generate_gemini_analysis(ai_repo, settings_repo, request, None)
        .await
        .map_err(|e| format!("Failed to generate AI insights: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_enabled(app_state: State<'_, AppState>) -> Result<bool, String> {
    let repo = app_state.settings_repo.clone();
    let val = repo
        .get_setting("gemini_enabled")
        .await
        .map_err(|e| format!("Failed to check AI settings: {}", e))?;

    // Default to true if not set
    Ok(val.map(|v| v != "false").unwrap_or(true))
}

#[command]
#[specta::specta]
pub async fn set_gemini_enabled(
    enabled: bool,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = app_state.settings_repo.clone();
    repo.set_setting("gemini_enabled", if enabled { "true" } else { "false" })
        .await
        .map_err(|e| format!("Failed to update AI settings: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_api_key(app_state: State<'_, AppState>) -> Result<Option<String>, String> {
    let repo = app_state.settings_repo.clone();
    repo.get_setting("gemini_api_key")
        .await
        .map_err(|e| format!("Failed to get API key: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_api_key(
    api_key: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = app_state.settings_repo.clone();
    repo.set_setting("gemini_api_key", api_key.as_str())
        .await
        .map_err(|e| format!("Failed to set API key: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_persona(app_state: State<'_, AppState>) -> Result<Option<String>, String> {
    let repo = app_state.settings_repo.clone();
    repo.get_setting("gemini_persona")
        .await
        .map_err(|e| format!("Failed to get persona: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_persona(
    persona: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = app_state.settings_repo.clone();
    repo.set_setting("gemini_persona", persona.as_str())
        .await
        .map_err(|e| format!("Failed to set persona: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_requirements(
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let repo = app_state.settings_repo.clone();
    repo.get_setting("gemini_requirements")
        .await
        .map_err(|e| format!("Failed to get requirements: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_requirements(
    requirements: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = app_state.settings_repo.clone();
    repo.set_setting("gemini_requirements", requirements.as_str())
        .await
        .map_err(|e| format!("Failed to set requirements: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_context_options(
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let repo = app_state.settings_repo.clone();
    repo.get_setting("gemini_context_options")
        .await
        .map_err(|e| format!("Failed to get context options: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_context_options(
    options: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = app_state.settings_repo.clone();
    repo.set_setting("gemini_context_options", options.as_str())
        .await
        .map_err(|e| format!("Failed to set context options: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_prompt_blocks(
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let repo = app_state.settings_repo.clone();
    repo.get_setting("gemini_prompt_blocks")
        .await
        .map_err(|e| format!("Failed to get prompt blocks: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_prompt_blocks(
    blocks: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let repo = app_state.settings_repo.clone();
    repo.set_setting("gemini_prompt_blocks", blocks.as_str())
        .await
        .map_err(|e| format!("Failed to set prompt blocks: {}", e))
}

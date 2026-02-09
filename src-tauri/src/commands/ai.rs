use tauri::{command, State};

use crate::db::DbState;
use crate::service::{generate_gemini_analysis, GeminiRequest};
use crate::lifecycle::SettingsState;

#[command]
#[specta::specta]
pub async fn get_gemini_insights(
    db: State<'_, DbState>,
    request: GeminiRequest,
    settings: State<'_, SettingsState>,
) -> Result<String, String> {
    // Check if AI is enabled globally
    log::info!("Analysis Id for AI insight: {:?}", request.analysis_id);
    let repo = settings.0.clone();
    let enabled = repo
        .get_setting("gemini_enabled")
        .await
        .map_err(|e| format!("Failed to check AI settings: {}", e))?;

    if let Some(val) = enabled {
        if val == "false" {
            log::info!("AI analysis skipped (disabled by user)");
            return Ok("".to_string());
        }
    }

    let ai_repo = std::sync::Arc::new(crate::repository::sqlite::AiRepository::new(db.0.clone()));

    generate_gemini_analysis(ai_repo, &db.0, request, None)
        .await
        .map_err(|e| format!("Failed to generate AI insights: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_enabled(
    _db: State<'_, DbState>,
    settings: State<'_, SettingsState>,
) -> Result<bool, String> {
    let repo = settings.0.clone();
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
    _db: State<'_, DbState>,
    enabled: bool,
    settings: State<'_, SettingsState>,
) -> Result<(), String> {
    let repo = settings.0.clone();
    repo.set_setting("gemini_enabled", if enabled { "true" } else { "false" })
        .await
        .map_err(|e| format!("Failed to update AI settings: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_api_key(
    _db: State<'_, DbState>,
    settings: State<'_, SettingsState>,
) -> Result<Option<String>, String> {
    let repo = settings.0.clone();
    repo.get_setting("gemini_api_key")
        .await
        .map_err(|e| format!("Failed to get API key: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_api_key(
    _db: State<'_, DbState>,
    api_key: String,
    settings: State<'_, SettingsState>,
) -> Result<(), String> {
    let repo = settings.0.clone();
    repo.set_setting("gemini_api_key", &api_key)
        .await
        .map_err(|e| format!("Failed to set API key: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_persona(
    _db: State<'_, DbState>,
    settings: State<'_, SettingsState>,
) -> Result<Option<String>, String> {
    let repo = settings.0.clone();
    repo.get_setting("gemini_persona")
        .await
        .map_err(|e| format!("Failed to get persona: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_persona(
    _db: State<'_, DbState>,
    persona: String,
    settings: State<'_, SettingsState>,
) -> Result<(), String> {
    let repo = settings.0.clone();
    repo.set_setting("gemini_persona", &persona)
        .await
        .map_err(|e| format!("Failed to set persona: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_requirements(
    _db: State<'_, DbState>,
    settings: State<'_, SettingsState>,
) -> Result<Option<String>, String> {
    let repo = settings.0.clone();
    repo.get_setting("gemini_requirements")
        .await
        .map_err(|e| format!("Failed to get requirements: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_requirements(
    _db: State<'_, DbState>,
    requirements: String,
    settings: State<'_, SettingsState>,
) -> Result<(), String> {
    let repo = settings.0.clone();
    repo.set_setting("gemini_requirements", &requirements)
        .await
        .map_err(|e| format!("Failed to set requirements: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_context_options(
    _db: State<'_, DbState>,
    settings: State<'_, SettingsState>,
) -> Result<Option<String>, String> {
    let repo = settings.0.clone();
    repo.get_setting("gemini_context_options")
        .await
        .map_err(|e| format!("Failed to get context options: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_context_options(
    _db: State<'_, DbState>,
    options: String,
    settings: State<'_, SettingsState>,
) -> Result<(), String> {
    let repo = settings.0.clone();
    repo.set_setting("gemini_context_options", &options)
        .await
        .map_err(|e| format!("Failed to set context options: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_prompt_blocks(
    _db: State<'_, DbState>,
    settings: State<'_, SettingsState>,
) -> Result<Option<String>, String> {
    let repo = settings.0.clone();
    repo.get_setting("gemini_prompt_blocks")
        .await
        .map_err(|e| format!("Failed to get prompt blocks: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_prompt_blocks(
    _db: State<'_, DbState>,
    blocks: String,
    settings: State<'_, SettingsState>,
) -> Result<(), String> {
    let repo = settings.0.clone();
    repo.set_setting("gemini_prompt_blocks", &blocks)
        .await
        .map_err(|e| format!("Failed to set prompt blocks: {}", e))
}

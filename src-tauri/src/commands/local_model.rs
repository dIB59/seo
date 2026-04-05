use tauri::{command, State};

use crate::{contexts::local_model::ModelInfo, lifecycle::app_state::AppState};

trait ResultExt<T> {
    fn context(self, msg: &str) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> Result<T, String> {
        self.map_err(|e| format!("{}: {}", msg, e))
    }
}

#[command]
#[specta::specta]
pub async fn list_local_models(app_state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    app_state
        .local_model_context
        .list_models()
        .await
        .context("Failed to list local models")
}

#[command]
#[specta::specta]
pub async fn download_local_model(
    model_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state
        .local_model_context
        .download_model(&model_id)
        .await
        .context("Failed to start model download")
}

#[command]
#[specta::specta]
pub async fn cancel_model_download(
    model_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state.local_model_context.cancel_download(&model_id);
    Ok(())
}

#[command]
#[specta::specta]
pub async fn delete_local_model(
    model_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state
        .local_model_context
        .delete_model(&model_id)
        .await
        .context("Failed to delete model")
}

#[command]
#[specta::specta]
pub async fn get_active_local_model(
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    app_state
        .local_model_context
        .get_active_model_id()
        .await
        .context("Failed to get active model")
}

#[command]
#[specta::specta]
pub async fn set_active_local_model(
    model_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    app_state
        .local_model_context
        .set_active_model(&model_id)
        .await
        .context("Failed to set active model")
}

#[command]
#[specta::specta]
pub async fn generate_local_insights(
    request: crate::service::GeminiRequest,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    app_state
        .local_model_context
        .generate_insights(&request)
        .await
        .context("Failed to generate local AI insights")
}

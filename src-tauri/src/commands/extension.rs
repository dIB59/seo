use tauri::{command, State};

use crate::{
    contexts::extension::{
        CustomCheck, CustomCheckParams, CustomExtractor, CustomExtractorParams,
    }, error::CommandError, lifecycle::app_state::AppState
};

#[command]
#[specta::specta]
pub async fn list_custom_checks(
    app_state: State<'_, AppState>,
) -> Result<Vec<CustomCheck>, CommandError> {
    app_state
        .extension_repo
        .list_checks()
        .await
        .map_err(CommandError::from)
}

#[command]
#[specta::specta]
pub async fn create_custom_check(
    params: CustomCheckParams,
    app_state: State<'_, AppState>,
) -> Result<CustomCheck, CommandError> {
    app_state
        .extension_repo
        .create_check(&params)
        .await
        .map_err(CommandError::from)
}

#[command]
#[specta::specta]
pub async fn update_custom_check(
    id: String,
    params: CustomCheckParams,
    app_state: State<'_, AppState>,
) -> Result<CustomCheck, CommandError> {
    app_state
        .extension_repo
        .update_check(&id, &params)
        .await
        .map_err(CommandError::from)
}

#[command]
#[specta::specta]
pub async fn delete_custom_check(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<(), CommandError> {
    app_state
        .extension_repo
        .delete_check(&id)
        .await
        .map_err(CommandError::from)
}

// --- Custom Extractors ---

#[command]
#[specta::specta]
pub async fn list_custom_extractors(
    app_state: State<'_, AppState>,
) -> Result<Vec<CustomExtractor>, CommandError> {
    app_state
        .extension_repo
        .list_extractors()
        .await
        .map_err(CommandError::from)
}

#[command]
#[specta::specta]
pub async fn create_custom_extractor(
    params: CustomExtractorParams,
    app_state: State<'_, AppState>,
) -> Result<CustomExtractor, CommandError> {
    app_state
        .extension_repo
        .create_extractor(&params)
        .await
        .map_err(CommandError::from)
}

#[command]
#[specta::specta]
pub async fn update_custom_extractor(
    id: String,
    params: CustomExtractorParams,
    app_state: State<'_, AppState>,
) -> Result<CustomExtractor, CommandError> {
    app_state
        .extension_repo
        .update_extractor(&id, &params)
        .await
        .map_err(CommandError::from)
}

#[command]
#[specta::specta]
pub async fn delete_custom_extractor(
    id: String,
    app_state: State<'_, AppState>,
) -> Result<(), CommandError> {
    app_state
        .extension_repo
        .delete_extractor(&id)
        .await
        .map_err(CommandError::from)
}

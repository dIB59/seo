use tauri::{command, State};

use crate::{
    contexts::extension::{
        CustomCheck, CustomCheckParams, CustomExtractor, CustomExtractorParams,
    },
    contexts::tags::{Tag, TagRegistry, TagScope},
    error::CommandError,
    lifecycle::app_state::AppState,
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

// --- Tags ---

/// Return the full tag catalog so the frontend can render tag pickers
/// / autocomplete in the custom-check editor, template editor, and the
/// Settings → Tags panel.
///
/// `scope` is optional: when present the result is filtered to tags
/// valid in that authoring surface (e.g. `CheckField`). When absent
/// every tag is returned.
#[command]
#[specta::specta]
pub async fn list_tags(
    scope: Option<TagScope>,
    app_state: State<'_, AppState>,
) -> Result<Vec<Tag>, CommandError> {
    let registry = TagRegistry::build(app_state.extension_repo.as_ref())
        .await
        .map_err(CommandError::from)?;
    match scope {
        Some(s) => Ok(registry.in_scope(s).into_iter().cloned().collect()),
        None => Ok(registry.into_tags()),
    }
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

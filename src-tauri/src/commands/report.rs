use crate::contexts::report::{ReportData, ReportPattern, ReportPatternParams};
use crate::error::CommandError;
use crate::lifecycle::app_state::AppState;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn list_report_patterns(
    state: State<'_, AppState>,
) -> Result<Vec<ReportPattern>, CommandError> {
    state
        .report_pattern_repo
        .list_patterns()
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn create_report_pattern(
    params: ReportPatternParams,
    state: State<'_, AppState>,
) -> Result<ReportPattern, CommandError> {
    state
        .report_pattern_repo
        .create_pattern(&params)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn update_report_pattern(
    id: String,
    params: ReportPatternParams,
    state: State<'_, AppState>,
) -> Result<ReportPattern, CommandError> {
    state
        .report_pattern_repo
        .update_pattern(&id, &params)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_report_pattern(
    id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    state
        .report_pattern_repo
        .toggle_pattern(&id, enabled)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_report_pattern(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    state
        .report_pattern_repo
        .delete_pattern(&id)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn generate_report_data(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<ReportData, CommandError> {
    state
        .report_context
        .generate_report(&job_id)
        .await
        .map_err(CommandError::from)
}

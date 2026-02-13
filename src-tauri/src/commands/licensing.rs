use crate::domain::licensing::LicenseTier;
use crate::error::CommandError;
use crate::lifecycle::app_state::AppState;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn activate_license(
    license_json: String,
    state: State<'_, AppState>,
) -> Result<LicenseTier, CommandError> {
    let tier = state
        .licensing_service
        .activate(&license_json)
        .await
        .map_err(|e| CommandError::from(e))?;

    state.update_from_tier(tier);
    Ok(tier)
}

#[tauri::command]
#[specta::specta]
pub async fn activate_with_key(
    key: String,
    state: State<'_, AppState>,
) -> Result<LicenseTier, CommandError> {
    let tier = state
        .licensing_service
        .activate_with_key(&key)
        .await
        .map_err(|e| CommandError::from(e))?;

    state.update_from_tier(tier);
    Ok(tier)
}

#[tauri::command]
#[specta::specta]
pub async fn get_license_tier(state: State<'_, AppState>) -> Result<LicenseTier, CommandError> {
    Ok(state.permissions.read().map(|p| p.tier).unwrap_or_default())
}

#[tauri::command]
#[specta::specta]
pub async fn get_machine_id() -> Result<String, CommandError> {
    Ok(crate::service::hardware::HardwareService::get_machine_id())
}

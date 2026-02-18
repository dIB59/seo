use crate::domain::permissions::LicenseTier;
use crate::domain::TierPolicy;
use crate::error::{self, CommandError};
use crate::lifecycle::app_state::AppState;
use crate::service::hardware::HardwareService;
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn activate_license(
    license_key: String,
    state: State<'_, AppState>,
) -> Result<crate::domain::permissions::Policy, CommandError> {
    let tier = state
        .licensing_service
        .activate_with_key(&license_key)
        .await
        .map_err(|e| {
            tracing::error!("[MOCK] Failed to activate license: {}", e);
            CommandError::from(e)
        })?;

    state.update_from_tier(tier);
    Ok(tier.get_policy())
}

#[tauri::command]
#[specta::specta]
pub async fn activate_with_key(
    key: String,
    state: State<'_, AppState>,
) -> Result<crate::domain::permissions::Policy, CommandError> {
    let tier = state
        .licensing_service
        .activate_with_key(&key)
        .await
        .map_err(|e| {
            tracing::error!("[MOCK] Failed to activate license: {}", e);
            CommandError::from(e)
        })?;

    state.update_from_tier(tier);
    Ok(tier.get_policy())
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_policy(
    state: State<'_, AppState>,
) -> Result<crate::domain::permissions::Policy, CommandError> {
    Ok(state.permissions.read().unwrap().clone())
}

#[tauri::command]
#[specta::specta]
pub async fn get_license_tier(state: State<'_, AppState>) -> Result<LicenseTier, CommandError> {
    state
        .permissions
        .read()
        .map_err(|_| {
            CommandError::from(error::AppError::ServiceError {
                service: "Hardware",
                message: "Failed to get machine id".to_string(),
            })
        })
        .map(|policy| policy.tier)
}

#[tauri::command]
#[specta::specta]
pub async fn get_machine_id() -> Result<String, CommandError> {
    Ok(HardwareService::get_machine_id())
}

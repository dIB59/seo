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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::permissions::{Feature, Policy};
    use crate::repository::sqlite_settings_repo;
    use crate::service::licensing::MockLicensingService;
    use crate::test_utils::fixtures::setup_test_db;
    use std::sync::{Arc, RwLock};
    use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
    use tauri::Manager;

    pub struct MockSpider;
    #[async_trait::async_trait]
    impl crate::service::spider::SpiderAgent for MockSpider {
        async fn fetch_html(&self, _url: &str) -> anyhow::Result<String> {
            Ok(String::new())
        }
        async fn get(&self, _url: &str) -> anyhow::Result<crate::service::spider::SpiderResponse> {
            Ok(crate::service::spider::SpiderResponse {
                status: 200,
                body: String::new(),
                url: String::new(),
            })
        }
        async fn post_json(
            &self,
            _url: &str,
            _payload: &serde_json::Value,
        ) -> anyhow::Result<crate::service::spider::SpiderResponse> {
            Ok(crate::service::spider::SpiderResponse {
                status: 200,
                body: String::new(),
                url: String::new(),
            })
        }
    }

    struct NilEmitter;
    impl crate::service::processor::reporter::ProgressEmitter for NilEmitter {
        fn emit(&self, _event: crate::service::processor::reporter::ProgressEvent) {}
    }

    async fn create_test_app() -> tauri::App<MockRuntime> {
        let pool = setup_test_db().await;
        let settings_repo = sqlite_settings_repo(pool.clone());
        let licensing_service = Arc::new(MockLicensingService::new(settings_repo.clone()));

        let state = AppState {
            settings_repo,
            ai_repo: crate::repository::sqlite_ai_repo(pool.clone()),
            job_repo: crate::repository::sqlite_job_repo(pool.clone()),
            results_repo: crate::repository::sqlite_results_repo(pool.clone()),
            standard_spider: Arc::new(MockSpider),
            heavy_spider: Arc::new(MockSpider),
            job_processor: Arc::new(crate::service::JobProcessor::new(
                crate::repository::sqlite_job_repo(pool.clone()),
                crate::repository::sqlite_link_repo(pool.clone()),
                crate::service::processor::AnalyzerService::new(
                    crate::repository::sqlite_page_repo(pool.clone()),
                    crate::repository::sqlite_issue_repo(pool.clone()),
                    Arc::new(MockSpider),
                ),
                crate::service::processor::Crawler::new(Arc::new(MockSpider)),
                Arc::new(NilEmitter),
            )),
            permissions: RwLock::new(Policy::default()),
            licensing_service,
        };

        mock_builder()
            .manage(state)
            .invoke_handler(tauri::generate_handler![
                activate_license,
                get_user_policy,
                get_license_tier,
                get_machine_id
            ])
            .build(mock_context(noop_assets()))
            .expect("failed to build app")
    }

    #[tokio::test]
    async fn test_get_user_policy_ipc() {
        let app = create_test_app().await;
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "get_user_policy".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<Policy>().unwrap())
        .unwrap();

        assert_eq!(res.tier, LicenseTier::Free);
        assert_eq!(res.max_pages, 1);
    }

    #[tokio::test]
    async fn test_activate_license_ipc_success() {
        let app = create_test_app().await;
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        // Get a valid key from the mock service in state
        let state = webview.state::<AppState>();
        let mock_service = MockLicensingService::new(state.settings_repo.clone());
        let valid_key = mock_service.generate_short_key(LicenseTier::Premium);

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "activate_license".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: serde_json::json!({ "licenseKey": valid_key }).into(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<Policy>().unwrap())
        .unwrap();

        assert_eq!(res.tier, LicenseTier::Premium);
        assert!(res.enabled_features.contains(&Feature::LinkAnalysis));
    }

    #[tokio::test]
    async fn test_get_machine_id_ipc() {
        let app = create_test_app().await;
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "get_machine_id".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<String>().unwrap())
        .unwrap();

        assert!(!res.is_empty());
    }
}

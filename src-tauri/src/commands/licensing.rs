use crate::contexts::licensing::{LicenseTier, Policy};
use crate::error::CommandError;
use crate::lifecycle::app_state::AppState;
use crate::service::hardware::HardwareService;
use tauri::State;

fn read_policy(state: &AppState) -> Policy {
    state.permissions.read().map(|p| p.clone()).unwrap_or_default()
}

async fn do_activate(key: &str, state: &AppState) -> Result<Policy, CommandError> {
    let status = state
        .licensing_context
        .activate_with_key(key)
        .await
        .map_err(|e| {
            tracing::error!("Failed to activate license: {}", e);
            CommandError::from(e)
        })?;
    state.update_from_status(status);
    Ok(read_policy(state))
}

#[tauri::command]
#[specta::specta]
pub async fn activate_license(
    license_key: String,
    state: State<'_, AppState>,
) -> Result<Policy, CommandError> {
    do_activate(&license_key, &state).await
}

#[tauri::command]
#[specta::specta]
pub async fn activate_with_key(
    key: String,
    state: State<'_, AppState>,
) -> Result<Policy, CommandError> {
    do_activate(&key, &state).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_policy(state: State<'_, AppState>) -> Result<Policy, CommandError> {
    Ok(read_policy(&state))
}

#[tauri::command]
#[specta::specta]
pub async fn get_license_tier(state: State<'_, AppState>) -> Result<LicenseTier, CommandError> {
    Ok(read_policy(&state).tier)
}

#[tauri::command]
#[specta::specta]
pub async fn get_machine_id() -> Result<String, CommandError> {
    Ok(HardwareService::get_machine_id())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::licensing::{Feature, Policy};
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
        async fn stream_get(&self, _url: &str) -> anyhow::Result<crate::service::spider::StreamResponse> {
            use crate::service::spider::MockSpider as Ms;
            let ms = Ms { html_response: String::new(), generic_response: crate::service::spider::SpiderResponse { status: 200, body: String::new(), url: String::new() } };
            ms.stream_get(_url).await
        }
        async fn stream_get_range(&self, _url: &str, _start_byte: u64) -> anyhow::Result<crate::service::spider::StreamResponse> {
            self.stream_get(_url).await
        }
    }

    struct NilEmitter;
    impl crate::service::processor::reporter::ProgressEmitter for NilEmitter {
        fn emit(&self, _event: crate::service::processor::reporter::ProgressEvent) {}
    }

    async fn create_test_app() -> tauri::App<MockRuntime> {
        let pool = setup_test_db().await;
        let settings_repo = sqlite_settings_repo(pool.clone());
        let ai_repo = crate::repository::sqlite_ai_repo(pool.clone());
        let job_repo = crate::repository::sqlite_job_repo(pool.clone());
        let results_repo = crate::repository::sqlite_results_repo(pool.clone());
        let licensing_service = Arc::new(MockLicensingService::new(settings_repo.clone()));

        // Create the new context-based analysis service
        let analysis_context = crate::contexts::analysis::AnalysisServiceFactory::with_repositories(
            job_repo.clone(),
            results_repo.clone(),
        );

        // Create the new context-based AI service
        let ai_context = crate::contexts::ai::AiServiceFactory::from_repositories(
            ai_repo.clone(),
            settings_repo.clone(),
        );

        let state = AppState {
            standard_spider: Arc::new(MockSpider),
            heavy_spider: Arc::new(MockSpider),
            job_processor: Arc::new(crate::service::JobProcessor::new(
                crate::repository::sqlite_job_repo(pool.clone()),
                crate::repository::sqlite_link_repo(pool.clone()),
                crate::repository::sqlite_page_queue_repo(pool.clone()),
                crate::service::processor::AnalyzerService::new(
                    crate::repository::sqlite_page_repo(pool.clone()),
                    crate::repository::sqlite_issue_repo(pool.clone()),
                    Arc::new(MockSpider),
                    Arc::new(crate::extractor::data_extractor::ExtractorRegistry::new()),
                ),
                crate::service::processor::Crawler::new(Arc::new(MockSpider)),
                Arc::new(NilEmitter),
            )),
            permissions: RwLock::new(Policy::default()),
            licensing_context: licensing_service,
            analysis_context,
            ai_context,
            local_model_context: {
                struct NilEmitter;
                impl crate::service::local_model::DownloadEmitter for NilEmitter {
                    fn emit(&self, _: crate::service::local_model::ModelDownloadEvent) {}
                }
                Arc::new(crate::contexts::local_model::LocalModelService::new(
                    settings_repo.clone(),
                    std::path::PathBuf::from("/tmp"),
                    Arc::new(crate::service::local_model::ModelDownloader::new(
                        Arc::new(crate::service::spider::MockSpider {
                            html_response: String::new(),
                            generic_response: crate::service::spider::SpiderResponse {
                                status: 200,
                                body: String::new(),
                                url: String::new(),
                            },
                        }),
                        Arc::new(NilEmitter),
                    )),
                    Arc::new(crate::service::local_model::LlamaInferenceEngine::new()),
                ))
            },
            extension_repo: crate::repository::sqlite_extension_repo(pool.clone()),
            report_pattern_repo: crate::repository::sqlite_report_pattern_repo(pool.clone()),
            report_context: crate::contexts::report::ReportService::new(
                crate::repository::sqlite_report_pattern_repo(pool.clone()),
                results_repo.clone(),
                settings_repo.clone(),
            ),
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

        // Get a valid key from the mock service using the licensing context
        let _state = webview.state::<AppState>();
        // Create a standalone mock service to generate a test key
        let temp_pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        let temp_settings_repo = crate::repository::sqlite_settings_repo(temp_pool);
        let valid_key = MockLicensingService::new(temp_settings_repo).generate_license_key(
            LicenseTier::Premium,
            Some(chrono::Utc::now() + chrono::Duration::days(365)),
        );

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

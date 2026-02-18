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

    generate_gemini_analysis(
        ai_repo,
        settings_repo,
        request,
        app_state.standard_spider.clone(),
        None,
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::permissions::Policy;
    use crate::lifecycle::app_state::AppState;
    use crate::repository::{sqlite_ai_repo, sqlite_settings_repo};
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
        let ai_repo = sqlite_ai_repo(pool.clone());
        let licensing_service = Arc::new(crate::service::licensing::MockLicensingService::new(
            settings_repo.clone(),
        ));

        let state = AppState {
            settings_repo,
            ai_repo,
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
                get_gemini_insights,
                get_gemini_enabled,
                set_gemini_enabled,
                get_gemini_api_key,
                set_gemini_api_key,
                get_gemini_persona,
                set_gemini_persona,
                get_gemini_requirements,
                set_gemini_requirements
            ])
            .build(mock_context(noop_assets()))
            .expect("failed to build app")
    }

    #[tokio::test]
    async fn test_gemini_settings_management() {
        let app = create_test_app().await;
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        // Test set/get enabled
        tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "set_gemini_enabled".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: serde_json::json!({ "enabled": false }).into(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .unwrap();

        let enabled = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "get_gemini_enabled".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<bool>().unwrap())
        .unwrap();

        assert!(!enabled);

        // Test set/get API key
        tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "set_gemini_api_key".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: serde_json::json!({ "apiKey": "test-key" }).into(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .unwrap();

        let api_key = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "get_gemini_api_key".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<Option<String>>().unwrap())
        .unwrap();

        assert_eq!(api_key, Some("test-key".to_string()));
    }

    #[tokio::test]
    async fn test_get_gemini_insights_disabled_skip() {
        let app = create_test_app().await;
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        // Disable Gemini
        webview
            .state::<AppState>()
            .settings_repo
            .set_setting("gemini_enabled", "false")
            .await
            .unwrap();

        let res = tauri::test::get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "get_gemini_insights".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: serde_json::json!({ "request": crate::test_utils::fixtures::minimal_gemini_request() }).into(),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        ).map(|b| b.deserialize::<String>().unwrap()).unwrap();

        assert_eq!(res, "");
    }
}

use tauri::{command, State};

use crate::{
    lifecycle::app_state::AppState,
    service::GeminiRequest,
};

#[command]
#[specta::specta]
pub async fn get_gemini_insights(
    request: GeminiRequest,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    tracing::info!("Analysis Id for AI insight: {:?}", request.analysis_id);
    
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .generate_insights(request)
        .await
        .map_err(|e| format!("Failed to generate AI insights: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_enabled(app_state: State<'_, AppState>) -> Result<bool, String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .is_enabled()
        .await
        .map_err(|e| format!("Failed to check AI settings: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_enabled(
    enabled: bool,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .set_enabled(enabled)
        .await
        .map_err(|e| format!("Failed to update AI settings: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_api_key(app_state: State<'_, AppState>) -> Result<Option<String>, String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .get_api_key()
        .await
        .map_err(|e| format!("Failed to get API key: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_api_key(
    api_key: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .set_api_key(&api_key)
        .await
        .map_err(|e| format!("Failed to set API key: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_persona(app_state: State<'_, AppState>) -> Result<Option<String>, String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .get_persona()
        .await
        .map_err(|e| format!("Failed to get persona: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_persona(
    persona: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .set_persona(&persona)
        .await
        .map_err(|e| format!("Failed to set persona: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_requirements(
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .get_requirements()
        .await
        .map_err(|e| format!("Failed to get requirements: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_requirements(
    requirements: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .set_requirements(&requirements)
        .await
        .map_err(|e| format!("Failed to set requirements: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_context_options(
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .get_context_options()
        .await
        .map_err(|e| format!("Failed to get context options: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_context_options(
    options: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .set_context_options(&options)
        .await
        .map_err(|e| format!("Failed to set context options: {}", e))
}

#[command]
#[specta::specta]
pub async fn get_gemini_prompt_blocks(
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .get_prompt_blocks()
        .await
        .map_err(|e| format!("Failed to get prompt blocks: {}", e))
}

#[command]
#[specta::specta]
pub async fn set_gemini_prompt_blocks(
    blocks: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // Use the context service (Strangler Fig pattern)
    app_state
        .ai_context
        .set_prompt_blocks(&blocks)
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
        let job_repo = crate::repository::sqlite_job_repo(pool.clone());
        let results_repo = crate::repository::sqlite_results_repo(pool.clone());
        let licensing_service = Arc::new(crate::service::licensing::MockLicensingService::new(
            settings_repo.clone(),
        ));

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
                ),
                crate::service::processor::Crawler::new(Arc::new(MockSpider)),
                Arc::new(NilEmitter),
            )),
            permissions: RwLock::new(Policy::default()),
            licensing_context: licensing_service,
            analysis_context,
            ai_context,
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

        // Disable Gemini using the context service
        webview
            .state::<AppState>()
            .ai_context
            .set_enabled(false)
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

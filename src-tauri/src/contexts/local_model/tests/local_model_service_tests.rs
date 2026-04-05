use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use async_trait::async_trait;

use crate::contexts::local_model::{LocalModelService, ModelInfo};
use crate::repository::SettingsRepository;
use crate::service::local_model::{
    DownloadEmitter, InferenceEngine, InferenceRequest, LlamaInferenceEngine, ModelDownloadEvent,
    ModelDownloader,
};
use crate::service::spider::{MockSpider, SpiderResponse};

// ── Shared stubs ─────────────────────────────────────────────────────────────

struct MockSettingsRepo {
    store: RwLock<HashMap<String, String>>,
}

impl MockSettingsRepo {
    fn new() -> Arc<Self> {
        Arc::new(Self { store: RwLock::new(HashMap::new()) })
    }
}

#[async_trait]
impl SettingsRepository for MockSettingsRepo {
    async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self.store.read().unwrap().get(key).cloned())
    }
    async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.store.write().unwrap().insert(key.to_string(), value.to_string());
        Ok(())
    }
}

struct NilEmitter;
impl DownloadEmitter for NilEmitter {
    fn emit(&self, _: ModelDownloadEvent) {}
}

fn nil_spider() -> Arc<MockSpider> {
    Arc::new(MockSpider {
        html_response: String::new(),
        generic_response: SpiderResponse { status: 200, body: String::new(), url: String::new() },
    })
}

struct MockInferenceEngine {
    response: String,
}

impl MockInferenceEngine {
    fn returns(s: impl Into<String>) -> Arc<Self> {
        Arc::new(Self { response: s.into() })
    }
}

#[async_trait]
impl InferenceEngine for MockInferenceEngine {
    async fn infer(&self, _request: InferenceRequest) -> Result<String> {
        Ok(self.response.clone())
    }
}

struct FailingInferenceEngine;

#[async_trait]
impl InferenceEngine for FailingInferenceEngine {
    async fn infer(&self, _request: InferenceRequest) -> Result<String> {
        Err(anyhow::anyhow!("inference failed"))
    }
}

/// Build a service with a temp directory so file-existence checks work.
fn make_service(
    settings: Arc<dyn SettingsRepository>,
    models_dir: PathBuf,
    engine: Arc<dyn InferenceEngine>,
) -> LocalModelService {
    LocalModelService::new(
        settings,
        models_dir,
        Arc::new(ModelDownloader::new(nil_spider(), Arc::new(NilEmitter))),
        engine,
    )
}

fn minimal_gemini_request() -> crate::service::GeminiRequest {
    crate::service::GeminiRequest {
        analysis_id: "job-1".to_string(),
        url: "https://example.com".to_string(),
        seo_score: 75,
        pages_count: 5,
        total_issues: 3,
        critical_issues: 1,
        warning_issues: 1,
        suggestion_issues: 1,
        top_issues: vec!["Missing meta descriptions".to_string()],
        avg_load_time: 1.5,
        total_words: 2000,
        ssl_certificate: true,
        sitemap_found: true,
        robots_txt_found: true,
    }
}

// ── list_models ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_models_returns_all_registry_models() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    let models = svc.list_models().await.unwrap();
    assert_eq!(models.len(), 3);
}

#[tokio::test]
async fn list_models_none_downloaded_when_dir_absent() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    let models = svc.list_models().await.unwrap();
    assert!(models.iter().all(|m| !m.is_downloaded));
}

#[tokio::test]
async fn list_models_none_active_by_default() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    let models = svc.list_models().await.unwrap();
    assert!(models.iter().all(|m| !m.is_active));
}

#[tokio::test]
async fn list_models_tiers_are_correct() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    let models = svc.list_models().await.unwrap();
    let tiers: Vec<&str> = models.iter().map(|m| m.entry.tier.as_str()).collect();
    assert!(tiers.contains(&"large"));
    assert!(tiers.contains(&"medium"));
    assert!(tiers.contains(&"small"));
}

// ── get_active_model_id ───────────────────────────────────────────────────────

#[tokio::test]
async fn get_active_model_id_none_by_default() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    assert_eq!(svc.get_active_model_id().await.unwrap(), None);
}

#[tokio::test]
async fn get_active_model_id_returns_empty_as_none() {
    // Explicitly storing empty string should still surface as None
    let repo = MockSettingsRepo::new();
    repo.set_setting("local_model_active_id", "").await.unwrap();
    let svc = make_service(repo, PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    assert_eq!(svc.get_active_model_id().await.unwrap(), None);
}

// ── set_active_model ──────────────────────────────────────────────────────────

#[tokio::test]
async fn set_active_model_fails_if_not_downloaded() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    let err = svc.set_active_model("llama-3.2-1b-instruct-q4").await.unwrap_err();
    assert!(err.to_string().contains("not downloaded"), "unexpected error: {err}");
}

#[tokio::test]
async fn set_active_model_fails_for_unknown_id() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    let err = svc.set_active_model("nonexistent-model").await.unwrap_err();
    assert!(err.to_string().contains("Unknown model"), "unexpected error: {err}");
}

#[tokio::test]
async fn set_active_model_succeeds_when_file_present() {
    let dir = tempfile::tempdir().unwrap();
    let model_id = "llama-3.2-1b-instruct-q4";
    let filename = "Llama-3.2-1B-Instruct-Q4_K_M.gguf";

    // Create a fake model file so is_downloaded() returns true
    std::fs::write(dir.path().join(filename), b"fake").unwrap();

    let repo = MockSettingsRepo::new();
    let svc = make_service(repo, dir.path().to_path_buf(), MockInferenceEngine::returns(""));
    svc.set_active_model(model_id).await.unwrap();
    assert_eq!(svc.get_active_model_id().await.unwrap(), Some(model_id.to_string()));
}

#[tokio::test]
async fn set_active_model_is_reflected_in_list_models() {
    let dir = tempfile::tempdir().unwrap();
    let model_id = "llama-3.2-1b-instruct-q4";
    let filename = "Llama-3.2-1B-Instruct-Q4_K_M.gguf";
    std::fs::write(dir.path().join(filename), b"fake").unwrap();

    let svc = make_service(MockSettingsRepo::new(), dir.path().to_path_buf(), MockInferenceEngine::returns(""));
    svc.set_active_model(model_id).await.unwrap();

    let models = svc.list_models().await.unwrap();
    let active: Vec<&ModelInfo> = models.iter().filter(|m| m.is_active).collect();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].entry.id, model_id);
}

// ── delete_model ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_model_fails_for_unknown_id() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    let err = svc.delete_model("does-not-exist").await.unwrap_err();
    assert!(err.to_string().contains("Unknown model"), "unexpected error: {err}");
}

#[tokio::test]
async fn delete_model_removes_file() {
    let dir = tempfile::tempdir().unwrap();
    let filename = "Llama-3.2-1B-Instruct-Q4_K_M.gguf";
    let path = dir.path().join(filename);
    std::fs::write(&path, b"fake").unwrap();
    assert!(path.exists());

    let svc = make_service(MockSettingsRepo::new(), dir.path().to_path_buf(), MockInferenceEngine::returns(""));
    svc.delete_model("llama-3.2-1b-instruct-q4").await.unwrap();
    assert!(!path.exists());
}

#[tokio::test]
async fn delete_model_clears_active_setting() {
    let dir = tempfile::tempdir().unwrap();
    let model_id = "llama-3.2-1b-instruct-q4";
    let filename = "Llama-3.2-1B-Instruct-Q4_K_M.gguf";
    std::fs::write(dir.path().join(filename), b"fake").unwrap();

    let svc = make_service(MockSettingsRepo::new(), dir.path().to_path_buf(), MockInferenceEngine::returns(""));
    svc.set_active_model(model_id).await.unwrap();
    assert_eq!(svc.get_active_model_id().await.unwrap(), Some(model_id.to_string()));

    svc.delete_model(model_id).await.unwrap();
    assert_eq!(svc.get_active_model_id().await.unwrap(), None);
}

#[tokio::test]
async fn delete_model_does_not_clear_active_if_different() {
    let dir = tempfile::tempdir().unwrap();
    // Put two fake models on disk
    std::fs::write(dir.path().join("Llama-3.2-1B-Instruct-Q4_K_M.gguf"), b"fake").unwrap();
    std::fs::write(dir.path().join("microsoft_Phi-4-mini-instruct-Q4_K_M.gguf"), b"fake").unwrap();

    let svc = make_service(MockSettingsRepo::new(), dir.path().to_path_buf(), MockInferenceEngine::returns(""));
    svc.set_active_model("phi-4-mini-instruct-q4").await.unwrap();
    svc.delete_model("llama-3.2-1b-instruct-q4").await.unwrap();

    // Phi is still active
    assert_eq!(
        svc.get_active_model_id().await.unwrap(),
        Some("phi-4-mini-instruct-q4".to_string())
    );
}

// ── download_model ────────────────────────────────────────────────────────────

#[tokio::test]
async fn download_model_fails_for_unknown_id() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/tmp"), MockInferenceEngine::returns(""));
    let err = svc.download_model("not-a-model").await.unwrap_err();
    assert!(err.to_string().contains("Unknown model"), "unexpected error: {err}");
}

#[tokio::test]
async fn download_model_is_noop_if_already_downloaded() {
    // If the file exists, download_model should return Ok without touching the network.
    let dir = tempfile::tempdir().unwrap();
    let filename = "Llama-3.2-1B-Instruct-Q4_K_M.gguf";
    std::fs::write(dir.path().join(filename), b"fake").unwrap();

    let svc = make_service(MockSettingsRepo::new(), dir.path().to_path_buf(), MockInferenceEngine::returns(""));
    // If this tried to download it would fail (no real HTTP server) — but it should skip.
    svc.download_model("llama-3.2-1b-instruct-q4").await.unwrap();
}

// ── generate_insights ─────────────────────────────────────────────────────────

#[tokio::test]
async fn generate_insights_fails_when_no_active_model() {
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/nonexistent"), MockInferenceEngine::returns(""));
    let err = svc.generate_insights(&minimal_gemini_request()).await.unwrap_err();
    assert!(err.to_string().contains("No local model"), "unexpected error: {err}");
}

#[tokio::test]
async fn generate_insights_fails_when_active_model_not_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    // Set active via settings directly without a file present
    let repo = MockSettingsRepo::new();
    repo.set_setting("local_model_active_id", "llama-3.2-1b-instruct-q4").await.unwrap();

    let svc = make_service(repo, dir.path().to_path_buf(), MockInferenceEngine::returns(""));
    let err = svc.generate_insights(&minimal_gemini_request()).await.unwrap_err();
    assert!(err.to_string().contains("not downloaded"), "unexpected error: {err}");
}

#[tokio::test]
async fn generate_insights_returns_engine_output() {
    let dir = tempfile::tempdir().unwrap();
    let filename = "Llama-3.2-1B-Instruct-Q4_K_M.gguf";
    std::fs::write(dir.path().join(filename), b"fake").unwrap();

    let repo = MockSettingsRepo::new();
    repo.set_setting("local_model_active_id", "llama-3.2-1b-instruct-q4").await.unwrap();

    let svc = make_service(repo, dir.path().to_path_buf(), MockInferenceEngine::returns("Great SEO score!"));
    let result = svc.generate_insights(&minimal_gemini_request()).await.unwrap();
    assert_eq!(result, "Great SEO score!");
}

#[tokio::test]
async fn generate_insights_propagates_engine_error() {
    let dir = tempfile::tempdir().unwrap();
    let filename = "Llama-3.2-1B-Instruct-Q4_K_M.gguf";
    std::fs::write(dir.path().join(filename), b"fake").unwrap();

    let repo = MockSettingsRepo::new();
    repo.set_setting("local_model_active_id", "llama-3.2-1b-instruct-q4").await.unwrap();

    let svc = LocalModelService::new(
        repo,
        dir.path().to_path_buf(),
        Arc::new(ModelDownloader::new(nil_spider(), Arc::new(NilEmitter))),
        Arc::new(FailingInferenceEngine),
    );
    let err = svc.generate_insights(&minimal_gemini_request()).await.unwrap_err();
    assert!(err.to_string().contains("inference failed"), "unexpected error: {err}");
}

// ── cancel_download ───────────────────────────────────────────────────────────

#[test]
fn cancel_download_is_noop_when_not_downloading() {
    // Should not panic when cancelling a model that is not actively downloading.
    let svc = make_service(MockSettingsRepo::new(), PathBuf::from("/tmp"), MockInferenceEngine::returns(""));
    svc.cancel_download("llama-3.2-1b-instruct-q4");
}

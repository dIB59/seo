use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use crate::contexts::local_model::domain::{ModelEntry, ModelInfo, MODEL_REGISTRY};
use crate::repository::SettingsRepository;
use crate::service::gemini::GeminiRequest;
use crate::service::local_model::{InferenceEngine, InferenceRequest, ModelDownloader};
use crate::service::prompt::{build_prompt_from_blocks, load_persona, load_prompt_blocks};

const ACTIVE_MODEL_SETTING: &str = "local_model_active_id";

pub struct LocalModelService {
    settings_repo: Arc<dyn SettingsRepository>,
    models_dir: PathBuf,
    pub(crate) downloader: Arc<ModelDownloader>,
    pub(crate) inference_engine: Arc<dyn InferenceEngine>,
}

impl LocalModelService {
    pub fn new(
        settings_repo: Arc<dyn SettingsRepository>,
        models_dir: PathBuf,
        downloader: Arc<ModelDownloader>,
        inference_engine: Arc<dyn InferenceEngine>,
    ) -> Self {
        Self { settings_repo, models_dir, downloader, inference_engine }
    }

    /// All registry models with their downloaded + active status.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let active_id = self.get_active_model_id().await?;
        Ok(MODEL_REGISTRY
            .iter()
            .map(|entry| ModelInfo {
                is_downloaded: self.is_downloaded(entry),
                is_active: active_id.as_deref() == Some(entry.id.as_str()),
                has_partial: self.has_partial(entry),
                entry: entry.clone(),
            })
            .collect())
    }

    /// Start downloading a model. Progress is emitted as `ModelDownloadEvent`.
    pub async fn download_model(&self, model_id: &str) -> Result<()> {
        let entry = ModelEntry::find_by_id(model_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown model id: {model_id}"))?;

        if self.is_downloaded(entry) {
            return Ok(());
        }

        let dest = self.model_path(entry);
        self.downloader.download(model_id, &entry.download_url, &dest).await
    }

    /// Cancel an in-progress download.
    pub fn cancel_download(&self, model_id: &str) {
        self.downloader.cancel(model_id);
    }

    /// Delete the model file (and any partial `.tmp`) from disk, and clear
    /// the active setting if this model was active.
    pub async fn delete_model(&self, model_id: &str) -> Result<()> {
        let entry = ModelEntry::find_by_id(model_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown model id: {model_id}"))?;

        let path = self.model_path(entry);
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }

        // Also clean up any partial download.
        let tmp = path.with_extension("tmp");
        match std::fs::remove_file(&tmp) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }

        // Clear active setting if this was the active model
        if self.get_active_model_id().await?.as_deref() == Some(model_id) {
            self.settings_repo.set_setting(ACTIVE_MODEL_SETTING, "").await?;
        }

        Ok(())
    }

    pub async fn get_active_model_id(&self) -> Result<Option<String>> {
        let val = self.settings_repo.get_setting(ACTIVE_MODEL_SETTING).await?;
        Ok(val.filter(|s| !s.is_empty()))
    }

    pub async fn set_active_model(&self, model_id: &str) -> Result<()> {
        let entry = ModelEntry::find_by_id(model_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown model id: {model_id}"))?;

        if !self.is_downloaded(entry) {
            anyhow::bail!("Model {model_id} is not downloaded yet");
        }

        self.settings_repo.set_setting(ACTIVE_MODEL_SETTING, model_id).await?;
        Ok(())
    }

    /// Generate SEO insights using the active local model.
    ///
    /// Uses the same persona and prompt blocks configured in Settings → AI Instructions,
    /// so Gemini and the local model produce prompts in exactly the same shape.
    pub async fn generate_insights(&self, request: &GeminiRequest) -> Result<String> {
        let model_id = self.get_active_model_id().await?
            .ok_or_else(|| anyhow::anyhow!("No local model selected. Download and activate a model first."))?;

        let entry = ModelEntry::find_by_id(&model_id)
            .ok_or_else(|| anyhow::anyhow!("Active model not found in registry"))?;

        if !self.is_downloaded(entry) {
            anyhow::bail!("Active model is not downloaded. Please re-download it.");
        }

        let model_path = self.model_path(entry);

        // Load persona + blocks — same settings keys used by Gemini
        let persona = load_persona(self.settings_repo.as_ref()).await?;
        let blocks = load_prompt_blocks(self.settings_repo.as_ref()).await?;

        let prompt = build_prompt_from_blocks(&persona, &blocks, request);

        self.inference_engine.infer(InferenceRequest {
            model_path,
            prompt,
            max_tokens: 1024,
            temperature: 0.7,
        }).await
    }

    /// Expose the models directory path (needed by ReportService for direct inference).
    pub fn models_dir(&self) -> &PathBuf {
        &self.models_dir
    }

    /// Low-level inference call — bypasses GeminiRequest wrapping.
    /// Used by ReportService for phased AI brief generation.
    pub async fn infer_raw(&self, request: InferenceRequest) -> Result<String> {
        self.inference_engine.infer(request).await
    }

    // --- private helpers ---

    fn model_path(&self, entry: &ModelEntry) -> PathBuf {
        self.models_dir.join(&entry.filename)
    }

    fn is_downloaded(&self, entry: &ModelEntry) -> bool {
        self.model_path(entry).exists()
    }

    fn has_partial(&self, entry: &ModelEntry) -> bool {
        self.model_path(entry).with_extension("tmp").exists()
    }
}


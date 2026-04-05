use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use crate::contexts::local_model::domain::{ModelEntry, ModelInfo, MODEL_REGISTRY};
use crate::repository::SettingsRepository;
use crate::service::local_model::{InferenceEngine, InferenceRequest, ModelDownloader};
use crate::service::gemini::GeminiRequest;

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
                entry: entry.clone(),
                is_downloaded: self.is_downloaded(entry),
                is_active: active_id.as_deref() == Some(entry.id.as_str()),
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

    /// Delete the model file from disk and clear active if it was active.
    pub async fn delete_model(&self, model_id: &str) -> Result<()> {
        let entry = ModelEntry::find_by_id(model_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown model id: {model_id}"))?;

        let path = self.model_path(entry);
        if path.exists() {
            std::fs::remove_file(&path)?;
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
    pub async fn generate_insights(&self, request: &GeminiRequest) -> Result<String> {
        let model_id = self.get_active_model_id().await?
            .ok_or_else(|| anyhow::anyhow!("No local model selected. Download and activate a model first."))?;

        let entry = ModelEntry::find_by_id(&model_id)
            .ok_or_else(|| anyhow::anyhow!("Active model not found in registry"))?;

        if !self.is_downloaded(entry) {
            anyhow::bail!("Active model is not downloaded. Please re-download it.");
        }

        let model_path = self.model_path(entry);
        let prompt = build_seo_prompt(request);

        self.inference_engine.infer(InferenceRequest {
            model_path,
            prompt,
            max_tokens: 1024,
            temperature: 0.7,
        }).await
    }

    // --- private helpers ---

    fn model_path(&self, entry: &ModelEntry) -> PathBuf {
        self.models_dir.join(&entry.filename)
    }

    fn is_downloaded(&self, entry: &ModelEntry) -> bool {
        self.model_path(entry).exists()
    }
}

fn build_seo_prompt(req: &GeminiRequest) -> String {
    format!(
        "You are an expert SEO consultant. Analyze the following SEO audit results and provide \
        actionable insights and recommendations.\n\n\
        Website: {url}\n\
        SEO Score: {score}/100\n\
        Pages Analyzed: {pages}\n\
        Total Issues: {issues} (Critical: {critical}, Warnings: {warnings}, Info: {info})\n\
        Average Load Time: {load_time:.2}s\n\
        Total Words: {words}\n\
        SSL Certificate: {ssl}\n\
        Sitemap Found: {sitemap}\n\
        Robots.txt Found: {robots}\n\
        Top Issues: {top_issues}\n\n\
        Provide a concise professional analysis with the most important improvements the site owner should make.",
        url = req.url,
        score = req.seo_score,
        pages = req.pages_count,
        issues = req.total_issues,
        critical = req.critical_issues,
        warnings = req.warning_issues,
        info = req.suggestion_issues,
        load_time = req.avg_load_time,
        words = req.total_words,
        ssl = req.ssl_certificate,
        sitemap = req.sitemap_found,
        robots = req.robots_txt_found,
        top_issues = req.top_issues.join(", "),
    )
}

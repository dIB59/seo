use std::path::PathBuf;
use std::sync::Arc;

use crate::repository::SettingsRepository;
use crate::service::local_model::{LlamaInferenceEngine, ModelDownloader};
use super::services::LocalModelService;

pub struct LocalModelServiceFactory;

impl LocalModelServiceFactory {
    pub fn new(
        settings_repo: Arc<dyn SettingsRepository>,
        models_dir: PathBuf,
        app_handle: tauri::AppHandle,
    ) -> LocalModelService {
        let downloader = Arc::new(
            ModelDownloader::with_handle(app_handle)
                .expect("Failed to build model downloader HTTP client"),
        );
        let inference_engine = Arc::new(LlamaInferenceEngine::new());
        LocalModelService::new(settings_repo, models_dir, downloader, inference_engine)
    }
}

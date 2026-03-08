// AiService Factory
// Creates AiService instances with proper dependency injection

use std::sync::Arc;
use super::services::AiService;
use crate::repository::{AiRepository, SettingsRepository};

/// Factory for creating AiService instances
pub struct AiServiceFactory;

impl AiServiceFactory {
    /// Create an AiService from existing repositories
    pub fn from_repositories(
        ai_repo: Arc<dyn AiRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
    ) -> AiService {
        AiService::new(ai_repo, settings_repo)
    }
}

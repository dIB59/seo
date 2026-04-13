// AiService - Service Layer for AI Bounded Context
// Single point of coupling for external modules

use std::sync::Arc;
use anyhow::Result;

use super::super::domain::*;
use crate::repository::{AiRepository, SettingsRepository};
use crate::service::prompt::{PERSONA_SETTING_KEY, PROMPT_BLOCKS_SETTING_KEY};

/// Setting key for the active AI backend selector.
const AI_SOURCE_SETTING_KEY: &str = "ai_source";
/// Setting key for the Gemini API key.
const GEMINI_API_KEY_SETTING_KEY: &str = "gemini_api_key";
/// Setting key for the user-supplied AI requirements text.
const GEMINI_REQUIREMENTS_SETTING_KEY: &str = "gemini_requirements";
/// Setting key for the user-supplied context-options JSON.
const GEMINI_CONTEXT_OPTIONS_SETTING_KEY: &str = "gemini_context_options";
/// Setting key for the AI-enabled feature flag.
const GEMINI_ENABLED_SETTING_KEY: &str = "gemini_enabled";
use crate::service::spider::SpiderAgent;
use crate::service::{generate_gemini_analysis, GeminiRequest};

/// Service layer for the AI bounded context.
/// Single point of coupling - external modules interact only through this service.
pub struct AiService {
    ai_repo: Arc<dyn AiRepository>,
    settings_repo: Arc<dyn SettingsRepository>,
    spider: Option<Arc<dyn SpiderAgent>>,
}

impl AiService {
    /// Create a new AiService
    pub fn new(
        ai_repo: Arc<dyn AiRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
    ) -> Self {
        Self { ai_repo, settings_repo, spider: None }
    }

    /// Create a new AiService with spider for AI generation
    pub fn with_spider(
        ai_repo: Arc<dyn AiRepository>,
        settings_repo: Arc<dyn SettingsRepository>,
        spider: Arc<dyn SpiderAgent>,
    ) -> Self {
        Self { ai_repo, settings_repo, spider: Some(spider) }
    }

    // === Insight Generation ===

    /// Generate AI insights for an analysis
    pub async fn generate_insights(&self, request: GeminiRequest) -> Result<String> {
        // Check if AI is enabled
        let enabled = self.is_enabled().await?;
        if !enabled {
            tracing::info!("AI analysis skipped (disabled by user)");
            return Ok(String::new());
        }

        // Get spider or return error
        let spider = self.spider.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Spider not configured for AI service"))?;

        // Use the existing generate_gemini_analysis function
        generate_gemini_analysis(
            self.ai_repo.clone(),
            self.settings_repo.clone(),
            request,
            spider.clone(),
            None,
        )
        .await
    }

    /// Get stored AI insights for a job
    pub async fn get_insights(&self, job_id: &str) -> Result<Option<AiInsight>> {
        let insights = self.ai_repo.get_ai_insights(job_id).await?;
        let now = chrono::Utc::now();
        Ok(insights.map(|s| AiInsight {
            id: 0, // Default ID for migrated data
            job_id: job_id.to_string(),
            summary: Some(s),
            recommendations: None,
            raw_response: None,
            model: None,
            created_at: now,
            updated_at: now,
        }))
    }

    // === Configuration ===

    /// Set the API key for AI services
    pub async fn set_api_key(&self, key: &str) -> Result<()> {
        Ok(self.settings_repo.set_setting(GEMINI_API_KEY_SETTING_KEY, key).await?)
    }

    /// Get the stored API key
    pub async fn get_api_key(&self) -> Result<Option<String>> {
        Ok(self.settings_repo.get_setting(GEMINI_API_KEY_SETTING_KEY).await?)
    }

    /// Set the AI persona
    pub async fn set_persona(&self, persona: &str) -> Result<()> {
        Ok(self.settings_repo.set_setting(PERSONA_SETTING_KEY, persona).await?)
    }

    /// Get the stored persona
    pub async fn get_persona(&self) -> Result<Option<String>> {
        Ok(self.settings_repo.get_setting(PERSONA_SETTING_KEY).await?)
    }

    /// Set the AI requirements
    pub async fn set_requirements(&self, requirements: &str) -> Result<()> {
        Ok(self.settings_repo.set_setting(GEMINI_REQUIREMENTS_SETTING_KEY, requirements).await?)
    }

    /// Get the stored requirements
    pub async fn get_requirements(&self) -> Result<Option<String>> {
        Ok(self.settings_repo.get_setting(GEMINI_REQUIREMENTS_SETTING_KEY).await?)
    }

    /// Set the AI context options
    pub async fn set_context_options(&self, options: &str) -> Result<()> {
        Ok(self.settings_repo.set_setting(GEMINI_CONTEXT_OPTIONS_SETTING_KEY, options).await?)
    }

    /// Get the stored context options
    pub async fn get_context_options(&self) -> Result<Option<String>> {
        Ok(self.settings_repo.get_setting(GEMINI_CONTEXT_OPTIONS_SETTING_KEY).await?)
    }

    /// Set the AI prompt blocks
    pub async fn set_prompt_blocks(&self, blocks: &str) -> Result<()> {
        Ok(self.settings_repo.set_setting(PROMPT_BLOCKS_SETTING_KEY, blocks).await?)
    }

    /// Get the stored prompt blocks
    pub async fn get_prompt_blocks(&self) -> Result<Option<String>> {
        Ok(self.settings_repo.get_setting(PROMPT_BLOCKS_SETTING_KEY).await?)
    }

    /// Set whether AI is enabled
    pub async fn set_enabled(&self, enabled: bool) -> Result<()> {
        Ok(self
            .settings_repo
            .set_setting(GEMINI_ENABLED_SETTING_KEY, if enabled { "true" } else { "false" })
            .await?)
    }

    /// Check if AI is enabled
    pub async fn is_enabled(&self) -> Result<bool> {
        let val = self.settings_repo.get_setting(GEMINI_ENABLED_SETTING_KEY).await?;
        Ok(val.map(|v| v != "false").unwrap_or(true))
    }

    /// Get the active AI source. Defaults to [`AiSource::Gemini`] if unset
    /// or if a corrupted/legacy value is in the settings row (logs a
    /// warning in that case so the drift is visible).
    pub async fn get_ai_source_typed(&self) -> Result<AiSource> {
        use std::str::FromStr;
        let val = self.settings_repo.get_setting(AI_SOURCE_SETTING_KEY).await?;
        Ok(match val {
            None => AiSource::default(),
            Some(s) => AiSource::from_str(&s).unwrap_or_else(|e| {
                tracing::warn!("ai: unknown ai_source setting '{s}' ({e}); defaulting to gemini");
                AiSource::default()
            }),
        })
    }

    /// Wire-format wrapper kept stable for the existing Tauri command and
    /// frontend bindings — internally delegates to [`Self::get_ai_source_typed`].
    pub async fn get_ai_source(&self) -> Result<String> {
        Ok(self.get_ai_source_typed().await?.as_str().to_string())
    }

    /// Persist the active AI source from a typed enum.
    pub async fn set_ai_source_typed(&self, source: AiSource) -> Result<()> {
        Ok(self
            .settings_repo
            .set_setting(AI_SOURCE_SETTING_KEY, source.as_str())
            .await?)
    }

    /// Wire-format wrapper for the Tauri command. Rejects unknown values
    /// at the boundary instead of silently writing junk into the settings
    /// row — previously a typo from the frontend would persist and the
    /// drift wouldn't surface until the next read.
    pub async fn set_ai_source(&self, source: &str) -> Result<()> {
        use std::str::FromStr;
        let parsed = AiSource::from_str(source)
            .map_err(|e| anyhow::anyhow!("invalid ai_source: {e}"))?;
        self.set_ai_source_typed(parsed).await
    }

}

#[cfg(test)]
mod tests {
    // Tests are in tests/ai_service_tests.rs
}

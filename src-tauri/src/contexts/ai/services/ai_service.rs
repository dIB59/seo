// AiService - Service Layer for AI Bounded Context
// Single point of coupling for external modules

use std::sync::Arc;
use anyhow::Result;

use super::super::domain::*;
use crate::repository::{AiRepository, SettingsRepository};
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
        self.settings_repo.set_setting("gemini_api_key", key).await
    }

    /// Get the stored API key
    pub async fn get_api_key(&self) -> Result<Option<String>> {
        self.settings_repo.get_setting("gemini_api_key").await
    }

    /// Set the AI persona
    pub async fn set_persona(&self, persona: &str) -> Result<()> {
        self.settings_repo.set_setting("gemini_persona", persona).await
    }

    /// Get the stored persona
    pub async fn get_persona(&self) -> Result<Option<String>> {
        self.settings_repo.get_setting("gemini_persona").await
    }

    /// Set the AI requirements
    pub async fn set_requirements(&self, requirements: &str) -> Result<()> {
        self.settings_repo.set_setting("gemini_requirements", requirements).await
    }

    /// Get the stored requirements
    pub async fn get_requirements(&self) -> Result<Option<String>> {
        self.settings_repo.get_setting("gemini_requirements").await
    }

    /// Set the AI context options
    pub async fn set_context_options(&self, options: &str) -> Result<()> {
        self.settings_repo.set_setting("gemini_context_options", options).await
    }

    /// Get the stored context options
    pub async fn get_context_options(&self) -> Result<Option<String>> {
        self.settings_repo.get_setting("gemini_context_options").await
    }

    /// Set the AI prompt blocks
    pub async fn set_prompt_blocks(&self, blocks: &str) -> Result<()> {
        self.settings_repo.set_setting("gemini_prompt_blocks", blocks).await
    }

    /// Get the stored prompt blocks
    pub async fn get_prompt_blocks(&self) -> Result<Option<String>> {
        self.settings_repo.get_setting("gemini_prompt_blocks").await
    }

    /// Set whether AI is enabled
    pub async fn set_enabled(&self, enabled: bool) -> Result<()> {
        self.settings_repo.set_setting("gemini_enabled", if enabled { "true" } else { "false" }).await
    }

    /// Check if AI is enabled
    pub async fn is_enabled(&self) -> Result<bool> {
        let val = self.settings_repo.get_setting("gemini_enabled").await?;
        Ok(val.map(|v| v != "false").unwrap_or(true))
    }

    /// Get the active AI source ("gemini" | "local"). Defaults to "gemini".
    pub async fn get_ai_source(&self) -> Result<String> {
        let val = self.settings_repo.get_setting("ai_source").await?;
        Ok(val.unwrap_or_else(|| "gemini".to_string()))
    }

    /// Set the active AI source ("gemini" | "local").
    pub async fn set_ai_source(&self, source: &str) -> Result<()> {
        self.settings_repo.set_setting("ai_source", source).await
    }

    // === Prompt Configuration ===

    /// Get the current prompt configuration
    /// Note: This is a stub that will be fully implemented in Phase 3
    pub async fn get_prompt_config(&self) -> Result<PromptConfig> {
        // TODO: Load from settings
        Ok(PromptConfig::default())
    }

    /// Update the prompt configuration
    /// Note: This is a stub that will be fully implemented in Phase 3
    pub async fn set_prompt_config(&self, _config: &PromptConfig) -> Result<()> {
        // TODO: Save to settings
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Tests are in tests/ai_service_tests.rs
}

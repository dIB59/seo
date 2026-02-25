// TDD Tests for AiService
// These tests define the expected interface and behavior of the AiService

use std::sync::Arc;
use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;

use crate::contexts::ai::{
    AiService, PromptConfig, PromptBlock,
};
use crate::repository::{AiRepository, SettingsRepository};

// ============================================================================
// Mock Repositories for Testing
// ============================================================================

/// Mock AiRepository for testing
struct MockAiRepository {
    insights: RwLock<HashMap<String, String>>,
}

impl MockAiRepository {
    fn new() -> Self {
        Self {
            insights: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl AiRepository for MockAiRepository {
    async fn get_ai_insights(&self, job_id: &str) -> anyhow::Result<Option<String>> {
        Ok(self.insights.read().unwrap().get(job_id).cloned())
    }

    async fn save_ai_insights(&self, job_id: &str, insights: &str) -> anyhow::Result<()> {
        self.insights.write().unwrap().insert(job_id.to_string(), insights.to_string());
        Ok(())
    }
}

/// Mock SettingsRepository for testing
struct MockSettingsRepository {
    settings: RwLock<HashMap<String, String>>,
}

impl MockSettingsRepository {
    fn new() -> Self {
        Self {
            settings: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SettingsRepository for MockSettingsRepository {
    async fn get_setting(&self, key: &str) -> anyhow::Result<Option<String>> {
        Ok(self.settings.read().unwrap().get(key).cloned())
    }

    async fn set_setting(&self, key: &str, value: &str) -> anyhow::Result<()> {
        self.settings.write().unwrap().insert(key.to_string(), value.to_string());
        Ok(())
    }
}

// ============================================================================
// Tests for AiService Interface
// ============================================================================

/// Test: AiService can be created
#[test]
fn test_ai_service_can_be_created() {
    // Arrange
    let ai_repo = Arc::new(MockAiRepository::new());
    let settings_repo = Arc::new(MockSettingsRepository::new());
    
    // Act
    let _service = AiService::new(ai_repo, settings_repo);
    
    // Assert - Service was created successfully
}

/// Test: AiService can set and get API key
#[tokio::test]
async fn test_ai_service_api_key() {
    // Arrange
    let ai_repo = Arc::new(MockAiRepository::new());
    let settings_repo = Arc::new(MockSettingsRepository::new());
    let service = AiService::new(ai_repo, settings_repo);
    
    // Act - Set API key
    service.set_api_key("test-api-key").await.expect("Failed to set API key");
    
    // Assert - Get API key
    let key = service.get_api_key().await.expect("Failed to get API key");
    assert_eq!(key, Some("test-api-key".to_string()));
}

/// Test: AiService can set and get persona
#[tokio::test]
async fn test_ai_service_persona() {
    // Arrange
    let ai_repo = Arc::new(MockAiRepository::new());
    let settings_repo = Arc::new(MockSettingsRepository::new());
    let service = AiService::new(ai_repo, settings_repo);
    
    // Act - Set persona
    service.set_persona("SEO Expert").await.expect("Failed to set persona");
    
    // Assert - Get persona
    let persona = service.get_persona().await.expect("Failed to get persona");
    assert_eq!(persona, Some("SEO Expert".to_string()));
}

/// Test: AiService returns None for persona when not set
#[tokio::test]
async fn test_ai_service_persona_default() {
    // Arrange
    let ai_repo = Arc::new(MockAiRepository::new());
    let settings_repo = Arc::new(MockSettingsRepository::new());
    let service = AiService::new(ai_repo, settings_repo);
    
    // Act
    let persona = service.get_persona().await.expect("Failed to get persona");
    
    // Assert
    assert_eq!(persona, None);
}

/// Test: AiService can set and check enabled status
#[tokio::test]
async fn test_ai_service_enabled() {
    // Arrange
    let ai_repo = Arc::new(MockAiRepository::new());
    let settings_repo = Arc::new(MockSettingsRepository::new());
    let service = AiService::new(ai_repo, settings_repo);
    
    // Act - Set enabled
    service.set_enabled(true).await.expect("Failed to set enabled");
    
    // Assert - Check enabled
    let enabled = service.is_enabled().await.expect("Failed to check enabled");
    assert!(enabled);
}

/// Test: AiService enabled by default (no setting means enabled)
#[tokio::test]
async fn test_ai_service_enabled_by_default() {
    // Arrange
    let ai_repo = Arc::new(MockAiRepository::new());
    let settings_repo = Arc::new(MockSettingsRepository::new());
    let service = AiService::new(ai_repo, settings_repo);
    
    // Act
    let enabled = service.is_enabled().await.expect("Failed to check enabled");
    
    // Assert - Default is enabled (no setting = true)
    assert!(enabled);
}

/// Test: AiService can get stored insights
#[tokio::test]
async fn test_ai_service_get_insights() {
    // Arrange
    let ai_repo = Arc::new(MockAiRepository::new());
    ai_repo.save_ai_insights("job-123", "Test insights").await.unwrap();
    let settings_repo = Arc::new(MockSettingsRepository::new());
    let service = AiService::new(ai_repo, settings_repo);
    
    // Act
    let insights = service.get_insights("job-123").await.expect("Failed to get insights");
    
    // Assert
    assert!(insights.is_some());
    assert_eq!(insights.unwrap().summary, Some("Test insights".to_string()));
}

/// Test: AiService returns None for non-existent insights
#[tokio::test]
async fn test_ai_service_get_insights_none() {
    // Arrange
    let ai_repo = Arc::new(MockAiRepository::new());
    let settings_repo = Arc::new(MockSettingsRepository::new());
    let service = AiService::new(ai_repo, settings_repo);
    
    // Act
    let insights = service.get_insights("nonexistent").await.expect("Failed to get insights");
    
    // Assert
    assert!(insights.is_none());
}

/// Test: AiService generate_insights returns empty when disabled
#[tokio::test]
async fn test_ai_service_generate_insights_disabled() {
    // Arrange
    let ai_repo = Arc::new(MockAiRepository::new());
    let settings_repo = Arc::new(MockSettingsRepository::new());
    let service = AiService::new(ai_repo, settings_repo);
    
    // Disable AI
    service.set_enabled(false).await.expect("Failed to disable AI");
    
    // Create a minimal request with correct fields
    let request = crate::service::GeminiRequest {
        analysis_id: "job-123".to_string(),
        url: "https://example.com".to_string(),
        seo_score: 80,
        pages_count: 1,
        total_issues: 0,
        critical_issues: 0,
        warning_issues: 0,
        suggestion_issues: 0,
        top_issues: vec![],
        avg_load_time: 0.0,
        total_words: 0,
        ssl_certificate: true,
        sitemap_found: false,
        robots_txt_found: false,
    };
    
    // Act
    let result = service.generate_insights(request).await;
    
    // Assert - Should return empty string when disabled
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

// ============================================================================
// Tests for PromptConfig
// ============================================================================

/// Test: PromptConfig default values
#[test]
fn test_prompt_config_defaults() {
    let config = PromptConfig::default();
    
    assert!(config.persona.is_none());
    assert!(config.requirements.is_none());
    assert!(config.prompt_blocks.is_empty());
}

/// Test: PromptBlock creation
#[test]
fn test_prompt_block_creation() {
    let block = PromptBlock::new("test-id", "Test Title", "Test content");
    
    assert_eq!(block.id, "test-id");
    assert_eq!(block.title, "Test Title");
    assert_eq!(block.content, "Test content");
    assert!(block.enabled);
    assert_eq!(block.order, 0);
}

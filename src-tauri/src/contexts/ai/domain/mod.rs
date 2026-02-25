// AI Context Domain Models
// These are the core domain types for the AI bounded context.

mod insight;

// ============================================================================
// Insight Types
// ============================================================================

pub use insight::AiInsight;

// ============================================================================
// Configuration Types
// ============================================================================

/// Configuration for AI prompts
#[derive(Debug, Clone, Default)]
pub struct PromptConfig {
    pub persona: Option<String>,
    pub requirements: Option<String>,
    pub context_options: ContextOptions,
    pub prompt_blocks: Vec<PromptBlock>,
}

/// Context options for AI analysis
#[derive(Debug, Clone)]
pub struct ContextOptions {
    pub include_issues: bool,
    pub include_links: bool,
    pub include_performance: bool,
    pub include_seo_details: bool,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            include_issues: true,
            include_links: true,
            include_performance: true,
            include_seo_details: true,
        }
    }
}

/// A block of prompt content
#[derive(Debug, Clone)]
pub struct PromptBlock {
    pub id: String,
    pub title: String,
    pub content: String,
    pub enabled: bool,
    pub order: i32,
}

impl PromptBlock {
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            enabled: true,
            order: 0,
        }
    }
}

// AI Context Domain Models
// These are the core domain types for the AI bounded context.

mod insight;

// ============================================================================
// Insight Types
// ============================================================================

pub use insight::AiInsight;

// ============================================================================
// AI source selection
// ============================================================================

/// Which backend the user has selected for AI analysis. Replaces the
/// stringly-typed `String` previously returned by `AiService::get_ai_source`.
/// Wire format stays `"gemini"` / `"local"` so frontend bindings and the
/// `ai_source` settings row are unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiSource {
    #[default]
    Gemini,
    Local,
}

/// Returned by [`AiSource::from_str`] when the input doesn't map to a
/// known backend. Carries the offending value so logs / decoder errors
/// surface what was actually seen.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("invalid AI source: '{0}'")]
pub struct ParseAiSourceError(pub String);

impl AiSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::Local => "local",
        }
    }
}

impl std::str::FromStr for AiSource {
    type Err = ParseAiSourceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gemini" => Ok(Self::Gemini),
            "local" => Ok(Self::Local),
            other => Err(ParseAiSourceError(other.to_string())),
        }
    }
}

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

// AI Bounded Context
// Handles AI-powered insights, prompt management, and configuration

mod domain;
mod services;
mod factory;

#[cfg(test)]
mod tests;

// Public API - what external modules can use
pub use services::AiService;
pub use factory::AiServiceFactory;

// Domain types exposed to external contexts
pub use domain::{AiInsight, PromptConfig, PromptBlock};

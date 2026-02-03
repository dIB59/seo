//! Error types for the SEO analyzer application.
//!
//! This module provides structured error handling with:
//! - `AppError`: Domain-specific errors for application operations
//! - `CommandError`: Wrapper for Tauri command errors (serializable)
//! - `Result<T>`: Type alias for Results using AppError

use serde::Serialize;
use std::fmt;
use thiserror::Error;

// ============================================================================
// DOMAIN ERROR TYPE
// ============================================================================

/// Domain-specific errors for application operations.
#[derive(Debug, Error)]
pub enum AppError {
    /// Invalid or malformed URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    /// Network request failed
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Failed to parse HTML content
    #[error("HTML parsing error: {0}")]
    ParseError(String),
    
    /// Database operation failed
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Job not found
    #[error("Job not found: {0}")]
    JobNotFound(i64),
    
    /// Analysis result not found
    #[error("Analysis not found: {0}")]
    AnalysisNotFound(String),
    
    /// External service error (Lighthouse, Gemini, etc.)
    #[error("Service error ({service}): {message}")]
    ServiceError { service: &'static str, message: String },
    
    /// Job was cancelled
    #[error("Job cancelled")]
    Cancelled,
    
    /// Generic error with context
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl AppError {
    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::NetworkError(msg.into())
    }
    
    /// Create a service error
    pub fn service(service: &'static str, msg: impl Into<String>) -> Self {
        Self::ServiceError { service, message: msg.into() }
    }
    
    /// Create a database error
    pub fn database(msg: impl Into<String>) -> Self {
        Self::DatabaseError(msg.into())
    }
}

/// Result type alias using AppError.
pub type Result<T> = std::result::Result<T, AppError>;

// ============================================================================
// COMMAND ERROR (FOR TAURI)
// ============================================================================

/// Wrapper for errors returned from Tauri commands.
/// This type is serializable and can be sent to the frontend.
#[derive(Debug)]
pub struct CommandError(pub anyhow::Error);

impl std::error::Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:#}", self.0))
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(error: anyhow::Error) -> Self {
        Self(error)
    }
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        Self(error.into())
    }
}

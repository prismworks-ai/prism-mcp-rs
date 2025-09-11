//! Utility functions and helpers for the MCP Rust SDK
//!
//! This module provides various utility functions for URI handling, validation,
//! string manipulation, async helpers, and other common operations used throughout the SDK.

pub mod async_helpers;
pub mod string_utils;
pub mod time;
pub mod uri;
pub mod validation;

// Re-export commonly used utilities
pub use async_helpers::*;
pub use string_utils::*;
pub use time::*;
pub use uri::*;
pub use validation::*;

/// Common result type for utility functions
pub type UtilResult<T> = Result<T, UtilError>;

/// Utility function errors
#[derive(Debug, thiserror::Error)]
pub enum UtilError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Timeout occurred after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

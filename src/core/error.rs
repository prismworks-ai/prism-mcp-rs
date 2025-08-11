// ! Error types for the MCP Rust SDK
// !
// ! Module defines all error types that can occur within the MCP SDK,
// ! providing structured error handling with detailed context.

use thiserror::Error;

/// The main error type for the MCP SDK
#[derive(Error, Debug, Clone)]
pub enum McpError {
    /// Transport-related errors (connection, I/O, etc.)
    #[error("Transport error: {0}")]
    Transport(String),

    /// Protocol-level errors (invalid messages, unexpected responses, etc.)
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// JSON serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid URI format or content
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    /// Requested tool was not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Requested resource was not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Requested prompt was not found
    #[error("Prompt not found: {0}")]
    PromptNotFound(String),

    /// Method not found (JSON-RPC error)
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Invalid parameters (JSON-RPC error)
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Connection-related errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Authentication/authorization errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// OAuth 2.1 authorization errors
    #[error("Authorization error: {0}")]
    Auth(String),

    /// Input validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// I/O errors from the standard library
    #[error("I/O error: {0}")]
    Io(String),

    /// URL parsing errors
    #[error("URL error: {0}")]
    Url(String),

    /// HTTP-related errors when using HTTP transport
    #[cfg(feature = "http")]
    #[error("HTTP error: {0}")]
    Http(String),

    /// WebSocket-related errors when using WebSocket transport
    #[cfg(feature = "websocket")]
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// JSON Schema validation errors
    #[cfg(feature = "validation")]
    #[error("Schema validation error: {0}")]
    SchemaValidation(String),

    /// Timeout errors
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Cancellation errors
    #[error("Operation cancelled: {0}")]
    Cancelled(String),

    /// Internal errors that shouldn't normally occur
    #[error("Internal error: {0}")]
    Internal(String),
}

// Manual From implementations for types that don't implement Clone
impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        McpError::Io(err.to_string())
    }
}

impl From<url::ParseError> for McpError {
    fn from(err: url::ParseError) -> Self {
        McpError::Url(err.to_string())
    }
}

/// Result type alias for MCP operations
pub type McpResult<T> = Result<T, McpError>;

impl McpError {
    /// Create a new transport error
    pub fn transport<S: Into<String>>(message: S) -> Self {
        Self::Transport(message.into())
    }

    /// Create a new protocol error
    pub fn protocol<S: Into<String>>(message: S) -> Self {
        Self::Protocol(message.into())
    }

    /// Create a new validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    /// Create a new connection error
    pub fn connection<S: Into<String>>(message: S) -> Self {
        Self::Connection(message.into())
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    /// Create a new IO error from std::io::Error
    pub fn io(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }

    /// Create a new serialization error from serde_json::Error
    pub fn serialization(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        Self::Timeout(message.into())
    }

    /// Create a connection error (compatibility method)
    pub fn connection_error<S: Into<String>>(message: S) -> Self {
        Self::Connection(message.into())
    }

    /// Create a protocol error (compatibility method)
    pub fn protocol_error<S: Into<String>>(message: S) -> Self {
        Self::Protocol(message.into())
    }

    /// Create a validation error (compatibility method)
    pub fn validation_error<S: Into<String>>(message: S) -> Self {
        Self::Validation(message.into())
    }

    /// Create a timeout error (compatibility method)
    pub fn timeout_error() -> Self {
        Self::Timeout("Operation timed out".to_string())
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            McpError::Transport(_) => false,
            McpError::Protocol(_) => false,
            McpError::Connection(_) => true,
            McpError::Timeout(_) => true,
            McpError::Validation(_) => false,
            McpError::ToolNotFound(_) => false,
            McpError::ResourceNotFound(_) => false,
            McpError::PromptNotFound(_) => false,
            McpError::MethodNotFound(_) => false,
            McpError::InvalidParams(_) => false,
            McpError::Authentication(_) => false,
            McpError::Serialization(_) => false,
            McpError::InvalidUri(_) => false,
            McpError::Io(_) => true,
            McpError::Url(_) => false,
            #[cfg(feature = "http")]
            McpError::Http(_) => true,
            #[cfg(feature = "websocket")]
            McpError::WebSocket(_) => true,
            #[cfg(feature = "validation")]
            McpError::SchemaValidation(_) => false,
            McpError::Cancelled(_) => false,
            McpError::Auth(_) => false,
            McpError::Internal(_) => false,
        }
    }

    /// Get the error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            McpError::Transport(_) => "transport",
            McpError::Protocol(_) => "protocol",
            McpError::Connection(_) => "connection",
            McpError::Timeout(_) => "timeout",
            McpError::Validation(_) => "validation",
            McpError::ToolNotFound(_) => "not_found",
            McpError::ResourceNotFound(_) => "not_found",
            McpError::PromptNotFound(_) => "not_found",
            McpError::MethodNotFound(_) => "not_found",
            McpError::InvalidParams(_) => "validation",
            McpError::Authentication(_) => "auth",
            McpError::Serialization(_) => "serialization",
            McpError::InvalidUri(_) => "validation",
            McpError::Io(_) => "io",
            McpError::Url(_) => "validation",
            #[cfg(feature = "http")]
            McpError::Http(_) => "http",
            #[cfg(feature = "websocket")]
            McpError::WebSocket(_) => "websocket",
            #[cfg(feature = "validation")]
            McpError::SchemaValidation(_) => "validation",
            McpError::Cancelled(_) => "cancelled",
            McpError::Auth(_) => "auth",
            McpError::Internal(_) => "internal",
        }
    }
}

// Convert common HTTP errors when the feature is enabled
#[cfg(feature = "http")]
impl From<reqwest::Error> for McpError {
    fn from(err: reqwest::Error) -> Self {
        McpError::Http(err.to_string())
    }
}

// Convert common WebSocket errors when the feature is enabled
#[cfg(feature = "websocket")]
impl From<tokio_tungstenite::tungstenite::Error> for McpError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        McpError::WebSocket(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = McpError::transport("Connection failed");
        assert_eq!(error.to_string(), "Transport error: Connection failed");
        assert_eq!(error.category(), "transport");
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_error_recovery() {
        assert!(McpError::connection("timeout").is_recoverable());
        assert!(!McpError::validation("invalid input").is_recoverable());
        assert!(McpError::timeout("request timeout").is_recoverable());
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(McpError::protocol("bad message").category(), "protocol");
        assert_eq!(
            McpError::ToolNotFound("missing".to_string()).category(),
            "not_found"
        );
        assert_eq!(
            McpError::Authentication("unauthorized".to_string()).category(),
            "auth"
        );
    }
}

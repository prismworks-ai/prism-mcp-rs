//! Enhanced error types for better ergonomics
//! Replaces generic String messages with structured errors

use std::time::Duration;
use thiserror::Error;

/// Structured transport error types
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Process start failed: {command} - {reason}")]
    ProcessStartFailed { command: String, reason: String },

    #[error("I/O operation failed: {operation}")]
    IoError {
        operation: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Transport not connected")]
    NotConnected,

    #[error("Transport already closed")]
    AlreadyClosed,
}

/// Structured protocol error types
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid message format: {message}")]
    InvalidFormat { message: String },

    #[error("Method not found: {method}")]
    MethodNotFound { method: String },

    #[error("Invalid parameters for {method}: {reason}")]
    InvalidParameters { method: String, reason: String },

    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
}

/// Enhanced MCP error with structured types
#[derive(Error, Debug)]
pub enum McpError {
    #[error("Transport error: {source}")]
    Transport {
        #[from]
        source: TransportError,
    },

    #[error("Protocol error: {source}")]
    Protocol {
        #[from]
        source: ProtocolError,
    },

    #[error("Operation timeout after {duration:?}")]
    Timeout { duration: Duration },

    #[error("Connection error: {message}")]
    Connection { message: String },

    #[error("Serialization error: {message}")]
    Serialization { message: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Result type alias for convenience
pub type McpResult<T> = Result<T, McpError>;

/// Implementations for backward compatibility and convenience
impl McpError {
    /// Create a transport error with message
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            source: TransportError::ConnectionFailed {
                message: message.into(),
            },
        }
    }

    /// Create a protocol error with message
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            source: ProtocolError::InvalidFormat {
                message: message.into(),
            },
        }
    }

    /// Create a timeout error
    pub fn timeout(_message: impl Into<String>) -> Self {
        Self::Timeout {
            duration: Duration::from_secs(30), // Default timeout
        }
    }

    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Check if error is recoverable (for retry logic)
    pub fn is_recoverable(&self) -> bool {
        match self {
            McpError::Transport { source } => matches!(
                source,
                TransportError::ConnectionFailed { .. } | TransportError::IoError { .. }
            ),
            McpError::Timeout { .. } => true,
            McpError::Connection { .. } => true,
            _ => false,
        }
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        Self::Transport {
            source: TransportError::IoError {
                operation: "I/O operation".to_string(),
                source: err,
            },
        }
    }
}

/// Convert from serde_json::Error  
impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: err.to_string(),
        }
    }
}

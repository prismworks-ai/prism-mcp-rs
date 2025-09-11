//! Prelude module for convenient imports
//!
//! This module re-exports the most commonly used types and traits from the SDK,
//! allowing users to import everything they need with a single `use` statement.
//!
//! # Usage
//!
//! ```rust
//! use prism_mcp_rs::prelude::*;
//!
//! // Now you have access to all common types
//! let client = McpClient::new("my-client".to_string(), "1.0.0".to_string());
//! let server = McpServer::new("my-server".to_string(), "1.0.0".to_string());
//! ```

// Core types and traits
pub use crate::core::{
    error::{McpError, McpResult},
    tool::{Tool, ToolHandler, ToolBuilder, EchoTool},
    resource::{Resource, ResourceHandler},
    prompt::{Prompt, PromptHandler},
    completion::{CompletionHandler, CompletionProvider},
    validation::Validator,
    health::{HealthChecker, HealthStatus},
    metrics::{MetricsCollector, PerformanceMetrics as CoreMetrics},
    retry::{RetryConfig as CoreRetryConfig, RetryPolicy as CoreRetryPolicy},
};

// Client and server
pub use crate::{
    client::{McpClient, McpClientBuilder, ClientSession, ClientRequestHandler},
    server::{McpServer, ServerBuilder},
};

// Protocol types
pub use crate::protocol::{
    types::*,
    messages::*,
    JsonRpcRequest, JsonRpcResponse, JsonRpcError, JsonRpcMessage,
    JSONRPC_VERSION, LATEST_PROTOCOL_VERSION,
};

// Transport layer
pub use crate::transport::{
    Transport, ServerTransport, TransportConfig,
    ConnectionState, TransportEvent, TransportStats,
};

// Feature-gated transport re-exports
#[cfg(feature = "stdio")]
pub use crate::transport::{StdioClientTransport, StdioServerTransport};

#[cfg(feature = "http")]
pub use crate::transport::{HttpClientTransport, HttpServerTransport};

#[cfg(feature = "websocket")]
pub use crate::transport::{WebSocketClientTransport, WebSocketServerTransport};

// Authentication (when available)
pub use crate::auth::{AuthConfig, AuthError, AuthResult};

// Plugin system
pub use crate::plugin::{
    PluginManager, PluginLoader, PluginConfig,
    ToolPlugin, PluginError, PluginResult,
};

// Utilities
pub use crate::utils::*;

// Common external re-exports
pub use async_trait::async_trait;
pub use serde_json::{json, Value};
pub use std::collections::HashMap;
pub use tokio;

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, McpError>;

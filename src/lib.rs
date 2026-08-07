// Copyright (c) 2025 Prismworks AI Inc.
// SPDX-License-Identifier: MIT

//! # Prism MCP Rust SDK
//!
//! Async client and server primitives for the
//! [Model Context Protocol](https://modelcontextprotocol.io/) 2025-11-25.
//! The crate includes protocol types, tools, resources, prompts, sampling,
//! completion, roots, replaceable transports, and opt-in production controls.
//!
//! STDIO is enabled by default. HTTP, WebSocket, SSE, authentication helpers,
//! TLS/mTLS, OpenTelemetry, compression, and trusted native plugins are
//! feature-gated. Security controls are not enabled automatically: a host must
//! authenticate callers and install its request policy.
//!
//! ## Quick Start
//!
//! The easiest way to get started is with the prelude module:
//!
//! ```rust
//! use prism_mcp_rs::prelude::*;
//! ```
//!
//! This imports all the commonly used types and traits.
//!
//! ### Server Example
//!
//! ```rust,no_run
//! use prism_mcp_rs::prelude::*;
//! use std::collections::HashMap;
//!
//! struct EchoHandler;
//!
//! #[async_trait]
//! impl ToolHandler for EchoHandler {
//!     async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
//!         let message = arguments.get("message")
//!             .and_then(|v| v.as_str())
//!             .unwrap_or_default();
//!
//!         Ok(ToolResult {
//!             content: vec![ContentBlock::text(message)],
//!             is_error: Some(false),
//!             structured_content: None,
//!             meta: None,
//!         })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> McpResult<()> {
//!     let server = McpServer::create("echo-server", "1.0.0");
//!
//!     server.add_tool(
//!         "echo",
//!         Some("Echo a message"),
//!         json!({
//!             "type": "object",
//!             "properties": {
//!                 "message": { "type": "string" }
//!             }
//!         }),
//!         EchoHandler,
//!     ).await?;
//!
//!     server.run_with_transport(StdioServerTransport::new()).await
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`core`]: Core abstractions for resources, tools, prompts, and errors
//! - `plugin`: trusted native dynamic tool loading (feature-gated)
//! - [`protocol`]: MCP protocol types and message definitions (2025-11-25)
//! - [`transport`]: Transport layer implementations (STDIO, HTTP, WebSocket)
//! - [`server`]: MCP server implementation and lifecycle management
//! - [`client`]: MCP client implementation and session management
//! - [`security`]: request identity, authorization, and rate limiting
//! - [`utils`]: utility functions and helpers

#[cfg(feature = "http")]
pub mod auth;
pub mod client;
pub mod core;
#[cfg(feature = "plugin")]
pub mod plugin;
pub mod protocol;
pub mod security;
pub mod server;
#[cfg(feature = "otel")]
pub mod telemetry;
pub mod transport;
pub mod utils;

// Re-export commonly used types for convenience
pub use core::error::{McpError, McpResult};
pub use protocol::types::*;
pub use protocol::{
    ErrorObject, JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, ServerCapabilities,
};

/// Prelude module for convenient imports (2025-11-25)
///
/// Module re-exports the most commonly used types and traits for easy access.
/// Use `use prism_mcp_rs::prelude::*;` to import everything you need.
pub mod prelude {
    // Core types and traits
    pub use crate::core::{
        error::{McpError, McpResult},
        prompt::{Prompt, PromptHandler},
        resource::{Resource, ResourceHandler},
        tool::{Tool, ToolHandler},
    };
    pub use crate::security::{
        Permission, Principal, RateLimitConfig, RateLimiter, RbacAuthorizer, RequestContext,
        RequestPolicy, RequestTarget,
    };

    // Protocol types and messages
    pub use crate::protocol::error_codes;
    pub use crate::protocol::error_helpers::IntoJsonRpcMessage;
    pub use crate::protocol::messages::*;
    pub use crate::protocol::missing_types::*;
    pub use crate::protocol::types::*;

    // Client and completion handlers
    pub use crate::client::{
        AutomatedClientRequestHandler, ClientRequestHandler, InteractiveClientRequestHandler,
    };
    pub use crate::core::completion::{
        CompletionHandler, PromptCompletionHandler, ResourceUriCompletionHandler,
    };
    pub use crate::core::completion_handlers::{
        CompositeCompletionHandler as ExtendedCompositeCompletionHandler,
        FileSystemCompletionHandler, FuzzyCompletionHandler, SchemaCompletionHandler,
    };

    // Server and Client
    pub use crate::client::McpClient;
    pub use crate::server::{McpServer, ServerBuilder, ServerConfig};

    // Transport layer implementations
    #[cfg(feature = "stdio")]
    pub use crate::transport::{StdioClientTransport, StdioServerTransport};

    #[cfg(feature = "http")]
    pub use crate::transport::{HttpClientTransport, HttpServerTransport};

    #[cfg(feature = "websocket")]
    pub use crate::transport::{WebSocketClientTransport, WebSocketServerTransport};

    pub use crate::transport::{EndpointPoolConfig, EndpointPoolTransport};

    // Plugin system
    #[cfg(feature = "plugin")]
    pub use crate::plugin::{LoadResult, LoadedPluginInfo, PluginManager};

    // Core builders
    pub use crate::core::prompt::PromptBuilder;
    pub use crate::core::resource::ResourceBuilder;
    pub use crate::core::tool::ToolBuilder;

    // Essential external types
    pub use async_trait::async_trait;
    pub use serde_json::{json, Value};
    pub use std::collections::HashMap;
}

// Testing utilities (only available in tests)
#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Basic smoke test to ensure all modules are accessible
        let _error = McpError::Protocol("test".to_string());
    }
}

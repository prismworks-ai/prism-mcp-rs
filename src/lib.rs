// Copyright (c) 2025 Prismworks AI Inc.
// SPDX-License-Identifier: MIT

//! # MCP Rust SDK (2025-06-18)
//!
//! A comprehensive Rust SDK for the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/)
//! version 2025-06-18, providing both server and client implementations with MCP specification
//! compliance including audio content, annotations, and improved capabilities.
// !
//! ## Features
//!
//! - ⚡ **High Performance**: Built with Rust's zero-cost abstractions and async/await
//! - 🛡️ **Type Safety**: Leverages Rust's type system to prevent runtime errors
//! - 🔌 **Multiple Transports**: Support for STDIO, HTTP/SSE, and WebSocket transports
//! - ✅ **MCP 2025-06-18 Compliance**: Comprehensive implementation of the latest MCP specification
//! - 🚀 **Rich Ecosystem**: Tools, resources, prompts, and sampling support
//! - 🎵 **Audio Support**: NEW in 2025-06-18 - Audio content support for multimodal interactions
//! - 🏷️ **Annotations**: NEW in 2025-06-18 - Tool and content annotations for improved metadata
//! - 💡 **Autocompletion**: NEW in 2025-06-18 - Argument autocompletion capabilities
//! - 📁 **Roots Support**: NEW in 2025-06-18 - File system roots for improved resource access
// !
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
//! ```rust,ignore
//! use prism_mcp_rs::prelude::*;
//! use std::collections::HashMap;
//! use serde_json::{json, Value};
//! use async_trait::async_trait;
//!
//! struct EchoHandler;
//!
//! #[async_trait]
//! impl ToolHandler for EchoHandler {
//!     async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
//!         let message = arguments.get("message")
//!             .and_then(|v| v.as_str())
//!             .unwrap_or("Hello, World!");
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
//!     let mut server = McpServer::new("echo-server".to_string(), "1.0.0".to_string());
//!
//!     server.add_tool(
//!         "echo".to_string(),
//!         Some("Echo a message".to_string()),
//!         json!({
//!             "type": "object",
//!             "properties": {
//!                 "message": { "type": "string" }
//!             }
//!         }),
//!         EchoHandler,
//!     ).await?;
//!
//!     #[cfg(feature = "stdio")]
//!     server.run_with_stdio().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Client Example
//!
//! ```rust,ignore
//! use prism_mcp_rs::prelude::*;
//! use std::collections::HashMap;
//! use serde_json::{json, Value};
//!
//! #[tokio::main]
//! async fn main() -> McpResult<()> {
//!     // Create a client
//!     let mut client = McpClient::new("my-client".to_string(), "1.0.0".to_string());
//!     
//!     // Set up transport (stdio feature required)
//!     #[cfg(feature = "stdio")]
//!     {
//!         use prism_mcp_rs::transport::StdioClientTransport;
//!         let transport = StdioClientTransport::new("server-command", vec!["arg1"]).await?;
//!         client.connect(transport).await?;
//!     }
//!     
//!     // List available tools  
//!     let tools = client.list_tools(None).await?;
//!     println!("Available tools: {:?}", tools);
//!     
//!     // Call a tool
//!     let mut args = HashMap::new();
//!     args.insert("message".to_string(), json!("Hello from client!"));
//!     
//!     let result = client.call_tool(
//!         "echo".to_string(),
//!         Some(args)
//!     ).await?;
//!     
//!     println!("Tool result: {:?}", result);
//!     Ok(())
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`core`]: Core abstractions for resources, tools, prompts, and errors
//! - plugin: Plugin system for dynamic tool loading
//! - [`protocol`]: MCP protocol types and message definitions (2025-06-18)
//! - [`transport`]: Transport layer implementations (STDIO, HTTP, WebSocket)
//! - [`server`]: MCP server implementation and lifecycle management
//! - [`client`]: MCP client implementation and session management
//! - [`utils`]: Utility functions and helpers

#[cfg(feature = "http")]
pub mod auth;
pub mod client;
pub mod core;
#[cfg(feature = "plugin")]
pub mod plugin;
pub mod protocol;
pub mod server;
pub mod transport;
pub mod utils;

// Re-export commonly used types for convenience
pub use core::error::{McpError, McpResult};
pub use protocol::types::*;
pub use protocol::{
    ErrorObject, JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, ServerCapabilities,
};

/// Prelude module for convenient imports (2025-06-18)
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
        CompositeCompletionHandler as completeCompositeCompletionHandler,
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

    // Plugin system
    #[cfg(feature = "plugin")]
    pub use crate::plugin::{LoadResult, LoadedPluginInfo, PluginManager};

    // Tool builder
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

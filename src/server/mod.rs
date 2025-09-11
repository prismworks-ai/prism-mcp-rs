//! MCP server implementation
//!
//! This module provides the complete server-side implementation of the Model Context Protocol.
//!
//! # Architecture
//!
//! The server module is organized as follows:
//! - [`McpServer`] - Main server struct handling protocol lifecycle
//! - [`ServerBuilder`] - Fluent API for server configuration
//! - `handlers` - Request/response handlers for each protocol method
//! - `lifecycle` - Server startup, shutdown, and state management
//! - `discovery_handler` - Protocol discovery and capability exchange
//!
//! # Usage Patterns
//!
//! ## Simple Server

//! ```no_run
//! use prism_mcp_rs::server::McpServer;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let server = McpServer::new("server".to_string(), "1.0.0".to_string());
//! // Use: server.start(transport).await?; then handle lifecycle manually
//! # Ok(())
//! # }
//! ```
//!
//! ## Using ServerBuilder

//! ```
//! use prism_mcp_rs::server::{ServerBuilder, ServerConfig};
//! use prism_mcp_rs::core::{Tool, Resource, Prompt};
//!
//! let server = ServerBuilder::new()
//!     .name("complex-server")
//!     .version("2.0.0")
//!     .with_tools()  // Enable tools capability
//!     .with_resources()  // Enable resources capability
//!     .with_prompts()  // Enable prompts capability
//!     .config(ServerConfig {
//!         validate_requests: true,
//!         enable_logging: false,
//!         max_concurrent_requests: 100,
//!         request_timeout_ms: 30000,
//!     })
//!     .build();
//! ```
//!
//! ## Custom Request Handling
//! ```no_run
//! use prism_mcp_rs::server::McpServer;
//! use prism_mcp_rs::protocol::{JsonRpcRequest, JsonRpcResponse};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let server = McpServer::new("server".to_string(), "1.0.0".to_string());
//!
//! // Handle requests programmatically
//! let request = JsonRpcRequest::new(
//!     "1".into(),
//!     "initialize".to_string(),
//!     None::<()>,
//! )?;
//!
//! // Process the request
//! // let response = server.handle_request(request).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - **Protocol Compliance**: Full MCP 2025-06-18 specification support
//! - **Capability Management**: Dynamic capability negotiation
//! - **Lifecycle Handling**: Proper initialization and shutdown
//! - **Error Handling**: Comprehensive error responses
//! - **Transport Agnostic**: Works with stdio, HTTP, WebSocket

pub mod async_methods;
pub mod builder;
pub mod discovery_handler;
pub mod handlers;
pub mod mcp_server;

// Test types for complete testing
#[cfg(test)]
pub mod test_types;

// HTTP-specific server implementation (when HTTP feature is enabled)
#[cfg(feature = "http")]
pub mod http_server;

// Re-export the main server type and builder
pub use builder::{ServerBuilder, ServerBuilderError};
pub use mcp_server::{McpServer, ServerConfig, ServerState};

// Re-export HTTP server when feature is enabled
#[cfg(feature = "http")]
pub use http_server::HttpMcpServer;

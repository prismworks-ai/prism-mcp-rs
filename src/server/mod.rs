//! MCP server implementation
//!
//! This module provides the main server implementation for the Model Context Protocol.

pub mod discovery_handler;
pub mod handlers;
pub mod lifecycle;
pub mod mcp_server;

// Test types for complete testing
#[cfg(test)]
pub mod test_types;

// HTTP-specific server implementation (when HTTP feature is enabled)
#[cfg(feature = "http")]
pub mod http_server;

// Re-export the main server type
pub use mcp_server::McpServer;

// Re-export HTTP server when feature is enabled
#[cfg(feature = "http")]
pub use http_server::HttpMcpServer;
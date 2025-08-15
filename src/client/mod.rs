//! MCP client implementation
//!
//! This module provides the main client implementation for the Model Context Protocol.

pub mod builder;
pub mod mcp_client;
pub mod request_handler;
pub mod session;

// Re-export the main client type and builder
pub use builder::{ConnectionConfig, McpClientBuilder, RetryConfig};
pub use mcp_client::{ClientConfig, McpClient, TransportInfo, TransportUseCase};
pub use request_handler::{
    AutomatedClientRequestHandler, ClientRequestHandler, DefaultClientRequestHandler,
    InteractiveClientRequestHandler,
};
pub use session::{ClientSession, SessionConfig, SessionState};

// Legacy alias for test compatibility
pub type ClientBuilder = McpClientBuilder;

//! MCP client implementation
//!
//! This module provides the main client implementation for the Model Context Protocol.
//!
//! # Architecture
//!
//! The client module consists of:
//! - [`McpClient`]: Main client struct for protocol communication
//! - [`McpClientBuilder`]: Fluent API for client configuration
//! - [`ClientSession`]: Session management and state tracking
//! - [`ClientRequestHandler`]: Interface for handling server requests
//!
//! # Usage Patterns
//!
//! ## Simple Client
//! ```no_run
//! use prism_mcp_rs::client::McpClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = McpClient::new("my-client".to_string(), "1.0.0".to_string());
//!
//! // Connect and initialize
//! // client.connect_stdio().await?;
//! // client.initialize().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Using ClientBuilder
//! ```
//! use prism_mcp_rs::client::{McpClientBuilder, ConnectionConfig, RetryConfig};
//! use std::time::Duration;
//!
//! let client = McpClientBuilder::new()
//!     .name("advanced-client")
//!     .version("2.0.0")
//!     .connection_config(ConnectionConfig {
//!         timeout: Duration::from_secs(30),
//!         keep_alive: true,
//!         keep_alive_interval: Duration::from_secs(60),
//!     })
//!     .retry_config(RetryConfig {
//!         max_retries: 3,
//!         initial_delay: Duration::from_millis(100),
//!         max_delay: Duration::from_secs(5),
//!         exponential_base: 2.0,
//!     })
//!     .build();
//! ```
//!
//! ## Request Handlers
//!
//! Clients can handle requests from servers using request handlers:
//!
//! ```
//! use prism_mcp_rs::client::{ClientRequestHandler, DefaultClientRequestHandler};
//! use prism_mcp_rs::protocol::{JsonRpcRequest, JsonRpcResponse};
//! use async_trait::async_trait;
//!
//! struct MyHandler;
//!
//! #[async_trait]
//! impl ClientRequestHandler for MyHandler {
//!     async fn handle_request(
//!         &self,
//!         request: JsonRpcRequest,
//!     ) -> Result<JsonRpcResponse, Box<dyn std::error::Error + Send + Sync>> {
//!         // Handle server requests
//!         Ok(JsonRpcResponse::success_unchecked(
//!             request.id.clone(),
//!             serde_json::json!({}),
//!         ))
//!     }
//! }
//! ```
//!
//! # Session Management
//!
//! The client maintains session state throughout the connection lifecycle:
//!
//! ```
//! use prism_mcp_rs::client::{ClientSession, SessionState};
//!
//! # async fn example(client: prism_mcp_rs::client::McpClient) -> Result<(), Box<dyn std::error::Error>> {
//! // Session is automatically managed
//! let session = client.session();
//! match session.state() {
//!     SessionState::Connected => println!("Connected"),
//!     SessionState::Initialized => println!("Ready"),
//!     SessionState::Disconnected => println!("Not connected"),
//!     _ => {}
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - **Auto-reconnection**: Configurable retry with exponential backoff
//! - **Request Handling**: Bidirectional communication support
//! - **Session Tracking**: Automatic state management
//! - **Multiple Transports**: STDIO, HTTP, WebSocket support
//! - **Type Safety**: Strongly typed requests and responses

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
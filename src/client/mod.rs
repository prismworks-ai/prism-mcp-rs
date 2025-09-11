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
//! // EXAMPLE_END
//!
//! ## Using ClientBuilder

//! ```
//! use prism_mcp_rs::client::{McpClientBuilder, ConnectionConfig, RetryConfig};
//! use std::time::Duration;
//!
//! let client = McpClientBuilder::new()
//!     // .name() is not available
//!     .with_version("2.0.0")
//!     .with_connection_config(ConnectionConfig {
//!         timeout_ms: 30000,
//!         keep_alive: true,
//!         compression: false,
//!     })
//!     .with_retry_config(RetryConfig {
//!         max_attempts: Some(3),
//!         initial_delay_ms: 100,
//!         max_delay_ms: 5000,
//!         backoff_multiplier: 2.0,
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
//! use prism_mcp_rs::core::error::{McpError, McpResult};
//! use prism_mcp_rs::protocol::messages::{CreateMessageParams, ListRootsParams, ListRootsResult, ElicitParams, ElicitResult, PingParams, PingResult};
//! use prism_mcp_rs::protocol::types::{CreateMessageResult, Role, SamplingContent, StopReason, ElicitationAction};
//! use prism_mcp_rs::protocol::{JsonRpcRequest, JsonRpcResponse};
//! use async_trait::async_trait;
//! use serde_json;
//! use std::collections::HashMap;
//!
//! struct MyHandler;
//!
//! #[async_trait]
//! impl ClientRequestHandler for MyHandler {
//!     async fn handle_create_message(
//!         &self,
//!         params: CreateMessageParams,
//!     ) -> McpResult<CreateMessageResult> {
//!         // Handle server's request to create a message
//!         Ok(CreateMessageResult {
//!             model: "test-model".to_string(),
//!             stop_reason: Some(StopReason::EndTurn),
//!             role: Role::Assistant,
//!             content: SamplingContent::Text {
//!                 text: "Hello!".to_string(),
//!                 annotations: None,
//!                 meta: None,
//!             },
//!             meta: None,
//!         })
//!     }
//!
//!     async fn handle_list_roots(&self, _params: ListRootsParams) -> McpResult<ListRootsResult> {
//!         Ok(ListRootsResult {
//!             roots: vec![],
//!             meta: None,
//!         })
//!     }
//!
//!     async fn handle_elicit(&self, _params: ElicitParams) -> McpResult<ElicitResult> {
//!         Ok(ElicitResult {
//!             action: ElicitationAction::Accept,
//!             content: Some(HashMap::new()),
//!             meta: None,
//!         })
//!     }
//!
//!     async fn handle_ping(&self, _params: PingParams) -> McpResult<PingResult> {
//!         Ok(PingResult {
//!             meta: None,
//!         })
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
//! // Session state is managed internally
//! // You can check connection status after connecting
//! // let connected = client.is_connected();
//! println!("Client configured and ready to connect");
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

pub mod enhanced_builder;
pub mod enhanced_traits;
pub mod fluent_interfaces;
pub mod fluent_tools;
pub mod mcp_client;
pub mod request_handler;
pub mod session;

// Re-export enhanced APIs
pub use enhanced_builder::{ConnectionConfig, McpClientBuilder, RetryConfig};
pub use mcp_client::{ClientConfig, McpClient, TransportInfo, TransportUseCase};

// Legacy re-exports for backward compatibility

pub use request_handler::{
    AutomatedClientRequestHandler, ClientRequestHandler, DefaultClientRequestHandler,
    InteractiveClientRequestHandler,
};
pub use session::{ClientSession, SessionConfig, SessionState};

// Legacy alias for test compatibility
pub type ClientBuilder = McpClientBuilder;

//! Transport layer implementations
//!
//! This module provides concrete implementations of the transport traits
//! for different communication protocols including STDIO, HTTP, and WebSocket.
//!
//! # When to Use Transport Directly
//!
//! Most users should use the high-level server/client APIs (`McpServer`, `McpClient`)
//! which handle transport details automatically. Use Transport directly only when:
//!
//! - **Custom Transport**: Implementing new transport mechanisms
//! - **Transport Middleware**: Adding logging, metrics, or transformations
//! - **Fine-Grained Control**: Managing connection lifecycle manually
//! - **Protocol Bridging**: Connecting different transport types
//!
//! # Available Transports
//!
//! ## STDIO Transport (Default)

//! ```no_run
//! # #[cfg(feature = "stdio")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use prism_mcp_rs::transport::StdioServerTransport;
//! use prism_mcp_rs::server::McpServer;
//!
//! let transport = StdioServerTransport::new();
//! let server = McpServer::new("server".to_string(), "1.0.0".to_string());
//! // server.run_with_transport(transport).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## HTTP Transport

//! ```no_run
//! # #[cfg(feature = "http")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use prism_mcp_rs::transport::{HttpServerTransport, HttpEndpoints};
//!
//! let endpoints = HttpEndpoints::new("http://localhost:8080");
//! let transport = HttpServerTransport::new(endpoints)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## WebSocket Transport

//! ```no_run
//! # #[cfg(feature = "websocket")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use prism_mcp_rs::transport::WebSocketServerTransport;
//!
//! let transport = WebSocketServerTransport::bind("127.0.0.1:9000").await?;
//! # Ok(())
//! # }
//! ```

//!
//! # Transport Traits
//!
//! The transport layer is built on several traits:
//!
//! - [`Transport`]: Core bidirectional message passing
//! - [`ServerTransport`]: Server-specific transport operations
//! - [`ReconnectableTransport`]: Auto-reconnection support
//! - [`EventEmittingTransport`]: Event notification system
//! - [`FilterableTransport`]: Message filtering and transformation
//!
//! # Advanced Features
//!
//! - **Authentication**: HTTP transport supports various auth mechanisms
//! - **Compression**: Streaming HTTP supports gzip/deflate
//! - **Reconnection**: Automatic reconnection with backoff
//! - **Metrics**: Performance monitoring and statistics
//! - **HTTP/2**: Multiplexed streaming support
//!
//! # Implementation Note
//!
//! The Transport trait is primarily an internal abstraction. Unless you're
//! implementing custom transports or need very specific control, use the
//! high-level APIs provided by `McpServer` and `McpClient`.

pub mod traits;

#[cfg(feature = "stdio")]
pub mod stdio;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "http")]
pub mod http_auth;

#[cfg(feature = "websocket")]
pub mod websocket;

// Advanced HTTP transport features (chunking, compression, HTTP/2)
#[cfg(any(
    feature = "chunked-encoding",
    feature = "compression",
    feature = "http2"
))]
pub mod streaming_http;

// Re-export commonly used types
pub use traits::{
    ConnectionState, EventEmittingTransport, FilterableTransport, ReconnectConfig,
    ReconnectableTransport, ServerTransport, Transport, TransportConfig, TransportEvent,
    TransportStats,
};

// Re-export transport implementations when features are enabled
#[cfg(feature = "stdio")]
pub use stdio::{StdioClientTransport, StdioServerTransport};

#[cfg(feature = "http")]
pub use http::{HttpClientTransport, HttpServerTransport};

#[cfg(feature = "http")]
pub use http_auth::{AuthorizedHttpTransport, AuthorizedHttpTransportBuilder};

#[cfg(feature = "http")]
pub mod http_convenience;

#[cfg(feature = "http")]
pub use http_convenience::{
    ConnectionStats, ErrorMetrics, HttpClientTransportBuilder, HttpEndpoints, PerformanceMetrics,
    RetryConfig, RetryPolicy, ServerInfo, TransportMetrics,
};

#[cfg(all(feature = "http", test))]
mod http_convenience_test;

#[cfg(feature = "websocket")]
pub use websocket::{WebSocketClientTransport, WebSocketServerTransport};

// Chunked encoding and streaming features
#[cfg(feature = "chunked-encoding")]
pub use streaming_http::{
    ContentAnalyzer, ContentType, StreamingAnalysis, StreamingConfig, StreamingHttpClientTransport,
    StreamingStrategy,
};

// Compression features
#[cfg(feature = "compression")]
pub use streaming_http::CompressionType;

// HTTP/2 specific re-exports
#[cfg(feature = "http2")]
pub use streaming_http::{Http2Config, Http2StreamManager, PushPromise, StreamInfo, StreamState};

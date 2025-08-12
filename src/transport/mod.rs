//! Transport layer implementations
//!
//! This module provides concrete implementations of the transport traits
//! for different communication protocols including STDIO, HTTP, and WebSocket.

pub mod traits;

#[cfg(feature = "stdio")]
pub mod stdio;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "http")]
pub mod http_auth;

#[cfg(feature = "websocket")]
pub mod websocket;

#[cfg(feature = "streaming-http")]
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

#[cfg(feature = "streaming-http")]
pub use streaming_http::{
    CompressionType, ContentAnalyzer, ContentType, StreamingAnalysis, StreamingConfig,
    StreamingHttpClientTransport, StreamingStrategy,
};

// HTTP/2 specific re-exports
#[cfg(feature = "streaming-http2")]
pub use streaming_http::{Http2Config, Http2StreamManager, PushPromise, StreamInfo, StreamState};

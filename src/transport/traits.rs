// ! Transport layer traits and abstractions
// !
// ! Module defines the core transport traits that enable MCP communication
// ! over different protocols like STDIO, HTTP, and WebSocket.

use crate::core::error::McpResult;
use crate::protocol::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use async_trait::async_trait;

/// Transport trait for MCP clients
///
/// Trait defines the interface for sending requests and receiving responses
/// in a client-side MCP connection.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON-RPC request and wait for a response
    ///
    /// # Arguments
    /// * `request` - The JSON-RPC request to send
    ///
    /// # Returns
    /// Result containing the JSON-RPC response or an error
    async fn send_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse>;

    /// Send a JSON-RPC notification (no response expected)
    ///
    /// # Arguments
    /// * `notification` - The JSON-RPC notification to send
    ///
    /// # Returns
    /// Result indicating success or an error
    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()>;

    /// Receive a notification from the server (non-blocking)
    ///
    /// # Returns
    /// Result containing an optional notification or an error
    async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>>;

    /// Handle incoming server request (for bidirectional communication)
    ///
    /// Method is called to check for incoming requests from the server.
    /// It should be non-blocking and return None if no request is available.
    ///
    /// # Returns
    /// Result containing an optional server request or an error
    async fn handle_incoming_request(&mut self) -> McpResult<Option<JsonRpcRequest>> {
        // Default implementation - no bidirectional support
        Ok(None)
    }

    /// Send response to server request (for bidirectional communication)
    ///
    /// Method sends a response back to the server for a server-initiated request.
    ///
    /// # Arguments
    /// * `response` - The JSON-RPC response to send
    ///
    /// # Returns
    /// Result indicating success or an error
    async fn send_response(&mut self, _response: JsonRpcResponse) -> McpResult<()> {
        // Default implementation - no bidirectional support
        Err(crate::core::error::McpError::MethodNotFound(
            "Bidirectional communication not supported by this transport".to_string(),
        ))
    }

    /// Close the transport connection
    ///
    /// # Returns
    /// Result indicating success or an error
    async fn close(&mut self) -> McpResult<()>;

    /// Check if the transport is connected
    ///
    /// # Returns
    /// True if the transport is connected and ready for communication
    fn is_connected(&self) -> bool {
        true // Default implementation - assume connected
    }

    /// Get connection information for debugging
    ///
    /// # Returns
    /// String describing the connection
    fn connection_info(&self) -> String {
        "Unknown transport".to_string()
    }
}

/// Server request handler function type
pub type ServerRequestHandler = std::sync::Arc<
    dyn Fn(
            JsonRpcRequest,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = McpResult<JsonRpcResponse>> + Send + 'static>,
        > + Send
        + Sync,
>;

/// Transport trait for MCP servers
///
/// Trait defines the interface for handling incoming requests and
/// sending responses in a server-side MCP connection.
#[async_trait]
pub trait ServerTransport: Send + Sync {
    /// Start the server transport and begin listening for connections
    ///
    /// # Returns
    /// Result indicating success or an error
    async fn start(&mut self) -> McpResult<()>;

    /// Set the request handler that will process incoming requests
    ///
    /// # Arguments
    /// * `handler` - The request handler function
    fn set_request_handler(&mut self, handler: ServerRequestHandler);

    /// Send a JSON-RPC notification to the client
    ///
    /// # Arguments
    /// * `notification` - The JSON-RPC notification to send
    ///
    /// # Returns
    /// Result indicating success or an error
    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()>;

    /// Stop the server transport
    ///
    /// # Returns
    /// Result indicating success or an error
    async fn stop(&mut self) -> McpResult<()>;

    /// Check if the server is running
    ///
    /// # Returns
    /// True if the server is running and accepting connections
    fn is_running(&self) -> bool {
        true // Default implementation - assume running
    }

    /// Get server information for debugging
    ///
    /// # Returns
    /// String describing the server state
    fn server_info(&self) -> String {
        "Unknown server transport".to_string()
    }
}

/// Transport configuration options
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: Option<u64>,
    /// Read timeout in milliseconds
    pub read_timeout_ms: Option<u64>,
    /// Write timeout in milliseconds
    pub write_timeout_ms: Option<u64>,
    /// Maximum message size in bytes
    pub max_message_size: Option<usize>,
    /// Keep-alive interval in milliseconds
    pub keep_alive_ms: Option<u64>,
    /// Whether to enable compression
    pub compression: bool,
    /// Custom headers for HTTP-based transports
    pub headers: std::collections::HashMap<String, String>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: Some(30_000),         // 30 seconds
            read_timeout_ms: Some(60_000),            // 60 seconds
            write_timeout_ms: Some(30_000),           // 30 seconds
            max_message_size: Some(16 * 1024 * 1024), // 16 MB
            keep_alive_ms: Some(30_000),              // 30 seconds
            compression: false,
            headers: std::collections::HashMap::new(),
        }
    }
}

/// Connection state for transports
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Transport is disconnected
    Disconnected,
    /// Transport is connecting
    Connecting,
    /// Transport is connected and ready
    Connected,
    /// Transport is reconnecting after an error
    Reconnecting,
    /// Transport is closing
    Closing,
    /// Transport has encountered an error
    Error(String),
}

/// Transport statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct TransportStats {
    /// Number of requests sent
    pub requests_sent: u64,
    /// Number of responses received
    pub responses_received: u64,
    /// Number of notifications sent
    pub notifications_sent: u64,
    /// Number of notifications received
    pub notifications_received: u64,
    /// Number of connection errors
    pub connection_errors: u64,
    /// Number of protocol errors
    pub protocol_errors: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Connection uptime in milliseconds
    pub uptime_ms: u64,
}

/// Trait for transports that support statistics
pub trait TransportStats_: Send + Sync {
    /// Get current transport statistics
    fn stats(&self) -> TransportStats;

    /// Reset transport statistics
    fn reset_stats(&mut self);
}

/// Trait for transports that support reconnection
#[async_trait]
pub trait ReconnectableTransport: Transport {
    /// Attempt to reconnect the transport
    ///
    /// # Returns
    /// Result indicating success or an error
    async fn reconnect(&mut self) -> McpResult<()>;

    /// Set the reconnection configuration
    ///
    /// # Arguments
    /// * `config` - Reconnection configuration
    fn set_reconnect_config(&mut self, config: ReconnectConfig);

    /// Get the current connection state
    fn connection_state(&self) -> ConnectionState;
}

/// Configuration for automatic reconnection
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Whether automatic reconnection is enabled
    pub enabled: bool,
    /// Maximum number of reconnection attempts
    pub max_attempts: Option<u32>,
    /// Initial delay before first reconnection attempt (milliseconds)
    pub initial_delay_ms: u64,
    /// Maximum delay between reconnection attempts (milliseconds)
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Jitter factor for randomizing delays (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: Some(5),
            initial_delay_ms: 1000, // 1 second
            max_delay_ms: 30_000,   // 30 seconds
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Trait for transports that support message filtering
pub trait FilterableTransport: Send + Sync {
    /// Set a message filter function
    ///
    /// # Arguments
    /// * `filter` - Function that returns true if message should be processed
    fn set_message_filter(&mut self, filter: Box<dyn Fn(&JsonRpcRequest) -> bool + Send + Sync>);

    /// Clear the message filter
    fn clear_message_filter(&mut self);
}

/// Transport event for monitoring and debugging
#[derive(Debug, Clone)]
pub enum TransportEvent {
    /// Connection established
    Connected,
    /// Connection lost
    Disconnected,
    /// Message sent
    MessageSent {
        /// Message type
        message_type: String,
        /// Message size in bytes
        size: usize,
    },
    /// Message received
    MessageReceived {
        /// Message type
        message_type: String,
        /// Message size in bytes
        size: usize,
    },
    /// Error occurred
    Error {
        /// Error message
        message: String,
    },
}

/// Trait for transports that support event listeners
pub trait EventEmittingTransport: Send + Sync {
    /// Add an event listener
    ///
    /// # Arguments
    /// * `listener` - Event listener function
    fn add_event_listener(&mut self, listener: Box<dyn Fn(TransportEvent) + Send + Sync>);

    /// Remove all event listeners
    fn clear_event_listeners(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.connect_timeout_ms, Some(30_000));
        assert_eq!(config.read_timeout_ms, Some(60_000));
        assert_eq!(config.max_message_size, Some(16 * 1024 * 1024));
        assert!(!config.compression);
    }

    #[test]
    fn test_reconnect_config_default() {
        let config = ReconnectConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_attempts, Some(5));
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30_000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.jitter_factor, 0.1);
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);

        let error1 = ConnectionState::Error("test".to_string());
        let error2 = ConnectionState::Error("test".to_string());
        let error3 = ConnectionState::Error("other".to_string());
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_transport_stats_default() {
        let stats = TransportStats::default();
        assert_eq!(stats.requests_sent, 0);
        assert_eq!(stats.responses_received, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
    }
}

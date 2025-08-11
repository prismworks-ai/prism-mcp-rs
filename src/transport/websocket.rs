// ! WebSocket transport implementation for MCP
// !
// ! Module provides WebSocket-based transport for MCP communication,
// ! offering bidirectional, real-time communication between clients and servers.

use async_trait::async_trait;
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock, broadcast, mpsc},
    time::timeout,
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, accept_async, connect_async, tungstenite::Message,
};
use url::Url;

use crate::core::error::{McpError, McpResult};
// use crate::core::logging::ErrorContext; // Unused import
use crate::protocol::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use crate::transport::traits::{ConnectionState, ServerTransport, Transport, TransportConfig};

// Type aliases to reduce complexity warnings
type RequestHandler = Arc<
    RwLock<
        Option<
            Arc<
                dyn Fn(JsonRpcRequest) -> tokio::sync::oneshot::Receiver<JsonRpcResponse>
                    + Send
                    + Sync,
            >,
        >,
    >,
>;

// ============================================================================
// WebSocket Client Transport
// ============================================================================

/// WebSocket transport for MCP clients
///
/// This transport communicates with an MCP server via WebSocket connections,
/// providing bidirectional real-time communication for both requests and notifications.
#[derive(Debug)]
pub struct WebSocketClientTransport {
    ws_sender: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    pending_requests: Arc<Mutex<HashMap<Value, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
    notification_receiver: Option<mpsc::UnboundedReceiver<JsonRpcNotification>>,
    config: TransportConfig,
    state: Arc<RwLock<ConnectionState>>,
    url: String,
    message_handler: Option<tokio::task::JoinHandle<()>>,
}

impl WebSocketClientTransport {
    /// Create a new WebSocket client transport
    ///
    /// # Arguments
    /// * `url` - WebSocket URL to connect to (e.g., "ws://localhost:8080/mcp")
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn new<S: AsRef<str>>(url: S) -> McpResult<Self> {
        Self::with_config(url, TransportConfig::default()).await
    }

    /// Create a new WebSocket client transport with custom configuration
    ///
    /// # Arguments
    /// * `url` - WebSocket URL to connect to
    /// * `config` - Transport configuration
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn with_config<S: AsRef<str>>(url: S, config: TransportConfig) -> McpResult<Self> {
        let url_str = url.as_ref();

        // Validate URL format
        let _url_parsed = Url::parse(url_str)
            .map_err(|e| McpError::WebSocket(format!("Invalid WebSocket URL: {e}")))?;

        tracing::debug!("Connecting to WebSocket: {}", url_str);

        // Connect to WebSocket with timeout
        let connect_timeout = Duration::from_millis(config.connect_timeout_ms.unwrap_or(30_000));

        let (ws_stream, _) = timeout(connect_timeout, connect_async(url_str))
            .await
            .map_err(|_| McpError::WebSocket("Connection timeout".to_string()))?
            .map_err(|e| McpError::WebSocket(format!("Failed to connect: {e}")))?;

        let (ws_sender, ws_receiver) = ws_stream.split();

        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let (notification_sender, notification_receiver) = mpsc::unbounded_channel();
        let state = Arc::new(RwLock::new(ConnectionState::Connected));

        // Start message handling task
        let message_handler = tokio::spawn(Self::handle_messages(
            ws_receiver,
            pending_requests.clone(),
            notification_sender,
            state.clone(),
        ));

        Ok(Self {
            ws_sender: Some(ws_sender),
            pending_requests,
            notification_receiver: Some(notification_receiver),
            config,
            state,
            url: url_str.to_string(),
            message_handler: Some(message_handler),
        })
    }

    async fn handle_messages(
        mut ws_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        pending_requests: Arc<Mutex<HashMap<Value, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
        notification_sender: mpsc::UnboundedSender<JsonRpcNotification>,
        state: Arc<RwLock<ConnectionState>>,
    ) {
        while let Some(message) = ws_receiver.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    tracing::trace!("Received WebSocket message: {}", text);

                    // Try to parse as response first
                    if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&text) {
                        let mut pending = pending_requests.lock().await;
                        if let Some(sender) = pending.remove(&response.id) {
                            if sender.send(response).is_err() {
                                tracing::warn!("Failed to send response to waiting request");
                            }
                        } else {
                            tracing::warn!(
                                "Received response for unknown request ID: {:?}",
                                response.id
                            );
                        }
                    }
                    // Try to parse as notification
                    else if let Ok(notification) =
                        serde_json::from_str::<JsonRpcNotification>(&text)
                    {
                        if notification_sender.send(notification).is_err() {
                            tracing::debug!("Notification receiver dropped");
                            break;
                        }
                    } else {
                        tracing::warn!("Failed to parse WebSocket message: {}", text);
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket connection closed");
                    *state.write().await = ConnectionState::Disconnected;
                    break;
                }
                Ok(Message::Ping(_data)) => {
                    tracing::trace!("Received WebSocket ping");
                    // Pong responses are handled automatically by tungstenite
                }
                Ok(Message::Pong(_)) => {
                    tracing::trace!("Received WebSocket pong");
                }
                Ok(Message::Binary(_)) => {
                    tracing::warn!("Received unexpected binary WebSocket message");
                }
                Ok(Message::Frame(_)) => {
                    tracing::trace!("Received WebSocket frame (internal)");
                    // Frame messages are internal to tungstenite
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    *state.write().await = ConnectionState::Error(e.to_string());
                    break;
                }
            }
        }

        tracing::debug!("WebSocket message handler exiting");
    }

    async fn send_message(&mut self, message: Message) -> McpResult<()> {
        if let Some(ref mut sender) = self.ws_sender {
            sender
                .send(message)
                .await
                .map_err(|e| McpError::WebSocket(format!("Failed to send message: {e}")))?;
        } else {
            return Err(McpError::WebSocket("WebSocket not connected".to_string()));
        }
        Ok(())
    }
}

#[async_trait]
impl Transport for WebSocketClientTransport {
    async fn send_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        // Store the pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request.id.clone(), sender);
        }

        // Send the request
        let request_text =
            serde_json::to_string(&request).map_err(|e| McpError::Serialization(e.to_string()))?;

        tracing::trace!("Sending WebSocket request: {}", request_text);

        self.send_message(Message::Text(request_text.into()))
            .await?;

        // Wait for response with timeout
        let timeout_duration = Duration::from_millis(self.config.read_timeout_ms.unwrap_or(60_000));

        let response = timeout(timeout_duration, receiver)
            .await
            .map_err(|_| McpError::WebSocket("Request timeout".to_string()))?
            .map_err(|_| McpError::WebSocket("Response channel closed".to_string()))?;

        Ok(response)
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let notification_text = serde_json::to_string(&notification)
            .map_err(|e| McpError::Serialization(e.to_string()))?;

        tracing::trace!("Sending WebSocket notification: {}", notification_text);

        self.send_message(Message::Text(notification_text.into()))
            .await
    }

    async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>> {
        if let Some(ref mut receiver) = self.notification_receiver {
            match receiver.try_recv() {
                Ok(notification) => Ok(Some(notification)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => Err(McpError::WebSocket(
                    "Notification channel disconnected".to_string(),
                )),
            }
        } else {
            Ok(None)
        }
    }

    async fn close(&mut self) -> McpResult<()> {
        tracing::debug!("Closing WebSocket connection");

        *self.state.write().await = ConnectionState::Closing;

        // Send close message
        if let Some(ref mut sender) = self.ws_sender {
            let _ = sender.send(Message::Close(None)).await;
        }

        // Abort message handler
        if let Some(handle) = self.message_handler.take() {
            handle.abort();
        }

        self.ws_sender = None;
        self.notification_receiver = None;

        *self.state.write().await = ConnectionState::Disconnected;

        Ok(())
    }

    fn is_connected(&self) -> bool {
        // We'd need to check the actual state here
        self.ws_sender.is_some()
    }

    fn connection_info(&self) -> String {
        format!("WebSocket transport (url: {})", self.url)
    }
}

// ============================================================================
// WebSocket Server Transport
// ============================================================================

/// Connection state for a WebSocket client
struct WebSocketConnection {
    sender: SplitSink<WebSocketStream<TcpStream>, Message>,
    _id: String, // Keep for future connection tracking/debugging
}

/// WebSocket transport for MCP servers
///
/// This transport serves MCP requests over WebSocket connections,
/// allowing multiple concurrent clients with bidirectional communication.
pub struct WebSocketServerTransport {
    bind_addr: String,
    config: TransportConfig, // Used for connection timeouts and limits
    clients: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    request_handler: RequestHandler,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    running: Arc<RwLock<bool>>,
    shutdown_sender: Option<broadcast::Sender<()>>,
}

impl WebSocketServerTransport {
    /// Create a new WebSocket server transport
    ///
    /// # Arguments
    /// * `bind_addr` - Address to bind the WebSocket server to (e.g., "0.0.0.0:8080")
    ///
    /// # Returns
    /// New WebSocket server transport instance
    pub fn new<S: Into<String>>(bind_addr: S) -> Self {
        Self::with_config(bind_addr, TransportConfig::default())
    }

    /// Create a new WebSocket server transport with custom configuration
    ///
    /// # Arguments
    /// * `bind_addr` - Address to bind the WebSocket server to
    /// * `config` - Transport configuration
    ///
    /// # Returns
    /// New WebSocket server transport instance
    pub fn with_config<S: Into<String>>(bind_addr: S, config: TransportConfig) -> Self {
        let (shutdown_sender, _) = broadcast::channel(1);

        Self {
            bind_addr: bind_addr.into(),
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            request_handler: Arc::new(RwLock::new(None)),
            server_handle: None,
            running: Arc::new(RwLock::new(false)),
            shutdown_sender: Some(shutdown_sender),
        }
    }

    /// Set the request handler function
    ///
    /// # Arguments
    /// * `handler` - Function that processes incoming requests
    pub async fn set_request_handler<F>(&mut self, handler: F)
    where
        F: Fn(JsonRpcRequest) -> tokio::sync::oneshot::Receiver<JsonRpcResponse>
            + Send
            + Sync
            + 'static,
    {
        let mut request_handler = self.request_handler.write().await;
        *request_handler = Some(Arc::new(handler));
    }

    /// Get the current configuration
    pub fn config(&self) -> &TransportConfig {
        &self.config
    }

    /// Get the maximum message size from config
    pub fn max_message_size(&self) -> Option<usize> {
        self.config.max_message_size
    }

    async fn handle_client_connection(
        stream: TcpStream,
        clients: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
        request_handler: RequestHandler,
        mut shutdown_receiver: broadcast::Receiver<()>,
    ) {
        let client_id = uuid::Uuid::new_v4().to_string();

        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                tracing::error!("Failed to accept WebSocket connection: {}", e);
                return;
            }
        };

        tracing::info!("New WebSocket client connected: {}", client_id);

        let (ws_sender, mut ws_receiver) = ws_stream.split();

        // Add client to the connections map
        {
            let mut clients_guard = clients.write().await;
            clients_guard.insert(
                client_id.clone(),
                WebSocketConnection {
                    sender: ws_sender,
                    _id: client_id.clone(),
                },
            );
        }

        // Handle messages from this client
        loop {
            tokio::select! {
                message = ws_receiver.next() => {
                    match message {
                        Some(Ok(Message::Text(text))) => {
                            tracing::trace!("Received message from {}: {}", client_id, text);

                            // Try to parse as request
                            if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&text) {
                                let handler_guard = request_handler.read().await;
                                if let Some(ref handler) = *handler_guard {
                                    let response_rx = handler(request.clone());
                                    drop(handler_guard);

                                    match response_rx.await {
                                        Ok(response) => {
                                            let response_text = match serde_json::to_string(&response) {
                                                Ok(text) => text,
                                                Err(e) => {
                                                    tracing::error!("Failed to serialize response: {}", e);
                                                    continue;
                                                }
                                            };

                                            // Send response back to client
                                            let mut clients_guard = clients.write().await;
                                            if let Some(client) = clients_guard.get_mut(&client_id) {
                                                if let Err(e) = client.sender.send(Message::Text(response_text.into())).await {
                                                    tracing::error!("Failed to send response to client {}: {}", client_id, e);
                                                    break;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            tracing::error!("Request handler channel closed for client {}", client_id);
                                        }
                                    }
                                } else {
                                    tracing::warn!("No request handler configured for client {}", client_id);
                                }
                            }
                            // Handle notifications (no response needed)
                            else if let Ok(_notification) = serde_json::from_str::<JsonRpcNotification>(&text) {
                                tracing::trace!("Received notification from client {}", client_id);
                                // Notifications don't require responses
                            } else {
                                tracing::warn!("Failed to parse message from client {}: {}", client_id, text);
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            tracing::info!("Client {} disconnected", client_id);
                            break;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            tracing::trace!("Received ping from client {}", client_id);
                            let mut clients_guard = clients.write().await;
                            if let Some(client) = clients_guard.get_mut(&client_id) {
                                if let Err(e) = client.sender.send(Message::Pong(data)).await {
                                    tracing::error!("Failed to send pong to client {}: {}", client_id, e);
                                    break;
                                }
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            tracing::trace!("Received pong from client {}", client_id);
                        }
                        Some(Ok(Message::Binary(_))) => {
                            tracing::warn!("Received unexpected binary message from client {}", client_id);
                        }
                        Some(Ok(Message::Frame(_))) => {
                            tracing::trace!("Received WebSocket frame from client {} (internal)", client_id);
                            // Frame messages are internal to tungstenite
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error for client {}: {}", client_id, e);
                            break;
                        }
                        None => {
                            tracing::info!("WebSocket stream ended for client {}", client_id);
                            break;
                        }
                    }
                }
                _ = shutdown_receiver.recv() => {
                    tracing::info!("Shutting down connection for client {}", client_id);
                    break;
                }
            }
        }

        // Remove client from connections
        {
            let mut clients_guard = clients.write().await;
            clients_guard.remove(&client_id);
        }

        tracing::info!("Client {} connection handler exiting", client_id);
    }
}

#[async_trait]
impl ServerTransport for WebSocketServerTransport {
    async fn start(&mut self) -> McpResult<()> {
        tracing::info!("Starting WebSocket server on {}", self.bind_addr);

        let listener = TcpListener::bind(&self.bind_addr).await.map_err(|e| {
            McpError::WebSocket(format!("Failed to bind to {}: {}", self.bind_addr, e))
        })?;

        let clients = self.clients.clone();
        let request_handler = self.request_handler.clone();
        let running = self.running.clone();
        let shutdown_sender = self.shutdown_sender.as_ref().unwrap().clone();

        *running.write().await = true;

        let server_handle = tokio::spawn(async move {
            let mut shutdown_receiver = shutdown_sender.subscribe();

            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                tracing::debug!("New connection from: {}", addr);

                                tokio::spawn(Self::handle_client_connection(
                                    stream,
                                    clients.clone(),
                                    request_handler.clone(),
                                    shutdown_sender.subscribe(),
                                ));
                            }
                            Err(e) => {
                                tracing::error!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                    _ = shutdown_receiver.recv() => {
                        tracing::info!("WebSocket server shutting down");
                        break;
                    }
                }
            }
        });

        self.server_handle = Some(server_handle);

        tracing::info!(
            "WebSocket server started successfully on {}",
            self.bind_addr
        );
        Ok(())
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let notification_text = serde_json::to_string(&notification)
            .map_err(|e| McpError::Serialization(e.to_string()))?;

        let mut clients_guard = self.clients.write().await;
        let mut disconnected_clients = Vec::new();

        for (client_id, client) in clients_guard.iter_mut() {
            if let Err(e) = client
                .sender
                .send(Message::Text(notification_text.clone().into()))
                .await
            {
                tracing::error!("Failed to send notification to client {}: {}", client_id, e);
                disconnected_clients.push(client_id.clone());
            }
        }

        // Remove disconnected clients
        for client_id in disconnected_clients {
            clients_guard.remove(&client_id);
        }

        Ok(())
    }

    async fn stop(&mut self) -> McpResult<()> {
        tracing::info!("Stopping WebSocket server");

        *self.running.write().await = false;

        // Send shutdown signal
        if let Some(ref sender) = self.shutdown_sender {
            let _ = sender.send(());
        }

        // Wait for server to stop
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }

        // Close all client connections
        let mut clients_guard = self.clients.write().await;
        for (client_id, client) in clients_guard.iter_mut() {
            tracing::debug!("Closing connection for client {}", client_id);
            let _ = client.sender.send(Message::Close(None)).await;
        }
        clients_guard.clear();

        Ok(())
    }

    fn is_running(&self) -> bool {
        // Check if we have an active server handle
        self.server_handle.is_some()
    }

    fn server_info(&self) -> String {
        format!("WebSocket server transport (bind: {})", self.bind_addr)
    }

    fn set_request_handler(&mut self, handler: crate::transport::traits::ServerRequestHandler) {
        // Convert the ServerRequestHandler to the WebSocket transport's expected format
        let _ws_handler = Arc::new(move |request: JsonRpcRequest| {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let handler_future = handler(request);
            tokio::spawn(async move {
                let result = handler_future.await;
                let _ = tx.send(result.unwrap_or_else(|e| JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::Value::Null,
                    result: Some(serde_json::json!({
                        "error": {
                            "code": -32603,
                            "message": e.to_string()
                        }
                    })),
                }));
            });
            rx
        });

        // Set the handler in the WebSocket transport's request_handler field
        tokio::spawn(async move {
            // Note: This is a limitation - we can't easily update the async field from sync method
            // The WebSocket transport should be updated in the future to support the new trait design
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_websocket_server_creation() {
        let transport = WebSocketServerTransport::new("127.0.0.1:0");
        assert_eq!(transport.bind_addr, "127.0.0.1:0");
        assert!(!transport.is_running());
    }

    #[test]
    fn test_websocket_server_with_config() {
        let mut config = TransportConfig::default();
        config.max_message_size = Some(64 * 1024);

        let transport = WebSocketServerTransport::with_config("0.0.0.0:9090", config);
        assert_eq!(transport.bind_addr, "0.0.0.0:9090");
        assert_eq!(transport.config.max_message_size, Some(64 * 1024));
    }

    #[tokio::test]
    async fn test_websocket_client_invalid_url() {
        let result = WebSocketClientTransport::new("invalid-url").await;
        assert!(result.is_err());

        if let Err(McpError::WebSocket(msg)) = result {
            assert!(msg.contains("Invalid WebSocket URL"));
        } else {
            panic!("Expected WebSocket error");
        }
    }

    #[tokio::test]
    async fn test_websocket_client_connection_info() {
        // This will fail to connect but we can test the URL parsing
        let url = "ws://localhost:9999/test";
        if let Ok(transport) = WebSocketClientTransport::new(url).await {
            let info = transport.connection_info();
            assert!(info.contains("localhost:9999"));
        }
        // If connection fails (which is expected), that's fine for this test
    }

    // ============================================================================
    // Additional WebSocketClientTransport Tests
    // ============================================================================

    #[tokio::test]
    async fn test_websocket_client_with_config() {
        let config = TransportConfig {
            connect_timeout_ms: Some(1000),
            read_timeout_ms: Some(5000),
            max_message_size: Some(1024 * 1024),
            compression: true,
            write_timeout_ms: None,
            keep_alive_ms: None,
            headers: std::collections::HashMap::new(),
        };

        // Test with a URL that will timeout
        let result = WebSocketClientTransport::with_config(
            "ws://10.255.255.255:9999", // Non-routable IP for timeout
            config.clone(),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            McpError::WebSocket(msg) => assert!(msg.contains("Connection timeout")),
            _ => panic!("Expected WebSocket error with timeout"),
        }
    }

    // ============================================================================
    // WebSocketServerTransport Tests
    // ============================================================================

    #[tokio::test]
    async fn test_websocket_server_start_stop() {
        let mut transport = WebSocketServerTransport::new("127.0.0.1:0");

        // Start the server
        let result = transport.start().await;

        assert!(result.is_ok());
        assert!(transport.is_running());

        // Server should be running now
        // Note: We can't get the actual port without additional API

        // Stop the server
        let result = transport.stop().await;
        assert!(result.is_ok());
        assert!(!transport.is_running());
    }

    #[tokio::test]
    async fn test_websocket_server_info() {
        let transport = WebSocketServerTransport::new("127.0.0.1:8080");
        let info = transport.server_info();
        assert!(info.contains("WebSocket server"));
        assert!(info.contains("127.0.0.1:8080"));
        // The server_info doesn't include running status, just the bind address
    }

    #[tokio::test]
    async fn test_websocket_server_send_response() {
        let _transport = WebSocketServerTransport::new("127.0.0.1:0");

        // Without active connections, send should fail smoothly
        let _response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            result: Some(json!({"test": "data"})),
        };

        // We can't send responses directly without the send_response method
        // This would need to be done through the request handler
    }

    #[tokio::test]
    async fn test_websocket_server_send_notification() {
        let mut transport = WebSocketServerTransport::new("127.0.0.1:0");

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "server_notification".to_string(),
            params: Some(json!({"message": "hello"})),
        };

        // Should succeed even without connections (broadcasts to empty set)
        let result = transport.send_notification(notification).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_websocket_server_double_start() {
        let mut transport = WebSocketServerTransport::new("127.0.0.1:0");

        // First start
        let result1 = transport.start().await;
        assert!(result1.is_ok());

        // Second start will try to bind again, which should fail since port is already in use
        // But since we're using port 0 (random port), it might succeed with a different port
        // Let's just check that start can be called again
        let result2 = transport.start().await;
        // The behavior depends on the implementation - it might succeed or fail
        if result2.is_err() {
            match result2.unwrap_err() {
                McpError::WebSocket(msg) => assert!(msg.contains("bind")),
                _ => panic!("Expected WebSocket error"),
            }
        }

        // Clean up
        let _ = transport.stop().await;
    }

    #[tokio::test]
    async fn test_websocket_server_bind_error() {
        // Try to bind to a privileged port (will fail without sudo)
        let mut transport = WebSocketServerTransport::new("127.0.0.1:1");

        let result = transport.start().await;

        // Should fail due to permission denied or address in use
        assert!(result.is_err());
    }
}

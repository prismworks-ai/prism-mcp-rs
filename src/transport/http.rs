// ! HTTP transport implementation for MCP
// !
// ! Module provides HTTP-based transport for MCP communication,
// ! including Server-Sent Events (SSE) for real-time communication.

use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Sse, sse::Event},
    routing::{get, post},
};
use reqwest::Client;
use serde_json::Value;
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};

#[cfg(all(feature = "futures", feature = "tokio-stream"))]
use futures::stream::Stream;

#[cfg(feature = "tokio-stream")]
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::core::error::{McpError, McpResult};
use crate::core::logging::ErrorContext;
use crate::protocol::types::{
    JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, error_codes,
};
use crate::transport::traits::{ConnectionState, ServerTransport, Transport, TransportConfig};

// ============================================================================
// HTTP Client Transport
// ============================================================================

/// HTTP transport for MCP clients
///
/// This transport communicates with an MCP server via HTTP requests and
/// optionally uses Server-Sent Events for real-time notifications.
#[derive(Debug)]
pub struct HttpClientTransport {
    pub(crate) client: Client,
    pub(crate) base_url: String,
    pub(crate) sse_url: Option<String>,
    pub(crate) headers: HeaderMap,
    /// For tracking active requests (currently used for metrics/debugging)
    pending_requests: Arc<Mutex<HashMap<Value, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
    notification_receiver: Option<mpsc::UnboundedReceiver<JsonRpcNotification>>,
    pub(crate) config: TransportConfig,
    state: ConnectionState,
    request_id_counter: Arc<Mutex<u64>>,
}

impl HttpClientTransport {
    /// Create a new HTTP client transport
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the MCP server
    /// * `sse_url` - Optional URL for Server-Sent Events (for notifications)
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn new<S: AsRef<str>>(base_url: S, sse_url: Option<S>) -> McpResult<Self> {
        Self::with_config(base_url, sse_url, TransportConfig::default()).await
    }

    /// Create a new HTTP client transport with custom configuration
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the MCP server
    /// * `sse_url` - Optional URL for Server-Sent Events
    /// * `config` - Transport configuration
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn with_config<S: AsRef<str>>(
        base_url: S,
        sse_url: Option<S>,
        config: TransportConfig,
    ) -> McpResult<Self> {
        let client_builder = Client::builder()
            .timeout(Duration::from_millis(
                config.read_timeout_ms.unwrap_or(60_000),
            ))
            .connect_timeout(Duration::from_millis(
                config.connect_timeout_ms.unwrap_or(30_000),
            ));

        // Note: reqwest doesn't have a gzip() method, it's enabled by default with features

        let client = client_builder
            .build()
            .map_err(|e| McpError::Http(format!("Failed to create HTTP client: {e}")))?;

        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());

        // Add custom headers from config
        for (key, value) in &config.headers {
            if let (Ok(header_name), Ok(header_value)) = (
                key.parse::<axum::http::HeaderName>(),
                value.parse::<axum::http::HeaderValue>(),
            ) {
                headers.insert(header_name, header_value);
            }
        }

        let (notification_sender, notification_receiver) = mpsc::unbounded_channel();

        // Set up SSE connection for notifications if URL provided
        if let Some(sse_url) = &sse_url {
            let sse_url = sse_url.as_ref().to_string();
            let client_clone = client.clone();
            let headers_clone = headers.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_sse_stream(
                    client_clone,
                    sse_url,
                    headers_clone,
                    notification_sender,
                )
                .await
                {
                    tracing::error!("SSE stream error: {}", e);
                }
            });
        }

        Ok(Self {
            client,
            base_url: base_url.as_ref().to_string(),
            sse_url: sse_url.map(|s| s.as_ref().to_string()),
            headers,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            notification_receiver: Some(notification_receiver),
            config,
            state: ConnectionState::Connected,
            request_id_counter: Arc::new(Mutex::new(0)),
        })
    }

    async fn handle_sse_stream(
        client: Client,
        sse_url: String,
        headers: HeaderMap,
        notification_sender: mpsc::UnboundedSender<JsonRpcNotification>,
    ) -> McpResult<()> {
        let mut request = client.get(&sse_url);
        for (name, value) in headers.iter() {
            // Convert axum headers to reqwest headers
            let name_str = name.as_str();
            let value_bytes = value.as_bytes();
            request = request.header(name_str, value_bytes);
        }

        let response = request
            .send()
            .await
            .map_err(|e| McpError::Http(format!("SSE connection failed: {e}")))?;

        let mut stream = response.bytes_stream();

        #[cfg(feature = "tokio-stream")]
        {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                // Remove "data: " prefix
                                if let Ok(notification) =
                                    serde_json::from_str::<JsonRpcNotification>(data)
                                {
                                    if notification_sender.send(notification).is_err() {
                                        tracing::debug!("Notification receiver dropped");
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("SSE stream error: {}", e);
                        break;
                    }
                }
            }
        }

        #[cfg(not(feature = "tokio-stream"))]
        {
            tracing::warn!("SSE streaming requires tokio-stream feature");
        }

        Ok(())
    }

    pub async fn next_request_id(&self) -> u64 {
        let mut counter = self.request_id_counter.lock().await;
        *counter += 1;
        *counter
    }

    /// Track request for metrics/debugging purposes
    async fn track_request(&self, request_id: &Value) {
        // For HTTP transport, we mainly use this for debugging and metrics
        // Since HTTP is synchronous request/response, we don't need the async
        // tracking that WebSocket uses, but we keep the interface for consistency
        let mut pending = self.pending_requests.lock().await;
        let (sender, _receiver) = tokio::sync::oneshot::channel();
        pending.insert(request_id.clone(), sender);
    }

    /// Remove tracked request
    async fn untrack_request(&self, request_id: &Value) {
        let mut pending = self.pending_requests.lock().await;
        pending.remove(request_id);
    }

    /// Get count of active requests (for debugging/metrics)
    pub async fn active_request_count(&self) -> usize {
        let pending = self.pending_requests.lock().await;
        pending.len()
    }

    #[cfg(test)]
    pub fn has_notification_receiver(&self) -> bool {
        self.notification_receiver.is_some()
    }
}

#[async_trait]
impl Transport for HttpClientTransport {
    async fn send_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        // Generate request ID if not present or ensure we have a valid ID
        let request_with_id = if request.id == Value::Null {
            let request_id = self.next_request_id().await;
            JsonRpcRequest {
                id: Value::from(request_id),
                ..request
            }
        } else {
            request
        };

        // Create error context for logging
        let context = ErrorContext::new("http_send_request")
            .with_transport("http")
            .with_method(&request_with_id.method)
            .with_extra("request_id", request_with_id.id.clone())
            .with_extra("base_url", serde_json::Value::String(self.base_url.clone()));

        // Track the request for debugging/metrics
        self.track_request(&request_with_id.id).await;

        let url = format!("{}/mcp", self.base_url);

        let mut http_request = self.client.post(&url);

        // Apply headers from config and defaults
        for (name, value) in self.headers.iter() {
            let name_str = name.as_str();
            let value_bytes = value.as_bytes();
            http_request = http_request.header(name_str, value_bytes);
        }

        // Apply timeout from config if specified
        if let Some(timeout_ms) = self.config.read_timeout_ms {
            http_request = http_request.timeout(Duration::from_millis(timeout_ms));
        }

        let response = http_request
            .json(&request_with_id)
            .send()
            .await
            .map_err(|e| {
                // Untrack request on error
                let request_id = request_with_id.id.clone();
                let pending_requests = self.pending_requests.clone();
                tokio::spawn(async move {
                    let mut pending = pending_requests.lock().await;
                    pending.remove(&request_id);
                });

                // Create appropriate error based on the reqwest error
                let error = if e.is_timeout() {
                    McpError::timeout("HTTP request timeout")
                } else if e.is_connect() {
                    McpError::connection(format!("HTTP connection failed: {e}"))
                } else {
                    McpError::Http(format!("HTTP request failed: {e}"))
                };

                // Log error with context
                let error_clone = error.clone();
                let context_clone = context.clone();
                tokio::spawn(async move {
                    error_clone.log_with_context(context_clone).await;
                });

                error
            })?;

        if !response.status().is_success() {
            // Untrack request on HTTP error
            self.untrack_request(&request_with_id.id).await;

            let error = McpError::Http(format!(
                "HTTP error: {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("Unknown")
            ));

            // Log HTTP status error
            error.log_with_context(context).await;
            return Err(error);
        }

        let json_response: JsonRpcResponse = response.json().await.map_err(|e| {
            // Untrack request on parse error
            let request_id = request_with_id.id.clone();
            let pending_requests = self.pending_requests.clone();
            tokio::spawn(async move {
                let mut pending = pending_requests.lock().await;
                pending.remove(&request_id);
            });

            let error = McpError::connection(format!("Request serialization failed: {e}"));

            // Log parse error
            let error_clone = error.clone();
            let context_clone = context.clone();
            tokio::spawn(async move {
                error_clone.log_with_context(context_clone).await;
            });

            error
        })?;

        // Validate response ID matches request ID
        if json_response.id != request_with_id.id {
            self.untrack_request(&request_with_id.id).await;
            return Err(McpError::Http(format!(
                "Response ID {:?} does not match request ID {:?}",
                json_response.id, request_with_id.id
            )));
        }

        // Untrack successful request
        self.untrack_request(&request_with_id.id).await;

        Ok(json_response)
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let url = format!("{}/mcp/notify", self.base_url);

        let mut http_request = self.client.post(&url);

        // Apply headers from config and defaults
        for (name, value) in self.headers.iter() {
            let name_str = name.as_str();
            let value_bytes = value.as_bytes();
            http_request = http_request.header(name_str, value_bytes);
        }

        // Apply write timeout from config if specified
        if let Some(timeout_ms) = self.config.write_timeout_ms {
            http_request = http_request.timeout(Duration::from_millis(timeout_ms));
        }

        let response = http_request
            .json(&notification)
            .send()
            .await
            .map_err(|e| McpError::Http(format!("HTTP notification failed: {e}")))?;

        if !response.status().is_success() {
            return Err(McpError::Http(format!(
                "HTTP notification error: {} {}",
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        Ok(())
    }

    async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>> {
        if let Some(ref mut receiver) = self.notification_receiver {
            match receiver.try_recv() {
                Ok(notification) => Ok(Some(notification)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => Err(McpError::Http(
                    "Notification channel disconnected".to_string(),
                )),
            }
        } else {
            Ok(None)
        }
    }

    async fn close(&mut self) -> McpResult<()> {
        self.state = ConnectionState::Disconnected;
        self.notification_receiver = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    fn connection_info(&self) -> String {
        format!(
            "HTTP transport (base: {}, sse: {:?}, state: {:?})",
            self.base_url, self.sse_url, self.state
        )
    }
}

// ============================================================================
// HTTP Server Transport
// ============================================================================

/// Shared state for HTTP server transport
#[derive(Clone)]
struct HttpServerState {
    notification_sender: broadcast::Sender<JsonRpcNotification>,
    request_handler: Option<
        Arc<
            dyn Fn(JsonRpcRequest) -> tokio::sync::oneshot::Receiver<JsonRpcResponse> + Send + Sync,
        >,
    >,
}

/// HTTP transport for MCP servers
///
/// This transport serves MCP requests over HTTP and provides Server-Sent Events
/// for real-time notifications to clients.
pub struct HttpServerTransport {
    bind_addr: String,
    config: TransportConfig,
    state: Arc<RwLock<HttpServerState>>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    running: Arc<RwLock<bool>>,
}

impl HttpServerTransport {
    /// Create a new HTTP server transport
    ///
    /// # Arguments
    /// * `bind_addr` - Address to bind the HTTP server to (e.g., "0.0.0.0:3000")
    ///
    /// # Returns
    /// New HTTP server transport instance
    pub fn new<S: Into<String>>(bind_addr: S) -> Self {
        Self::with_config(bind_addr, TransportConfig::default())
    }

    /// Create a new HTTP server transport with custom configuration
    ///
    /// # Arguments
    /// * `bind_addr` - Address to bind the HTTP server to
    /// * `config` - Transport configuration
    ///
    /// # Returns
    /// New HTTP server transport instance
    pub fn with_config<S: Into<String>>(bind_addr: S, config: TransportConfig) -> Self {
        let (notification_sender, _) = broadcast::channel(1000);

        Self {
            bind_addr: bind_addr.into(),
            config,
            state: Arc::new(RwLock::new(HttpServerState {
                notification_sender,
                request_handler: None,
            })),
            server_handle: None,
            running: Arc::new(RwLock::new(false)),
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
        let mut state = self.state.write().await;
        state.request_handler = Some(Arc::new(handler));
    }

    #[cfg(test)]
    pub fn get_bind_addr(&self) -> &str {
        &self.bind_addr
    }

    #[cfg(test)]
    pub fn get_config(&self) -> &TransportConfig {
        &self.config
    }
}

#[async_trait]
impl ServerTransport for HttpServerTransport {
    async fn start(&mut self) -> McpResult<()> {
        tracing::info!("Starting HTTP server on {}", self.bind_addr);

        let state = self.state.clone();
        let bind_addr = self.bind_addr.clone();
        let running = self.running.clone();
        let _config = self.config.clone();

        // Create the Axum app with configuration-based settings
        let mut app = Router::new()
            .route("/mcp", post(handle_mcp_request))
            .route("/mcp/notify", post(handle_mcp_notification))
            .route("/mcp/events", get(handle_sse_events))
            .route("/health", get(handle_health_check))
            .with_state(state);

        // Apply CORS configuration
        let cors_layer = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        app = app.layer(ServiceBuilder::new().layer(cors_layer).into_inner());

        // Note: Timeout configuration is handled at the HTTP client level
        // Server-side timeouts are managed by the underlying Axum/Hyper stack

        // Start the server
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| McpError::Http(format!("Failed to bind to {bind_addr}: {e}")))?;

        *running.write().await = true;

        let server_handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("HTTP server error: {}", e);
            }
        });

        self.server_handle = Some(server_handle);

        tracing::info!("HTTP server started successfully on {}", self.bind_addr);
        Ok(())
    }

    fn set_request_handler(&mut self, handler: crate::transport::traits::ServerRequestHandler) {
        // Convert the ServerRequestHandler to the HTTP transport's expected format
        let _http_handler = Arc::new(move |request: JsonRpcRequest| {
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

        // Set the handler using the existing async method
        tokio::spawn(async move {
            // Note: This is a limitation - we can't call async methods from a sync trait method
            // The HTTP transport should be updated in the future to support the new trait design
        });
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let state = self.state.read().await;

        if state.notification_sender.send(notification).is_err() {
            tracing::warn!("No SSE clients connected to receive notification");
        }

        Ok(())
    }

    async fn stop(&mut self) -> McpResult<()> {
        tracing::info!("Stopping HTTP server");

        *self.running.write().await = false;

        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }

        Ok(())
    }

    fn is_running(&self) -> bool {
        // Check if we have an active server handle
        self.server_handle.is_some()
    }

    fn server_info(&self) -> String {
        format!("HTTP server transport (bind: {})", self.bind_addr)
    }
}

// ============================================================================
// HTTP Route Handlers
// ============================================================================

/// Handle MCP JSON-RPC requests
async fn handle_mcp_request(
    State(state): State<Arc<RwLock<HttpServerState>>>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcMessage>, StatusCode> {
    let state_guard = state.read().await;

    if let Some(ref handler) = state_guard.request_handler {
        let response_rx = handler(request);
        drop(state_guard); // Release the lock

        match response_rx.await {
            Ok(response) => Ok(Json(JsonRpcMessage::Response(response))),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        let error_response = JsonRpcError::error(
            request.id,
            error_codes::METHOD_NOT_FOUND,
            "No request handler configured".to_string(),
            None,
        );
        Ok(Json(JsonRpcMessage::Error(error_response)))
    }
}

/// Handle MCP notification requests
async fn handle_mcp_notification(Json(_notification): Json<JsonRpcNotification>) -> StatusCode {
    // Notifications don't require a response
    StatusCode::OK
}

/// Handle Server-Sent Events for real-time notifications
#[cfg(all(feature = "tokio-stream", feature = "futures"))]
async fn handle_sse_events(
    State(state): State<Arc<RwLock<HttpServerState>>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let state_guard = state.read().await;
    let receiver = state_guard.notification_sender.subscribe();
    drop(state_guard);

    let stream = BroadcastStream::new(receiver).map(|result| {
        match result {
            Ok(notification) => match serde_json::to_string(&notification) {
                Ok(json) => Ok(Event::default().data(json)),
                Err(e) => {
                    tracing::error!("Failed to serialize notification: {}", e);
                    Ok(Event::default().data("{}"))
                }
            },
            Err(_) => Ok(Event::default().data("{}")), // Lagged or closed
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    )
}

/// Handle Server-Sent Events (fallback when features not available)
#[cfg(not(all(feature = "tokio-stream", feature = "futures")))]
async fn handle_sse_events(_state: State<Arc<RwLock<HttpServerState>>>) -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

/// Handle health check requests
async fn handle_health_check() -> Json<Value> {
    #[cfg(feature = "chrono")]
    let timestamp = chrono::Utc::now().to_rfc3339();
    #[cfg(not(feature = "chrono"))]
    let timestamp = "unavailable";

    Json(serde_json::json!({
        "status": "healthy",
        "transport": "http",
        "timestamp": timestamp
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_http_client_creation() {
        let transport = HttpClientTransport::new("http://localhost:3000", None).await;
        assert!(transport.is_ok());

        let transport = transport.unwrap();
        assert!(transport.is_connected());
        assert_eq!(transport.base_url, "http://localhost:3000");
    }

    #[tokio::test]
    async fn test_http_server_creation() {
        let transport = HttpServerTransport::new("127.0.0.1:0");
        assert_eq!(transport.bind_addr, "127.0.0.1:0");
        assert!(!transport.is_running());
    }

    #[test]
    fn test_http_server_with_config() {
        let config = TransportConfig {
            compression: true,
            ..Default::default()
        };

        let transport = HttpServerTransport::with_config("0.0.0.0:8080", config);
        assert_eq!(transport.bind_addr, "0.0.0.0:8080");
        assert!(transport.config.compression);
    }

    #[tokio::test]
    async fn test_http_client_with_sse() {
        let transport = HttpClientTransport::new(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
        )
        .await;

        assert!(transport.is_ok());
        let transport = transport.unwrap();
        assert!(transport.sse_url.is_some());
        assert_eq!(transport.sse_url.unwrap(), "http://localhost:3000/events");
    }

    // Add complete tests for maximum coverage
    #[tokio::test]
    async fn test_request_id_generation_sequence() {
        let transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        let id1 = transport.next_request_id().await;
        let id2 = transport.next_request_id().await;
        let id3 = transport.next_request_id().await;

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[tokio::test]
    async fn test_request_tracking_complete() {
        let transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // Initially no active requests
        assert_eq!(transport.active_request_count().await, 0);

        // Track multiple requests with different ID types
        let request_ids = vec![
            Value::from(123),
            Value::String("string-id".to_string()),
            Value::Null,
            Value::Array(vec![Value::from(1), Value::from(2)]),
        ];

        for id in &request_ids {
            transport.track_request(id).await;
        }
        assert_eq!(transport.active_request_count().await, request_ids.len());

        // Untrack all requests
        for id in &request_ids {
            transport.untrack_request(id).await;
        }
        assert_eq!(transport.active_request_count().await, 0);

        // Untrack non-existent request (should not panic)
        transport.untrack_request(&Value::from(999)).await;
        assert_eq!(transport.active_request_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_state_management() {
        let mut transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // Initially connected
        assert!(transport.is_connected());
        assert!(transport.has_notification_receiver());

        let info_before = transport.connection_info();
        assert!(info_before.contains("Connected"));

        // Close transport
        let result = transport.close().await;
        assert!(result.is_ok());

        // Should be disconnected
        assert!(!transport.is_connected());
        assert!(!transport.has_notification_receiver());

        let info_after = transport.connection_info();
        assert!(info_after.contains("Disconnected"));
    }

    #[tokio::test]
    async fn test_receive_notification_states() {
        let mut transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // Without SSE URL, the notification channel gets disconnected
        // This should return an error indicating disconnection
        let result = transport.receive_notification().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disconnected"));

        // After closing, should still return None (receiver is set to None)
        transport.close().await.unwrap();
        let result = transport.receive_notification().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Test again after close to ensure consistent behavior
        let result2 = transport.receive_notification().await;
        assert!(result2.is_ok());
        assert!(result2.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_http_server_lifecycle_complete() {
        let mut transport = HttpServerTransport::new("127.0.0.1:0");

        // Check initial state
        assert_eq!(transport.get_bind_addr(), "127.0.0.1:0");
        assert!(!transport.is_running());

        let info = transport.server_info();
        assert!(info.contains("HTTP server transport"));
        assert!(info.contains("127.0.0.1:0"));

        // Start server
        let result = transport.start().await;
        assert!(result.is_ok());
        assert!(transport.is_running());

        // Send notification while running
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_notification".to_string(),
            params: Some(serde_json::json!({"test": true})),
        };
        let result = transport.send_notification(notification).await;
        assert!(result.is_ok());

        // Stop server
        let result = transport.stop().await;
        assert!(result.is_ok());
        assert!(!transport.is_running());

        // Should be able to stop again without error
        let result = transport.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_http_server_request_handler() {
        let mut transport = HttpServerTransport::new("127.0.0.1:0");

        let handler = |request: JsonRpcRequest| {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::json!({
                    "method_received": request.method,
                    "handled": true
                })),
            };
            let _ = tx.send(response);
            rx
        };

        transport.set_request_handler(handler).await;
        // Handler should be set successfully (we can't easily test actual handling)
    }

    #[tokio::test]
    async fn test_http_server_with_custom_config() {
        let mut config = TransportConfig {
            compression: true,
            ..Default::default()
        };
        config
            .headers
            .insert("Server".to_string(), "MCP-Test/1.0".to_string());

        let transport = HttpServerTransport::with_config("0.0.0.0:8080", config);

        assert_eq!(transport.get_bind_addr(), "0.0.0.0:8080");
        assert!(transport.get_config().compression);
        assert_eq!(
            transport.get_config().headers.get("Server"),
            Some(&"MCP-Test/1.0".to_string())
        );
    }

    #[tokio::test]
    async fn test_http_client_with_custom_config() {
        let mut config = TransportConfig {
            read_timeout_ms: Some(5000),
            connect_timeout_ms: Some(2000),
            write_timeout_ms: Some(3000),
            ..Default::default()
        };
        config
            .headers
            .insert("X-Custom-Header".to_string(), "test-value".to_string());
        config
            .headers
            .insert("Authorization".to_string(), "Bearer token123".to_string());

        let transport = HttpClientTransport::with_config(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
            config,
        )
        .await;

        assert!(transport.is_ok());
        let transport = transport.unwrap();
        assert_eq!(transport.config.read_timeout_ms, Some(5000));
        assert_eq!(transport.config.connect_timeout_ms, Some(2000));
        assert_eq!(transport.config.write_timeout_ms, Some(3000));
        assert!(transport.sse_url.is_some());
    }

    // Route handler tests
    #[tokio::test]
    async fn test_handle_health_check() {
        let result = handle_health_check().await;

        let Json(health_data) = result;
        assert_eq!(health_data["status"], "healthy");
        assert_eq!(health_data["transport"], "http");
        assert!(health_data["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_handle_mcp_notification() {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_notification".to_string(),
            params: Some(serde_json::json!({"test": "notification"})),
        };
        let json_notification = Json(notification);

        let result = handle_mcp_notification(json_notification).await;

        // Notifications should always return OK
        assert_eq!(result, StatusCode::OK);
    }

    #[cfg(not(all(feature = "tokio-stream", feature = "futures")))]
    #[tokio::test]
    async fn test_handle_sse_events_not_implemented() {
        let (notification_sender, _) = broadcast::channel(100);

        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender,
            request_handler: None,
        }));

        let state_extract = State(state);

        let result = handle_sse_events(state_extract).await;

        // Should return NOT_IMPLEMENTED when features are not available
        assert_eq!(result, StatusCode::NOT_IMPLEMENTED);
    }

    // Edge cases and complete coverage tests
    #[tokio::test]
    async fn test_transport_config_variations() {
        // Test default config
        let default_config = TransportConfig::default();
        assert_eq!(default_config.read_timeout_ms, Some(60_000));
        assert_eq!(default_config.write_timeout_ms, Some(30_000));
        assert_eq!(default_config.connect_timeout_ms, Some(30_000));
        assert!(default_config.headers.is_empty());

        // Test config with all options
        let mut full_config = TransportConfig {
            read_timeout_ms: Some(10000),
            write_timeout_ms: Some(5000),
            connect_timeout_ms: Some(3000),
            compression: true,
            ..Default::default()
        };
        full_config
            .headers
            .insert("Test-Header".to_string(), "test-value".to_string());

        let transport =
            HttpClientTransport::with_config("http://localhost:3000", None, full_config)
                .await
                .unwrap();

        assert_eq!(transport.config.read_timeout_ms, Some(10000));
        assert_eq!(transport.config.write_timeout_ms, Some(5000));
        assert_eq!(transport.config.connect_timeout_ms, Some(3000));
        assert!(transport.config.compression);
    }

    #[tokio::test]
    async fn test_sse_url_variations() {
        // Test with SSE URL as &str
        let transport1 = HttpClientTransport::new(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
        )
        .await
        .unwrap();
        assert!(transport1.sse_url.is_some());
        assert_eq!(
            transport1.sse_url.as_ref().unwrap(),
            "http://localhost:3000/events"
        );

        // Test with SSE URL as String
        let transport2 = HttpClientTransport::new(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
        )
        .await
        .unwrap();
        assert!(transport2.sse_url.is_some());

        // Test without SSE URL
        let transport3 = HttpClientTransport::new("http://localhost:3000", None::<&str>)
            .await
            .unwrap();
        assert!(transport3.sse_url.is_none());

        // Test connection info formatting
        let info1 = transport1.connection_info();
        assert!(info1.contains("http://localhost:3000/events"));

        let info3 = transport3.connection_info();
        assert!(info3.contains("sse: None"));
    }

    #[tokio::test]
    async fn test_concurrent_request_id_generation() {
        let transport = std::sync::Arc::new(
            HttpClientTransport::new("http://localhost:3000", None)
                .await
                .unwrap(),
        );

        let mut handles = vec![];

        // Spawn multiple tasks generating request IDs concurrently
        for _ in 0..3 {
            let transport_clone = transport.clone();
            let handle = tokio::spawn(async move {
                let mut ids = vec![];
                for _ in 0..3 {
                    ids.push(transport_clone.next_request_id().await);
                }
                ids
            });
            handles.push(handle);
        }

        let mut all_ids = vec![];
        for handle in handles {
            let ids = handle.await.unwrap();
            all_ids.extend(ids);
        }

        // All IDs should be unique
        all_ids.sort();
        let mut unique_ids = all_ids.clone();
        unique_ids.dedup();

        assert_eq!(all_ids.len(), unique_ids.len());
        assert_eq!(all_ids.len(), 9); // 3 tasks * 3 IDs each
    }

    #[tokio::test]
    async fn test_server_bind_addresses() {
        let test_cases = vec!["127.0.0.1:0", "0.0.0.0:8080", "localhost:9000"];

        for addr in test_cases {
            let server = HttpServerTransport::new(addr);
            assert_eq!(server.get_bind_addr(), addr);
            assert!(!server.is_running());

            let info = server.server_info();
            assert!(info.contains("HTTP server transport"));
            assert!(info.contains(addr));
        }
    }

    // Mock server tests for actual Transport trait implementation coverage
    #[tokio::test]
    async fn test_transport_send_request_with_mock() {
        let mock_server = MockServer::start().await;

        // Set up mock response
        let expected_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::from(42),
            result: Some(serde_json::json!({
                "capabilities": {
                    "tools": true,
                    "resources": true
                }
            })),
        };

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&expected_response))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(42),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            })),
        };

        let result = transport.send_request(request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, Value::from(42));
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_transport_send_notification_with_mock() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp/notify"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "initialized".to_string(),
            params: Some(serde_json::json!({})),
        };

        let result = transport.send_notification(notification).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transport_request_auto_id() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"status": "ok"}
            })))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        // Request with null ID should get auto-generated ID
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::Null,
            method: "ping".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, Value::from(1));
    }

    #[tokio::test]
    async fn test_transport_error_scenarios() {
        let mock_server = MockServer::start().await;

        // Test HTTP 500 error
        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(1),
            method: "test".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_err());

        if let Err(McpError::Http(msg)) = result {
            assert!(msg.contains("HTTP error: 500"));
        } else {
            panic!("Expected HTTP error");
        }
    }

    #[tokio::test]
    async fn test_transport_notification_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp/notify"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_notification".to_string(),
            params: None,
        };

        let result = transport.send_notification(notification).await;
        assert!(result.is_err());

        if let Err(McpError::Http(msg)) = result {
            assert!(msg.contains("HTTP notification error: 400"));
        } else {
            panic!("Expected HTTP notification error");
        }
    }

    #[tokio::test]
    async fn test_transport_connection_failure() {
        // Use invalid port to trigger connection error
        let mut transport = HttpClientTransport::new("http://127.0.0.1:1", None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(1),
            method: "test".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_err());
        // Connection errors can manifest as different error types
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transport_invalid_json_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(1),
            method: "test".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_err());

        if let Err(McpError::Connection(msg)) = result {
            assert!(msg.contains("Request serialization failed"));
        } else {
            // Accept other error types for JSON parsing failures
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_transport_response_id_mismatch() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 999, // Different from request ID
                "result": {"success": true}
            })))
            .mount(&mock_server)
            .await;

        let mut transport = HttpClientTransport::new(mock_server.uri(), None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(1),
            method: "test".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_err());

        if let Err(McpError::Http(msg)) = result {
            assert!(msg.contains("Response ID") && msg.contains("does not match request ID"));
        } else {
            panic!("Expected HTTP error for ID mismatch");
        }
    }
}

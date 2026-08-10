//! MCP client implementation
//!
//! Module provides the main MCP client that can connect to MCP servers,
//! initialize connections, and perform operations like calling tools, reading resources,
//! and executing prompts according to the Model Context Protocol specification.
//!
//! # Standards-track and proprietary HTTP
//!
//! Use [`McpClient::connect_with_http`] for standards-track MCP, including
//! MCP 2026 request-scoped subscription SSE. The optional `chunked-encoding`
//! feature exposes historical Prism-specific endpoints:
//! ```toml
//! [dependencies]
//! prism-mcp-rs = { version = "3", features = ["chunked-encoding"] }
//! ```
//!
//! Those helpers require explicit [`ProtocolMode::LegacyOnly`] and a peer that
//! implements the same proprietary routes; automatic transport selection never
//! chooses them.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::client::request_handler::{ClientRequestHandler, DefaultClientRequestHandler};
use crate::core::error::{McpError, McpResult};
use crate::protocol::tasks::{
    has_tasks_extension, CancelTaskParams, CreateTaskResult, GetTaskParams, GetTaskResult, Task,
    TaskAcknowledgement, TaskStatus, UpdateTaskParams, TASKS_EXTENSION_ID,
};
use crate::protocol::{messages::*, methods, types::*, validation::*, version::*};
use crate::transport::traits::{ClientSubscription, Transport};

/// Configuration for the MCP client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Whether to validate outgoing requests
    pub validate_requests: bool,
    /// Whether to validate incoming responses
    pub validate_responses: bool,
    /// Runtime MCP revision policy.
    pub protocol_mode: ProtocolMode,
    /// Maximum automatic multi-round-trip input cycles.
    pub max_mrtr_rounds: u8,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            request_timeout_ms: 30000,
            max_retries: 3,
            retry_delay_ms: 1000,
            validate_requests: true,
            validate_responses: true,
            protocol_mode: ProtocolMode::Auto,
            max_mrtr_rounds: 10,
        }
    }
}

/// Main MCP client implementation
pub struct McpClient {
    /// Client information
    info: ClientInfo,
    /// Client capabilities
    capabilities: ClientCapabilities,
    /// Client configuration
    config: ClientConfig,
    /// Active transport
    transport: Arc<Mutex<Option<Box<dyn Transport>>>>,
    /// Server capabilities (available after initialization)
    server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    /// Server information (available after initialization)
    server_info: Arc<RwLock<Option<ServerInfo>>>,
    /// Request ID counter
    request_counter: Arc<Mutex<u64>>,
    /// Connection state
    connected: Arc<RwLock<bool>>,
    /// Request handler for server-initiated requests
    request_handler: Arc<dyn ClientRequestHandler>,
    /// Protocol selected for the active connection.
    negotiated_protocol: Arc<RwLock<Option<NegotiatedProtocol>>>,
}

impl McpClient {
    /// Internal constructor - use builder() instead
    pub(crate) fn from_parts(
        info: ClientInfo,
        capabilities: ClientCapabilities,
        config: ClientConfig,
    ) -> Self {
        Self {
            info,
            capabilities,
            config,
            transport: Arc::new(Mutex::new(None)),
            server_capabilities: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            request_counter: Arc::new(Mutex::new(0)),
            connected: Arc::new(RwLock::new(false)),
            request_handler: Arc::new(DefaultClientRequestHandler),
            negotiated_protocol: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new client builder (primary constructor)
    pub fn builder() -> crate::client::enhanced_builder::McpClientBuilder {
        crate::client::enhanced_builder::McpClientBuilder::new()
    }

    /// Create a new MCP client with name and version
    pub fn new(name: String, version: String) -> Self {
        let info = ClientInfo::new(name, version);
        Self::with_client_info(info)
    }

    /// Create a new MCP client with a specific ClientInfo
    pub fn with_client_info(info: ClientInfo) -> Self {
        Self {
            info,
            capabilities: ClientCapabilities::default(),
            config: ClientConfig::default(),
            transport: Arc::new(Mutex::new(None)),
            server_capabilities: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            request_counter: Arc::new(Mutex::new(0)),
            connected: Arc::new(RwLock::new(false)),
            request_handler: Arc::new(DefaultClientRequestHandler),
            negotiated_protocol: Arc::new(RwLock::new(None)),
        }
    }

    // ========================================================================
    // Modern Fluent Interface (Primary API)
    // ========================================================================

    /// Access tools with fluent interface
    pub fn tools(&self) -> crate::client::fluent_tools::ToolsBuilder<'_> {
        crate::client::fluent_tools::ToolsBuilder::new(self)
    }

    /// Access resources with fluent interface
    pub fn resources(&self) -> crate::client::fluent_interfaces::ResourcesBuilder<'_> {
        crate::client::fluent_interfaces::ResourcesBuilder::new(self)
    }

    /// Access prompts with fluent interface
    pub fn prompts(&self) -> crate::client::fluent_interfaces::PromptsBuilder<'_> {
        crate::client::fluent_interfaces::PromptsBuilder::new(self)
    }

    // ========================================================================
    // Deprecated Convenience Methods (Backward Compatibility)
    // ========================================================================

    /// Convenience method: call_tool with &str name and serde_json::Value arguments
    ///
    /// # Deprecated
    /// Use `client.tools().call(name).args(arguments).execute().await` instead
    #[deprecated(
        since = "0.2.0",
        note = "Use fluent interface: client.tools().call(name).args(args).execute().await"
    )]
    pub async fn call_tool_simple(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> McpResult<CallToolResult> {
        let args_map = if let Some(obj) = arguments.as_object() {
            obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            HashMap::new()
        };

        self.call_tool(name.to_string(), Some(args_map)).await
    }

    /// Create a new MCP client with custom configuration
    pub fn with_config(name: String, version: String, config: ClientConfig) -> Self {
        let mut client = Self::new(name, version);
        client.config = config;
        client
    }

    /// Set client capabilities
    pub fn set_capabilities(&mut self, capabilities: ClientCapabilities) {
        self.capabilities = capabilities;
    }

    /// Declare support for the official MCP Tasks extension.
    pub fn enable_tasks_extension(&mut self) {
        self.capabilities
            .extensions
            .get_or_insert_with(HashMap::new)
            .insert(TASKS_EXTENSION_ID.to_string(), serde_json::json!({}));
    }

    /// Set custom request handler for server-initiated requests
    ///
    /// This enables bidirectional MCP communication by allowing the server
    /// to initiate requests to the client for sampling, elicitation, etc.
    ///
    /// # Arguments
    /// * `handler` - Custom implementation of ClientRequestHandler
    ///
    /// # Example
    /// ```rust
    /// use prism_mcp_rs::client::{McpClient, InteractiveClientRequestHandler};
    ///
    /// let mut client = McpClient::new("my-app".to_string(), "1.0.0".to_string());
    /// let handler = InteractiveClientRequestHandler::new("my-app")
    /// .add_root("file:///home/user", Some("Home"))
    /// .auto_accept_elicitation(true);
    /// client.set_request_handler(handler);
    /// ```
    pub fn set_request_handler<H>(&mut self, handler: H)
    where
        H: ClientRequestHandler + 'static,
    {
        self.request_handler = Arc::new(handler);
    }

    /// Handle incoming server request
    ///
    /// Method processes server-initiated requests and returns appropriate
    /// responses. It's called automatically by the transport layer when the
    /// server sends a request to the client.
    ///
    /// # Arguments
    /// * `request` - The JSON-RPC request from the server
    ///
    /// # Returns
    /// JSON-RPC response to send back to the server
    pub async fn handle_server_request(
        &self,
        request: JsonRpcRequest,
    ) -> McpResult<JsonRpcResponse> {
        let result =
            match request.method.as_str() {
                methods::SAMPLING_CREATE_MESSAGE => {
                    let params: CreateMessageParams =
                        serde_json::from_value(request.params.ok_or_else(|| {
                            McpError::InvalidParams("Missing params".to_string())
                        })?)?;
                    let result = self.request_handler.handle_create_message(params).await?;
                    serde_json::to_value(result)?
                }
                methods::ROOTS_LIST => {
                    let params: ListRootsParams = request
                        .params
                        .map(serde_json::from_value)
                        .transpose()?
                        .unwrap_or(ListRootsParams { meta: None });
                    let result = self.request_handler.handle_list_roots(params).await?;
                    serde_json::to_value(result)?
                }
                methods::ELICITATION_CREATE => {
                    let params: ElicitParams =
                        serde_json::from_value(request.params.ok_or_else(|| {
                            McpError::InvalidParams("Missing params".to_string())
                        })?)?;
                    let result = self.request_handler.handle_elicit(params).await?;
                    serde_json::to_value(result)?
                }
                methods::PING => {
                    let params: PingParams = request
                        .params
                        .map(serde_json::from_value)
                        .transpose()?
                        .unwrap_or(PingParams { meta: None });
                    let result = self.request_handler.handle_ping(params).await?;
                    serde_json::to_value(result)?
                }
                _ => {
                    return Err(McpError::MethodNotFound(format!(
                        "Unknown method: {}",
                        request.method
                    )));
                }
            };

        Ok(JsonRpcResponse::success(request.id, result)?)
    }

    /// Get client information
    pub fn info(&self) -> &ClientInfo {
        &self.info
    }

    /// Get client capabilities
    pub fn capabilities(&self) -> &ClientCapabilities {
        &self.capabilities
    }

    /// Get client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Set the protocol selection policy before connecting.
    pub fn set_protocol_mode(&mut self, mode: ProtocolMode) {
        self.config.protocol_mode = mode;
    }

    /// Return the protocol selected for the active connection.
    pub async fn negotiated_protocol(&self) -> Option<NegotiatedProtocol> {
        self.negotiated_protocol.read().await.clone()
    }

    /// Get server capabilities (if connected)
    pub async fn server_capabilities(&self) -> Option<ServerCapabilities> {
        let capabilities = self.server_capabilities.read().await;
        capabilities.clone()
    }

    /// Get server information (if connected)
    pub async fn server_info(&self) -> Option<ServerInfo> {
        let info = self.server_info.read().await;
        info.clone()
    }

    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        let connected = self.connected.read().await;
        *connected
    }

    // ========================================================================
    // Connection Management
    // ========================================================================

    /// Connect to an MCP server using the provided transport
    pub async fn connect<T>(&mut self, transport: T) -> McpResult<ConnectResult>
    where
        T: Transport + 'static,
    {
        // Set the transport
        {
            let mut transport_guard = self.transport.lock().await;
            *transport_guard = Some(Box::new(transport));
        }

        let connection = match self.config.protocol_mode {
            ProtocolMode::ModernOnly => self.discover_modern().await?,
            ProtocolMode::LegacyOnly => self.initialize_legacy().await?,
            ProtocolMode::Auto => match self.discover_modern().await {
                Ok(result) => result,
                Err(error) if is_method_not_found(&error) => {
                    tracing::info!("server/discover unavailable; using MCP 2025-11-25");
                    self.initialize_legacy().await?
                }
                Err(error) => return Err(error),
            },
        };

        // Mark as connected
        {
            let mut connected = self.connected.write().await;
            *connected = true;
        }

        Ok(connection)
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) -> McpResult<()> {
        // Close the transport
        {
            let mut transport_guard = self.transport.lock().await;
            if let Some(transport) = transport_guard.as_mut() {
                transport.close().await?;
            }
            *transport_guard = None;
        }

        // Clear server information
        {
            let mut server_capabilities = self.server_capabilities.write().await;
            *server_capabilities = None;
        }
        {
            let mut server_info = self.server_info.write().await;
            *server_info = None;
        }

        // Mark as disconnected
        {
            let mut connected = self.connected.write().await;
            *connected = false;
        }
        *self.negotiated_protocol.write().await = None;

        Ok(())
    }

    /// Negotiate the stateless MCP 2026-07-28 lifecycle.
    async fn discover_modern(&self) -> McpResult<ConnectResult> {
        let params = DiscoverParams {
            meta: RequestMetaObject::modern(self.info.clone(), self.capabilities.clone()),
        };
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::SERVER_DISCOVER.to_string(),
            Some(params.clone()),
        )?;
        let response = match self.send_request(request).await {
            Err(McpError::UnsupportedProtocolVersion { supported, .. })
                if supported
                    .iter()
                    .any(|version| version == MODERN_PROTOCOL_VERSION) =>
            {
                tracing::info!(
                    protocol.version = MODERN_PROTOCOL_VERSION,
                    "retrying server/discover with a mutually supported version"
                );
                let retry = JsonRpcRequest::new(
                    Value::from(self.next_request_id().await),
                    methods::SERVER_DISCOVER.to_string(),
                    Some(params),
                )?;
                self.send_request(retry).await?
            }
            result => result?,
        };
        let result: DiscoverResult = serde_json::from_value(
            response
                .result
                .ok_or_else(|| McpError::Protocol("Missing discover result".to_string()))?,
        )?;
        if !result
            .supported_versions
            .iter()
            .any(|version| version == MODERN_PROTOCOL_VERSION)
        {
            return Err(McpError::UnsupportedProtocolVersion {
                requested: MODERN_PROTOCOL_VERSION.to_string(),
                supported: result.supported_versions,
            });
        }

        let protocol = NegotiatedProtocol::modern();
        let server_info = result.server_info();
        *self.server_capabilities.write().await = Some(result.capabilities.clone());
        *self.server_info.write().await = server_info.clone();
        *self.negotiated_protocol.write().await = Some(protocol.clone());

        Ok(ConnectResult {
            protocol,
            capabilities: result.capabilities,
            server_info,
            instructions: result.instructions,
        })
    }

    /// Initialize the legacy MCP 2025-11-25 lifecycle.
    async fn initialize_legacy(&self) -> McpResult<ConnectResult> {
        let params = InitializeParams::new(
            LEGACY_PROTOCOL_VERSION.to_string(),
            self.capabilities.clone(),
            self.info.clone(),
        );

        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::INITIALIZE.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;

        // The send_request method will return an error if there was a JSON-RPC error
        // so we can safely extract the result here

        let result: InitializeResult = serde_json::from_value(
            response
                .result
                .ok_or_else(|| McpError::Protocol("Missing initialize result".to_string()))?,
        )?;

        // Store server information
        {
            let mut server_capabilities = self.server_capabilities.write().await;
            *server_capabilities = Some(result.capabilities.clone());
        }
        {
            let mut server_info = self.server_info.write().await;
            *server_info = Some(result.server_info.clone());
        }

        let protocol = NegotiatedProtocol::legacy();
        *self.negotiated_protocol.write().await = Some(protocol.clone());
        Ok(ConnectResult {
            protocol,
            capabilities: result.capabilities,
            server_info: Some(result.server_info),
            instructions: result.instructions,
        })
    }

    // ========================================================================
    // Connection Convenience Methods
    // ========================================================================

    /// Connect to an MCP server over STDIO (convenience method)
    ///
    /// This is a convenience method that:
    /// 1. Creates a STDIO transport
    /// 2. Connects to the server
    /// 3. Returns the initialization result
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("my-client".to_string(), "1.0.0".to_string());
    /// let init_result = client.connect_with_stdio("my-mcp-server", vec!["--verbose"]).await?;
    /// println!("Protocol: {}", init_result.protocol.version);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "stdio")]
    pub async fn connect_with_stdio(
        &mut self,
        command: &str,
        args: Vec<&str>,
    ) -> McpResult<ConnectResult> {
        use crate::transport::stdio::StdioClientTransport;

        if self.config.protocol_mode != ProtocolMode::Auto {
            let transport = StdioClientTransport::new(command, args).await?;
            return self.connect(transport).await;
        }

        // Some legacy stdio servers terminate after an unknown-method probe.
        // Probe a disposable child and start a clean sibling for initialization.
        let probe = StdioClientTransport::new(command, args.clone()).await?;
        *self.transport.lock().await = Some(Box::new(probe));
        match self.discover_modern().await {
            Ok(connection) => {
                *self.connected.write().await = true;
                Ok(connection)
            }
            Err(error) if is_method_not_found(&error) => {
                if let Some(transport) = self.transport.lock().await.as_mut() {
                    let _ = transport.close().await;
                }
                let legacy = StdioClientTransport::new(command, args).await?;
                *self.transport.lock().await = Some(Box::new(legacy));
                let connection = self.initialize_legacy().await?;
                *self.connected.write().await = true;
                Ok(connection)
            }
            Err(error) => {
                if let Some(transport) = self.transport.lock().await.as_mut() {
                    let _ = transport.close().await;
                }
                *self.transport.lock().await = None;
                Err(error)
            }
        }
    }

    /// Connect to an MCP server over HTTP (convenience method)
    ///
    /// This is a convenience method that:
    /// 1. Creates an HTTP transport to the specified URL
    /// 2. Connects to the server
    /// 3. Returns the initialization result
    ///
    /// # Arguments
    /// * `server_url` - The HTTP URL of the MCP server (e.g., "http://localhost:3000")
    /// * `sse_url` - Optional Server-Sent Events URL for notifications
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("my-client".to_string(), "1.0.0".to_string());
    /// let init_result = client.connect_with_http("http://localhost:3000", None).await?;
    /// println!("Protocol: {}", init_result.protocol.version);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "http")]
    pub async fn connect_with_http(
        &mut self,
        server_url: &str,
        sse_url: Option<&str>,
    ) -> McpResult<ConnectResult> {
        use crate::transport::http::HttpClientTransport;

        let transport = HttpClientTransport::new(server_url, sse_url).await?;
        self.connect(transport).await
    }

    /// Connect to an MCP server over STDIO with simple command (convenience method)
    ///
    /// This is a convenience method for the most common STDIO use case:
    /// connecting to a server with just a command and no arguments.
    ///
    /// # Arguments
    /// * `command` - Command to execute for the MCP server
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("my-client".to_string(), "1.0.0".to_string());
    /// let init_result = client.connect_with_stdio_simple("my-mcp-server").await?;
    /// println!("Protocol: {}", init_result.protocol.version);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "stdio")]
    pub async fn connect_with_stdio_simple(&mut self, command: &str) -> McpResult<ConnectResult> {
        self.connect_with_stdio(command, vec![]).await
    }

    /// Connect to an MCP server over WebSocket (convenience method)
    ///
    /// This is a convenience method that:
    /// 1. Creates a WebSocket transport to the specified URL
    /// 2. Connects to the server
    /// 3. Returns the initialization result
    ///
    /// # Arguments
    /// * `server_url` - The WebSocket URL of the MCP server (e.g., "ws://localhost:8080")
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("my-client".to_string(), "1.0.0".to_string());
    /// let init_result = client.connect_with_websocket("ws://localhost:8080").await?;
    /// println!("Protocol: {}", init_result.protocol.version);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "websocket")]
    pub async fn connect_with_websocket(&mut self, server_url: &str) -> McpResult<ConnectResult> {
        use crate::transport::websocket::WebSocketClientTransport;

        let transport = WebSocketClientTransport::new(server_url).await?;
        self.connect(transport).await
    }

    /// Connect to an MCP server and run interactive session over STDIO (convenience method)
    ///
    /// This is a convenience method that:
    /// 1. Creates a STDIO transport
    /// 2. Connects to the server
    /// 3. Runs an interactive session until Ctrl+C
    /// 4. smoothly disconnects
    ///
    /// # Arguments
    /// * `session_handler` - A closure that receives the connected client for operations
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    ///     let mut client = McpClient::new("my-client".to_string(), "1.0.0".to_string());
    ///
    ///     client.run_with_stdio("my-mcp-server", vec!["--verbose"], |_client| async move {
    ///         // Your client operations here
    ///         // Note: client operations would typically use _client parameter
    ///         println!("Client connected successfully");
    ///         Ok(())
    ///     }).await
    /// }
    /// ```
    #[cfg(feature = "stdio")]
    pub async fn run_with_stdio<F, Fut>(
        &mut self,
        command: &str,
        args: Vec<&str>,
        session_handler: F,
    ) -> McpResult<()>
    where
        F: FnOnce(&Self) -> Fut,
        Fut: std::future::Future<Output = McpResult<()>>,
    {
        // Connect with STDIO
        let init_result = self.connect_with_stdio(command, args).await?;
        if let Some(server_info) = &init_result.server_info {
            tracing::info!(
                "Connected to server: {} v{} using {}",
                server_info.name,
                server_info.version,
                init_result.protocol.version
            );
        } else {
            tracing::info!("Connected using {}", init_result.protocol.version);
        }

        // Set up Ctrl+C handler
        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            let _ = tx_clone.send(()).await;
        });

        // Run the session handler
        tokio::select! {
            result = session_handler(self) => {
                match result {
                    Ok(_) => tracing::info!("Session completed successfully"),
                    Err(e) => tracing::error!("Session error: {}", e),
                }
            }
            _ = rx.recv() => {
                tracing::info!("Shutdown signal received, disconnecting...");
            }
        }

        // Disconnect smoothly
        self.disconnect().await?;
        tracing::info!("Client disconnected");

        Ok(())
    }

    // ========================================================================
    // Streaming HTTP Transport Methods
    // ========================================================================

    #[cfg(feature = "chunked-encoding")]
    fn ensure_legacy_prism_streaming(&self) -> McpResult<()> {
        if self.config.protocol_mode != ProtocolMode::LegacyOnly {
            return Err(McpError::Transport(
                "Prism chunked/compressed endpoint helpers are legacy-only; use connect_with_http for standards-track MCP or set ProtocolMode::LegacyOnly explicitly"
                    .to_string(),
            ));
        }
        Ok(())
    }

    /// Connect to an MCP server with streaming HTTP transport for optimal efficiency
    ///
    /// This is a convenience method that:
    /// 1. Creates a streaming HTTP transport with smart content analysis
    /// 2. Connects to the server
    /// 3. Returns the initialization result
    ///
    /// Optimized for:
    /// - Large payload applications (>100KB)
    /// - Memory-constrained environments
    /// - High-performance requirements
    /// - Applications with mixed payload sizes
    ///
    /// Features:
    /// - Chunked transfer encoding for large payloads
    /// - Multiple compression algorithms (Gzip, Brotli, Zstd)
    /// - HTTP/2 Server Push support
    /// - smart content analysis
    /// - Automatic fallback to traditional HTTP
    ///
    /// # Arguments
    /// * `server_url` - The HTTP URL of the MCP server (e.g., "http://localhost:3000")
    /// * `config` - Streaming configuration options
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    /// use prism_mcp_rs::transport::StreamingConfig;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("data-processor".to_string(), "1.0.0".to_string());
    /// let config = StreamingConfig::performance_improved();
    /// let init = client.connect_with_chunked_encoding("http://localhost:3000", config).await?;
    /// println!("Connected with {}", init.protocol.version);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "chunked-encoding")]
    pub async fn connect_with_chunked_encoding(
        &mut self,
        server_url: &str,
        config: crate::transport::StreamingConfig,
    ) -> McpResult<ConnectResult> {
        use crate::transport::streaming_http::StreamingHttpClientTransport;

        self.ensure_legacy_prism_streaming()?;
        let transport = StreamingHttpClientTransport::with_config(server_url, config).await?;
        self.connect(transport).await
    }

    /// Connect with streaming HTTP using default configuration
    ///
    /// This is a convenience method that uses default streaming HTTP configuration
    /// improved for general use cases.
    ///
    /// # Arguments
    /// * `server_url` - The HTTP URL of the MCP server
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("my-app".to_string(), "1.0.0".to_string());
    /// let init = client.connect_with_chunked_encoding_default("http://localhost:3000").await?;
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "chunked-encoding")]
    pub async fn connect_with_chunked_encoding_default(
        &mut self,
        server_url: &str,
    ) -> McpResult<ConnectResult> {
        use crate::transport::streaming_http::StreamingHttpClientTransport;

        self.ensure_legacy_prism_streaming()?;
        let transport = StreamingHttpClientTransport::new(server_url).await?;
        self.connect(transport).await
    }

    /// Connect with memory-improved streaming HTTP configuration
    ///
    /// This configuration is improved for memory-constrained environments
    /// with smaller chunk sizes and conservative buffering.
    ///
    /// # Arguments
    /// * `server_url` - The HTTP URL of the MCP server
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("embedded-app".to_string(), "1.0.0".to_string());
    /// let init = client.connect_with_chunked_encoding_memory_improved("http://localhost:3000").await?;
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "chunked-encoding")]
    pub async fn connect_with_chunked_encoding_memory_improved(
        &mut self,
        server_url: &str,
    ) -> McpResult<ConnectResult> {
        use crate::transport::streaming_http::StreamingHttpClientTransport;
        use crate::transport::StreamingConfig;

        self.ensure_legacy_prism_streaming()?;
        let config = StreamingConfig::memory_improved();
        let transport = StreamingHttpClientTransport::with_config(server_url, config).await?;
        self.connect(transport).await
    }

    /// Connect with performance-improved streaming HTTP configuration
    ///
    /// This configuration is improved for high-performance scenarios
    /// with larger chunk sizes, multi-algorithm compression, and HTTP/2 features.
    ///
    /// # Arguments
    /// * `server_url` - The HTTP URL of the MCP server
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("high-perf-app".to_string(), "1.0.0".to_string());
    /// let init = client.connect_with_chunked_encoding_performance_improved("http://localhost:3000").await?;
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "chunked-encoding")]
    pub async fn connect_with_chunked_encoding_performance_improved(
        &mut self,
        server_url: &str,
    ) -> McpResult<ConnectResult> {
        use crate::transport::streaming_http::StreamingHttpClientTransport;
        use crate::transport::StreamingConfig;

        self.ensure_legacy_prism_streaming()?;
        let config = StreamingConfig::performance_improved();
        let transport = StreamingHttpClientTransport::with_config(server_url, config).await?;
        self.connect(transport).await
    }

    // ========================================================================
    // complete Transport Selection Guide
    // ========================================================================

    /// Choose the right transport automatically based on your use case
    ///
    /// This is a convenience method that selects the optimal transport based on
    /// your application characteristics. Use this if you want automatic selection.
    ///
    /// # Arguments
    /// * `use_case` - Your primary use case
    /// * `server_url` - Server URL (protocol will be adjusted automatically)
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    /// use prism_mcp_rs::client::TransportUseCase;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("my-app".to_string(), "1.0.0".to_string());
    /// let init = client.connect_with_recommended_transport(
    /// TransportUseCase::LargeDataProcessing,
    /// "http://localhost:3000"
    /// ).await?;
    /// Ok(())
    /// }
    /// ```
    #[allow(unused_variables)] // server_url may not be used depending on features
    pub async fn connect_with_recommended_transport(
        &mut self,
        use_case: TransportUseCase,
        server_url: &str,
    ) -> McpResult<ConnectResult> {
        match use_case {
            TransportUseCase::CommandLine
            | TransportUseCase::DesktopApp
            | TransportUseCase::Development => {
                // STDIO for command-line and desktop applications
                #[cfg(feature = "stdio")]
                {
                    self.connect_with_stdio_simple(server_url).await
                }
                #[cfg(not(feature = "stdio"))]
                {
                    Err(McpError::Transport(
                        "STDIO transport requested but feature not enabled".to_string(),
                    ))
                }
            }
            TransportUseCase::WebApplication
            | TransportUseCase::Mobile
            | TransportUseCase::Enterprise => {
                // HTTP for web applications, mobile, and enterprise environments
                #[cfg(feature = "http")]
                {
                    self.connect_with_http(server_url, None).await
                }
                #[cfg(not(feature = "http"))]
                {
                    Err(McpError::Connection(
                        "HTTP transport not available".to_string(),
                    ))
                }
            }
            TransportUseCase::LargeDataProcessing
            | TransportUseCase::MemoryConstrained
            | TransportUseCase::HighPerformance => {
                // Standard Streamable HTTP supports streaming responses while
                // remaining interoperable with conforming MCP peers.
                #[cfg(feature = "http")]
                {
                    self.connect_with_http(server_url, None).await
                }
                #[cfg(not(feature = "http"))]
                {
                    Err(McpError::Connection(
                        "HTTP transport not available".to_string(),
                    ))
                }
            }
            TransportUseCase::RealTime
            | TransportUseCase::HighFrequency
            | TransportUseCase::Interactive => {
                // MCP 2026 subscriptions are carried by standard HTTP SSE.
                #[cfg(feature = "http")]
                {
                    self.connect_with_http(server_url, None).await
                }
                #[cfg(not(feature = "http"))]
                {
                    Err(McpError::Connection(
                        "HTTP transport not available".to_string(),
                    ))
                }
            }
        }
    }

    /// Get transport recommendation for a use case (informational)
    ///
    /// Returns a human-readable recommendation for the best transport
    /// to use for a given use case.
    ///
    /// # Example
    /// ```rust
    /// use prism_mcp_rs::client::{McpClient, TransportUseCase};
    ///
    /// let client = McpClient::new("app".to_string(), "1.0.0".to_string());
    /// let recommendation = client.get_transport_recommendation(TransportUseCase::RealTime);
    /// println!("Recommendation: {}", recommendation);
    /// ```
    pub fn get_transport_recommendation(&self, use_case: TransportUseCase) -> &'static str {
        match use_case {
            TransportUseCase::CommandLine
            | TransportUseCase::DesktopApp
            | TransportUseCase::Development => {
                "STDIO Transport - complete for command-line tools, desktop apps, and local development. Direct process communication with zero network configuration."
            }
            TransportUseCase::WebApplication
            | TransportUseCase::Mobile
            | TransportUseCase::Enterprise => {
                "HTTP Transport - Ideal for web applications, mobile clients, and enterprise environments. Universal compatibility with firewalls and proxies."
            }
            TransportUseCase::LargeDataProcessing
            | TransportUseCase::MemoryConstrained
            | TransportUseCase::HighPerformance => {
                "Standard Streamable HTTP - interoperable MCP streaming with HTTP/2-capable clients, proxy compatibility, and subscription SSE."
            }
            TransportUseCase::RealTime
            | TransportUseCase::HighFrequency
            | TransportUseCase::Interactive => {
                "Standard Streamable HTTP subscriptions - interoperable real-time notifications over request-scoped SSE."
            }
        }
    }

    /// Get detailed transport comparison for decision making
    ///
    /// Returns detailed information about all available transports
    /// to help with transport selection decisions.
    ///
    /// # Example
    /// ```rust
    /// use prism_mcp_rs::client::McpClient;
    ///
    /// let client = McpClient::new("app".to_string(), "1.0.0".to_string());
    /// let comparison = client.get_transport_comparison();
    /// for transport in comparison {
    /// println!("{}: {}", transport.name, transport.description);
    /// }
    /// ```
    pub fn get_transport_comparison(&self) -> Vec<TransportInfo> {
        vec![
            TransportInfo {
                name: "STDIO".to_string(),
                description: "Direct process communication - spawn and communicate with MCP servers as child processes".to_string(),
                use_cases: vec!["Command-line tools".to_string(), "Desktop applications".to_string(), "Local development".to_string()],
                pros: vec!["Zero network configuration".to_string(), "Direct process lifecycle management".to_string(), "Highest security (local only)".to_string()],
                cons: vec!["Local only".to_string(), "Requires process spawning".to_string()],
                latency: "<1ms".to_string(),
                throughput: "High".to_string(),
                available: cfg!(feature = "stdio"),
            },
            TransportInfo {
                name: "HTTP".to_string(),
                description: "Traditional HTTP/1.1 with Server-Sent Events for notifications".to_string(),
                use_cases: vec!["Web applications".to_string(), "Mobile clients".to_string(), "Enterprise environments".to_string()],
                pros: vec!["Universal compatibility".to_string(), "Firewall friendly".to_string(), "Simple debugging".to_string()],
                cons: vec!["Higher latency".to_string(), "Request/response only".to_string()],
                latency: "10-50ms".to_string(),
                throughput: "Medium".to_string(),
                available: cfg!(feature = "http"),
            },
            TransportInfo {
                name: "WebSocket".to_string(),
                description: "Full-duplex real-time communication with automatic reconnection".to_string(),
                use_cases: vec!["Real-time applications".to_string(), "Live collaboration".to_string(), "High-frequency messaging".to_string()],
                pros: vec!["Lowest latency".to_string(), "Full-duplex".to_string(), "Real-time notifications".to_string()],
                cons: vec!["More complex".to_string(), "Firewall issues possible".to_string()],
                latency: "<5ms".to_string(),
                throughput: "High".to_string(),
                available: cfg!(feature = "websocket"),
            },
            TransportInfo {
                name: "Streaming HTTP".to_string(),
                description: "complete HTTP with chunked streaming, smart content analysis, and compression".to_string(),
                use_cases: vec!["Large data processing".to_string(), "Memory-constrained environments".to_string(), "High-performance applications".to_string()],
                pros: vec!["Memory efficient".to_string(), "complete compression (Gzip/Brotli/Zstd)".to_string(), "smart content analysis".to_string(), "Adaptive buffering".to_string()],
                cons: vec!["More complexity".to_string(), "Requires chunked-encoding feature".to_string()],
                latency: "10-30ms".to_string(),
                throughput: "Very High".to_string(),
                available: cfg!(feature = "chunked-encoding"),
            },
        ]
    }

    // ========================================================================
    // Tool Operations
    // ========================================================================

    /// List available tools from the server
    pub async fn list_tools(&self, cursor: Option<String>) -> McpResult<ListToolsResult> {
        self.ensure_connected().await?;

        let params = ListToolsParams { cursor, meta: None };
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::TOOLS_LIST.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    /// Call a tool on the server
    pub async fn call_tool(
        &self,
        name: String,
        arguments: Option<HashMap<String, Value>>,
    ) -> McpResult<CallToolResult> {
        self.ensure_connected().await?;

        let params = if let Some(args) = arguments {
            CallToolParams::new_with_arguments(name, args)
        } else {
            CallToolParams::new(name)
        };

        if self.config.validate_requests {
            validate_call_tool_params(&params)?;
        }

        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::TOOLS_CALL.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    // ========================================================================
    // Resource Operations
    // ========================================================================

    /// List available resources from the server
    pub async fn list_resources(&self, cursor: Option<String>) -> McpResult<ListResourcesResult> {
        self.ensure_connected().await?;

        let params = ListResourcesParams { cursor, meta: None };
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::RESOURCES_LIST.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    /// Read a resource from the server
    pub async fn read_resource(&self, uri: String) -> McpResult<ReadResourceResult> {
        self.ensure_connected().await?;

        let params = ReadResourceParams::new(uri);

        if self.config.validate_requests {
            validate_read_resource_params(&params)?;
        }

        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::RESOURCES_READ.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    /// Subscribe to resource updates
    pub async fn subscribe_resource(&self, uri: String) -> McpResult<SubscribeResourceResult> {
        self.ensure_connected().await?;

        let params = SubscribeResourceParams { uri, meta: None };
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::RESOURCES_SUBSCRIBE.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    /// Unsubscribe from resource updates
    pub async fn unsubscribe_resource(&self, uri: String) -> McpResult<UnsubscribeResourceResult> {
        self.ensure_connected().await?;

        let params = UnsubscribeResourceParams { uri, meta: None };
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::RESOURCES_UNSUBSCRIBE.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    /// List resource templates from the server (New in 2025-11-25)
    pub async fn list_resource_templates(
        &self,
        cursor: Option<String>,
    ) -> McpResult<ListResourceTemplatesResult> {
        self.ensure_connected().await?;

        let params = ListResourceTemplatesParams { cursor, meta: None };
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::RESOURCES_TEMPLATES_LIST.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    // ========================================================================
    // Completion Operations (New in 2025-11-25)
    // ========================================================================

    /// Request completion suggestions for an argument
    pub async fn complete_argument(
        &self,
        reference: CompletionReference,
        argument: CompletionArgument,
    ) -> McpResult<CompleteResult> {
        self.ensure_connected().await?;

        // Check if server supports completion
        {
            let server_capabilities = self.server_capabilities.read().await;
            if let Some(capabilities) = server_capabilities.as_ref() {
                if capabilities.completions.is_none() {
                    return Err(McpError::MethodNotFound(
                        "Server does not support completion".to_string(),
                    ));
                }
            }
        }

        let params = CompleteParams {
            reference,
            argument,
            meta: None,
        };

        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::COMPLETION_COMPLETE.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    /// Convenience method for prompt argument completion
    pub async fn complete_prompt_argument(
        &self,
        prompt_name: &str,
        argument_name: &str,
        current_value: &str,
    ) -> McpResult<Vec<String>> {
        let reference = CompletionReference::Prompt {
            name: prompt_name.to_string(),
        };

        let argument = CompletionArgument {
            name: argument_name.to_string(),
            value: current_value.to_string(),
        };

        let result = self.complete_argument(reference, argument).await?;
        Ok(result.completion.values)
    }

    /// Convenience method for resource URI completion
    pub async fn complete_resource_uri(
        &self,
        uri_template: &str,
        argument_name: &str,
        current_value: &str,
    ) -> McpResult<Vec<String>> {
        let reference = CompletionReference::Resource {
            uri: uri_template.to_string(),
        };

        let argument = CompletionArgument {
            name: argument_name.to_string(),
            value: current_value.to_string(),
        };

        let result = self.complete_argument(reference, argument).await?;
        Ok(result.completion.values)
    }

    /// Convenience method for tool argument completion
    pub async fn complete_tool_argument(
        &self,
        tool_name: &str,
        argument_name: &str,
        current_value: &str,
    ) -> McpResult<Vec<String>> {
        let reference = CompletionReference::Tool {
            name: tool_name.to_string(),
        };

        let argument = CompletionArgument {
            name: argument_name.to_string(),
            value: current_value.to_string(),
        };

        let result = self.complete_argument(reference, argument).await?;
        Ok(result.completion.values)
    }

    // ========================================================================
    // Prompt Operations
    // ========================================================================

    /// List available prompts from the server
    pub async fn list_prompts(&self, cursor: Option<String>) -> McpResult<ListPromptsResult> {
        self.ensure_connected().await?;

        let params = ListPromptsParams { cursor, meta: None };
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::PROMPTS_LIST.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    /// Get a prompt from the server
    pub async fn get_prompt(
        &self,
        name: String,
        arguments: Option<HashMap<String, String>>,
    ) -> McpResult<GetPromptResult> {
        self.ensure_connected().await?;

        let params = if let Some(args) = arguments {
            GetPromptParams::new_with_arguments(name, args)
        } else {
            GetPromptParams::new(name)
        };

        if self.config.validate_requests {
            validate_get_prompt_params(&params)?;
        }

        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::PROMPTS_GET.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    // ========================================================================
    // Sampling Operations (if supported by server)
    // ========================================================================

    /// Create a message using server-side sampling
    pub async fn create_message(
        &self,
        params: CreateMessageParams,
    ) -> McpResult<CreateMessageResult> {
        self.ensure_connected().await?;

        // Check if server supports sampling
        {
            let server_capabilities = self.server_capabilities.read().await;
            if let Some(capabilities) = server_capabilities.as_ref() {
                if capabilities.sampling.is_none() {
                    return Err(McpError::Protocol(
                        "Server does not support sampling".to_string(),
                    ));
                }
            } else {
                return Err(McpError::Protocol("Not connected to server".to_string()));
            }
        }

        if self.config.validate_requests {
            validate_create_message_params(&params)?;
        }

        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::SAMPLING_CREATE_MESSAGE.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    // ========================================================================
    // Utility Operations
    // ========================================================================

    /// Send a ping to the server
    pub async fn ping(&self) -> McpResult<PingResult> {
        self.ensure_connected().await?;

        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::PING.to_string(),
            Some(PingParams { meta: None }),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    /// Set the logging level on the server
    pub async fn set_logging_level(&self, level: LoggingLevel) -> McpResult<SetLoggingLevelResult> {
        self.ensure_connected().await?;

        let params = SetLoggingLevelParams { level, meta: None };
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::LOGGING_SET_LEVEL.to_string(),
            Some(params),
        )?;

        let response = self.send_request(request).await?;
        self.handle_response(response)
    }

    // ========================================================================
    // Tasks extension
    // ========================================================================

    async fn ensure_tasks_extension(&self) -> McpResult<()> {
        if !has_tasks_extension(&self.capabilities) {
            return Err(McpError::MissingRequiredClientCapability(
                serde_json::json!({"extensions": {(TASKS_EXTENSION_ID): {}}}),
            ));
        }
        if self
            .negotiated_protocol
            .read()
            .await
            .as_ref()
            .is_none_or(|protocol| protocol.era != ProtocolEra::Modern)
        {
            return Err(McpError::MethodNotFound(format!(
                "the Tasks extension requires MCP {MODERN_PROTOCOL_VERSION}"
            )));
        }
        let server_supports_tasks = self
            .server_capabilities
            .read()
            .await
            .as_ref()
            .and_then(|capabilities| capabilities.extensions.as_ref())
            .is_some_and(|extensions| extensions.contains_key(TASKS_EXTENSION_ID));
        if !server_supports_tasks {
            return Err(McpError::MethodNotFound(
                "the server did not advertise io.modelcontextprotocol/tasks".to_string(),
            ));
        }
        Ok(())
    }

    /// Retrieve the current state of a durable task.
    pub async fn get_task(&self, task_id: impl Into<String>) -> McpResult<Task> {
        self.ensure_connected().await?;
        self.ensure_tasks_extension().await?;
        let result: GetTaskResult = self
            .send_task_request(
                methods::TASKS_GET,
                GetTaskParams {
                    task_id: task_id.into(),
                    meta: HashMap::new(),
                },
            )
            .await?;
        result.task.validate().map_err(McpError::Protocol)?;
        Ok(result.task)
    }

    /// Submit responses for currently outstanding task input requests.
    pub async fn update_task(
        &self,
        task_id: impl Into<String>,
        input_responses: HashMap<String, Value>,
    ) -> McpResult<()> {
        self.ensure_connected().await?;
        self.ensure_tasks_extension().await?;
        let _: TaskAcknowledgement = self
            .send_task_request(
                methods::TASKS_UPDATE,
                UpdateTaskParams {
                    task_id: task_id.into(),
                    input_responses,
                    meta: HashMap::new(),
                },
            )
            .await?;
        Ok(())
    }

    /// Signal cooperative cancellation of a task.
    pub async fn cancel_task(&self, task_id: impl Into<String>) -> McpResult<()> {
        self.ensure_connected().await?;
        self.ensure_tasks_extension().await?;
        let _: TaskAcknowledgement = self
            .send_task_request(
                methods::TASKS_CANCEL,
                CancelTaskParams {
                    task_id: task_id.into(),
                    meta: HashMap::new(),
                },
            )
            .await?;
        Ok(())
    }

    // ========================================================================
    // Notification Handling
    // ========================================================================

    /// Receive notifications from the server
    pub async fn receive_notification(&self) -> McpResult<Option<JsonRpcNotification>> {
        let mut transport_guard = self.transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            transport.receive_notification().await
        } else {
            Err(McpError::Transport("Not connected".to_string()))
        }
    }

    /// Open a standards-track MCP 2026 notification stream.
    pub async fn listen(
        &self,
        notifications: crate::protocol::SubscriptionFilter,
    ) -> McpResult<ClientSubscription> {
        self.ensure_connected().await?;
        if self
            .negotiated_protocol
            .read()
            .await
            .as_ref()
            .is_none_or(|protocol| protocol.era != ProtocolEra::Modern)
        {
            return Err(McpError::MethodNotFound(
                "subscriptions/listen requires MCP 2026-07-28".to_string(),
            ));
        }
        if notifications.requests_tasks() {
            self.ensure_tasks_extension().await?;
        }
        let mut request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::SUBSCRIPTIONS_LISTEN.to_string(),
            Some(crate::protocol::SubscriptionsListenParams {
                notifications,
                meta: HashMap::new(),
            }),
        )?;
        decorate_modern_request(&mut request, &self.info, &self.capabilities)?;
        let mut transport = self.transport.lock().await;
        transport
            .as_mut()
            .ok_or_else(|| McpError::Transport("Not connected".to_string()))?
            .open_subscription(request)
            .await
    }

    /// Close an open subscription using transport-appropriate semantics.
    pub async fn cancel_subscription(&self, subscription: &ClientSubscription) -> McpResult<()> {
        let mut transport = self.transport.lock().await;
        transport
            .as_mut()
            .ok_or_else(|| McpError::Transport("Not connected".to_string()))?
            .cancel_subscription(subscription.id())
            .await
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Send a request and get a response
    async fn send_request(&self, mut request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let modern = request.method == methods::SERVER_DISCOVER
            || self
                .negotiated_protocol
                .read()
                .await
                .as_ref()
                .is_some_and(|protocol| protocol.era == ProtocolEra::Modern);
        if modern && is_legacy_only_method(&request.method) {
            return Err(McpError::MethodNotFound(format!(
                "{} is not part of MCP {MODERN_PROTOCOL_VERSION}",
                request.method
            )));
        }

        let mut round = 0_u8;
        loop {
            if modern {
                decorate_modern_request(&mut request, &self.info, &self.capabilities)?;
            }
            if self.config.validate_requests {
                validate_jsonrpc_request(&request)?;
                validate_mcp_request(&request.method, request.params.as_ref())?;
            }

            let mut response = {
                let mut transport_guard = self.transport.lock().await;
                let transport = transport_guard
                    .as_mut()
                    .ok_or_else(|| McpError::Transport("Not connected".to_string()))?;
                transport.send_request(request.clone()).await?
            };

            if self.config.validate_responses {
                validate_jsonrpc_response(&response)?;
            }
            if !modern {
                return Ok(response);
            }

            let result = response.result.as_ref().ok_or_else(|| {
                McpError::Protocol("modern response is missing a result".to_string())
            })?;
            match result.get("resultType").and_then(Value::as_str) {
                Some("complete") => return Ok(response),
                Some("task") if request.method == methods::TOOLS_CALL => {
                    self.ensure_tasks_extension().await?;
                    let created: CreateTaskResult = serde_json::from_value(result.clone())?;
                    created.task.validate().map_err(McpError::Protocol)?;
                    let completed = self.drive_task(created.task).await?;
                    response.result = Some(completed);
                    return Ok(response);
                }
                Some("task") => {
                    return Err(McpError::Protocol(format!(
                        "resultType task is invalid for {}",
                        request.method
                    )))
                }
                Some("input_required") => {
                    round = round.saturating_add(1);
                    if round > self.config.max_mrtr_rounds {
                        return Err(McpError::Protocol(format!(
                            "MCP input_required exceeded {} rounds",
                            self.config.max_mrtr_rounds
                        )));
                    }
                    let input: InputRequiredResult = serde_json::from_value(result.clone())?;
                    let mut input_responses = serde_json::Map::new();
                    for (key, input_request) in input.input_requests {
                        let response = self.fulfill_input_request(input_request).await?;
                        input_responses.insert(key, response);
                    }

                    let params = request
                        .params
                        .get_or_insert_with(|| Value::Object(serde_json::Map::new()))
                        .as_object_mut()
                        .ok_or_else(|| {
                            McpError::Protocol("MRTR request params must be an object".to_string())
                        })?;
                    if !input_responses.is_empty() {
                        params.insert("inputResponses".to_string(), Value::Object(input_responses));
                    }
                    if let Some(request_state) = input.request_state {
                        params.insert("requestState".to_string(), Value::String(request_state));
                    }
                    request.id = Value::from(self.next_request_id().await);
                }
                Some(other) => {
                    return Err(McpError::Protocol(format!(
                        "unsupported MCP resultType: {other}"
                    )))
                }
                None => {
                    return Err(McpError::Protocol(
                        "MCP 2026 response is missing resultType".to_string(),
                    ))
                }
            }
        }
    }

    async fn send_task_request<P, T>(&self, method: &str, params: P) -> McpResult<T>
    where
        P: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        let mut request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            method.to_string(),
            Some(params),
        )?;
        decorate_modern_request(&mut request, &self.info, &self.capabilities)?;
        if self.config.validate_requests {
            validate_jsonrpc_request(&request)?;
            validate_mcp_request(&request.method, request.params.as_ref())?;
        }
        let response = {
            let mut transport_guard = self.transport.lock().await;
            transport_guard
                .as_mut()
                .ok_or_else(|| McpError::Transport("Not connected".to_string()))?
                .send_request(request)
                .await?
        };
        if self.config.validate_responses {
            validate_jsonrpc_response(&response)?;
        }
        self.handle_response(response)
    }

    async fn drive_task(&self, mut task: Task) -> McpResult<Value> {
        loop {
            match task.status {
                TaskStatus::Completed => {
                    let mut result = task.result.ok_or_else(|| {
                        McpError::Protocol("completed task is missing result".to_string())
                    })?;
                    if let Some(object) = result.as_object_mut() {
                        object
                            .entry("resultType")
                            .or_insert_with(|| Value::String("complete".to_string()));
                    }
                    return Ok(result);
                }
                TaskStatus::Failed => {
                    return Err(McpError::Protocol(format!(
                        "task {} failed: {}",
                        task.task_id,
                        task.error
                            .as_ref()
                            .map(Value::to_string)
                            .unwrap_or_else(|| "unknown error".to_string())
                    )))
                }
                TaskStatus::Cancelled => {
                    return Err(McpError::Cancelled(format!(
                        "task {} was cancelled",
                        task.task_id
                    )))
                }
                TaskStatus::InputRequired => {
                    let mut responses = HashMap::new();
                    for (key, input_request) in &task.input_requests {
                        responses.insert(
                            key.clone(),
                            self.fulfill_input_request(input_request.clone()).await?,
                        );
                    }
                    self.update_task(task.task_id.clone(), responses).await?;
                }
                TaskStatus::Working => {}
            }
            let delay = task.poll_interval_ms.unwrap_or(1_000).max(1);
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            task = self.get_task(task.task_id.clone()).await?;
        }
    }

    async fn fulfill_input_request(&self, input_request: Value) -> McpResult<Value> {
        let object = input_request.as_object().ok_or_else(|| {
            McpError::Protocol("MRTR input request must be an object".to_string())
        })?;
        let method = object
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                McpError::Protocol("MRTR input request is missing method".to_string())
            })?;
        let params = object
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        match method {
            methods::SAMPLING_CREATE_MESSAGE => {
                let params: CreateMessageParams = serde_json::from_value(params)?;
                Ok(serde_json::to_value(
                    self.request_handler.handle_create_message(params).await?,
                )?)
            }
            methods::ROOTS_LIST => {
                let params: ListRootsParams = serde_json::from_value(params)?;
                Ok(serde_json::to_value(
                    self.request_handler.handle_list_roots(params).await?,
                )?)
            }
            methods::ELICITATION_CREATE => {
                let params: ElicitParams = serde_json::from_value(params)?;
                Ok(serde_json::to_value(
                    self.request_handler.handle_elicit(params).await?,
                )?)
            }
            _ => Err(McpError::MethodNotFound(format!(
                "unsupported MRTR input method: {method}"
            ))),
        }
    }

    /// Handle a JSON-RPC response and extract the result
    fn handle_response<T>(&self, response: JsonRpcResponse) -> McpResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // JsonRpcResponse only contains successful responses
        // Errors are handled separately by the transport layer
        let result = response
            .result
            .ok_or_else(|| McpError::Protocol("Missing result in response".to_string()))?;

        serde_json::from_value(result).map_err(|e| McpError::Serialization(e.to_string()))
    }

    /// Check client is connected
    async fn ensure_connected(&self) -> McpResult<()> {
        if !self.is_connected().await {
            return Err(McpError::Connection("Not connected to server".to_string()));
        }
        Ok(())
    }

    /// Get the next request ID
    async fn next_request_id(&self) -> u64 {
        let mut counter = self.request_counter.lock().await;
        *counter += 1;
        *counter
    }
}

/// Client builder for easier construction
pub struct McpClientBuilder {
    name: String,
    version: String,
    capabilities: ClientCapabilities,
    config: ClientConfig,
}

impl McpClientBuilder {
    /// Create a new client builder
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            capabilities: ClientCapabilities::default(),
            config: ClientConfig::default(),
        }
    }

    /// Set client capabilities
    pub fn capabilities(mut self, capabilities: ClientCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Set client configuration
    pub fn config(mut self, config: ClientConfig) -> Self {
        self.config = config;
        self
    }

    /// Set request timeout
    pub fn request_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.request_timeout_ms = timeout_ms;
        self
    }

    /// Set maximum retries
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Enable or disable request validation
    pub fn validate_requests(mut self, validate: bool) -> Self {
        self.config.validate_requests = validate;
        self
    }

    /// Enable or disable response validation
    pub fn validate_responses(mut self, validate: bool) -> Self {
        self.config.validate_responses = validate;
        self
    }

    /// Build the client
    pub fn build(self) -> McpClient {
        let mut client = McpClient::new(self.name, self.version);
        client.set_capabilities(self.capabilities);
        client.config = self.config;
        client
    }
}

/// Transport use case guide for automatic selection
///
/// This enum helps you choose the right transport by describing your
/// primary use case. The client can then automatically select the
/// most appropriate transport configuration.
///
/// # Examples
/// ```rust
/// use prism_mcp_rs::client::TransportUseCase;
///
/// let use_case = TransportUseCase::RealTime; // Will choose WebSocket
/// let use_case = TransportUseCase::CommandLine; // Will choose STDIO
/// let use_case = TransportUseCase::Enterprise; // Will choose HTTP
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportUseCase {
    /// Command-line tools and scripts - uses STDIO transport
    CommandLine,
    /// Desktop applications - uses STDIO transport
    DesktopApp,
    /// Local development and testing - uses STDIO transport
    Development,
    /// Web applications and browser integration - uses HTTP transport
    WebApplication,
    /// Mobile applications with battery constraints - uses HTTP transport
    Mobile,
    /// Enterprise environments with firewall restrictions - uses HTTP transport
    Enterprise,
    /// Applications processing large datasets - uses improved transport
    LargeDataProcessing,
    /// Memory-constrained environments - uses streaming transport
    MemoryConstrained,
    /// High-performance applications - uses improved transport
    HighPerformance,
    /// Real-time applications requiring low latency - uses WebSocket transport
    RealTime,
    /// High-frequency message exchange - uses WebSocket transport
    HighFrequency,
    /// Interactive applications and live collaboration - uses WebSocket transport
    Interactive,
}

/// Information about a transport for comparison
///
/// detailed information about transport characteristics
/// to help with selection decisions.
#[derive(Debug, Clone)]
pub struct TransportInfo {
    /// Transport name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Primary use cases
    pub use_cases: Vec<String>,
    /// Advantages
    pub pros: Vec<String>,
    /// Disadvantages
    pub cons: Vec<String>,
    /// Typical latency range
    pub latency: String,
    /// Throughput characteristics
    pub throughput: String,
    /// Whether this transport is available (compiled in)
    pub available: bool,
}

/// Default transport recommendations for different use cases
impl Default for TransportUseCase {
    fn default() -> Self {
        TransportUseCase::Development
    }
}

impl std::fmt::Display for TransportUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportUseCase::CommandLine => write!(f, "Command-line tool"),
            TransportUseCase::DesktopApp => write!(f, "Desktop application"),
            TransportUseCase::Development => write!(f, "Development and testing"),
            TransportUseCase::WebApplication => write!(f, "Web application"),
            TransportUseCase::Mobile => write!(f, "Mobile application"),
            TransportUseCase::Enterprise => write!(f, "Enterprise environment"),
            TransportUseCase::LargeDataProcessing => write!(f, "Large data processing"),
            TransportUseCase::MemoryConstrained => write!(f, "Memory-constrained environment"),
            TransportUseCase::HighPerformance => write!(f, "High-performance application"),
            TransportUseCase::RealTime => write!(f, "Real-time application"),
            TransportUseCase::HighFrequency => write!(f, "High-frequency messaging"),
            TransportUseCase::Interactive => write!(f, "Interactive application"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock transport for testing
    struct MockTransport {
        responses: Vec<JsonRpcResponse>,
        current: usize,
    }

    impl MockTransport {
        fn new(responses: Vec<JsonRpcResponse>) -> Self {
            Self {
                responses,
                current: 0,
            }
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn send_request(&mut self, _request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
            if self.current < self.responses.len() {
                let response = self.responses[self.current].clone();
                self.current += 1;
                Ok(response)
            } else {
                Err(McpError::Transport("No more responses".to_string()))
            }
        }

        async fn send_notification(&mut self, _notification: JsonRpcNotification) -> McpResult<()> {
            Ok(())
        }

        async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>> {
            Ok(None)
        }

        async fn close(&mut self) -> McpResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_client_builder() {
        let client = McpClientBuilder::new("test-client".to_string(), "1.0.0".to_string())
            .request_timeout(5000)
            .max_retries(5)
            .validate_requests(false)
            .build();

        assert_eq!(client.config().request_timeout_ms, 5000);
        assert_eq!(client.config().max_retries, 5);
        assert!(!client.config().validate_requests);
    }

    #[tokio::test]
    async fn test_mock_connection() {
        let init_result = InitializeResult::new(
            crate::protocol::LATEST_PROTOCOL_VERSION.to_string(),
            ServerCapabilities::default(),
            ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                title: Some("Test Server".to_string()),
                website_url: None,
                icons: None,
            },
        );

        let init_response = JsonRpcResponse::success(Value::from(1), init_result.clone()).unwrap();

        let transport = MockTransport::new(vec![init_response]);

        let mut client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        client.set_protocol_mode(ProtocolMode::LegacyOnly);
        let result = client.connect(transport).await.unwrap();

        assert_eq!(result.server_info.unwrap().name, "test-server");
        assert!(client.is_connected().await);
    }

    #[tokio::test]
    async fn test_disconnect() {
        let init_result = InitializeResult::new(
            crate::protocol::LATEST_PROTOCOL_VERSION.to_string(),
            ServerCapabilities::default(),
            ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                title: Some("Test Server".to_string()),
                website_url: None,
                icons: None,
            },
        );

        let init_response = JsonRpcResponse::success(Value::from(1), init_result).unwrap();

        let transport = MockTransport::new(vec![init_response]);

        let mut client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        client.set_protocol_mode(ProtocolMode::LegacyOnly);
        client.connect(transport).await.unwrap();

        assert!(client.is_connected().await);

        client.disconnect().await.unwrap();
        assert!(!client.is_connected().await);
        assert!(client.server_info().await.is_none());
        assert!(client.server_capabilities().await.is_none());
    }
}

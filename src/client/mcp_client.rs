// ! MCP client implementation
// !
// ! Module provides the main MCP client that can connect to MCP servers,
// ! initialize connections, and perform operations like calling tools, reading resources,
// ! and executing prompts according to the Model Context Protocol specification.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::client::request_handler::{ClientRequestHandler, DefaultClientRequestHandler};
use crate::core::error::{McpError, McpResult};
use crate::protocol::{messages::*, methods, types::*, validation::*};
use crate::transport::traits::Transport;

/// Configuration for the MCP client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Whether to validate all outgoing requests
    pub validate_requests: bool,
    /// Whether to validate all incoming responses
    pub validate_responses: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            request_timeout_ms: 30000,
            max_retries: 3,
            retry_delay_ms: 1000,
            validate_requests: true,
            validate_responses: true,
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
}

impl McpClient {
    /// Create a new MCP client with the given name and version
    pub fn new(name: String, version: String) -> Self {
        Self {
            info: ClientInfo::new(name, version),
            capabilities: ClientCapabilities::default(),
            config: ClientConfig::default(),
            transport: Arc::new(Mutex::new(None)),
            server_capabilities: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            request_counter: Arc::new(Mutex::new(0)),
            connected: Arc::new(RwLock::new(false)),
            request_handler: Arc::new(DefaultClientRequestHandler),
        }
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
    pub async fn connect<T>(&mut self, transport: T) -> McpResult<InitializeResult>
    where
        T: Transport + 'static,
    {
        // Set the transport
        {
            let mut transport_guard = self.transport.lock().await;
            *transport_guard = Some(Box::new(transport));
        }

        // Initialize the connection
        let init_result = self.initialize().await?;

        // Mark as connected
        {
            let mut connected = self.connected.write().await;
            *connected = true;
        }

        Ok(init_result)
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

        Ok(())
    }

    /// Initialize the connection with the server
    async fn initialize(&self) -> McpResult<InitializeResult> {
        let params = InitializeParams::new(
            crate::protocol::LATEST_PROTOCOL_VERSION.to_string(),
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

        Ok(result)
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
    /// println!("Connected to {}", init_result.server_info.name);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "stdio")]
    pub async fn connect_with_stdio(
        &mut self,
        command: &str,
        args: Vec<&str>,
    ) -> McpResult<InitializeResult> {
        use crate::transport::stdio::StdioClientTransport;

        let transport = StdioClientTransport::new(command, args).await?;
        self.connect(transport).await
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
    /// println!("Connected to {}", init_result.server_info.name);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "http")]
    pub async fn connect_with_http(
        &mut self,
        server_url: &str,
        sse_url: Option<&str>,
    ) -> McpResult<InitializeResult> {
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
    /// println!("Connected to {}", init_result.server_info.name);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "stdio")]
    pub async fn connect_with_stdio_simple(
        &mut self,
        command: &str,
    ) -> McpResult<InitializeResult> {
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
    /// println!("Connected to {}", init_result.server_info.name);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "websocket")]
    pub async fn connect_with_websocket(
        &mut self,
        server_url: &str,
    ) -> McpResult<InitializeResult> {
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
    /// ```rust,ignore
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut client = McpClient::new("my-client".to_string(), "1.0.0".to_string());
    ///
    /// client.run_with_stdio("my-mcp-server", vec!["--verbose"], |client| async move {
    /// // Your client operations here
    /// let tools = client.list_tools(None).await?;
    /// println!("Available tools: {:?}", tools);
    /// Ok(())
    /// }).await
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
        tracing::info!(
            "Connected to server: {} v{}",
            init_result.server_info.name,
            init_result.server_info.version
        );

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

    /// Connect to an MCP server with streaming HTTP transport - complete efficiency
    ///
    /// This is a convenience method that:
    /// 1. Creates a streaming HTTP transport with smart content analysis
    /// 2. Connects to the server
    /// 3. Returns the initialization result
    ///
    /// complete for:
    /// - Large payload applications (>100KB)
    /// - Memory-constrained environments
    /// - High-performance requirements
    /// - Applications with mixed payload sizes
    ///
    /// Features:
    /// - Chunked transfer encoding for large payloads
    /// - complete compression (Gzip, Brotli, Zstd)
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
    /// let init = client.connect_with_streaming_http("http://localhost:3000", config).await?;
    /// println!("Connected with streaming HTTP to {}", init.server_info.name);
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "streaming-http")]
    pub async fn connect_with_streaming_http(
        &mut self,
        server_url: &str,
        config: crate::transport::StreamingConfig,
    ) -> McpResult<InitializeResult> {
        use crate::transport::streaming_http::StreamingHttpClientTransport;

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
    /// let init = client.connect_with_streaming_http_default("http://localhost:3000").await?;
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "streaming-http")]
    pub async fn connect_with_streaming_http_default(
        &mut self,
        server_url: &str,
    ) -> McpResult<InitializeResult> {
        use crate::transport::streaming_http::StreamingHttpClientTransport;

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
    /// let init = client.connect_with_streaming_http_memory_improved("http://localhost:3000").await?;
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "streaming-http")]
    pub async fn connect_with_streaming_http_memory_improved(
        &mut self,
        server_url: &str,
    ) -> McpResult<InitializeResult> {
        use crate::transport::StreamingConfig;
        use crate::transport::streaming_http::StreamingHttpClientTransport;

        let config = StreamingConfig::memory_improved();
        let transport = StreamingHttpClientTransport::with_config(server_url, config).await?;
        self.connect(transport).await
    }

    /// Connect with performance-improved streaming HTTP configuration
    ///
    /// This configuration is improved for high-performance scenarios
    /// with larger chunk sizes, complete compression, and HTTP/2 features.
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
    /// let init = client.connect_with_streaming_http_performance_improved("http://localhost:3000").await?;
    /// Ok(())
    /// }
    /// ```
    #[cfg(feature = "streaming-http")]
    pub async fn connect_with_streaming_http_performance_improved(
        &mut self,
        server_url: &str,
    ) -> McpResult<InitializeResult> {
        use crate::transport::StreamingConfig;
        use crate::transport::streaming_http::StreamingHttpClientTransport;

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
    ) -> McpResult<InitializeResult> {
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
                    return Err(McpError::Transport(
                        "STDIO transport requested but feature not enabled".to_string(),
                    ));
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
                // Streaming HTTP transport for complete efficiency
                #[cfg(feature = "streaming-http")]
                {
                    match use_case {
                        TransportUseCase::MemoryConstrained => {
                            tracing::info!("Using memory-improved streaming HTTP transport");
                            self.connect_with_streaming_http_memory_improved(server_url)
                                .await
                        }
                        TransportUseCase::HighPerformance
                        | TransportUseCase::LargeDataProcessing => {
                            tracing::info!("Using performance-improved streaming HTTP transport");
                            self.connect_with_streaming_http_performance_improved(server_url)
                                .await
                        }
                        _ => self.connect_with_streaming_http_default(server_url).await,
                    }
                }
                #[cfg(not(feature = "streaming-http"))]
                {
                    tracing::warn!(
                        "Streaming HTTP requested but feature not enabled, using traditional HTTP"
                    );
                    #[cfg(feature = "http")]
                    {
                        self.connect_with_http(server_url, None).await
                    }
                    #[cfg(not(feature = "http"))]
                    {
                        Err(McpError::Connection(
                            "No suitable transport available".to_string(),
                        ))
                    }
                }
            }
            TransportUseCase::RealTime
            | TransportUseCase::HighFrequency
            | TransportUseCase::Interactive => {
                // WebSocket for real-time communication
                #[cfg(feature = "websocket")]
                {
                    let ws_url = server_url
                        .replace("http://", "ws://")
                        .replace("https://", "wss://");
                    self.connect_with_websocket(&ws_url).await
                }
                #[cfg(not(feature = "websocket"))]
                {
                    tracing::warn!("WebSocket requested but feature not enabled, using HTTP");
                    #[cfg(feature = "http")]
                    {
                        self.connect_with_http(server_url, None).await
                    }
                    #[cfg(not(feature = "http"))]
                    {
                        Err(McpError::Connection(
                            "No suitable transport available".to_string(),
                        ))
                    }
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
                "Streaming HTTP Transport - improved for large payloads and memory efficiency. complete chunking, compression (Gzip/Brotli/Zstd), and smart content analysis."
            }
            TransportUseCase::RealTime
            | TransportUseCase::HighFrequency
            | TransportUseCase::Interactive => {
                "WebSocket Transport - Best for real-time applications, live collaboration, and high-frequency messaging. Lowest latency with full-duplex communication."
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
                cons: vec!["More complexity".to_string(), "Requires streaming-http feature".to_string()],
                latency: "10-30ms".to_string(),
                throughput: "Very High".to_string(),
                available: cfg!(feature = "streaming-http"),
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

    /// List resource templates from the server (New in 2025-06-18)
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
    // Completion Operations (New in 2025-06-18)
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

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Send a request and get a response
    async fn send_request(&self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        if self.config.validate_requests {
            validate_jsonrpc_request(&request)?;
            validate_mcp_request(&request.method, request.params.as_ref())?;
        }

        let mut transport_guard = self.transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            let response = transport.send_request(request).await?;

            if self.config.validate_responses {
                validate_jsonrpc_response(&response)?;
            }

            Ok(response)
        } else {
            Err(McpError::Transport("Not connected".to_string()))
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
    async fn test_client_creation() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        assert_eq!(client.info().name, "test-client");
        assert_eq!(client.info().version, "1.0.0");
        assert!(!client.is_connected().await);
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
                title: Some("Test Server".to_string()),
            },
        );

        let init_response = JsonRpcResponse::success(Value::from(1), init_result.clone()).unwrap();

        let transport = MockTransport::new(vec![init_response]);

        let mut client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        let result = client.connect(transport).await.unwrap();

        assert_eq!(result.server_info.name, "test-server");
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
                title: Some("Test Server".to_string()),
            },
        );

        let init_response = JsonRpcResponse::success(Value::from(1), init_result).unwrap();

        let transport = MockTransport::new(vec![init_response]);

        let mut client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        client.connect(transport).await.unwrap();

        assert!(client.is_connected().await);

        client.disconnect().await.unwrap();
        assert!(!client.is_connected().await);
        assert!(client.server_info().await.is_none());
        assert!(client.server_capabilities().await.is_none());
    }
}

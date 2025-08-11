// ! MCP server implementation
// !
// ! Module provides the main MCP server implementation that handles client connections,
// ! manages resources, tools, and prompts, and processes JSON-RPC requests according to
// ! the Model Context Protocol specification.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::core::{
    PromptInfo, ResourceInfo, ToolInfo,
    completion::{CompletionContext, CompletionHandler},
    error::{McpError, McpResult},
    prompt::{Prompt, PromptHandler},
    resource::{Resource, ResourceHandler},
    tool::{Tool, ToolHandler},
};
use crate::protocol::{error_codes::*, messages::*, methods, types::*, validation::*};
use crate::transport::traits::ServerTransport;

/// Configuration for the MCP server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Whether to validate all incoming requests
    pub validate_requests: bool,
    /// Whether to enable detailed logging
    pub enable_logging: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            request_timeout_ms: 30000,
            validate_requests: true,
            enable_logging: true,
        }
    }
}

/// Main MCP server implementation
pub struct McpServer {
    /// Server information
    info: ServerInfo,
    /// Server capabilities
    capabilities: ServerCapabilities,
    /// Server configuration
    config: ServerConfig,
    /// Registered resources
    resources: Arc<RwLock<HashMap<String, Resource>>>,
    /// Registered tools
    tools: Arc<RwLock<HashMap<String, Tool>>>,
    /// Registered prompts
    prompts: Arc<RwLock<HashMap<String, Prompt>>>,
    /// Resource templates (New in 2025-06-18)
    resource_templates: Arc<RwLock<HashMap<String, ResourceTemplate>>>,
    /// Completion handlers (New in 2025-06-18)
    completion_handlers: Arc<RwLock<HashMap<String, Box<dyn CompletionHandler>>>>,
    /// Active transport
    transport: Arc<Mutex<Option<Box<dyn ServerTransport>>>>,
    /// Server state
    state: Arc<RwLock<ServerState>>,
    /// Request ID counter
    #[allow(dead_code)]
    request_counter: Arc<Mutex<u64>>,
}

/// Internal server state
#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    /// Server is not yet initialized
    Uninitialized,
    /// Server is initializing
    Initializing,
    /// Server is running and ready to accept requests
    Running,
    /// Server is shutting down
    Stopping,
    /// Server has stopped
    Stopped,
}

impl McpServer {
    /// Create a new MCP server with the given name and version
    pub fn new(name: String, version: String) -> Self {
        Self {
            info: ServerInfo::new(name, version),
            capabilities: ServerCapabilities {
                prompts: Some(PromptsCapability {
                    list_changed: Some(true),
                }),
                resources: Some(ResourcesCapability {
                    subscribe: Some(true),
                    list_changed: Some(true),
                }),
                tools: Some(ToolsCapability {
                    list_changed: Some(true),
                }),
                sampling: None,
                logging: None,
                completions: Some(CompletionsCapability::default()),
                experimental: None,
            },
            config: ServerConfig::default(),
            resources: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            prompts: Arc::new(RwLock::new(HashMap::new())),
            resource_templates: Arc::new(RwLock::new(HashMap::new())),
            completion_handlers: Arc::new(RwLock::new(HashMap::new())),
            transport: Arc::new(Mutex::new(None)),
            state: Arc::new(RwLock::new(ServerState::Uninitialized)),
            request_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a new MCP server with custom configuration
    pub fn with_config(name: String, version: String, config: ServerConfig) -> Self {
        let mut server = Self::new(name, version);
        server.config = config;
        server
    }

    /// Set server capabilities
    pub fn set_capabilities(&mut self, capabilities: ServerCapabilities) {
        self.capabilities = capabilities;
    }

    /// Get server information
    pub fn info(&self) -> &ServerInfo {
        &self.info
    }

    /// Get server name (for compatibility with tests)
    pub fn name(&self) -> &str {
        &self.info.name
    }

    /// Get server version (for compatibility with tests)
    pub fn version(&self) -> &str {
        &self.info.version
    }

    /// Get server capabilities
    pub fn capabilities(&self) -> &ServerCapabilities {
        &self.capabilities
    }

    /// Get server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    // ========================================================================
    // Resource Management
    // ========================================================================

    /// Add a resource to the server
    pub async fn add_resource<H>(&self, name: String, uri: String, handler: H) -> McpResult<()>
    where
        H: ResourceHandler + 'static,
    {
        let resource_info = ResourceInfo {
            uri: uri.clone(),
            name: name.clone(),
            description: None,
            mime_type: None,
            annotations: None,
            size: None,
            title: None,
            meta: None,
        };

        validate_resource_info(&resource_info)?;

        let resource = Resource::new(resource_info, handler);

        {
            let mut resources = self.resources.write().await;
            resources.insert(uri, resource);
        }

        // Emit list changed notification if we have an active transport
        self.emit_resources_list_changed().await?;

        Ok(())
    }

    /// Add a resource with detailed information
    pub async fn add_resource_detailed<H>(&self, info: ResourceInfo, handler: H) -> McpResult<()>
    where
        H: ResourceHandler + 'static,
    {
        validate_resource_info(&info)?;

        let uri = info.uri.clone();
        let resource = Resource::new(info, handler);

        {
            let mut resources = self.resources.write().await;
            resources.insert(uri, resource);
        }

        self.emit_resources_list_changed().await?;

        Ok(())
    }

    /// Remove a resource from the server
    pub async fn remove_resource(&self, uri: &str) -> McpResult<bool> {
        let removed = {
            let mut resources = self.resources.write().await;
            resources.remove(uri).is_some()
        };

        if removed {
            self.emit_resources_list_changed().await?;
        }

        Ok(removed)
    }

    /// List all registered resources
    pub async fn list_resources(&self) -> McpResult<Vec<ResourceInfo>> {
        let resources = self.resources.read().await;
        Ok(resources.values().map(|r| r.info.clone()).collect())
    }

    /// Read a resource
    pub async fn read_resource(&self, uri: &str) -> McpResult<Vec<ResourceContents>> {
        let resources = self.resources.read().await;

        match resources.get(uri) {
            Some(resource) => {
                let params = HashMap::new(); // URL parameter extraction will be implemented in future versions
                resource.handler.read(uri, &params).await
            }
            None => Err(McpError::ResourceNotFound(uri.to_string())),
        }
    }

    // ========================================================================
    // Tool Management
    // ========================================================================

    /// Add a tool to the server
    pub async fn add_tool<H>(
        &self,
        name: String,
        description: Option<String>,
        schema: Value,
        handler: H,
    ) -> McpResult<()>
    where
        H: ToolHandler + 'static,
    {
        let tool_schema = ToolInputSchema {
            schema_type: "object".to_string(),
            properties: schema
                .get("properties")
                .and_then(|p| p.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
            required: schema.get("required").and_then(|r| {
                r.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
            }),
            additional_properties: schema
                .as_object()
                .unwrap_or(&serde_json::Map::new())
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        };

        let tool_info = ToolInfo {
            name: name.clone(),
            description,
            input_schema: tool_schema,
            output_schema: None,
            annotations: None,
            title: None,
            meta: None,
        };

        validate_tool_info(&tool_info)?;

        let tool = Tool::new(
            name.clone(),
            tool_info.description.clone(),
            serde_json::to_value(&tool_info.input_schema)?,
            handler,
        );

        {
            let mut tools = self.tools.write().await;
            tools.insert(name, tool);
        }

        self.emit_tools_list_changed().await?;

        Ok(())
    }

    /// Add a simple tool with basic schema (convenience method)
    pub async fn add_simple_tool<F>(
        &self,
        name: &str,
        description: &str,
        handler: F,
    ) -> McpResult<()>
    where
        F: Fn(
                std::collections::HashMap<String, serde_json::Value>,
            ) -> McpResult<Vec<crate::protocol::types::ContentBlock>>
            + Send
            + Sync
            + 'static,
    {
        // Create a wrapper that implements ToolHandler
        struct SimpleHandler<F>(F);

        #[async_trait::async_trait]
        impl<F> crate::core::tool::ToolHandler for SimpleHandler<F>
        where
            F: Fn(
                    std::collections::HashMap<String, serde_json::Value>,
                ) -> McpResult<Vec<crate::protocol::types::ContentBlock>>
                + Send
                + Sync
                + 'static,
        {
            async fn call(
                &self,
                arguments: std::collections::HashMap<String, serde_json::Value>,
            ) -> McpResult<crate::protocol::types::ToolResult> {
                let content = (self.0)(arguments)?;
                Ok(crate::protocol::types::ToolResult {
                    content,
                    is_error: Some(false),
                    structured_content: None,
                    meta: None,
                })
            }
        }

        let simple_handler = SimpleHandler(handler);

        // Create a basic schema that accepts any object
        let schema = serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": true
        });

        self.add_tool(
            name.to_string(),
            Some(description.to_string()),
            schema,
            simple_handler,
        )
        .await
    }

    /// Add a tool with detailed information
    pub async fn add_tool_detailed<H>(&self, info: ToolInfo, handler: H) -> McpResult<()>
    where
        H: ToolHandler + 'static,
    {
        validate_tool_info(&info)?;

        let name = info.name.clone();
        let tool = Tool::new(
            name.clone(),
            info.description.clone(),
            serde_json::to_value(&info.input_schema)?,
            handler,
        );

        {
            let mut tools = self.tools.write().await;
            tools.insert(name, tool);
        }

        self.emit_tools_list_changed().await?;

        Ok(())
    }

    /// Remove a tool from the server
    pub async fn remove_tool(&self, name: &str) -> McpResult<bool> {
        let removed = {
            let mut tools = self.tools.write().await;
            tools.remove(name).is_some()
        };

        if removed {
            self.emit_tools_list_changed().await?;
        }

        Ok(removed)
    }

    /// List all registered tools
    pub async fn list_tools(&self) -> McpResult<Vec<ToolInfo>> {
        let tools = self.tools.read().await;
        Ok(tools.values().map(|t| t.info.clone()).collect())
    }

    /// Call a tool
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<HashMap<String, Value>>,
    ) -> McpResult<ToolResult> {
        let tools = self.tools.read().await;

        match tools.get(name) {
            Some(tool) => {
                if !tool.enabled {
                    return Err(McpError::ToolNotFound(format!("Tool '{name}' is disabled")));
                }

                let args = arguments.unwrap_or_default();
                tool.handler.call(args).await
            }
            None => Err(McpError::ToolNotFound(name.to_string())),
        }
    }

    // ========================================================================
    // Prompt Management
    // ========================================================================

    /// Add a prompt to the server
    pub async fn add_prompt<H>(&self, info: PromptInfo, handler: H) -> McpResult<()>
    where
        H: PromptHandler + 'static,
    {
        validate_prompt_info(&info)?;

        let name = info.name.clone();
        let prompt = Prompt::new(info, handler);

        {
            let mut prompts = self.prompts.write().await;
            prompts.insert(name, prompt);
        }

        self.emit_prompts_list_changed().await?;

        Ok(())
    }

    /// Remove a prompt from the server
    pub async fn remove_prompt(&self, name: &str) -> McpResult<bool> {
        let removed = {
            let mut prompts = self.prompts.write().await;
            prompts.remove(name).is_some()
        };

        if removed {
            self.emit_prompts_list_changed().await?;
        }

        Ok(removed)
    }

    /// List all registered prompts
    pub async fn list_prompts(&self) -> McpResult<Vec<PromptInfo>> {
        let prompts = self.prompts.read().await;
        Ok(prompts.values().map(|p| p.info.clone()).collect())
    }

    /// Get a prompt
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<HashMap<String, Value>>,
    ) -> McpResult<PromptResult> {
        let prompts = self.prompts.read().await;

        match prompts.get(name) {
            Some(prompt) => {
                let args = arguments.unwrap_or_default();
                prompt.handler.get(args).await
            }
            None => Err(McpError::PromptNotFound(name.to_string())),
        }
    }

    // ========================================================================
    // Resource Template Management (New in 2025-06-18)
    // ========================================================================

    /// Add a resource template to the server
    pub async fn add_resource_template(&self, template: ResourceTemplate) -> McpResult<()> {
        let name = template.name.clone();
        {
            let mut templates = self.resource_templates.write().await;
            templates.insert(name, template);
        }

        // Emit notification if supported
        self.emit_resource_templates_list_changed().await?;
        Ok(())
    }

    /// List resource templates
    pub async fn list_resource_templates(&self) -> McpResult<Vec<ResourceTemplate>> {
        let templates = self.resource_templates.read().await;
        Ok(templates.values().cloned().collect())
    }

    /// Remove a resource template
    pub async fn remove_resource_template(&self, name: &str) -> McpResult<bool> {
        let removed = {
            let mut templates = self.resource_templates.write().await;
            templates.remove(name).is_some()
        };

        if removed {
            self.emit_resource_templates_list_changed().await?;
        }

        Ok(removed)
    }

    /// Emit resource templates list changed notification
    async fn emit_resource_templates_list_changed(&self) -> McpResult<()> {
        // In a real implementation, this would send a notification
        // For now, we'll just return Ok
        Ok(())
    }

    // ========================================================================
    // Completion Management (New in 2025-06-18)
    // ========================================================================

    /// Add completion handler for a specific reference type
    pub async fn add_completion_handler<H>(&self, ref_type: String, handler: H) -> McpResult<()>
    where
        H: CompletionHandler + 'static,
    {
        let mut handlers = self.completion_handlers.write().await;
        handlers.insert(ref_type, Box::new(handler));
        Ok(())
    }

    /// Remove completion handler for a reference type
    pub async fn remove_completion_handler(&self, ref_type: &str) -> McpResult<bool> {
        let mut handlers = self.completion_handlers.write().await;
        Ok(handlers.remove(ref_type).is_some())
    }

    /// Handle completion request
    pub async fn handle_completion(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        let ref_type = match reference {
            CompletionReference::Prompt { .. } => "ref/prompt",
            CompletionReference::Resource { .. } => "ref/resource",
            CompletionReference::Tool { .. } => "ref/tool",
        };

        let handlers = self.completion_handlers.read().await;
        if let Some(handler) = handlers.get(ref_type) {
            handler.complete(reference, argument, context).await
        } else {
            // Return empty completions if no handler is registered
            Ok(vec![])
        }
    }

    // ========================================================================
    // Bidirectional Communication (New in 2025-06-18)
    // ========================================================================

    /// Send a request to the client (server-initiated)
    pub async fn send_client_request(
        &self,
        _request: JsonRpcRequest,
    ) -> McpResult<JsonRpcResponse> {
        let mut transport_guard = self.transport.lock().await;
        if let Some(_transport) = transport_guard.as_mut() {
            // For now, return an error - this would need transport-specific implementation
            Err(McpError::MethodNotFound(
                "Bidirectional communication not yet fully implemented".to_string(),
            ))
        } else {
            Err(McpError::Transport("Not connected".to_string()))
        }
    }

    /// Request LLM sampling from client
    pub async fn request_sampling(
        &self,
        params: CreateMessageParams,
    ) -> McpResult<CreateMessageResult> {
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::SAMPLING_CREATE_MESSAGE.to_string(),
            Some(params),
        )?;

        let response = self.send_client_request(request).await?;

        if let Some(result) = response.result {
            Ok(serde_json::from_value(result)?)
        } else {
            Err(McpError::Protocol("No result in response".to_string()))
        }
    }

    /// Request root directories from client
    pub async fn request_roots(&self) -> McpResult<ListRootsResult> {
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::ROOTS_LIST.to_string(),
            Some(ListRootsParams { meta: None }),
        )?;

        let response = self.send_client_request(request).await?;

        if let Some(result) = response.result {
            Ok(serde_json::from_value(result)?)
        } else {
            Err(McpError::Protocol("No result in response".to_string()))
        }
    }

    /// Request user input via elicitation
    pub async fn request_elicitation(&self, params: ElicitParams) -> McpResult<ElicitResult> {
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::ELICITATION_CREATE.to_string(),
            Some(params),
        )?;

        let response = self.send_client_request(request).await?;

        if let Some(result) = response.result {
            Ok(serde_json::from_value(result)?)
        } else {
            Err(McpError::Protocol("No result in response".to_string()))
        }
    }

    /// Request list of roots from the client
    ///
    /// Method allows the server to request access to specific directories
    /// or files on the client's file system.
    pub async fn request_list_roots(&self) -> McpResult<ListRootsResult> {
        let request = JsonRpcRequest::new(
            Value::from(self.next_request_id().await),
            methods::ROOTS_LIST.to_string(),
            None::<serde_json::Value>,
        )?;

        let response = self.send_client_request(request).await?;

        if let Some(result) = response.result {
            Ok(serde_json::from_value(result)?)
        } else {
            Err(McpError::Protocol("No result in response".to_string()))
        }
    }

    /// Send a notification that the roots list has changed
    ///
    /// This notification informs the client that the list of roots
    /// the server needs access to has changed.
    pub async fn notify_roots_list_changed(&self) -> McpResult<()> {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: methods::ROOTS_LIST_CHANGED.to_string(),
            params: None,
        };

        if let Some(ref mut transport) = *self.transport.lock().await {
            transport.send_notification(notification).await
        } else {
            Err(McpError::Connection("No transport available".to_string()))
        }
    }

    // ========================================================================
    // Server Lifecycle
    // ========================================================================

    /// Start the server with the given transport
    pub async fn start<T>(&mut self, mut transport: T) -> McpResult<()>
    where
        T: ServerTransport + 'static,
    {
        let mut state = self.state.write().await;

        match *state {
            ServerState::Uninitialized => {
                *state = ServerState::Initializing;
            }
            _ => return Err(McpError::Protocol("Server is already started".to_string())),
        }

        drop(state);

        // Create a request handler that delegates to this server
        let resources = self.resources.clone();
        let tools = self.tools.clone();
        let prompts = self.prompts.clone();
        let resource_templates = self.resource_templates.clone();
        let completion_handlers = self.completion_handlers.clone();
        let info = self.info.clone();
        let capabilities = self.capabilities.clone();
        let config = self.config.clone();

        let request_handler: crate::transport::traits::ServerRequestHandler =
            Arc::new(move |request| {
                let resources = resources.clone();
                let tools = tools.clone();
                let prompts = prompts.clone();
                let resource_templates = resource_templates.clone();
                let completion_handlers = completion_handlers.clone();
                let info = info.clone();
                let capabilities = capabilities.clone();
                let config = config.clone();

                Box::pin(async move {
                    // Create a temporary server instance to handle the request
                    let temp_server = McpServer {
                        info,
                        capabilities,
                        config,
                        resources,
                        tools,
                        prompts,
                        resource_templates,
                        completion_handlers,
                        transport: Arc::new(Mutex::new(None)),
                        state: Arc::new(RwLock::new(ServerState::Running)),
                        request_counter: Arc::new(Mutex::new(0)),
                    };
                    temp_server.handle_request(request).await
                })
            });

        // Set the request handler on the transport
        transport.set_request_handler(request_handler);

        // Set up the transport
        {
            let mut transport_guard = self.transport.lock().await;
            *transport_guard = Some(Box::new(transport));
        }

        // Start the transport
        {
            let mut transport_guard = self.transport.lock().await;
            if let Some(transport) = transport_guard.as_mut() {
                transport.start().await?;
            }
        }

        // Update state to running
        {
            let mut state = self.state.write().await;
            *state = ServerState::Running;
        }

        Ok(())
    }

    /// Stop the server
    pub async fn stop(&self) -> McpResult<()> {
        let mut state = self.state.write().await;

        match *state {
            ServerState::Running => {
                *state = ServerState::Stopping;
            }
            ServerState::Stopped => return Ok(()),
            _ => return Err(McpError::Protocol("Server is not running".to_string())),
        }

        drop(state);

        // Stop the transport
        {
            let mut transport_guard = self.transport.lock().await;
            if let Some(transport) = transport_guard.as_mut() {
                transport.stop().await?;
            }
        }

        // Update state to stopped
        {
            let mut state = self.state.write().await;
            *state = ServerState::Stopped;
        }

        Ok(())
    }

    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, ServerState::Running)
    }

    /// Get the current server state
    pub async fn state(&self) -> ServerState {
        let state = self.state.read().await;
        state.clone()
    }

    /// Start server with STDIO transport and run until interrupted (convenience method)
    ///
    /// This is a convenience method that:
    /// 1. Creates a STDIO transport
    /// 2. Starts the server
    /// 3. Waits for Ctrl+C signal
    /// 4. smoothly shuts down the server
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut server = McpServer::new("my-server".to_string(), "1.0.0".to_string());
    /// // . add tools, resources, prompts ...
    /// server.run_with_stdio().await
    /// }
    /// ```
    #[cfg(feature = "stdio")]
    pub async fn run_with_stdio(mut self) -> McpResult<()> {
        use crate::transport::stdio::StdioServerTransport;

        let transport = StdioServerTransport::new();
        self.start(transport).await?;

        tracing::info!("Server started with STDIO transport. Press Ctrl+C to stop.");

        // Wait for shutdown signal
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| McpError::internal(format!("Signal handling error: {e}")))?;

        tracing::info!("Shutdown signal received, stopping server...");
        self.stop().await
    }

    /// Start server with custom transport and run until interrupted
    ///
    /// This is a convenience method that:
    /// 1. Starts the server with the provided transport
    /// 2. Waits for Ctrl+C signal
    /// 3. smoothly shuts down the server
    ///
    /// # Arguments
    /// * `transport` - The transport to use for communication
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut server = McpServer::new("my-server".to_string(), "1.0.0".to_string());
    /// // . add tools, resources, prompts ...
    ///
    /// let transport = StdioServerTransport::new();
    /// server.run_with_transport(transport).await
    /// }
    /// ```
    pub async fn run_with_transport<T>(mut self, transport: T) -> McpResult<()>
    where
        T: ServerTransport + 'static,
    {
        self.start(transport).await?;

        tracing::info!("Server started. Press Ctrl+C to stop.");

        // Wait for shutdown signal
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| McpError::internal(format!("Signal handling error: {e}")))?;

        tracing::info!("Shutdown signal received, stopping server...");
        self.stop().await
    }

    /// Start server with HTTP transport and run until interrupted (convenience method)
    ///
    /// This is a convenience method that:
    /// 1. Creates an HTTP transport bound to the specified address
    /// 2. Starts the server
    /// 3. Waits for Ctrl+C signal
    /// 4. smoothly shuts down the server
    ///
    /// # Arguments
    /// * `bind_addr` - The address to bind the HTTP server to (e.g., "127.0.0.1:3000")
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut server = McpServer::new("my-server".to_string(), "1.0.0".to_string());
    /// // . add tools, resources, prompts ...
    /// server.run_with_http("127.0.0.1:3000").await
    /// }
    /// ```
    #[cfg(feature = "http")]
    pub async fn run_with_http(mut self, bind_addr: &str) -> McpResult<()> {
        use crate::transport::http::HttpServerTransport;

        let transport = HttpServerTransport::new(bind_addr.to_string());
        self.start(transport).await?;

        tracing::info!(
            "Server started with HTTP transport on {}. Press Ctrl+C to stop.",
            bind_addr
        );

        // Wait for shutdown signal
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| McpError::internal(format!("Signal handling error: {e}")))?;

        tracing::info!("Shutdown signal received, stopping server...");
        self.stop().await
    }

    /// Start server with WebSocket transport and run until interrupted (convenience method)
    ///
    /// This is a convenience method that:
    /// 1. Creates a WebSocket transport bound to the specified address
    /// 2. Starts the server
    /// 3. Waits for Ctrl+C signal
    /// 4. smoothly shuts down the server
    ///
    /// # Arguments
    /// * `bind_addr` - The address to bind the WebSocket server to (e.g., "127.0.0.1:8080")
    ///
    /// # Example
    /// ```rust,no_run
    /// use prism_mcp_rs::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> McpResult<()> {
    /// let mut server = McpServer::new("my-server".to_string(), "1.0.0".to_string());
    /// // . add tools, resources, prompts ...
    /// server.run_with_websocket("127.0.0.1:8080").await
    /// }
    /// ```
    #[cfg(feature = "websocket")]
    pub async fn run_with_websocket(mut self, bind_addr: &str) -> McpResult<()> {
        use crate::transport::websocket::WebSocketServerTransport;

        let transport = WebSocketServerTransport::new(bind_addr.to_string());
        self.start(transport).await?;

        tracing::info!(
            "Server started with WebSocket transport on {}. Press Ctrl+C to stop.",
            bind_addr
        );

        // Wait for shutdown signal
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| McpError::internal(format!("Signal handling error: {e}")))?;

        tracing::info!("Shutdown signal received, stopping server...");
        self.stop().await
    }

    // ========================================================================
    // Request Handling
    // ========================================================================

    /// Handle an incoming JSON-RPC request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        // Validate the request if configured to do so
        if self.config.validate_requests {
            validate_jsonrpc_request(&request)?;
            validate_mcp_request(&request.method, request.params.as_ref())?;
        }

        // Route the request to the appropriate handler
        let result = match request.method.as_str() {
            methods::INITIALIZE => self.handle_initialize(request.params).await,
            methods::PING => self.handle_ping().await,
            methods::TOOLS_LIST => self.handle_tools_list(request.params).await,
            methods::TOOLS_CALL => self.handle_tools_call(request.params).await,
            methods::RESOURCES_LIST => self.handle_resources_list(request.params).await,
            methods::RESOURCES_READ => self.handle_resources_read(request.params).await,
            methods::RESOURCES_SUBSCRIBE => self.handle_resources_subscribe(request.params).await,
            methods::RESOURCES_UNSUBSCRIBE => {
                self.handle_resources_unsubscribe(request.params).await
            }
            methods::PROMPTS_LIST => self.handle_prompts_list(request.params).await,
            methods::PROMPTS_GET => self.handle_prompts_get(request.params).await,
            methods::RESOURCES_TEMPLATES_LIST => {
                self.handle_resource_templates_list(request.params).await
            }
            methods::COMPLETION_COMPLETE => self.handle_completion_complete(request.params).await,
            methods::LOGGING_SET_LEVEL => self.handle_logging_set_level(request.params).await,
            methods::RPC_DISCOVER => self.handle_rpc_discover(request.params).await,
            _ => {
                let method = &request.method;
                Err(McpError::Protocol(format!("Unknown method: {method}")))
            }
        };

        // Convert the result to a JSON-RPC response
        match result {
            Ok(result_value) => Ok(JsonRpcResponse::success(request.id, result_value)?),
            Err(error) => {
                let (code, message) = match error {
                    McpError::ToolNotFound(_) => (TOOL_NOT_FOUND, error.to_string()),
                    McpError::ResourceNotFound(_) => (RESOURCE_NOT_FOUND, error.to_string()),
                    McpError::PromptNotFound(_) => (PROMPT_NOT_FOUND, error.to_string()),
                    McpError::Validation(_) => (INVALID_PARAMS, error.to_string()),
                    _ => (INTERNAL_ERROR, error.to_string()),
                };
                // Return proper JSON-RPC error response
                Err(McpError::Protocol(format!(
                    "JSON-RPC error {code}: {message}"
                )))
            }
        }
    }

    // ========================================================================
    // Individual Request Handlers
    // ========================================================================

    async fn handle_initialize(&self, params: Option<Value>) -> McpResult<Value> {
        let params: InitializeParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => {
                return Err(McpError::Validation(
                    "Missing initialize parameters".to_string(),
                ));
            }
        };

        validate_initialize_params(&params)?;

        let result = InitializeResult::new(
            crate::protocol::LATEST_PROTOCOL_VERSION.to_string(),
            self.capabilities.clone(),
            self.info.clone(),
        );

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_ping(&self) -> McpResult<Value> {
        Ok(serde_json::to_value(PingResult { meta: None })?)
    }

    async fn handle_tools_list(&self, params: Option<Value>) -> McpResult<Value> {
        let _params: ListToolsParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => ListToolsParams::default(),
        };

        let tools = self.list_tools().await?;
        let result = ListToolsResult {
            tools,
            next_cursor: None, // Pagination support will be added in future versions
            meta: None,
        };

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_tools_call(&self, params: Option<Value>) -> McpResult<Value> {
        let params: CallToolParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => {
                return Err(McpError::Validation(
                    "Missing tool call parameters".to_string(),
                ));
            }
        };

        validate_call_tool_params(&params)?;

        let result = self.call_tool(&params.name, params.arguments).await?;
        Ok(serde_json::to_value(result)?)
    }

    async fn handle_resources_list(&self, params: Option<Value>) -> McpResult<Value> {
        let _params: ListResourcesParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => ListResourcesParams::default(),
        };

        let resources = self.list_resources().await?;
        let result = ListResourcesResult {
            resources,
            next_cursor: None, // Pagination support will be added in future versions
            meta: None,
        };

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_resources_read(&self, params: Option<Value>) -> McpResult<Value> {
        let params: ReadResourceParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => {
                return Err(McpError::Validation(
                    "Missing resource read parameters".to_string(),
                ));
            }
        };

        validate_read_resource_params(&params)?;

        let contents = self.read_resource(&params.uri).await?;
        let result = ReadResourceResult {
            contents,
            meta: None,
        };

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_resources_subscribe(&self, params: Option<Value>) -> McpResult<Value> {
        let params: SubscribeResourceParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => {
                return Err(McpError::Validation(
                    "Missing resource subscribe parameters".to_string(),
                ));
            }
        };

        // Resource subscriptions functionality planned for future implementation
        let _uri = params.uri;
        let result = SubscribeResourceResult { meta: None };

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_resources_unsubscribe(&self, params: Option<Value>) -> McpResult<Value> {
        let params: UnsubscribeResourceParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => {
                return Err(McpError::Validation(
                    "Missing resource unsubscribe parameters".to_string(),
                ));
            }
        };

        // Resource subscriptions functionality planned for future implementation
        let _uri = params.uri;
        let result = UnsubscribeResourceResult { meta: None };

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_prompts_list(&self, params: Option<Value>) -> McpResult<Value> {
        let _params: ListPromptsParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => ListPromptsParams::default(),
        };

        let prompts = self.list_prompts().await?;
        let result = ListPromptsResult {
            prompts,
            next_cursor: None, // Pagination support will be added in future versions
            meta: None,
        };

        Ok(serde_json::to_value(result)?)
    }

    async fn handle_prompts_get(&self, params: Option<Value>) -> McpResult<Value> {
        let params: GetPromptParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => {
                return Err(McpError::Validation(
                    "Missing prompt get parameters".to_string(),
                ));
            }
        };

        validate_get_prompt_params(&params)?;

        let arguments = params.arguments.map(|args| {
            args.into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect()
        });
        let result = self.get_prompt(&params.name, arguments).await?;
        Ok(serde_json::to_value(result)?)
    }

    async fn handle_logging_set_level(&self, params: Option<Value>) -> McpResult<Value> {
        let _params: SetLoggingLevelParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => {
                return Err(McpError::Validation(
                    "Missing logging level parameters".to_string(),
                ));
            }
        };

        // Logging level management feature planned for future implementation
        let result = SetLoggingLevelResult { meta: None };
        Ok(serde_json::to_value(result)?)
    }

    /// Handle resource templates list request (New in 2025-06-18)
    async fn handle_resource_templates_list(&self, params: Option<Value>) -> McpResult<Value> {
        let _params: ListResourceTemplatesParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => ListResourceTemplatesParams::default(),
        };

        let templates = self.list_resource_templates().await?;
        let result = ListResourceTemplatesResult {
            resource_templates: templates,
            next_cursor: None, // Pagination support will be added in future versions
            meta: None,
        };

        Ok(serde_json::to_value(result)?)
    }

    /// Handle completion request (New in 2025-06-18)
    async fn handle_completion_complete(&self, params: Option<Value>) -> McpResult<Value> {
        let params: CompleteParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => {
                return Err(McpError::Validation(
                    "Missing completion parameters".to_string(),
                ));
            }
        };

        let completions = self
            .handle_completion(
                &params.reference,
                &params.argument,
                None, // Context support to be added
            )
            .await?;

        let result = CompleteResult {
            completion: CompletionData {
                values: completions,
                total: None,
                has_more: None,
            },
            meta: None,
        };

        Ok(serde_json::to_value(result)?)
    }

    /// Handle RPC discovery request (Optional discovery mechanism)
    async fn handle_rpc_discover(&self, params: Option<Value>) -> McpResult<Value> {
        use crate::server::discovery_handler::DiscoveryHandler;

        let handler = DiscoveryHandler::new();
        let result = handler
            .handle(&self.info, &self.capabilities, params)
            .await?;
        Ok(serde_json::to_value(result)?)
    }

    // ========================================================================
    // Notification Helpers
    // ========================================================================

    async fn emit_resources_list_changed(&self) -> McpResult<()> {
        let notification = JsonRpcNotification::new(
            methods::RESOURCES_LIST_CHANGED.to_string(),
            Some(ResourceListChangedParams { meta: None }),
        )?;

        self.send_notification(notification).await
    }

    async fn emit_tools_list_changed(&self) -> McpResult<()> {
        let notification = JsonRpcNotification::new(
            methods::TOOLS_LIST_CHANGED.to_string(),
            Some(ToolListChangedParams { meta: None }),
        )?;

        self.send_notification(notification).await
    }

    async fn emit_prompts_list_changed(&self) -> McpResult<()> {
        let notification = JsonRpcNotification::new(
            methods::PROMPTS_LIST_CHANGED.to_string(),
            Some(PromptListChangedParams { meta: None }),
        )?;

        self.send_notification(notification).await
    }

    /// Send a notification through the transport
    async fn send_notification(&self, notification: JsonRpcNotification) -> McpResult<()> {
        let mut transport_guard = self.transport.lock().await;
        if let Some(transport) = transport_guard.as_mut() {
            transport.send_notification(notification).await?;
        }
        Ok(())
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    async fn next_request_id(&self) -> u64 {
        let mut counter = self.request_counter.lock().await;
        *counter += 1;
        *counter
    }

    // ========================================================================
    // State Management & Convenience Methods
    // ========================================================================

    /// Check if the server has been initialized
    pub async fn is_initialized(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, ServerState::Running)
    }

    /// Check if the server is stopping or stopped
    pub async fn is_stopped(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, ServerState::Stopping | ServerState::Stopped)
    }

    /// Get the current server state (alias for existing state method)
    pub async fn get_state(&self) -> ServerState {
        self.state().await
    }

    /// Check if the server has any registered tools
    pub async fn has_tools(&self) -> bool {
        let tools = self.tools.read().await;
        !tools.is_empty()
    }

    /// Check if the server has any registered resources
    pub async fn has_resources(&self) -> bool {
        let resources = self.resources.read().await;
        !resources.is_empty()
    }

    /// Check if the server has any registered prompts
    pub async fn has_prompts(&self) -> bool {
        let prompts = self.prompts.read().await;
        !prompts.is_empty()
    }

    /// Get the number of registered tools
    pub async fn tool_count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }

    /// Get the number of registered resources
    pub async fn resource_count(&self) -> usize {
        let resources = self.resources.read().await;
        resources.len()
    }

    /// Get the number of registered prompts
    pub async fn prompt_count(&self) -> usize {
        let prompts = self.prompts.read().await;
        prompts.len()
    }

    /// Check if a specific tool is registered
    pub async fn has_tool(&self, name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(name)
    }

    /// Check if a specific resource is registered
    pub async fn has_resource(&self, uri: &str) -> bool {
        let resources = self.resources.read().await;
        resources.contains_key(uri)
    }

    /// Check if a specific prompt is registered
    pub async fn has_prompt(&self, name: &str) -> bool {
        let prompts = self.prompts.read().await;
        prompts.contains_key(name)
    }

    /// Set the server state (internal use)
    async fn set_state(&self, new_state: ServerState) {
        let mut state = self.state.write().await;
        *state = new_state;
    }

    /// Initialize the server (sets state to Running)
    pub async fn initialize(&self) -> McpResult<()> {
        self.set_state(ServerState::Running).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_server_creation() {
        let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        assert_eq!(server.info().name, "test-server");
        assert_eq!(server.info().version, "1.0.0");
        assert!(!server.is_running().await);
    }

    #[tokio::test]
    async fn test_tool_management() {
        let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());

        // Add a tool
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });

        struct TestToolHandler;

        #[async_trait::async_trait]
        impl ToolHandler for TestToolHandler {
            async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
                Ok(ToolResult {
                    content: vec![Content::text("Hello from tool")],
                    is_error: None,
                    structured_content: None,
                    meta: None,
                })
            }
        }

        server
            .add_tool(
                "test_tool".to_string(),
                Some("A test tool".to_string()),
                schema,
                TestToolHandler,
            )
            .await
            .unwrap();

        // List tools
        let tools = server.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");

        // Call tool
        let result = server.call_tool("test_tool", None).await.unwrap();
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_initialize_request() {
        let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());

        let init_params = InitializeParams::new(
            crate::protocol::LATEST_PROTOCOL_VERSION.to_string(),
            ClientCapabilities::default(),
            ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                title: Some("Test Client".to_string()),
            },
        );

        let request =
            JsonRpcRequest::new(json!(1), methods::INITIALIZE.to_string(), Some(init_params))
                .unwrap();

        let response = server.handle_request(request).await.unwrap();
        assert!(response.result.is_some());
    }
}

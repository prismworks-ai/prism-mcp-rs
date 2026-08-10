//! STDIO transport implementation for MCP
//!
//! Module provides STDIO-based transport for MCP communication,
//! which is commonly used for command-line tools and process communication.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio::time::{timeout, Duration};

use crate::core::error::{McpError, McpResult};
use crate::protocol::types::{
    error_codes, JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};
use crate::protocol::{
    has_tasks_extension, json_rpc_error_details, methods, modern_request_context,
    ServerCapabilities, SubscriptionFilter, SubscriptionsAcknowledgedParams,
    SubscriptionsListenParams, SubscriptionsListenResult, HEADER_MISMATCH,
    MISSING_REQUIRED_CLIENT_CAPABILITY, SUBSCRIPTION_ID_META_KEY, TASKS_EXTENSION_ID,
    UNSUPPORTED_PROTOCOL_VERSION,
};
use crate::transport::traits::{
    ClientSubscription, ConnectionState, ServerRequestHandler, ServerTransport, Transport,
    TransportConfig,
};

fn add_subscription_id(notification: &mut JsonRpcNotification, id: &Value) {
    let params = notification
        .params
        .get_or_insert_with(|| Value::Object(serde_json::Map::new()));
    let Some(params) = params.as_object_mut() else {
        return;
    };
    let meta = params
        .entry("_meta")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    if let Some(meta) = meta.as_object_mut() {
        meta.insert(SUBSCRIPTION_ID_META_KEY.to_string(), id.clone());
    }
}

async fn write_stdio_line<T: serde::Serialize>(
    writer: &Arc<Mutex<BufWriter<tokio::io::Stdout>>>,
    message: &T,
) -> McpResult<()> {
    let line = serde_json::to_string(message).map_err(McpError::serialization)?;
    let mut writer = writer.lock().await;
    writer
        .write_all(line.as_bytes())
        .await
        .map_err(McpError::io)?;
    writer.write_all(b"\n").await.map_err(McpError::io)?;
    writer.flush().await.map_err(McpError::io)
}

fn accepted_subscription_filter(
    requested: &SubscriptionFilter,
    capabilities: &ServerCapabilities,
) -> SubscriptionFilter {
    let tasks_enabled = capabilities
        .extensions
        .as_ref()
        .is_some_and(|extensions| extensions.contains_key(TASKS_EXTENSION_ID));
    SubscriptionFilter {
        tools_list_changed: (requested.tools_list_changed == Some(true)
            && capabilities
                .tools
                .as_ref()
                .is_some_and(|value| value.list_changed == Some(true)))
        .then_some(true),
        prompts_list_changed: (requested.prompts_list_changed == Some(true)
            && capabilities
                .prompts
                .as_ref()
                .is_some_and(|value| value.list_changed == Some(true)))
        .then_some(true),
        resources_list_changed: (requested.resources_list_changed == Some(true)
            && capabilities
                .resources
                .as_ref()
                .is_some_and(|value| value.list_changed == Some(true)))
        .then_some(true),
        resource_subscriptions: if capabilities
            .resources
            .as_ref()
            .is_some_and(|value| value.subscribe == Some(true))
        {
            requested.resource_subscriptions.clone()
        } else {
            Vec::new()
        },
        task_ids: if tasks_enabled {
            requested.task_ids.clone()
        } else {
            Vec::new()
        },
    }
}

/// STDIO transport for MCP clients
///
/// This transport communicates with an MCP server via STDIO (standard input/output).
/// It's typically used when the server is a separate process.
#[derive(Debug)]
pub struct StdioClientTransport {
    child: Option<Child>,
    stdin_writer: Option<BufWriter<tokio::process::ChildStdin>>,
    #[allow(dead_code)]
    stdout_reader: Option<BufReader<tokio::process::ChildStdout>>,
    notification_receiver: Option<mpsc::UnboundedReceiver<JsonRpcNotification>>,
    pending_requests:
        Arc<Mutex<HashMap<Value, tokio::sync::oneshot::Sender<McpResult<JsonRpcResponse>>>>>,
    subscription_senders: Arc<Mutex<HashMap<Value, mpsc::UnboundedSender<JsonRpcNotification>>>>,
    config: TransportConfig,
    state: ConnectionState,
}

impl StdioClientTransport {
    /// Create a new STDIO client transport
    ///
    /// # Arguments
    /// * `command` - Command to execute for the MCP server
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn new<S: AsRef<str>>(command: S, args: Vec<S>) -> McpResult<Self> {
        Self::with_config(command, args, TransportConfig::default()).await
    }

    /// Create a new STDIO client transport with command string and string args
    ///
    /// # Arguments
    /// * `command` - Command to execute for the MCP server
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn new_with_command(command: &str, args: &[String]) -> McpResult<Self> {
        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        Self::new(command, args_str).await
    }

    /// Create a new STDIO client transport with environment variables
    ///
    /// # Arguments
    /// * `command` - Command to execute for the MCP server
    /// * `args` - Arguments to pass to the command
    /// * `env` - Environment variables to set for the process
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn with_env<S: AsRef<str>>(
        command: S,
        args: Vec<S>,
        env: HashMap<String, String>,
    ) -> McpResult<Self> {
        Self::with_config_and_env(command, args, TransportConfig::default(), Some(env)).await
    }

    /// Create a new STDIO client transport with custom configuration
    ///
    /// # Arguments
    /// * `command` - Command to execute for the MCP server
    /// * `args` - Arguments to pass to the command
    /// * `config` - Transport configuration
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn with_config<S: AsRef<str>>(
        command: S,
        args: Vec<S>,
        config: TransportConfig,
    ) -> McpResult<Self> {
        Self::with_config_and_env(command, args, config, None).await
    }

    /// Create a new STDIO client transport with custom configuration and environment
    ///
    /// # Arguments
    /// * `command` - Command to execute for the MCP server
    /// * `args` - Arguments to pass to the command
    /// * `config` - Transport configuration
    /// * `env` - Optional environment variables to set for the process
    ///
    /// # Returns
    /// Result containing the transport or an error
    pub async fn with_config_and_env<S: AsRef<str>>(
        command: S,
        args: Vec<S>,
        config: TransportConfig,
        env: Option<HashMap<String, String>>,
    ) -> McpResult<Self> {
        let command_str = command.as_ref();
        let args_str: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

        tracing::debug!("Starting MCP server: {} {:?}", command_str, args_str);

        let mut cmd = Command::new(command_str);
        cmd.args(&args_str)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables if provided
        if let Some(env_vars) = env {
            cmd.envs(env_vars);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| McpError::transport(format!("Failed to start server process: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::transport("Failed to get stdin handle"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::transport("Failed to get stdout handle"))?;

        let stdin_writer = BufWriter::new(stdin);
        let stdout_reader = BufReader::new(stdout);

        let (notification_sender, notification_receiver) = mpsc::unbounded_channel();
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let subscription_senders = Arc::new(Mutex::new(HashMap::new()));

        // Start message processing task
        let reader_pending_requests = pending_requests.clone();
        let reader_subscription_senders = subscription_senders.clone();
        let reader = stdout_reader;
        tokio::spawn(async move {
            Self::message_processor(
                reader,
                notification_sender,
                reader_pending_requests,
                reader_subscription_senders,
            )
            .await;
        });

        Ok(Self {
            child: Some(child),
            stdin_writer: Some(stdin_writer),
            stdout_reader: None, // Moved to processor task
            notification_receiver: Some(notification_receiver),
            pending_requests,
            subscription_senders,
            config,
            state: ConnectionState::Connected,
        })
    }

    async fn message_processor(
        mut reader: BufReader<tokio::process::ChildStdout>,
        notification_sender: mpsc::UnboundedSender<JsonRpcNotification>,
        pending_requests: Arc<
            Mutex<HashMap<Value, tokio::sync::oneshot::Sender<McpResult<JsonRpcResponse>>>>,
        >,
        subscription_senders: Arc<
            Mutex<HashMap<Value, mpsc::UnboundedSender<JsonRpcNotification>>>,
        >,
    ) {
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    tracing::debug!("STDIO reader reached EOF");
                    break;
                }
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    tracing::trace!("Received: {}", line);

                    let parsed_value = serde_json::from_str::<Value>(line).ok();
                    if parsed_value
                        .as_ref()
                        .and_then(|value| value.get("error"))
                        .is_some()
                    {
                        let Ok(error_response) = serde_json::from_str::<JsonRpcError>(line) else {
                            tracing::warn!("Failed to parse JSON-RPC error: {}", line);
                            continue;
                        };
                        let error = match error_response.error.code {
                            error_codes::METHOD_NOT_FOUND => {
                                McpError::MethodNotFound(error_response.error.message)
                            }
                            HEADER_MISMATCH => {
                                McpError::HeaderMismatch(error_response.error.message)
                            }
                            MISSING_REQUIRED_CLIENT_CAPABILITY => {
                                let required = error_response
                                    .error
                                    .data
                                    .and_then(|value| value.get("requiredCapabilities").cloned())
                                    .unwrap_or_else(|| serde_json::json!({}));
                                McpError::MissingRequiredClientCapability(required)
                            }
                            UNSUPPORTED_PROTOCOL_VERSION => {
                                let data = error_response.error.data.unwrap_or_default();
                                McpError::UnsupportedProtocolVersion {
                                    requested: data
                                        .get("requested")
                                        .and_then(Value::as_str)
                                        .unwrap_or("unknown")
                                        .to_string(),
                                    supported: data
                                        .get("supported")
                                        .and_then(Value::as_array)
                                        .into_iter()
                                        .flatten()
                                        .filter_map(Value::as_str)
                                        .map(str::to_string)
                                        .collect(),
                                }
                            }
                            code => McpError::Protocol(format!(
                                "JSON-RPC error {code}: {}",
                                error_response.error.message
                            )),
                        };
                        let mut pending = pending_requests.lock().await;
                        if let Some(sender) = pending.remove(&error_response.id) {
                            let _ = sender.send(Err(error));
                        }
                        subscription_senders.lock().await.remove(&error_response.id);
                    }
                    // Try to parse as response.
                    else if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(line) {
                        let mut pending = pending_requests.lock().await;
                        match pending.remove(&response.id) {
                            Some(sender) => {
                                let response_id = response.id.clone();
                                let _ = sender.send(Ok(response));
                                subscription_senders.lock().await.remove(&response_id);
                            }
                            _ => {
                                tracing::warn!(
                                    "Received response for unknown request ID: {:?}",
                                    response.id
                                );
                            }
                        }
                    }
                    // Try to parse as notification
                    else if let Ok(notification) =
                        serde_json::from_str::<JsonRpcNotification>(line)
                    {
                        if let Some(subscription_id) = notification
                            .params
                            .as_ref()
                            .and_then(|params| params.get("_meta"))
                            .and_then(|meta| meta.get(SUBSCRIPTION_ID_META_KEY))
                        {
                            if let Some(sender) = subscription_senders
                                .lock()
                                .await
                                .get(subscription_id)
                                .cloned()
                            {
                                let _ = sender.send(notification.clone());
                            }
                        }
                        if notification_sender.send(notification).is_err() {
                            tracing::debug!("Notification receiver dropped");
                            break;
                        }
                    } else {
                        tracing::warn!("Failed to parse message: {}", line);
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading from stdout: {}", e);
                    break;
                }
            }
        }
    }
}

#[async_trait]
impl Transport for StdioClientTransport {
    async fn send_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        let writer = self
            .stdin_writer
            .as_mut()
            .ok_or_else(|| McpError::transport("Transport not connected"))?;

        let (sender, receiver) = tokio::sync::oneshot::channel();

        // Store the pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request.id.clone(), sender);
        }

        // Send the request
        let request_line = serde_json::to_string(&request).map_err(McpError::serialization)?;

        tracing::trace!("Sending: {}", request_line);

        writer
            .write_all(request_line.as_bytes())
            .await
            .map_err(|e| McpError::transport(format!("Failed to write request: {e}")))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| McpError::transport(format!("Failed to write newline: {e}")))?;
        writer
            .flush()
            .await
            .map_err(|e| McpError::transport(format!("Failed to flush: {e}")))?;

        // Wait for response with timeout
        let timeout_duration = Duration::from_millis(self.config.read_timeout_ms.unwrap_or(60_000));

        let response = timeout(timeout_duration, receiver)
            .await
            .map_err(|_| McpError::timeout("Request timeout"))?
            .map_err(|_| McpError::transport("Response channel closed"))??;

        Ok(response)
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let writer = self
            .stdin_writer
            .as_mut()
            .ok_or_else(|| McpError::transport("Transport not connected"))?;

        let notification_line =
            serde_json::to_string(&notification).map_err(McpError::serialization)?;

        tracing::trace!("Sending notification: {}", notification_line);

        writer
            .write_all(notification_line.as_bytes())
            .await
            .map_err(|e| McpError::transport(format!("Failed to write notification: {e}")))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| McpError::transport(format!("Failed to write newline: {e}")))?;
        writer
            .flush()
            .await
            .map_err(|e| McpError::transport(format!("Failed to flush: {e}")))?;

        Ok(())
    }

    async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>> {
        if let Some(ref mut receiver) = self.notification_receiver {
            match receiver.try_recv() {
                Ok(notification) => Ok(Some(notification)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    Err(McpError::transport("Notification channel disconnected"))
                }
            }
        } else {
            Ok(None)
        }
    }

    async fn open_subscription(
        &mut self,
        request: JsonRpcRequest,
    ) -> McpResult<ClientSubscription> {
        if request.method != methods::SUBSCRIPTIONS_LISTEN {
            return Err(McpError::InvalidParams(
                "open_subscription requires subscriptions/listen".to_string(),
            ));
        }
        let writer = self
            .stdin_writer
            .as_mut()
            .ok_or_else(|| McpError::transport("Transport not connected"))?;
        let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
        let (notification_sender, notification_receiver) = mpsc::unbounded_channel();
        self.pending_requests
            .lock()
            .await
            .insert(request.id.clone(), response_sender);
        self.subscription_senders
            .lock()
            .await
            .insert(request.id.clone(), notification_sender);
        let line = serde_json::to_string(&request).map_err(McpError::serialization)?;
        if let Err(error) = async {
            writer.write_all(line.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await
        }
        .await
        {
            self.pending_requests.lock().await.remove(&request.id);
            self.subscription_senders.lock().await.remove(&request.id);
            return Err(McpError::io(error));
        }
        Ok(ClientSubscription::new(
            request.id,
            notification_receiver,
            response_receiver,
        ))
    }

    async fn cancel_subscription(&mut self, request_id: &Value) -> McpResult<()> {
        let notification = JsonRpcNotification::new(
            methods::CANCELLED.to_string(),
            Some(serde_json::json!({"requestId": request_id})),
        )?;
        self.send_notification(notification).await
    }

    async fn close(&mut self) -> McpResult<()> {
        tracing::debug!("Closing STDIO transport");

        self.state = ConnectionState::Closing;

        // Close stdin to signal the server to shut down
        if let Some(mut writer) = self.stdin_writer.take() {
            let _ = writer.shutdown().await;
        }

        // Wait for the child process to exit
        if let Some(mut child) = self.child.take() {
            match timeout(Duration::from_secs(5), child.wait()).await {
                Ok(Ok(status)) => {
                    tracing::debug!("Server process exited with status: {}", status);
                }
                Ok(Err(e)) => {
                    tracing::warn!("Error waiting for server process: {}", e);
                }
                Err(_) => {
                    tracing::warn!("Timeout waiting for server process, killing it");
                    let _ = child.kill().await;
                }
            }
        }

        self.state = ConnectionState::Disconnected;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    fn connection_info(&self) -> String {
        let state = &self.state;
        format!("STDIO transport (state: {state:?})")
    }
}

/// STDIO transport for MCP servers
///
/// This transport communicates with an MCP client via STDIO (standard input/output).
/// It reads requests from stdin and writes responses to stdout.
pub struct StdioServerTransport {
    stdin_reader: Option<BufReader<tokio::io::Stdin>>,
    stdout_writer: Option<Arc<Mutex<BufWriter<tokio::io::Stdout>>>>,
    #[allow(dead_code)]
    config: TransportConfig,
    running: bool,
    request_handler: Option<ServerRequestHandler>,
    capabilities: ServerCapabilities,
    subscriptions: Arc<RwLock<HashMap<Value, SubscriptionFilter>>>,
    task_notifications: Option<broadcast::Receiver<JsonRpcNotification>>,
}

impl StdioServerTransport {
    /// Create a new STDIO server transport
    ///
    /// # Returns
    /// New STDIO server transport instance
    pub fn new() -> Self {
        Self::with_config(TransportConfig::default())
    }

    /// Create a new STDIO server transport with custom configuration
    ///
    /// # Arguments
    /// * `config` - Transport configuration
    ///
    /// # Returns
    /// New STDIO server transport instance
    pub fn with_config(config: TransportConfig) -> Self {
        let stdin_reader = BufReader::new(tokio::io::stdin());
        let stdout_writer = Arc::new(Mutex::new(BufWriter::new(tokio::io::stdout())));

        Self {
            stdin_reader: Some(stdin_reader),
            stdout_writer: Some(stdout_writer),
            config,
            running: false,
            request_handler: None,
            capabilities: ServerCapabilities::default(),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            task_notifications: None,
        }
    }
}

#[async_trait]
impl ServerTransport for StdioServerTransport {
    async fn start(&mut self) -> McpResult<()> {
        tracing::debug!("Starting STDIO server transport");

        let mut reader = self
            .stdin_reader
            .take()
            .ok_or_else(|| McpError::transport("STDIN reader already taken"))?;
        let writer = self
            .stdout_writer
            .as_ref()
            .cloned()
            .ok_or_else(|| McpError::transport("STDOUT writer is unavailable"))?;

        self.running = true;
        let request_handler = self.request_handler.clone();
        let subscriptions = self.subscriptions.clone();
        if let Some(mut task_notifications) = self.task_notifications.take() {
            let task_writer = writer.clone();
            let task_subscriptions = subscriptions.clone();
            tokio::spawn(async move {
                while let Ok(notification) = task_notifications.recv().await {
                    let entries = task_subscriptions.read().await.clone();
                    for (id, filter) in entries {
                        if !filter.matches(&notification.method, notification.params.as_ref()) {
                            continue;
                        }
                        let mut notification = notification.clone();
                        add_subscription_id(&mut notification, &id);
                        let _ = write_stdio_line(&task_writer, &notification).await;
                    }
                }
            });
        }

        let mut line = String::new();
        loop {
            line.clear();

            match reader.read_line(&mut line).await {
                Ok(0) => {
                    tracing::debug!("STDIN closed, stopping server");
                    break;
                }
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    tracing::trace!("Received: {}", line);

                    let parsed: Value = match serde_json::from_str(line) {
                        Ok(value) => value,
                        Err(error) => {
                            tracing::warn!(%error, "failed to parse STDIO JSON");
                            continue;
                        }
                    };
                    if parsed.get("method").and_then(Value::as_str) == Some(methods::CANCELLED)
                        && parsed.get("id").is_none()
                    {
                        if let Some(request_id) = parsed
                            .get("params")
                            .and_then(|params| params.get("requestId"))
                            .cloned()
                        {
                            if subscriptions.write().await.remove(&request_id).is_some() {
                                let mut meta = HashMap::new();
                                meta.insert(
                                    SUBSCRIPTION_ID_META_KEY.to_string(),
                                    request_id.clone(),
                                );
                                let response = JsonRpcResponse::success(
                                    request_id,
                                    serde_json::to_value(SubscriptionsListenResult {
                                        result_type: "complete".to_string(),
                                        meta,
                                    })?,
                                )?;
                                write_stdio_line(&writer, &response).await?;
                            }
                        }
                        continue;
                    }

                    // Parse the request
                    match serde_json::from_value::<JsonRpcRequest>(parsed) {
                        Ok(request) => {
                            if request.method == methods::SUBSCRIPTIONS_LISTEN {
                                let context =
                                    modern_request_context(&request)?.ok_or_else(|| {
                                        McpError::InvalidParams(
                                            "subscriptions/listen requires modern metadata"
                                                .to_string(),
                                        )
                                    })?;
                                let params: SubscriptionsListenParams = serde_json::from_value(
                                    request.params.clone().ok_or_else(|| {
                                        McpError::InvalidParams(
                                            "missing subscription filter".to_string(),
                                        )
                                    })?,
                                )?;
                                if params.notifications.requests_tasks()
                                    && !has_tasks_extension(&context.client_capabilities)
                                {
                                    let error = McpError::MissingRequiredClientCapability(
                                        serde_json::json!({"extensions": {(TASKS_EXTENSION_ID): {}}}),
                                    );
                                    let (code, data) = json_rpc_error_details(&error);
                                    let response = JsonRpcError::error(
                                        request.id,
                                        code,
                                        error.to_string(),
                                        data,
                                    );
                                    write_stdio_line(&writer, &response).await?;
                                    continue;
                                }
                                let accepted = accepted_subscription_filter(
                                    &params.notifications,
                                    &self.capabilities,
                                );
                                subscriptions
                                    .write()
                                    .await
                                    .insert(request.id.clone(), accepted.clone());
                                let mut meta = HashMap::new();
                                meta.insert(
                                    SUBSCRIPTION_ID_META_KEY.to_string(),
                                    request.id.clone(),
                                );
                                let acknowledgement = JsonRpcNotification::new(
                                    methods::SUBSCRIPTIONS_ACKNOWLEDGED.to_string(),
                                    Some(SubscriptionsAcknowledgedParams {
                                        notifications: accepted,
                                        meta,
                                    }),
                                )?;
                                write_stdio_line(&writer, &acknowledgement).await?;
                                continue;
                            }
                            let response_result = if let Some(ref handler) = request_handler {
                                // Use the provided request handler
                                handler(request.clone()).await
                            } else {
                                // Fall back to error if no handler is set
                                Err(McpError::protocol(format!(
                                    "Method '{}' not found",
                                    request.method
                                )))
                            };

                            let response_or_error = match response_result {
                                Ok(response) => serde_json::to_string(&response),
                                Err(error) => {
                                    // Convert McpError to JsonRpcError
                                    let (code, data) = json_rpc_error_details(&error);
                                    let json_rpc_error = crate::protocol::types::JsonRpcError {
                                        jsonrpc: "2.0".to_string(),
                                        id: request.id,
                                        error: crate::protocol::types::ErrorObject {
                                            code,
                                            message: error.to_string(),
                                            data,
                                        },
                                    };
                                    serde_json::to_string(&json_rpc_error)
                                }
                            };

                            let response_line =
                                response_or_error.map_err(McpError::serialization)?;

                            tracing::trace!("Sending: {}", response_line);

                            let mut writer_guard = writer.lock().await;
                            writer_guard
                                .write_all(response_line.as_bytes())
                                .await
                                .map_err(|e| {
                                    McpError::transport(format!("Failed to write response: {e}"))
                                })?;
                            writer_guard.write_all(b"\n").await.map_err(|e| {
                                McpError::transport(format!("Failed to write newline: {e}"))
                            })?;
                            writer_guard.flush().await.map_err(|e| {
                                McpError::transport(format!("Failed to flush: {e}"))
                            })?;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse request: {} - Error: {}", line, e);
                            // Send parse error response if we can extract an ID
                            // For now, just continue
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading from stdin: {}", e);
                    return Err(McpError::io(e));
                }
            }
        }

        Ok(())
    }

    fn set_request_handler(&mut self, handler: ServerRequestHandler) {
        self.request_handler = Some(handler);
    }

    fn set_server_capabilities(&mut self, capabilities: ServerCapabilities) -> McpResult<()> {
        self.capabilities = capabilities;
        Ok(())
    }

    fn set_task_notifications(
        &mut self,
        receiver: broadcast::Receiver<JsonRpcNotification>,
    ) -> McpResult<()> {
        self.task_notifications = Some(receiver);
        Ok(())
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let writer = self
            .stdout_writer
            .as_ref()
            .ok_or_else(|| McpError::transport("STDOUT writer not available"))?;

        let subscriptions = self.subscriptions.read().await.clone();
        if subscriptions.is_empty() {
            // Preserve legacy MCP behavior when no 2026 subscription is open.
            tracing::trace!(method = %notification.method, "sending legacy notification");
            write_stdio_line(writer, &notification).await?;
        } else {
            for (id, filter) in subscriptions {
                if !filter.matches(&notification.method, notification.params.as_ref()) {
                    continue;
                }
                let mut routed = notification.clone();
                add_subscription_id(&mut routed, &id);
                write_stdio_line(writer, &routed).await?;
            }
        }

        Ok(())
    }

    async fn stop(&mut self) -> McpResult<()> {
        tracing::debug!("Stopping STDIO server transport");
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn server_info(&self) -> String {
        format!("STDIO server transport (running: {})", self.running)
    }
}

// Backward compatibility method for tests
impl StdioServerTransport {
    /// Backward compatibility method for tests
    /// a default response for testing purposes
    pub async fn handle_request(&mut self, request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
        // Default implementation for tests - return method not found error
        Err(McpError::protocol(format!(
            "Method '{}' not found (test mode)",
            request.method
        )))
    }
}

impl Default for StdioServerTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for StdioClientTransport {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Try to kill the child process if it's still running
            let _ = child.start_kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::{mpsc, Mutex};

    #[test]
    fn test_stdio_server_creation() {
        let transport = StdioServerTransport::new();
        assert!(!transport.is_running());
        assert!(transport.stdin_reader.is_some());
        assert!(transport.stdout_writer.is_some());
    }

    #[test]
    fn test_stdio_server_with_config() {
        let config = TransportConfig {
            read_timeout_ms: Some(30_000),
            ..Default::default()
        };

        let transport = StdioServerTransport::with_config(config);
        assert_eq!(transport.config.read_timeout_ms, Some(30_000));
    }

    #[tokio::test]
    async fn test_stdio_server_handle_request() {
        let mut transport = StdioServerTransport::new();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "unknown_method".to_string(),
            params: None,
        };

        let result = transport.handle_request(request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            McpError::Protocol(msg) => assert!(msg.contains("unknown_method")),
            _ => panic!("Expected Protocol error"),
        }
    }

    // ============================================================================
    // StdioClientTransport Tests
    // ============================================================================

    #[tokio::test]
    async fn test_client_transport_creation_failure() {
        // Test with invalid command
        let result = StdioClientTransport::new("/nonexistent/command", vec!["arg1"]).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::Transport(msg) => assert!(msg.contains("Failed to start server process")),
            _ => panic!("Expected Transport error"),
        }
    }

    #[tokio::test]
    async fn test_client_transport_with_config() {
        let config = TransportConfig {
            read_timeout_ms: Some(5000),
            max_message_size: Some(2048),
            ..Default::default()
        };

        // Test with echo command (available on most systems)
        let result = StdioClientTransport::with_config("echo", vec!["test"], config.clone()).await;

        // The command should start but may exit immediately
        // We're testing the transport creation logic
        if let Ok(transport) = result {
            assert_eq!(transport.config.read_timeout_ms, Some(5000));
            assert_eq!(transport.config.max_message_size, Some(2048));
        }
    }

    #[tokio::test]
    async fn test_client_send_request_disconnected() {
        let mut transport = StdioClientTransport {
            child: None,
            stdin_writer: None,
            stdout_reader: None,
            notification_receiver: None,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            subscription_senders: Arc::new(Mutex::new(HashMap::new())),
            config: TransportConfig::default(),
            state: ConnectionState::Disconnected,
        };

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test_method".to_string(),
            params: None,
        };

        let result = transport.send_request(request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::Transport(msg) => assert!(msg.contains("not connected")),
            _ => panic!("Expected Transport error"),
        }
    }

    #[tokio::test]
    async fn test_client_receive_notification() {
        let (tx, rx) = mpsc::unbounded_channel();

        let mut transport = StdioClientTransport {
            child: None,
            stdin_writer: None,
            stdout_reader: None,
            notification_receiver: Some(rx),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            subscription_senders: Arc::new(Mutex::new(HashMap::new())),
            config: TransportConfig::default(),
            state: ConnectionState::Connected,
        };

        // Send a notification through the channel
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_notification".to_string(),
            params: Some(json!({"test": true})),
        };
        tx.send(notification.clone()).unwrap();

        let received = transport.receive_notification().await.unwrap();
        assert_eq!(received.unwrap().method, "test_notification");
    }

    #[tokio::test]
    async fn test_client_receive_notification_timeout() {
        let (_tx, rx) = mpsc::unbounded_channel();

        let mut transport = StdioClientTransport {
            child: None,
            stdin_writer: None,
            stdout_reader: None,
            notification_receiver: Some(rx),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            subscription_senders: Arc::new(Mutex::new(HashMap::new())),
            config: TransportConfig {
                read_timeout_ms: Some(100),
                ..Default::default()
            },
            state: ConnectionState::Connected,
        };

        let result = transport.receive_notification().await;
        // When no notification is available, it returns Ok(None) not an error
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // Note: Integration tests with actual processes would go in tests/integration/
}

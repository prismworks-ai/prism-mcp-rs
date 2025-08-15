// ! STDIO transport implementation for MCP
// !
// ! Module provides STDIO-based transport for MCP communication,
// ! which is commonly used for command-line tools and process communication.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc};
use tokio::time::{Duration, timeout};

use crate::core::error::{McpError, McpResult};
use crate::protocol::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::transport::traits::{
    ConnectionState, ServerRequestHandler, ServerTransport, Transport, TransportConfig,
};

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
    pending_requests: Arc<Mutex<HashMap<Value, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
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
        let command_str = command.as_ref();
        let args_str: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();

        tracing::debug!("Starting MCP server: {} {:?}", command_str, args_str);

        let mut child = Command::new(command_str)
            .args(&args_str)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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

        // Start message processing task
        let reader_pending_requests = pending_requests.clone();
        let reader = stdout_reader;
        tokio::spawn(async move {
            Self::message_processor(reader, notification_sender, reader_pending_requests).await;
        });

        Ok(Self {
            child: Some(child),
            stdin_writer: Some(stdin_writer),
            stdout_reader: None, // Moved to processor task
            notification_receiver: Some(notification_receiver),
            pending_requests,
            config,
            state: ConnectionState::Connected,
        })
    }

    async fn message_processor(
        mut reader: BufReader<tokio::process::ChildStdout>,
        notification_sender: mpsc::UnboundedSender<JsonRpcNotification>,
        pending_requests: Arc<Mutex<HashMap<Value, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
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

                    // Try to parse as response first
                    if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(line) {
                        let mut pending = pending_requests.lock().await;
                        match pending.remove(&response.id) {
                            Some(sender) => {
                                let _ = sender.send(response);
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
            .map_err(|_| McpError::transport("Response channel closed"))?;

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
    stdout_writer: Option<BufWriter<tokio::io::Stdout>>,
    #[allow(dead_code)]
    config: TransportConfig,
    running: bool,
    request_handler: Option<ServerRequestHandler>,
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
        let stdout_writer = BufWriter::new(tokio::io::stdout());

        Self {
            stdin_reader: Some(stdin_reader),
            stdout_writer: Some(stdout_writer),
            config,
            running: false,
            request_handler: None,
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
        let mut writer = self
            .stdout_writer
            .take()
            .ok_or_else(|| McpError::transport("STDOUT writer already taken"))?;

        self.running = true;
        let request_handler = self.request_handler.clone();

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

                    // Parse the request
                    match serde_json::from_str::<JsonRpcRequest>(line) {
                        Ok(request) => {
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
                                    let json_rpc_error = crate::protocol::types::JsonRpcError {
                                        jsonrpc: "2.0".to_string(),
                                        id: request.id,
                                        error: crate::protocol::types::ErrorObject {
                                            code: match error {
                                                McpError::Protocol(ref msg) if msg.contains("not found") => {
                                                    error_codes::METHOD_NOT_FOUND
                                                }
                                                _ => crate::protocol::types::error_codes::INTERNAL_ERROR,
                                            },
                                            message: error.to_string(),
                                            data: None,
                                        },
                                    };
                                    serde_json::to_string(&json_rpc_error)
                                }
                            };

                            let response_line =
                                response_or_error.map_err(McpError::serialization)?;

                            tracing::trace!("Sending: {}", response_line);

                            writer
                                .write_all(response_line.as_bytes())
                                .await
                                .map_err(|e| {
                                    McpError::transport(format!("Failed to write response: {e}"))
                                })?;
                            writer.write_all(b"\n").await.map_err(|e| {
                                McpError::transport(format!("Failed to write newline: {e}"))
                            })?;
                            writer.flush().await.map_err(|e| {
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

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let writer = self
            .stdout_writer
            .as_mut()
            .ok_or_else(|| McpError::transport("STDOUT writer not available"))?;

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
    use tokio::sync::{Mutex, mpsc};

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
        if result.is_ok() {
            let transport = result.unwrap();
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

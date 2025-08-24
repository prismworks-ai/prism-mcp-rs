//! Example demonstrating custom transport implementation
//!
//! This example shows how to implement a custom transport for MCP communication.
//! The custom transport uses in-memory channels for testing and demonstration.

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::transport::traits::{ServerRequestHandler, ServerTransport};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Custom in-memory transport for testing
#[derive(Clone)]
struct MemoryTransport {
    /// Incoming message queue
    incoming: Arc<Mutex<VecDeque<JsonRpcMessage>>>,
    /// Outgoing message queue
    outgoing: Arc<Mutex<VecDeque<JsonRpcMessage>>>,
    /// Request handler
    handler: Arc<RwLock<Option<ServerRequestHandler>>>,
    /// Running state
    running: Arc<RwLock<bool>>,
}

impl MemoryTransport {
    /// Create a new memory transport
    fn new() -> Self {
        Self {
            incoming: Arc::new(Mutex::new(VecDeque::new())),
            outgoing: Arc::new(Mutex::new(VecDeque::new())),
            handler: Arc::new(RwLock::new(None)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Send a message to the transport (simulating client sending to server)
    async fn send_message(&self, message: JsonRpcMessage) {
        let mut incoming = self.incoming.lock().await;
        incoming.push_back(message);
    }

    /// Receive a message from the transport (simulating server response)
    async fn receive_message(&self) -> Option<JsonRpcMessage> {
        let mut outgoing = self.outgoing.lock().await;
        outgoing.pop_front()
    }

    /// Process messages in the background
    #[allow(clippy::collapsible_match)]
    async fn process_messages(&self) {
        while *self.running.read().await {
            // Get next incoming message
            let message = {
                let mut incoming = self.incoming.lock().await;
                incoming.pop_front()
            };

            if let Some(message) = message {
                // Process the message
                if let JsonRpcMessage::Request(request) = message {
                    // Get the handler
                    let handler = self.handler.read().await;
                    if let Some(ref handler) = *handler {
                        // Process the request
                        match handler(request).await {
                            Ok(response) => {
                                let mut outgoing = self.outgoing.lock().await;
                                outgoing.push_back(JsonRpcMessage::Response(response));
                            }
                            Err(err) => {
                                eprintln!("Error processing request: {}", err);
                            }
                        }
                    }
                }
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
}

#[async_trait]
impl ServerTransport for MemoryTransport {
    async fn start(&mut self) -> McpResult<()> {
        *self.running.write().await = true;

        // Start background processing
        let self_clone = Self {
            incoming: Arc::clone(&self.incoming),
            outgoing: Arc::clone(&self.outgoing),
            handler: Arc::clone(&self.handler),
            running: Arc::clone(&self.running),
        };

        tokio::spawn(async move {
            self_clone.process_messages().await;
        });

        Ok(())
    }

    async fn stop(&mut self) -> McpResult<()> {
        *self.running.write().await = false;
        Ok(())
    }

    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        let mut outgoing = self.outgoing.lock().await;
        outgoing.push_back(JsonRpcMessage::Notification(notification));
        Ok(())
    }

    fn set_request_handler(&mut self, handler: ServerRequestHandler) {
        let handler_clone = Arc::clone(&self.handler);
        tokio::spawn(async move {
            let mut h = handler_clone.write().await;
            *h = Some(handler);
        });
    }
}

/// Custom tool handler for demonstration
struct EchoTool;

#[async_trait]
impl ToolHandler for EchoTool {
    async fn call(
        &self,
        arguments: std::collections::HashMap<String, Value>,
    ) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message provided");

        Ok(ToolResult {
            content: vec![Content::text(format!("Echo: {}", message))],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Custom Transport Example");
    println!("============================\n");

    // Create server with custom configuration
    let mut server = McpServer::new("custom-transport-server".to_string(), "1.0.0".to_string());

    // Add a tool
    server
        .add_tool(
            "echo",
            Some("Echo back a message"),
            json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo"
                    }
                },
                "required": ["message"]
            }),
            EchoTool,
        )
        .await?;

    println!("✅ Server configured with echo tool");

    // Create custom transport
    let transport = MemoryTransport::new();

    // Start server with custom transport
    server.start(transport.clone()).await?;
    println!("✅ Server started with custom memory transport\n");

    // Simulate client interactions
    println!("📤 Simulating client requests...\n");

    // Send initialization request
    let init_request = JsonRpcRequest::new(
        json!(1),
        "initialize".to_string(),
        Some(InitializeParams::new(
            LATEST_PROTOCOL_VERSION.to_string(),
            ClientCapabilities::default(),
            ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                title: Some("Test Client".to_string()),
            },
        )),
    )?;

    transport
        .send_message(JsonRpcMessage::Request(init_request))
        .await;

    // Wait for response
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    if let Some(response) = transport.receive_message().await {
        println!("📥 Received initialization response: {:?}\n", response);
    }

    // Send tool list request
    let list_tools_request =
        JsonRpcRequest::new(json!(2), "tools/list".to_string(), None::<Value>)?;

    transport
        .send_message(JsonRpcMessage::Request(list_tools_request))
        .await;

    // Wait for response
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    if let Some(response) = transport.receive_message().await {
        println!("📥 Received tools list response: {:?}\n", response);
    }

    // Call the echo tool
    let call_tool_request = JsonRpcRequest::new(
        json!(3),
        "tools/call".to_string(),
        Some(CallToolParams {
            name: "echo".to_string(),
            arguments: Some({
                let mut args = std::collections::HashMap::new();
                args.insert("message".to_string(), json!("Hello from custom transport!"));
                args
            }),
            meta: None,
        }),
    )?;

    transport
        .send_message(JsonRpcMessage::Request(call_tool_request))
        .await;

    // Wait for response
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    if let Some(response) = transport.receive_message().await {
        println!("📥 Received tool call response: {:?}\n", response);
    }

    // Stop the server
    server.stop().await?;
    println!("✅ Server stopped successfully");

    Ok(())
}

// ! WebSocket Server Example
// !
// ! This example demonstrates how to create an MCP server that communicates
// ! over WebSocket connections for real-time bidirectional communication.
// !
// ! ## Required Features
// ! This example requires the following features to be enabled:
// ! ```toml
// ! [dependencies]
// ! prism-mcp-rs = { version = "*", features = ["websocket-server"] }
// ! ```
// !
// ! ## Running this Example
// ! ```bash
// ! cargo run --example websocket_server --features "websocket-server"
// ! ```

use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::HashMap;

use prism_mcp_rs::{
    core::{
        error::{McpError, McpResult},
        resource::ResourceHandler,
        tool::ToolHandler,
    },
    protocol::types::{Content, ResourceContents, ResourceInfo, ToolResult},
    server::McpServer,
    transport::websocket::WebSocketServerTransport,
};

/// WebSocket echo tool with connection info
struct WebSocketEchoHandler;

#[async_trait]
impl ToolHandler for WebSocketEchoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello from WebSocket!");

        let add_timestamp = arguments
            .get("add_timestamp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let add_connection_info = arguments
            .get("add_connection_info")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let broadcast = arguments
            .get("broadcast")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut response = message.to_string();

        if add_timestamp {
            #[cfg(feature = "chrono")]
            {
                response = format!(
                    "[{}] {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                    response
                );
            }
            #[cfg(not(feature = "chrono"))]
            {
                response = format!("[timestamp] {}", response);
            }
        }

        if add_connection_info {
            response = format!("{response} (via WebSocket)");
        }

        if broadcast {
            response = format!("🔊 BROADCAST: {response}");
        }

        Ok(ToolResult {
            content: vec![Content::text(response)],
            is_error: None,
            structured_content: None,
            meta: None,
        })
    }
}

/// Real-time chat tool for WebSocket connections
struct WebSocketChatHandler;

#[async_trait]
impl ToolHandler for WebSocketChatHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let username = arguments
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("Anonymous");

        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation("Missing 'message' parameter".to_string()))?;

        let room = arguments
            .get("room")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        Ok(ToolResult {
            content: vec![Content::text(format!(
                "Chat: [{room}] {username}: {message}"
            ))],
            is_error: None,
            structured_content: None,
            meta: None,
        })
    }
}

/// WebSocket connection status resource
struct WebSocketStatusHandler;

#[async_trait]
impl ResourceHandler for WebSocketStatusHandler {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        match uri {
            "ws://server/status" => {
                let status = json!({
                    "transport": "WebSocket",
                    "protocol": "MCP over WebSocket",
                    "features": ["bidirectional", "real-time", "low-latency"],
                    "connection_info": {
                        "active_connections": 2,
                        "total_connections": 15,
                        "uptime": "5 minutes"
                    },
                    "capabilities": [
                        "instant messaging",
                        "real-time notifications",
                        "persistent connections",
                        "automatic reconnection"
                    ]
                });

                Ok(vec![ResourceContents::Text {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: serde_json::to_string_pretty(&status)?,
                    meta: None,
                }])
            }
            "ws://server/connections" => {
                let connections = json!({
                    "active_connections": [
                        {
                            "id": "conn_001",
                            "client": "WebSocket Client",
                            "connected_at": "2024-01-15T10:30:00Z",
                            "messages_sent": 42,
                            "messages_received": 38
                        },
                        {
                            "id": "conn_002",
                            "client": "Chat Client",
                            "connected_at": "2024-01-15T10:32:15Z",
                            "messages_sent": 15,
                            "messages_received": 23
                        }
                    ],
                    "total_messages": 118,
                    "protocol_version": "MCP/WebSocket 1.0"
                });

                Ok(vec![ResourceContents::Text {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: serde_json::to_string_pretty(&connections)?,
                    meta: None,
                }])
            }
            _ => Err(McpError::ResourceNotFound(uri.to_string())),
        }
    }

    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        Ok(vec![
            ResourceInfo {
                uri: "ws://server/status".to_string(),
                name: "WebSocket Server Status".to_string(),
                description: Some(
                    "Current status and capabilities of WebSocket server".to_string(),
                ),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                size: None,
                title: None,
                meta: None,
            },
            ResourceInfo {
                uri: "ws://server/connections".to_string(),
                name: "Active WebSocket Connections".to_string(),
                description: Some(
                    "Information about currently connected WebSocket clients".to_string(),
                ),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                size: None,
                title: None,
                meta: None,
            },
        ])
    }

    async fn subscribe(&self, _uri: &str) -> McpResult<()> {
        // In a real implementation, this would set up real-time updates
        Ok(())
    }

    async fn unsubscribe(&self, _uri: &str) -> McpResult<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    #[cfg(feature = "tracing-subscriber")]
    tracing_subscriber::fmt::init();

    let mut server = McpServer::new("websocket-mcp-server".to_string(), "1.0.0".to_string());

    // Add WebSocket echo tool
    server
        .add_tool(
            "ws_echo".to_string(),
            Some("improved echo tool with WebSocket-specific features".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo back"
                    },
                    "add_timestamp": {
                        "type": "boolean",
                        "description": "Add timestamp to the echoed message",
                        "default": false
                    },
                    "add_connection_info": {
                        "type": "boolean",
                        "description": "Add WebSocket connection information",
                        "default": false
                    },
                    "broadcast": {
                        "type": "boolean",
                        "description": "Mark message as broadcast to all clients",
                        "default": false
                    }
                },
                "required": ["message"]
            }),
            WebSocketEchoHandler,
        )
        .await?;

    // Add WebSocket chat tool
    server
        .add_tool(
            "ws_chat".to_string(),
            Some("Real-time chat tool for WebSocket connections".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "username": {
                        "type": "string",
                        "description": "Username of the chat participant",
                        "default": "Anonymous"
                    },
                    "message": {
                        "type": "string",
                        "description": "Chat message to send"
                    },
                    "room": {
                        "type": "string",
                        "description": "Chat room name",
                        "default": "general"
                    }
                },
                "required": ["message"]
            }),
            WebSocketChatHandler,
        )
        .await?;

    // Add WebSocket status resources
    server
        .add_resource_detailed(
            ResourceInfo {
                uri: "ws://server/".to_string(),
                name: "WebSocket Server Resources".to_string(),
                description: Some("WebSocket server status and connection information".to_string()),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                size: None,
                title: None,
                meta: None,
            },
            WebSocketStatusHandler,
        )
        .await?;

    // Start WebSocket server
    tracing::info!("Starting WebSocket MCP server on ws://localhost:8081");
    tracing::info!("Features:");
    tracing::info!("  - Bidirectional real-time communication");
    tracing::info!("  - Multiple concurrent connections");
    tracing::info!("  - Automatic message routing");
    tracing::info!("  - Low-latency responses");

    let transport = WebSocketServerTransport::new("0.0.0.0:8081");
    server.start(transport).await?;

    tracing::info!("WebSocket MCP server is running!");
    tracing::info!("Connect with a WebSocket client to: ws://localhost:8081");
    tracing::info!("Test tools: ws_echo, ws_chat");
    tracing::info!("Test resources: ws://server/status, ws://server/connections");

    // Keep running until interrupted
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");
    server.stop().await?;

    Ok(())
}

// ! HTTP Server Example
// !
// ! This example demonstrates how to create an MCP server that communicates
// ! over HTTP with Server-Sent Events for real-time notifications.
// !
// ! ## Required Features
// ! This example requires the following features to be enabled:
// ! ```toml
// ! [dependencies]
// ! prism-mcp-rs = { version = "*", features = ["http-server"] }
// ! ```
// !
// ! ## Running this Example
// ! ```bash
// ! cargo run --example http_server --features "http-server"
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
    server::HttpMcpServer,
    transport::http::HttpServerTransport,
};

/// HTTP-aware calculator tool
struct HttpCalculatorHandler;

#[async_trait]
impl ToolHandler for HttpCalculatorHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("add");

        let a = arguments
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::Validation("Missing or invalid 'a' parameter".to_string()))?;

        let b = arguments
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::Validation("Missing or invalid 'b' parameter".to_string()))?;

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Ok(ToolResult {
                        content: vec![Content::text("Error: Division by zero")],
                        is_error: Some(true),
                        structured_content: None,
                        meta: None,
                    });
                }
                a / b
            }
            "power" => a.powf(b),
            "modulo" => a % b,
            _ => {
                return Ok(ToolResult {
                    content: vec![Content::text(format!(
                        "Error: Unknown operation '{operation}'"
                    ))],
                    is_error: Some(true),
                    structured_content: None,
                    meta: None,
                });
            }
        };

        Ok(ToolResult {
            content: vec![Content::text(format!("{a} {operation} {b} = {result}"))],
            is_error: None,
            structured_content: None,
            meta: None,
        })
    }
}

/// HTTP status resource handler
struct HttpStatusHandler;

#[async_trait]
impl ResourceHandler for HttpStatusHandler {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        match uri {
            "http://server/status" => {
                let status = json!({
                    "transport": "HTTP",
                    "protocol": "MCP over HTTP",
                    "features": ["requests", "notifications", "sse"],
                    "endpoints": {
                        "requests": "/mcp",
                        "notifications": "/mcp/notify",
                        "events": "/mcp/events",
                        "health": "/health"
                    },
                    "uptime": "running",
                    "clients": "connected via HTTP"
                });

                Ok(vec![ResourceContents::Text {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: serde_json::to_string_pretty(&status)?,
                    meta: None,
                }])
            }
            "http://server/metrics" => {
                let metrics = json!({
                    "requests_processed": 42,
                    "notifications_sent": 15,
                    "sse_connections": 3,
                    "average_response_time_ms": 12.5,
                    "transport_type": "http"
                });

                Ok(vec![ResourceContents::Text {
                    uri: uri.to_string(),
                    mime_type: Some("application/json".to_string()),
                    text: serde_json::to_string_pretty(&metrics)?,
                    meta: None,
                }])
            }
            _ => Err(McpError::ResourceNotFound(uri.to_string())),
        }
    }

    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        Ok(vec![
            ResourceInfo {
                uri: "http://server/status".to_string(),
                name: "HTTP Server Status".to_string(),
                description: Some("Current status of the HTTP MCP server".to_string()),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                size: None,
                title: None,
                meta: None,
            },
            ResourceInfo {
                uri: "http://server/metrics".to_string(),
                name: "HTTP Server Metrics".to_string(),
                description: Some("Performance metrics for the HTTP transport".to_string()),
                mime_type: Some("application/json".to_string()),
                annotations: None,
                size: None,
                title: None,
                meta: None,
            },
        ])
    }

    async fn subscribe(&self, _uri: &str) -> McpResult<()> {
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

    let mut http_server = HttpMcpServer::new("http-mcp-server".to_string(), "1.0.0".to_string());

    // Get a reference to the underlying server for adding tools and resources
    let server = http_server.server().await;

    // Add HTTP-aware calculator tool
    {
        let server_guard = server.lock().await;
        server_guard
            .add_tool(
                "http_calculator".to_string(),
                Some("complete calculator with HTTP transport support".to_string()),
                json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["add", "subtract", "multiply", "divide", "power", "modulo"],
                            "description": "Mathematical operation to perform",
                            "default": "add"
                        },
                        "a": {
                            "type": "number",
                            "description": "First operand"
                        },
                        "b": {
                            "type": "number",
                            "description": "Second operand"
                        }
                    },
                    "required": ["a", "b"]
                }),
                HttpCalculatorHandler,
            )
            .await?;

        // Add HTTP status resources individually
        server_guard
            .add_resource_detailed(
                ResourceInfo {
                    uri: "http://server/status".to_string(),
                    name: "HTTP Server Status".to_string(),
                    description: Some("Current status of the HTTP MCP server".to_string()),
                    mime_type: Some("application/json".to_string()),
                    annotations: None,
                    size: None,
                    title: None,
                    meta: None,
                },
                HttpStatusHandler,
            )
            .await?;

        server_guard
            .add_resource_detailed(
                ResourceInfo {
                    uri: "http://server/metrics".to_string(),
                    name: "HTTP Server Metrics".to_string(),
                    description: Some("Performance metrics for the HTTP transport".to_string()),
                    mime_type: Some("application/json".to_string()),
                    annotations: None,
                    size: None,
                    title: None,
                    meta: None,
                },
                HttpStatusHandler,
            )
            .await?;
    }

    // Start HTTP server
    tracing::info!("Starting HTTP MCP server on http://localhost:3000");
    tracing::info!("Endpoints:");
    tracing::info!("  - POST /mcp - JSON-RPC requests");
    tracing::info!("  - POST /mcp/notify - Notifications");
    tracing::info!("  - GET /mcp/events - Server-Sent Events");
    tracing::info!("  - GET /health - Health check");

    let transport = HttpServerTransport::new("0.0.0.0:3000");
    http_server.start(transport).await?;

    tracing::info!("HTTP MCP server is running!");
    tracing::info!(
        "Test with: curl -X POST http://localhost:3000/mcp -H 'Content-Type: application/json' -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}}'"
    );

    // Keep running until interrupted
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");
    http_server.stop().await?;

    Ok(())
}

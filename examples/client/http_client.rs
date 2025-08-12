// ! HTTP Client Example
// !
// ! This example demonstrates how to create an MCP client that connects to
// ! an MCP server over HTTP with Server-Sent Events for notifications.
// !
// ! ## Required Features
// ! This example requires the following features to be enabled:
// ! ```toml
// ! [dependencies]
// ! prism-mcp-rs = { version = "*", features = ["http-client"] }
// ! ```
// !
// ! ## Running this Example
// ! ```bash
// ! cargo run --example http_client --features "http-client"
// ! ```

use serde_json::json;
use std::collections::HashMap;

use prism_mcp_rs::{
    client::session::SessionConfig,
    client::{ClientSession, McpClient},
    core::error::McpResult,
    protocol::types::ContentBlock as Content,
    transport::http::HttpClientTransport,
};

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    #[cfg(feature = "tracing-subscriber")]
    tracing_subscriber::fmt::init();

    tracing::info!("Starting HTTP MCP client example...");

    // Create client
    let client = McpClient::new("http-demo-client".to_string(), "1.0.0".to_string());

    // Create session for HTTP connection
    let session_config = SessionConfig {
        auto_reconnect: true,
        max_reconnect_attempts: 3,
        reconnect_delay_ms: 2000,
        connection_timeout_ms: 10000,
        heartbeat_interval_ms: 30000,
        ..Default::default()
    };

    let session = ClientSession::with_config(client, session_config);

    // Connect to HTTP server with SSE support
    tracing::info!("Connecting to HTTP server...");

    let transport = HttpClientTransport::new(
        "http://localhost:3000",
        Some("http://localhost:3000/mcp/events"), // SSE endpoint
    )
    .await?;

    match session.connect(transport).await {
        Ok(init_result) => {
            tracing::info!(
                "Connected to HTTP server: {} v{}",
                init_result.server_info.name,
                init_result.server_info.version
            );
            tracing::info!("Server capabilities: {:?}", init_result.capabilities);
        }
        Err(e) => {
            tracing::error!("Failed to connect to HTTP server: {}", e);
            return Err(e);
        }
    }

    // Get the client for operations
    let client = session.client();

    // Demonstrate HTTP-specific operations
    match demonstrate_http_operations(&client).await {
        Ok(_) => tracing::info!("All HTTP operations completed successfully"),
        Err(e) => tracing::error!("HTTP operation failed: {}", e),
    }

    // Disconnect from the server
    tracing::info!("Disconnecting from HTTP server...");
    session.disconnect().await?;

    tracing::info!("HTTP client example completed");
    Ok(())
}

async fn demonstrate_http_operations(
    client: &std::sync::Arc<tokio::sync::Mutex<McpClient>>,
) -> McpResult<()> {
    // 1. List available tools
    tracing::info!("=== Listing Tools via HTTP ===");
    {
        let client_guard = client.lock().await;
        let tools_result = client_guard.list_tools(None).await?;

        tracing::info!("Available tools via HTTP:");
        for tool in &tools_result.tools {
            tracing::info!(
                "  - {}: {}",
                tool.name,
                tool.description.as_deref().unwrap_or("No description")
            );
        }
    }

    // 2. Call the HTTP calculator tool
    tracing::info!("=== Calling HTTP Calculator Tool ===");
    {
        let client_guard = client.lock().await;
        let mut args = HashMap::new();
        args.insert("operation".to_string(), json!("multiply"));
        args.insert("a".to_string(), json!(25.5));
        args.insert("b".to_string(), json!(4.0));

        match client_guard
            .call_tool("http_calculator".to_string(), Some(args))
            .await
        {
            Ok(result) => {
                tracing::info!("HTTP Calculator result:");
                for content in &result.content {
                    match content {
                        Content::Text { text, .. } => {
                            tracing::info!("  {}", text);
                        }
                        _ => tracing::info!("  (non-text content)"),
                    }
                }
            }
            Err(e) => tracing::error!("HTTP Calculator tool failed: {}", e),
        }
    }

    // 3. Test with power operation
    tracing::info!("=== Testing Power Operation ===");
    {
        let client_guard = client.lock().await;
        let mut args = HashMap::new();
        args.insert("operation".to_string(), json!("power"));
        args.insert("a".to_string(), json!(2.0));
        args.insert("b".to_string(), json!(8.0));

        match client_guard
            .call_tool("http_calculator".to_string(), Some(args))
            .await
        {
            Ok(result) => {
                tracing::info!("Power operation result:");
                for content in &result.content {
                    match content {
                        Content::Text { text, .. } => {
                            tracing::info!("  {}", text);
                        }
                        _ => tracing::info!("  (non-text content)"),
                    }
                }
            }
            Err(e) => tracing::error!("Power operation failed: {}", e),
        }
    }

    // 4. List HTTP server resources
    tracing::info!("=== Listing HTTP Resources ===");
    {
        let client_guard = client.lock().await;
        let resources_result = client_guard.list_resources(None).await?;

        tracing::info!("Available HTTP resources:");
        for resource in &resources_result.resources {
            tracing::info!(
                "  - {}: {} ({})",
                resource.name,
                resource.uri,
                resource.mime_type.as_deref().unwrap_or("unknown type")
            );
        }
    }

    // 5. Read HTTP server status
    tracing::info!("=== Reading HTTP Server Status ===");
    {
        let client_guard = client.lock().await;
        match client_guard
            .read_resource("http://server/status".to_string())
            .await
        {
            Ok(result) => {
                tracing::info!("HTTP Server status:");
                for content in &result.contents {
                    match content {
                        prism_mcp_rs::ResourceContents::Text { text, .. } => {
                            tracing::info!("  {}", text);
                        }
                        prism_mcp_rs::ResourceContents::Blob { .. } => {
                            tracing::info!("  (binary content)");
                        }
                    }
                }
            }
            Err(e) => tracing::error!("Failed to read HTTP server status: {}", e),
        }
    }

    // 6. Read HTTP server metrics
    tracing::info!("=== Reading HTTP Server Metrics ===");
    {
        let client_guard = client.lock().await;
        match client_guard
            .read_resource("http://server/metrics".to_string())
            .await
        {
            Ok(result) => {
                tracing::info!("HTTP Server metrics:");
                for content in &result.contents {
                    match content {
                        prism_mcp_rs::ResourceContents::Text { text, .. } => {
                            tracing::info!("  {}", text);
                        }
                        prism_mcp_rs::ResourceContents::Blob { .. } => {
                            tracing::info!("  (binary content)");
                        }
                    }
                }
            }
            Err(e) => tracing::error!("Failed to read HTTP server metrics: {}", e),
        }
    }

    // 7. Test ping over HTTP
    tracing::info!("=== Testing HTTP Ping ===");
    {
        let client_guard = client.lock().await;
        match client_guard.ping().await {
            Ok(_) => tracing::info!("HTTP Ping successful"),
            Err(e) => tracing::error!("HTTP Ping failed: {}", e),
        }
    }

    Ok(())
}

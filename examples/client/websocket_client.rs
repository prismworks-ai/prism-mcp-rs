// ! WebSocket Client Example
// !
// ! This example demonstrates how to create an MCP client that connects to
// ! an MCP server over WebSocket for real-time bidirectional communication.
// !
// ! ## Required Features
// ! This example requires the following features to be enabled:
// ! ```toml
// ! [dependencies]
// ! prism-mcp-rs = { version = "*", features = ["websocket-client"] }
// ! ```
// !
// ! ## Running this Example
// ! ```bash
// ! cargo run --example websocket_client --features "websocket-client"
// ! ```

use serde_json::json;
use std::collections::HashMap;

use prism_mcp_rs::{
    client::session::SessionConfig,
    client::{ClientSession, McpClient},
    core::error::McpResult,
    protocol::types::ContentBlock as Content,
    transport::websocket::WebSocketClientTransport,
};

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    #[cfg(feature = "tracing-subscriber")]
    tracing_subscriber::fmt::init();

    tracing::info!("Starting WebSocket MCP client example...");

    // Create client
    let client = McpClient::new("websocket-demo-client".to_string(), "1.0.0".to_string());

    // Create session for WebSocket connection
    let session_config = SessionConfig {
        auto_reconnect: true,
        max_reconnect_attempts: 5,
        reconnect_delay_ms: 1000,
        connection_timeout_ms: 15000,
        heartbeat_interval_ms: 20000,
        ..Default::default()
    };

    let session = ClientSession::with_config(client, session_config);

    // Connect to WebSocket server
    tracing::info!("Connecting to WebSocket server...");

    let transport = WebSocketClientTransport::new("ws://localhost:8081").await?;

    match session.connect(transport).await {
        Ok(init_result) => {
            tracing::info!(
                "Connected to WebSocket server: {} v{}",
                init_result.server_info.name,
                init_result.server_info.version
            );
            tracing::info!("Server capabilities: {:?}", init_result.capabilities);
        }
        Err(e) => {
            tracing::error!("Failed to connect to WebSocket server: {}", e);
            return Err(e);
        }
    }

    // Get the client for operations
    let client = session.client();

    // Demonstrate WebSocket-specific operations
    match demonstrate_websocket_operations(&client).await {
        Ok(_) => tracing::info!("All WebSocket operations completed successfully"),
        Err(e) => tracing::error!("WebSocket operation failed: {}", e),
    }

    // Disconnect from the server
    tracing::info!("Disconnecting from WebSocket server...");
    session.disconnect().await?;

    tracing::info!("WebSocket client example completed");
    Ok(())
}

async fn demonstrate_websocket_operations(
    client: &std::sync::Arc<tokio::sync::Mutex<McpClient>>,
) -> McpResult<()> {
    // 1. List available tools
    tracing::info!("=== Listing Tools via WebSocket ===");
    {
        let client_guard = client.lock().await;
        let tools_result = client_guard.list_tools(None).await?;

        tracing::info!("Available tools via WebSocket:");
        for tool in &tools_result.tools {
            tracing::info!(
                "  - {}: {}",
                tool.name,
                tool.description.as_deref().unwrap_or("No description")
            );
        }
    }

    // 2. Test WebSocket echo tool with basic message
    tracing::info!("=== Testing WebSocket Echo Tool ===");
    {
        let client_guard = client.lock().await;
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("Hello from WebSocket client!"));
        args.insert("add_timestamp".to_string(), json!(true));
        args.insert("add_connection_info".to_string(), json!(true));

        match client_guard
            .call_tool("ws_echo".to_string(), Some(args))
            .await
        {
            Ok(result) => {
                tracing::info!("WebSocket Echo result:");
                for content in &result.content {
                    match content {
                        Content::Text { text, .. } => {
                            tracing::info!("  {}", text);
                        }
                        _ => tracing::info!("  (non-text content)"),
                    }
                }
            }
            Err(e) => tracing::error!("WebSocket Echo tool failed: {}", e),
        }
    }

    // 3. Test WebSocket broadcast message
    tracing::info!("=== Testing WebSocket Broadcast ===");
    {
        let client_guard = client.lock().await;
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("Important announcement!"));
        args.insert("broadcast".to_string(), json!(true));
        args.insert("add_timestamp".to_string(), json!(true));

        match client_guard
            .call_tool("ws_echo".to_string(), Some(args))
            .await
        {
            Ok(result) => {
                tracing::info!("WebSocket Broadcast result:");
                for content in &result.content {
                    match content {
                        Content::Text { text, .. } => {
                            tracing::info!("  {}", text);
                        }
                        _ => tracing::info!("  (non-text content)"),
                    }
                }
            }
            Err(e) => tracing::error!("WebSocket Broadcast failed: {}", e),
        }
    }

    // 4. Test WebSocket chat functionality
    tracing::info!("=== Testing WebSocket Chat ===");
    {
        let client_guard = client.lock().await;
        let mut args = HashMap::new();
        args.insert("username".to_string(), json!("Alice"));
        args.insert("message".to_string(), json!("Hello everyone in the chat!"));
        args.insert("room".to_string(), json!("mcp-demo"));

        match client_guard
            .call_tool("ws_chat".to_string(), Some(args))
            .await
        {
            Ok(result) => {
                tracing::info!("WebSocket Chat result:");
                for content in &result.content {
                    match content {
                        Content::Text { text, .. } => {
                            tracing::info!("  {}", text);
                        }
                        _ => tracing::info!("  (non-text content)"),
                    }
                }
            }
            Err(e) => tracing::error!("WebSocket Chat failed: {}", e),
        }
    }

    // 5. Another chat message with different user
    tracing::info!("=== Testing Chat with Different User ===");
    {
        let client_guard = client.lock().await;
        let mut args = HashMap::new();
        args.insert("username".to_string(), json!("Bob"));
        args.insert(
            "message".to_string(),
            json!("WebSocket communication is so fast!"),
        );
        args.insert("room".to_string(), json!("mcp-demo"));

        match client_guard
            .call_tool("ws_chat".to_string(), Some(args))
            .await
        {
            Ok(result) => {
                tracing::info!("WebSocket Chat (Bob) result:");
                for content in &result.content {
                    match content {
                        Content::Text { text, .. } => {
                            tracing::info!("  {}", text);
                        }
                        _ => tracing::info!("  (non-text content)"),
                    }
                }
            }
            Err(e) => tracing::error!("WebSocket Chat (Bob) failed: {}", e),
        }
    }

    // 6. List WebSocket server resources
    tracing::info!("=== Listing WebSocket Resources ===");
    {
        let client_guard = client.lock().await;
        let resources_result = client_guard.list_resources(None).await?;

        tracing::info!("Available WebSocket resources:");
        for resource in &resources_result.resources {
            tracing::info!(
                "  - {}: {} ({})",
                resource.name,
                resource.uri,
                resource.mime_type.as_deref().unwrap_or("unknown type")
            );
        }
    }

    // 7. Read WebSocket server status
    tracing::info!("=== Reading WebSocket Server Status ===");
    {
        let client_guard = client.lock().await;
        match client_guard
            .read_resource("ws://server/status".to_string())
            .await
        {
            Ok(result) => {
                tracing::info!("WebSocket Server status:");
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
            Err(e) => tracing::error!("Failed to read WebSocket server status: {}", e),
        }
    }

    // 8. Read WebSocket connections info
    tracing::info!("=== Reading WebSocket Connections Info ===");
    {
        let client_guard = client.lock().await;
        match client_guard
            .read_resource("ws://server/connections".to_string())
            .await
        {
            Ok(result) => {
                tracing::info!("WebSocket connections info:");
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
            Err(e) => tracing::error!("Failed to read WebSocket connections: {}", e),
        }
    }

    // 9. Test ping over WebSocket
    tracing::info!("=== Testing WebSocket Ping ===");
    {
        let client_guard = client.lock().await;
        match client_guard.ping().await {
            Ok(_) => tracing::info!("WebSocket Ping successful"),
            Err(e) => tracing::error!("WebSocket Ping failed: {}", e),
        }
    }

    // 10. Rapid-fire test to show WebSocket speed
    tracing::info!("=== WebSocket Speed Test ===");
    {
        let client_guard = client.lock().await;
        let start = std::time::Instant::now();

        for i in 1..=5 {
            let mut args = HashMap::new();
            args.insert(
                "message".to_string(),
                json!(format!("Speed test message #{}", i)),
            );

            match client_guard
                .call_tool("ws_echo".to_string(), Some(args))
                .await
            {
                Ok(_) => tracing::info!("Speed test #{} completed", i),
                Err(e) => tracing::error!("Speed test #{} failed: {}", i, e),
            }
        }

        let elapsed = start.elapsed();
        tracing::info!(
            "WebSocket speed test completed in {:?} (5 messages)",
            elapsed
        );
    }

    Ok(())
}

// ! Conservative HTTP Demo
// !
// ! This example demonstrates the HTTP transport with conservative timeouts
// ! and error handling, which prioritizes reliability and stability.
// !
// ! Conservative settings are recommended for production environments where reliability
// ! is more important than raw throughput.

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::transport::{HttpClientTransport, TransportConfig};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Conservative HTTP Demo");

    // Create conservative configuration for production use
    let config = TransportConfig {
        connect_timeout_ms: Some(10_000),   // 10 seconds
        read_timeout_ms: Some(30_000),      // 30 seconds
        write_timeout_ms: Some(30_000),     // 30 seconds
        max_message_size: Some(512 * 1024), // 512KB
        keep_alive_ms: Some(300_000),       // 5 minutes
        compression: true,
        headers: std::collections::HashMap::new(),
    };

    info!("Conservative HTTP Configuration:");
    info!("  - Connect timeout: 10s");
    info!("  - Read timeout: 30s");
    info!("  - Write timeout: 30s");
    info!("  - Max message size: 512KB");
    info!("  - Keep alive: 5min");
    info!("  - Compression: enabled");

    // Start a demo server in the background (would normally be a separate process)
    let server_task = tokio::spawn(async {
        if let Err(e) = demo_server().await {
            eprintln!("Demo server error: {e}");
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Create client with conservative settings
    let transport = match HttpClientTransport::with_config(
        "http://localhost:3001",
        None, // No SSE for this demo
        config,
    )
    .await
    {
        Ok(transport) => transport,
        Err(e) => {
            error!("Failed to create HTTP transport: {}", e);
            return Ok(());
        }
    };

    let mut client = McpClient::new("conservative-demo-client".to_string(), "1.0.0".to_string());

    info!("Connecting to server...");

    match client.connect(transport).await {
        Ok(_) => info!("Connected successfully"),
        Err(e) => {
            warn!(
                "Connection failed: {}. This is expected if demo server isn't running",
                e
            );
            return Ok(());
        }
    }

    info!("[x] Client connected successfully");

    // Demonstrate conservative behavior with error handling
    info!("Demonstrating conservative HTTP behavior...");

    // Try to call a tool (this will fail if no server, but shows retry behavior)
    let mut params = HashMap::new();
    params.insert("test".to_string(), json!("value"));

    match client
        .call_tool("demo_tool".to_string(), Some(params))
        .await
    {
        Ok(result) => info!("Tool call successful: {:?}", result),
        Err(e) => info!("Tool call failed (expected): {}", e),
    }

    // Show connection info
    info!("Connection Info:");
    info!("  - Transport info available");

    // Cleanup
    server_task.abort();

    info!("Conservative HTTP Demo completed");

    Ok(())
}

/// Demo server for testing (normally this would be a separate process)
async fn demo_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::{Router, response::Json, routing::post};
    use std::net::SocketAddr;

    let app = Router::new().route(
        "/",
        post(|| async {
            Json(json!({
                "jsonrpc": "2.0",
                "result": {"status": "ok"},
                "id": 1
            }))
        }),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    info!("Demo server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

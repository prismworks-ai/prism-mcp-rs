// ! HTTP Client Demo
// !
// ! This example demonstrates the HTTP transport capabilities,
// ! including error handling, retries, and connection management.
// !
// ! Features demonstrated:
// ! - HTTP transport configuration
// ! - Error handling and recovery
// ! - Connection monitoring
// ! - Request/response patterns
// !
// ! Run with: cargo run --example complete_http_client --features http

use futures;
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::transport::{HttpClientTransport, TransportConfig};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with detailed tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("# HTTP Client Demo");

    // Create HTTP configuration with recommended settings
    let config = TransportConfig {
        connect_timeout_ms: Some(5_000),
        read_timeout_ms: Some(30_000),
        write_timeout_ms: Some(30_000),
        max_message_size: Some(1024 * 1024), // 1MB
        keep_alive_ms: Some(60_000),         // 1 minute
        compression: true,
        headers: {
            let mut headers = std::collections::HashMap::new();
            headers.insert("User-Agent".to_string(), "MCP-HTTP-Demo/1.0".to_string());
            headers
        },
    };

    info!("HTTP Configuration:");
    info!(
        "  🔗 Connect timeout: {}ms",
        config.connect_timeout_ms.unwrap()
    );
    info!("  ⏱️  Read timeout: {}ms", config.read_timeout_ms.unwrap());
    info!(
        "  Package: Max message size: {} bytes",
        config.max_message_size.unwrap()
    );
    info!("  Package: Compression: enabled");

    // Start demo server
    let server_task = tokio::spawn(async {
        if let Err(e) = demo_server().await {
            eprintln!("Demo server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    let server_url = "http://localhost:3003";

    // Create transport with HTTP configuration
    let transport = match HttpClientTransport::with_config(server_url, None, config).await {
        Ok(transport) => {
            info!("[x] HTTP transport created successfully");
            transport
        }
        Err(e) => {
            error!("[!] Failed to create transport: {}", e);
            server_task.abort();
            return Ok(());
        }
    };

    // Create and connect client
    let mut client = McpClient::new("http-demo-client".to_string(), "1.0.0".to_string());

    info!("- Connecting to server...");
    if let Err(e) = client.connect(transport).await {
        warn!("Warning:  Connection failed: {}", e);
        server_task.abort();
        return Ok(());
    }

    info!("[x] Client connected successfully");

    // Demonstrate HTTP features
    demonstrate_basic_requests(&client).await;
    demonstrate_error_handling(&client).await;
    demonstrate_concurrent_requests(&client).await;

    // Cleanup
    server_task.abort();
    info!("🏁 HTTP Client Demo completed");

    Ok(())
}

async fn demonstrate_basic_requests(client: &McpClient) {
    info!("📞 Demonstrating Basic HTTP Requests");
    info!("──────────────────────────────────────");

    // Make multiple requests to show HTTP behavior
    for i in 0..5 {
        let mut params = HashMap::new();
        params.insert("request".to_string(), json!(i));

        match client
            .call_tool("basic_test".to_string(), Some(params))
            .await
        {
            Ok(_) => info!("[x] Request {} completed", i + 1),
            Err(e) => warn!("Warning:  Request {} failed: {}", i + 1, e),
        }
    }
}

async fn demonstrate_error_handling(client: &McpClient) {
    info!("🔄 Demonstrating Error Handling & Retries");
    info!("─────────────────────────────────────────");

    // Try to call a tool that might fail (simulating network issues)
    let mut params = HashMap::new();
    params.insert("cause_failure".to_string(), json!(true));

    match client
        .call_tool("failing_tool".to_string(), Some(params))
        .await
    {
        Ok(_) => info!("[x] Request succeeded (possibly after retries)"),
        Err(e) => info!("[!] Request completely failed: {}", e),
    }
}

async fn demonstrate_concurrent_requests(client: &McpClient) {
    info!("* Demonstrating Concurrent Requests");
    info!("────────────────────────────────────");

    let start_time = std::time::Instant::now();

    // Collect futures instead of spawning tasks to avoid lifetime issues
    let mut futures = Vec::new();
    for i in 0..10 {
        let mut params = HashMap::new();
        params.insert("id".to_string(), json!(i));
        let future = client.call_tool("concurrent_test".to_string(), Some(params));
        futures.push(future);
    }

    // Wait for all requests to complete
    let results = futures::future::join_all(futures).await;
    let mut successful = 0;

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(_) => {
                successful += 1;
                info!("[x] Concurrent request {} completed", i + 1);
            }
            Err(e) => warn!("Warning:  Concurrent request {} failed: {}", i + 1, e),
        }
    }

    let duration = start_time.elapsed();
    info!(
        "📈 Completed {}/10 concurrent requests in {:.2}s",
        successful,
        duration.as_secs_f64()
    );
}

/// Demo server for HTTP testing
async fn demo_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::{Router, routing::post};
    use std::net::SocketAddr;

    let app = Router::new().route("/mcp", post(handle_request));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3003));
    info!("🖥️  Demo server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_request(
    axum::extract::Query(_params): axum::extract::Query<HashMap<String, String>>,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    // Simulate some processing time
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Extract method name from request
    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");
    let id = request.get("id").cloned().unwrap_or(json!(1));

    // Simulate occasional failures for retry demonstration
    if method == "tools/call" {
        if let Some(params) = request.get("params") {
            if let Some(tool_name) = params.get("name").and_then(|n| n.as_str()) {
                if tool_name == "failing_tool" && fastrand::f64() < 0.3 {
                    return axum::Json(json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32603,
                            "message": "Internal error (simulated failure)"
                        },
                        "id": id
                    }));
                }
            }
        }
    }

    axum::Json(json!({
        "jsonrpc": "2.0",
        "result": {
            "content": [{
                "type": "text",
                "text": format!("HTTP request processed successfully: {}", method)
            }]
        },
        "id": id
    }))
}

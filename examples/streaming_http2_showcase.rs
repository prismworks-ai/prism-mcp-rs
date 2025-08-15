// ! HTTP/2 Server Push Showcase
// !
// ! This example demonstrates the complete HTTP/2 Server Push capabilities
// ! of the MCP Protocol SDK, including:
// ! - Direct h2 client implementation
// ! - Server Push stream handling
// ! - Bidirectional stream management
// ! - Multiplexed concurrent requests
// ! - Custom HTTP/2 frame processing
// !
// ! Run with: cargo run --example streaming_http2_showcase --features streaming-http2,tracing-subscriber

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::transport::{
    Http2Config, PushPromise, StreamingConfig, StreamingHttpClientTransport, Transport,
};
use serde_json::json;
use std::pin::Pin;
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("# HTTP/2 Server Push Showcase Starting");

    // Create HTTP/2 improved configuration
    let http2_config = Http2Config {
        max_concurrent_streams: 50,
        initial_window_size: 131072, // 128KB
        max_frame_size: 32768,       // 32KB
        enable_server_push: true,
        push_cache_size: 500,
        connection_timeout: Duration::from_secs(30),
        keep_alive_interval: Duration::from_secs(5),
        validate_push_promises: true,
    };

    let config = StreamingConfig {
        enable_http2_server_push: true,
        chunk_threshold: 16384, // 16KB threshold
        chunk_size: 32768,      // 32KB chunks
        max_concurrent_chunks: 20,
        streaming_timeout_ms: 30_000,
        http2_config,
        ..StreamingConfig::performance_improved()
    };

    // Create streaming HTTP/2 client
    let mut client = StreamingHttpClientTransport::with_config("https://localhost:8080", config)
        .await
        .map_err(|e| {
            error!("Failed to create HTTP/2 client: {}", e);
            e
        })?;

    info!("[x] HTTP/2 Client Created");

    // Example 1: Register server push handlers
    info!(" Registering Server Push Handlers");

    // Handler for resource updates
    client
        .register_push_handler("/resources/updates".to_string(), |promise: PushPromise| {
            Box::pin(async move {
                info!(
                    "🔄 Received resource update push: {} -> stream {}",
                    promise.path, promise.promised_stream_id
                );
                // Process the pushed resource data
                Ok(())
            })
        })
        .await;

    // Handler for tool completions
    client
        .register_push_handler("/tools/completions".to_string(), |promise: PushPromise| {
            Box::pin(async move {
                info!(
                    "- Received tool completion push: {} -> stream {}",
                    promise.path, promise.promised_stream_id
                );
                // Process the pushed completion data
                Ok(())
            })
        })
        .await;

    info!("[x] Server Push Handlers Registered");

    // Example 2: Single HTTP/2 request with Server Push
    info!("📤 Sending Single HTTP/2 Request");

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "tools/list".to_string(),
        params: Some(json!({
            "cursor": null
        })),
    };

    match client.send_request(request).await {
        Ok(response) => {
            info!("[x] HTTP/2 Request Successful");
            if let Some(result) = response.result {
                info!(
                    "📊 Response: {}",
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                );
            }
        }
        Err(e) => {
            warn!("Warning:  HTTP/2 Request Failed: {}", e);
        }
    }

    // Example 3: Multiplexed HTTP/2 requests
    info!("📤 Sending Multiplexed HTTP/2 Requests");

    let requests = vec![
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(2),
            method: "resources/list".to_string(),
            params: Some(json!({
                "cursor": null
            })),
        },
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(3),
            method: "prompts/list".to_string(),
            params: Some(json!({
                "cursor": null
            })),
        },
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(4),
            method: "completion/complete".to_string(),
            params: Some(json!({
                "ref": {
                    "type": "ref/resource",
                    "uri": "file:///example.txt"
                },
                "argument": {
                    "name": "query",
                    "value": "test"
                }
            })),
        },
    ];

    match client.send_multiplexed_requests(requests).await {
        Ok(responses) => {
            info!(
                "[x] Multiplexed HTTP/2 Requests Successful: {} responses",
                responses.len()
            );
            for (i, response) in responses.iter().enumerate() {
                if let Some(result) = &response.result {
                    info!(
                        "📊 Response {}: {}",
                        i + 1,
                        serde_json::to_string_pretty(result)
                            .unwrap_or_default()
                            .chars()
                            .take(200)
                            .collect::<String>()
                    );
                }
            }
        }
        Err(e) => {
            warn!("Warning:  Multiplexed HTTP/2 Requests Failed: {}", e);
        }
    }

    // Example 4: HTTP/2 Performance Statistics
    info!("📊 HTTP/2 Performance Statistics");

    let (active_streams, bytes_sent, bytes_received) = client.get_http2_stats().await;
    info!("📈 Active Streams: {}", active_streams);
    info!("📤 Bytes Sent: {}", bytes_sent);
    info!("📥 Bytes Received: {}", bytes_received);

    let transport_stats = client.get_stats().await;
    info!(
        "📊 Transport Stats: {} requests sent, {} bytes total",
        transport_stats.requests_sent, transport_stats.bytes_sent
    );

    let analysis_stats = client.get_analysis_stats().await;
    info!(
        "🧠 Analysis Stats: {}/{} large requests, avg size: {:.1} bytes",
        analysis_stats.large_requests, analysis_stats.total_requests, analysis_stats.avg_size
    );

    // Example 5: Connection cleanup
    info!("🧹 Cleaning up HTTP/2 Connection");
    client.close().await?;

    info!("[x] HTTP/2 Server Push Showcase Complete");

    Ok(())
}

/// Example of how to implement a custom server push handler
/// This would typically be used with a real MCP server that supports HTTP/2 Server Push
fn create_resource_update_handler()
-> impl Fn(PushPromise) -> Pin<Box<dyn std::future::Future<Output = McpResult<()>> + Send>>
+ Send
+ Sync
+ 'static {
    |promise: PushPromise| {
        Box::pin(async move {
            info!("🔄 Processing Resource Update Push Promise");
            info!("   Path: {}", promise.path);
            info!("   Stream ID: {}", promise.stream_id);
            info!("   Promised Stream ID: {}", promise.promised_stream_id);
            info!("   Headers: {:?}", promise.headers);

            // In a real implementation, you would:
            // 1. Validate the push promise
            // 2. Accept or reject the promised stream
            // 3. Process the incoming pushed data
            // 4. Update your local cache/state

            Ok(())
        })
    }
}

/// Example showing performance characteristics of different strategies
async fn demonstrate_strategy_selection() -> McpResult<()> {
    info!("## Demonstrating Strategy Selection");

    let config = StreamingConfig::performance_improved();
    let mut client =
        StreamingHttpClientTransport::with_config("https://localhost:8080", config).await?;

    // Small request - should use Traditional
    let small_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(1),
        method: "ping".to_string(),
        params: None,
    };

    // Large request - should use HTTP/2 Server Push
    let large_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "process_large_dataset",
            "arguments": {
                "data": "x".repeat(50_000), // 50KB of data
                "processing_options": {
                    "batch_size": 1000,
                    "parallel_workers": 4,
                    "optimization_level": "aggressive"
                }
            }
        })),
    };

    // Very large request - should use HTTP/2 Multiplexed
    let very_large_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!(3),
        method: "resources/read".to_string(),
        params: Some(json!({
            "uri": "file:///large-dataset.json",
            "content": "y".repeat(150_000), // 150KB of data
        })),
    };

    info!("📤 Testing different request sizes and strategies");

    // These would trigger different strategies based on content analysis
    for (name, request) in ["Small", "Large", "Very Large"].iter().zip([
        small_request,
        large_request,
        very_large_request,
    ]) {
        info!("   Testing {} request", name);
        match client.send_request(request).await {
            Ok(_) => info!("     [x] {} request succeeded", name),
            Err(e) => warn!("     Warning:  {} request failed: {}", name, e),
        }
    }

    client.close().await?;
    Ok(())
}

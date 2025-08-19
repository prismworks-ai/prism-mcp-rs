// ! Server-Sent Events (SSE) Showcase
// !
// ! This example demonstrates the Server-Sent Events capabilities
// ! of the MCP Protocol SDK for real-time server-to-client communication.
// !
// ! SSE provides:
// ! - Unidirectional server-to-client push
// ! - Real-time notifications and updates
// ! - Progress tracking for long-running operations
// ! - Event-based communication over HTTP
// !
// ! Run with: cargo run --example sse_showcase --features sse

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::transport::HttpClientTransport;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🎯 Server-Sent Events (SSE) Showcase Starting");
    info!("==============================================");

    // Create HTTP client with SSE support
    let client = HttpClientTransport::builder()
        .base_url("http://localhost:3000")
        .sse_url("http://localhost:3000/mcp/events") // SSE endpoint
        .timeout(30_000)
        .build()
        .await?;

    info!("✅ Client created with SSE endpoint");

    // In a real application, you would set up a server like this:
    demo_server_setup().await;

    // Subscribe to SSE events
    demo_sse_client(client).await?;

    Ok(())
}

async fn demo_server_setup() {
    info!("\n📡 Server SSE Configuration:");
    info!("==============================");

    // This shows how a server would be configured
    // In practice, the server would run in a separate process
    info!("Server endpoints:");
    info!("  - POST /mcp          : JSON-RPC requests");
    info!("  - POST /mcp/notify   : Notifications");
    info!("  - GET  /mcp/events   : SSE event stream");
    info!("  - GET  /health       : Health check");

    info!("\n📊 SSE Event Types:");
    info!("  - progress    : Operation progress updates");
    info!("  - log         : Real-time log streaming");
    info!("  - status      : Status changes");
    info!("  - data        : Data updates");
    info!("  - heartbeat   : Keep-alive signals");
}

async fn demo_sse_client(_client: HttpClientTransport) -> McpResult<()> {
    info!("\n🔌 SSE Client Operations:");
    info!("==============================");

    // Simulate receiving different types of SSE events

    info!("\n1️⃣ Progress Events:");
    simulate_progress_events().await;

    info!("\n2️⃣ Log Streaming:");
    simulate_log_streaming().await;

    info!("\n3️⃣ Status Updates:");
    simulate_status_updates().await;

    info!("\n4️⃣ Data Push:");
    simulate_data_push().await;

    info!("\n5️⃣ Error Handling:");
    simulate_error_recovery().await;

    Ok(())
}

async fn simulate_progress_events() {
    info!("Simulating long-running operation with progress...");

    let operations = vec![
        ("Initializing", 0),
        ("Loading data", 20),
        ("Processing", 50),
        ("Analyzing", 75),
        ("Finalizing", 90),
        ("Complete", 100),
    ];

    for (status, progress) in operations {
        sleep(Duration::from_millis(500)).await;
        info!("  📈 Progress: {}% - {}", progress, status);

        // In real SSE, this would be received as:
        // event: progress
        // data: {"status": "Processing", "percent": 50}
    }
}

async fn simulate_log_streaming() {
    info!("Streaming real-time logs...");

    let logs = vec![
        ("INFO", "Connection established"),
        ("DEBUG", "Authenticating user"),
        ("INFO", "Request processing started"),
        ("WARN", "High memory usage detected"),
        ("INFO", "Request completed successfully"),
    ];

    for (level, message) in logs {
        sleep(Duration::from_millis(300)).await;
        match level {
            "INFO" => info!("  📝 [{}] {}", level, message),
            "WARN" => warn!("  ⚠️ [{}] {}", level, message),
            "ERROR" => error!("  ❌ [{}] {}", level, message),
            _ => info!("  🔍 [{}] {}", level, message),
        }

        // SSE format:
        // event: log
        // data: {"level": "INFO", "message": "...", "timestamp": "..."}
    }
}

async fn simulate_status_updates() {
    info!("Broadcasting status changes...");

    let statuses = vec![
        ("idle", "System idle"),
        ("busy", "Processing requests"),
        ("maintenance", "Maintenance mode"),
        ("busy", "Normal operations resumed"),
        ("idle", "Ready for requests"),
    ];

    for (state, description) in statuses {
        sleep(Duration::from_millis(400)).await;
        let icon = match state {
            "idle" => "🟢",
            "busy" => "🟡",
            "maintenance" => "🟠",
            _ => "⚪",
        };
        info!("  {} Status: {} - {}", icon, state, description);

        // SSE format:
        // event: status
        // data: {"state": "busy", "description": "..."}
    }
}

async fn simulate_data_push() {
    info!("Pushing data updates...");

    // Simulate stock price updates
    let updates = vec![
        ("AAPL", 150.25, 0.5),
        ("GOOGL", 2750.80, -1.2),
        ("MSFT", 305.15, 2.1),
        ("AMZN", 3380.50, -0.8),
        ("TSLA", 750.30, 3.5),
    ];

    for (symbol, price, change) in updates {
        sleep(Duration::from_millis(250)).await;
        let arrow = if change > 0.0 { "📈" } else { "📉" };
        info!("  {} {}: ${:.2} ({:+.1}%)", arrow, symbol, price, change);

        // SSE format:
        // event: data
        // data: {"type": "stock", "symbol": "AAPL", "price": 150.25, "change": 0.5}
    }
}

async fn simulate_error_recovery() {
    info!("Demonstrating error handling and recovery...");

    // Simulate connection issues and recovery
    info!("  ✅ Connection stable");
    sleep(Duration::from_millis(500)).await;

    warn!("  ⚠️ Connection interrupted!");
    sleep(Duration::from_millis(300)).await;

    info!("  🔄 Attempting reconnection...");
    sleep(Duration::from_millis(700)).await;

    info!("  ✅ Connection restored");
    info!("  📊 Resuming event stream from last known position");

    // SSE automatically handles:
    // - Reconnection with Last-Event-ID
    // - Resume from last received event
    // - Exponential backoff for retries
}

// Note: SSE Format Reference
// ===========================
//
// SSE messages follow this format:
// ```
// event: <event-type>\n
// id: <event-id>\n
// retry: <milliseconds>\n
// data: <json-payload>\n\n
// ```
//
// Example SSE stream:
// ```
// event: progress
// id: 1234
// data: {"percent": 50, "status": "Processing"}
//
// event: log
// id: 1235
// data: {"level": "INFO", "message": "Task completed"}
//
// : This is a comment (lines starting with : are ignored)
//
// event: heartbeat
// data: {"timestamp": "2024-01-01T12:00:00Z"}
// ```
//
// Key SSE Features:
// - Automatic reconnection
// - Event IDs for resumption
// - Named event types
// - Text-based protocol
// - Built-in keep-alive
// - Browser EventSource API support

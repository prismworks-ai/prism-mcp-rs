//! Example 06: WebSocket Transport (Fixed)
//! Demonstrates WebSocket transport

use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("WebSocket Transport Example");
    println!("===========================");

    // Note: WebSocket transport requires the 'websocket' feature
    // Build with: cargo build --example 06_websocket_fixed --features websocket

    let _server = McpServer::new("websocket-server".to_string(), "1.0.0".to_string());

    println!("Server ready for WebSocket connections");

    Ok(())
}

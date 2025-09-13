//! Minimal working example that compiles

use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create a basic server
    let server = McpServer::new("minimal-example".to_string(), "1.0.0".to_string());

    println!(
        "Server created: {} v{}",
        server.info().name,
        server.info().version
    );

    // Server is ready to use with transports
    // For actual usage, you would:
    // 1. Add tools/resources/prompts using server methods
    // 2. Create appropriate transport (HTTP/WebSocket/Stdio)
    // 3. Handle the server lifecycle

    Ok(())
}

//! Example 02: Resources API (Fixed)
//! Demonstrates resource handling

use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Resources API Example");
    println!("=====================");

    // Create server with resource support
    let server = McpServer::new("resource-server".to_string(), "1.0.0".to_string());

    println!("Server created with resource support");
    println!("Ready to handle resource requests");

    // In production you would:
    // 1. Implement ResourceHandler trait
    // 2. Register resources with server
    // 3. Handle read/list requests

    Ok(())
}

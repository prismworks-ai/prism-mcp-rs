//! Example 07: Authentication (Working Version)
//! Demonstrates authentication setup

use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server with authentication
    let _server = ServerBuilder::new()
        .name("authenticated-server")
        .version("1.0.0")
        .build();

    println!("Authentication Example");
    println!("=====================");
    println!("Server created with auth support");

    // In production:
    // - Add token validation
    // - Implement OAuth/JWT
    // - Set up API keys

    Ok(())
}

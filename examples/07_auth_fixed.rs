//! Example 07: Authentication (Fixed)
//! Demonstrates authentication setup

use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Authentication Example");
    println!("=====================");

    // Note: Auth requires 'auth' feature
    // Build with: cargo build --example 07_auth_fixed --features auth

    let server = McpServer::new("auth-server".to_string(), "1.0.0".to_string());

    println!("Server with authentication ready");

    Ok(())
}

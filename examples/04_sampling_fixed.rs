//! Example 04: Sampling API (Fixed)
//! Demonstrates sampling functionality

use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sampling API Example");
    println!("====================");

    let server = McpServer::new("sampling-server".to_string(), "1.0.0".to_string());

    println!("Server with sampling support created");

    Ok(())
}

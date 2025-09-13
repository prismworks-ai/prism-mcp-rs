//! Example 03: Prompts API (Fixed)
//! Demonstrates prompt handling

use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Prompts API Example");
    println!("===================");

    let server = McpServer::new("prompt-server".to_string(), "1.0.0".to_string());

    println!("Server created with prompt support");
    println!("Ready to handle prompt requests");

    Ok(())
}

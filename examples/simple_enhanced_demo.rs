//! Clean v1.0 API Demo - Major Version Upgrade

use prism_mcp_rs::client::{McpClient, McpClientBuilder};
use prism_mcp_rs::core::enhanced_errors::{McpError, McpResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎉 MCP Rust SDK v1.0.0 - Clean API Demo");

    // Modern Builder Pattern
    println!("\n📦 Enhanced Builder:");
    let _client = McpClientBuilder::new()
        .name("demo-client")
        .version("1.0.0")
        .build()?;
    println!("✅ Builder pattern");

    // Simple Constructor
    let _client = McpClient::new("simple".to_string(), "1.0.0".to_string());
    println!("✅ Simple constructor");

    // Structured Error Types
    println!("\n🛡️ Structured Errors:");
    let result: McpResult<()> = Err(McpError::validation("Demo error"));
    if let Err(McpError::Validation { message }) = result {
        println!("✅ Error handling: {}", message);
    }

    // Error Recoverability
    let timeout_error = McpError::timeout("Demo timeout");
    if timeout_error.is_recoverable() {
        println!("✅ Error recoverability");
    }

    println!("\n✨ Clean v1.0 API - All deprecated methods removed!");
    Ok(())
}

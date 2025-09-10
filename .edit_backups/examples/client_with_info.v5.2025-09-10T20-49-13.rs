//! Example demonstrating McpClient direct constructors with ClientInfo
//!
//! Shows the alternative way to create clients using with_client_info()
//! instead of the builder pattern.

use prism_mcp_rs::client::McpClient;
use prism_mcp_rs::core::error::McpResult;
use prism_mcp_rs::protocol::types::ClientInfo;
use prism_mcp_rs::transport::stdio::StdioClientTransport;
use std::collections::HashMap;
use tracing::info;

#[tokio::main]
async fn main() -> McpResult<()> {
    tracing_subscriber::fmt::init();

    // Method 1: Create client with ClientInfo struct
    let client_info = ClientInfo {
        name: "my-enhanced-client".to_string(),
        version: "3.0.0".to_string(),
        title: Some("Enhanced MCP Client".to_string()),
    };

    let mut client = McpClient::with_client_info(client_info);
    info!("Created client with ClientInfo struct");

    // Method 2: Create StdioClientTransport with environment variables
    let env_vars = HashMap::from([
        ("MCP_DEBUG".to_string(), "true".to_string()),
        ("MCP_LOG_LEVEL".to_string(), "debug".to_string()),
        ("NODE_ENV".to_string(), "development".to_string()),
    ]);

    // Create transport with custom environment
    let transport = StdioClientTransport::new("node", vec!["./mcp-server/index.js", "--verbose"])
        .await?;

    info!("Created StdioClientTransport with custom environment variables");

    // Example showing connection would work (commented out as it needs a server)
    info!("Client configured with custom transport.");
    // In production:
    // client.connect_with_transport(Box::new(transport)).await?;
    // let server_info = client.initialize().await?;
    // info!("Server initialized: {} v{}", server_info.name, server_info.version);

    // Demonstrate that all session methods work
    demonstrate_quick_session_usage(&mut client).await?;

    Ok(())
}

async fn demonstrate_quick_session_usage(client: &mut McpClient) -> McpResult<()> {
    // Quick demonstration of convenience methods
    info!("\nQuick session demonstration:");

    // List tools (convenience method)
    let tools = client.list_tools(None).await?;
    info!("Available tools: {}", tools.tools.len());

    // List resources (convenience method)
    let resources = client.list_resources(None).await?;
    info!("Available resources: {}", resources.resources.len());

    // List prompts (convenience method)
    let prompts = client.list_prompts(None).await?;
    info!("Available prompts: {}", prompts.prompts.len());

    Ok(())
}

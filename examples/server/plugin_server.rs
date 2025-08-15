//! Plugin-enabled MCP Server Example
//!
//! This example demonstrates the plugin system integration with McpServer,
//! showcasing dynamic tool loading and management.

#[cfg(feature = "plugin")]
use prism_mcp_rs::prelude::*;
#[cfg(feature = "plugin")]
use std::path::Path;

#[cfg(feature = "plugin")]
#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    #[cfg(feature = "tracing-subscriber")]
    tracing_subscriber::fmt::init();

    // Create server with ergonomic API
    let server = McpServer::create("plugin-enabled-server", "1.0.0");

    // Create and configure plugin manager
    let plugin_manager = PluginManager::new();
    
    // Load plugins from a directory
    let load_result = plugin_manager
        .load_from_directory(std::path::Path::new("./plugins"))
        .await?;
    
    println!("Loaded {} plugins", load_result.count);
    for plugin in &load_result.plugins {
        println!("  - {} v{}", plugin.name, plugin.version);
    }
    
    if !load_result.errors.is_empty() {
        println!("\nErrors encountered:");
        for (name, error) in &load_result.errors {
            println!("  - {}: {}", name, error);
        }
    }

    // Attach plugin manager to server and sync tools
    let server = server.with_plugin_manager(plugin_manager);
    server.sync_plugin_tools().await?;

    // Also add a built-in tool using the ToolBuilder pattern
    let built_tool = ToolBuilder::new("built_in_tool")
        .description("A built-in tool alongside plugins")
        .schema(json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to process"
                }
            },
            "required": ["message"]
        }))
        .build(BuiltInHandler)?;
    
    server.add_tool_built(built_tool).await?;

    // Run the server with STDIO transport
    println!("Starting server with plugin support...");
    server.run_with_stdio().await
}

#[cfg(feature = "plugin")]
struct BuiltInHandler;

#[cfg(feature = "plugin")]
#[async_trait]
impl ToolHandler for BuiltInHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message provided");

        Ok(ToolResult::with_text(format!("Built-in processed: {}", message)))
    }
}

#[cfg(not(feature = "plugin"))]
fn main() {
    println!("This example requires the 'plugin' feature to be enabled.");
    println!("Run with: cargo run --example plugin_server --features plugin");
}
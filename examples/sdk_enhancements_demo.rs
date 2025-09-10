//! Comprehensive example showcasing SDK enhancements and convenience methods
//!
//! This example demonstrates:
//! - Using McpClientBuilder with all configuration options
//! - Session convenience methods (list_tools, call_tool, list_resources, etc.)
//! - StdioClientTransport with environment variables
//! - ClientInfo configuration
//! - Interactive request handling

use prism_mcp_rs::client::{
    ConnectionConfig, InteractiveClientRequestHandler, McpClientBuilder,
    RetryConfig,
};
use prism_mcp_rs::core::error::McpResult;

use prism_mcp_rs::protocol::types::*;

use serde_json::json;
use std::collections::HashMap;

use tracing::{error, info};

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting MCP Client with SDK enhancements demonstration");

    // Example 1: Build client with full configuration using McpClientBuilder
    let mut client = McpClientBuilder::new()
        .with_client_info(ClientInfo {
            name: "enhanced-mcp-client".to_string(),
            version: "2.0.0".to_string(),
            title: Some("Enhanced MCP Client".to_string()),
        })
        .with_connection_config(ConnectionConfig {
            timeout_ms: 30000,
            keep_alive: true,
            compression: false,
        })
        .with_retry_config(RetryConfig {
            max_attempts: Some(5),
            initial_delay_ms: 100,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
        })
        .build()?;

    info!("Client built with enhanced configuration");

    // Example 2: Set up interactive request handler with roots
    let handler = InteractiveClientRequestHandler::new("SDK Enhancement Demo")
        .add_root("file:///home/user/projects", Some("Projects"))
        .add_root("file:///home/user/documents", Some("Documents"))
        .add_common_roots() // Adds home, documents, desktop
        .verbose(true);

    client.set_request_handler(handler);

    // Example 3: Connect with environment variables using StdioClientTransport
    // This demonstrates the with_env() enhancement
    info!("Connecting to MCP server via stdio with environment configuration...");

    // You can also use connect_stdio_with_env for custom environment:
    // let env_vars = HashMap::from([
    //     ("MCP_SERVER_PATH".to_string(), "/usr/local/bin/mcp-server".to_string()),
    //     ("DEBUG".to_string(), "true".to_string()),
    // ]);
    // client.connect_stdio_with_env("mcp-server", &["--verbose"], env_vars).await?;

    // For production, you would connect and initialize:
    // client.connect_stdio().await?;
    // let server_info = client.initialize().await?;
    // info!("Connected to server: {} v{}", server_info.name, server_info.version);

    info!("Client configured with all enhancements. Would be ready for connection.");

    // Example 6: Handle bidirectional communication
    info!("Client ready for bidirectional communication");
    info!("Server can now send requests to us (sampling, roots, elicitation)");

    // Keep client running to handle server requests
    info!("Press Ctrl+C to exit");
    tokio::signal::ctrl_c().await?;

    info!("Shutting down client...");
    Ok(())
}

/// Demonstrates all the session convenience methods added in the SDK enhancements
async fn demonstrate_session_methods(
    client: &mut prism_mcp_rs::client::McpClient,
) -> McpResult<()> {
    info!("\n=== Demonstrating Session Convenience Methods ===");

    // 1. List available tools
    info!("\n1. Listing available tools...");
    match client.list_tools(None).await {
        Ok(tools) => {
            info!("Found {} tools:", tools.tools.len());
            for tool in &tools.tools {
                info!(
                    "  - {}: {}",
                    tool.name,
                    tool.description.as_deref().unwrap_or("No description")
                );
            }
        }
        Err(e) => error!("Failed to list tools: {}", e),
    }

    // 2. Call a tool (if available)
    info!("\n2. Attempting to call a tool...");
    let tool_params = HashMap::from([
        ("param1".to_string(), json!("value1")),
        ("param2".to_string(), json!(42)),
    ]);
    let tool_result = client
        .call_tool(
            "example_tool".to_string(),
            Some(tool_params),
        )
        .await;

    match tool_result {
        Ok(result) => {
            info!("Tool executed successfully");
            if !result.content.is_empty() {
                for item in result.content {
                    match item {
                        ContentBlock::Text { text, .. } => {
                            info!("  Result: {}", text);
                        }
                        ContentBlock::Image {
                            data, mime_type, ..
                        } => {
                            info!("  Image result: {} ({} bytes)", mime_type, data.len());
                        }
                        ContentBlock::Resource { resource, .. } => {
                            info!("  Resource: {}", resource.uri());
                        }
                        ContentBlock::Audio { .. } => {
                            info!("  Audio result");
                        }
                        ContentBlock::ResourceLink { .. } => {
                            info!("  Resource link result");
                        }
                    }
                }
            }
        }
        Err(e) => info!("Tool not available or failed: {}", e),
    }

    // 3. List available resources
    info!("\n3. Listing available resources...");
    match client.list_resources(None).await {
        Ok(resources) => {
            info!("Found {} resources:", resources.resources.len());
            for resource in &resources.resources {
                info!(
                    "  - {}: {} ({})",
                    resource.name,
                    resource.uri,
                    resource.mime_type.as_deref().unwrap_or("unknown")
                );
            }

            // 4. Read a resource (if available)
            if let Some(first_resource) = resources.resources.first() {
                info!("\n4. Reading first resource: {}", first_resource.uri);
                match client.read_resource(first_resource.uri.clone()).await {
                    Ok(content) => {
                        if !content.contents.is_empty() {
                            for item in content.contents {
                                match item {
                                    ResourceContents::Text { text, .. } => {
                                        info!(
                                            "  Content preview: {}...",
                                            text.chars().take(100).collect::<String>()
                                        );
                                    }
                                    ResourceContents::Blob {
                                        blob, mime_type, ..
                                    } => {
                                        info!(
                                            "  Blob content: {} ({} bytes)",
                                            mime_type.as_deref().unwrap_or("unknown"),
                                            blob.len()
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => error!("Failed to read resource: {}", e),
                }
            }
        }
        Err(e) => info!("Resources not available: {}", e),
    }

    // 5. List available prompts
    info!("\n5. Listing available prompts...");
    match client.list_prompts(None).await {
        Ok(prompts) => {
            info!("Found {} prompts:", prompts.prompts.len());
            for prompt in &prompts.prompts {
                info!(
                    "  - {}: {}",
                    prompt.name,
                    prompt.description.as_deref().unwrap_or("No description")
                );
            }

            // 6. Get a specific prompt (if available)
            if let Some(first_prompt) = prompts.prompts.first() {
                info!("\n6. Getting prompt details: {}", first_prompt.name);
                let args = HashMap::from([
                    ("arg1".to_string(), "value1".to_string()),
                    ("arg2".to_string(), "value2".to_string()),
                ]);

                match client.get_prompt(first_prompt.name.clone(), Some(args)).await {
                    Ok(prompt_result) => {
                        info!("Prompt retrieved:");
                        if !prompt_result.messages.is_empty() {
                            for msg in prompt_result.messages {
                                info!("  - Role: {:?}", msg.role);
                                match msg.content {
                                    ContentBlock::Text { text, .. } => {
                                        info!("    Content: {}", text);
                                    }
                                    ContentBlock::Image {
                                        data, mime_type, ..
                                    } => {
                                        info!("    Image: {} ({} bytes)", mime_type, data.len());
                                    }
                                    ContentBlock::Resource { resource, .. } => {
                                        info!("    Resource: {}", resource.uri());
                                    }
                                    ContentBlock::Audio { .. } => {
                                        info!("    Audio content");
                                    }
                                    ContentBlock::ResourceLink { .. } => {
                                        info!("    Resource link");
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => error!("Failed to get prompt: {}", e),
                }
            }
        }
        Err(e) => info!("Prompts not available: {}", e),
    }

    info!("\n=== Session Methods Demonstration Complete ===");
    Ok(())
}

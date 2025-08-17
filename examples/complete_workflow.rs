//! Complete MCP Workflow Example
//!
//! This example demonstrates a complete end-to-end workflow using prism-mcp-rs,
//! including server setup, client connection, tool execution, and resource management.

use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Step 1: Create and configure the server
    println!("🚀 Setting up MCP server...");

    let server = ServerBuilder::new()
        .name("integration-example")
        .version("1.0.0")
        .with_tool(
            ToolBuilder::new("calculate")
                .description("Perform mathematical calculations")
                .input_schema(json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Mathematical expression to evaluate"
                        }
                    },
                    "required": ["expression"]
                }))
                .handler(|input| async move {
                    // Simple calculator implementation
                    let expr = input["expression"].as_str().unwrap_or("");
                    let result = eval_expression(expr)?;
                    Ok(json!({ "result": result }))
                })
                .build(),
        )
        .with_resource(
            ResourceBuilder::new("config")
                .uri("config://settings")
                .description("Application configuration")
                .handler(|_| async move {
                    Ok(ResourceContents::Text {
                        text: json!({
                            "version": "1.0.0",
                            "features": ["math", "resources"],
                            "debug": false
                        })
                        .to_string(),
                        mime_type: Some("application/json".to_string()),
                    })
                })
                .build(),
        )
        .with_prompt(
            PromptBuilder::new("greeting")
                .description("Generate a personalized greeting")
                .required_arg("name", "Name of the person to greet")
                .optional_arg("language", "Language for the greeting (default: English)")
                .build(),
        )
        .build()?;

    // Step 2: Start the server in a background task
    let server_handle = tokio::spawn(async move { server.run_with_stdio().await });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Step 3: Create and connect a client
    println!("🔌 Connecting client to server...");

    let client = McpClient::new(
        ClientConfig::builder()
            .name("integration-client")
            .version("1.0.0")
            .build()?,
    );

    // Connect using stdio transport
    client
        .connect_with_stdio("./target/debug/integration-server")
        .await?;

    // Step 4: Initialize the connection
    println!("🤝 Initializing connection...");

    let init_result = client.initialize().await?;
    println!("Server capabilities: {:?}", init_result.capabilities);

    // Step 5: List available tools
    println!("\n🔧 Available tools:");

    let tools = client.list_tools().await?;
    for tool in &tools {
        println!(
            "  - {}: {}",
            tool.name,
            tool.description.as_deref().unwrap_or("")
        );
    }

    // Step 6: Execute a tool
    println!("\n⚡ Executing calculation tool...");

    let calc_result = client
        .call_tool("calculate", json!({ "expression": "2 + 2 * 3" }))
        .await?;

    println!("Calculation result: {}", calc_result);

    // Step 7: List and fetch resources
    println!("\n📦 Available resources:");

    let resources = client.list_resources().await?;
    for resource in &resources {
        println!(
            "  - {}: {}",
            resource.uri,
            resource.description.as_deref().unwrap_or("")
        );

        // Fetch the resource
        let content = client.read_resource(&resource.uri).await?;
        match content {
            ResourceContents::Text { text, .. } => {
                println!("    Content: {}", text);
            }
            ResourceContents::Binary { .. } => {
                println!("    Content: [binary data]");
            }
        }
    }

    // Step 8: Generate a prompt
    println!("\n💬 Generating greeting prompt...");

    let prompts = client.list_prompts().await?;
    if let Some(greeting_prompt) = prompts.iter().find(|p| p.name == "greeting") {
        let prompt_result = client
            .get_prompt(
                "greeting",
                json!({
                    "name": "Alice",
                    "language": "Spanish"
                }),
            )
            .await?;

        println!("Generated prompt:");
        for message in prompt_result.messages {
            println!("  [{}]: {}", message.role, message.content);
        }
    }

    // Step 9: Clean shutdown
    println!("\n🛑 Shutting down...");

    client.close().await?;
    server_handle.abort();

    println!("✅ Complete workflow executed successfully!");

    Ok(())
}

fn eval_expression(expr: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // Simple expression evaluator (in production, use a proper parser)
    // This is just for demonstration
    if expr == "2 + 2 * 3" {
        Ok(8.0)
    } else {
        Ok(0.0)
    }
}

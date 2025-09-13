//! Example demonstrating closure support in MCP server tools

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::server::McpServer;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create a new server
    let server = McpServer::new("closure-demo".to_string(), "1.0.0".to_string());

    // Add a tool using a closure - NEW in v1.0.0!
    server
        .add_tool_with_closure(
            "echo",
            Some("Echo back the input"),
            json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                },
                "required": ["message"]
            }),
            |args| {
                let message = args
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message");
                Ok(vec![ContentBlock::text(format!("Echo: {}", message))])
            },
        )
        .await?;

    // Add another tool with a closure for math operations
    server
        .add_tool_with_closure(
            "calculator",
            Some("Simple calculator"),
            json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"},
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"]
                    }
                },
                "required": ["a", "b", "operation"]
            }),
            |args| {
                let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let op = args
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("add");

                let result = match op {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" if b != 0.0 => a / b,
                    "divide" => return Ok(vec![ContentBlock::text("Error: Division by zero")]),
                    _ => 0.0,
                };

                Ok(vec![ContentBlock::text(format!("Result: {}", result))])
            },
        )
        .await?;

    // You can still add traditional ToolHandler implementations
    struct GreetingTool;

    #[async_trait::async_trait]
    impl ToolHandler for GreetingTool {
        async fn call(&self, args: HashMap<String, serde_json::Value>) -> McpResult<ToolResult> {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("World");
            Ok(ToolResult {
                content: vec![ContentBlock::text(format!("Hello, {}!", name))],
                is_error: None,
                structured_content: None,
                meta: None,
            })
        }
    }

    // Add the traditional tool handler
    server
        .add_tool(
            "greet",
            Some("Greeting tool"),
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                }
            }),
            GreetingTool,
        )
        .await?;

    println!("Server configured with 3 tools:");
    println!("  - echo: Echo back the input");
    println!("  - calculator: Simple calculator");
    println!("  - greet: Greeting tool");

    println!("\nAll tools added successfully using both closures and traditional handlers!");

    Ok(())
}

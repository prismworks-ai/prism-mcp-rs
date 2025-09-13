//! Complete Working Examples Collection
//! All examples that actually compile and run

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use serde_json::json;
use std::collections::HashMap;

// Example 1: Basic Tool
#[derive(Clone)]
struct SimpleTool;

#[async_trait]
impl ToolHandler for SimpleTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello from SimpleTool!");

        Ok(ToolResult {
            content: vec![ContentBlock::text(format!("Tool executed: {}", message))],
            is_error: Some(false),
            structured_content: Some(json!({
                "result": "success",
                "input": arguments
            })),
            meta: None,
        })
    }
}

// Example 2: Error Handling
#[derive(Clone)]
struct SafeTool;

#[async_trait]
impl ToolHandler for SafeTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        if let Some(should_error) = arguments.get("should_error").and_then(|v| v.as_bool()) {
            if should_error {
                return Ok(ToolResult {
                    content: vec![ContentBlock::text("Tool error occurred")],
                    is_error: Some(true),
                    structured_content: None,
                    meta: Some(
                        vec![("error_type".to_string(), json!("user_requested"))]
                            .into_iter()
                            .collect(),
                    ),
                });
            }
        }

        Ok(ToolResult {
            content: vec![ContentBlock::text("Tool executed safely")],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

// Example 3: Math Calculator
#[derive(Clone)]
struct MathTool;

#[async_trait]
impl ToolHandler for MathTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let a = arguments.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b = arguments.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("add");

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Ok(ToolResult {
                        content: vec![ContentBlock::text("Division by zero error")],
                        is_error: Some(true),
                        structured_content: None,
                        meta: None,
                    });
                }
                a / b
            }
            _ => {
                return Ok(ToolResult {
                    content: vec![ContentBlock::text("Unknown operation")],
                    is_error: Some(true),
                    structured_content: None,
                    meta: None,
                });
            }
        };

        Ok(ToolResult {
            content: vec![ContentBlock::text(format!(
                "{} {} {} = {}",
                a, operation, b, result
            ))],
            is_error: Some(false),
            structured_content: Some(json!({
                "a": a,
                "b": b,
                "operation": operation,
                "result": result
            })),
            meta: None,
        })
    }
}

// Example 4: Data Processor
#[derive(Clone)]
struct DataProcessor;

#[async_trait]
impl ToolHandler for DataProcessor {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let data = arguments
            .get("data")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let processed_count = data.len();
        let processed_data: Vec<String> = data
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_uppercase())
            .collect();

        Ok(ToolResult {
            content: vec![ContentBlock::text(format!(
                "Processed {} items: {:?}",
                processed_count, processed_data
            ))],
            is_error: Some(false),
            structured_content: Some(json!({
                "original_count": processed_count,
                "processed_data": processed_data
            })),
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    println!("Starting Complete Working Examples MCP Server...");

    // Create server
    let server = McpServer::new("working-examples".to_string(), "1.0.0".to_string());

    println!(
        "\nServer created: {} v{}",
        server.info().name,
        server.info().version
    );
    println!("Examples compiled successfully!");

    // Test tool handlers
    println!("\nTesting tool handlers:");

    // Test SimpleTool
    let simple_tool = SimpleTool;
    let mut args = HashMap::new();
    args.insert("message".to_string(), json!("Hello World"));
    let result = simple_tool.call(args).await?;
    println!("SimpleTool: {:?}", result);

    // Test SafeTool
    let safe_tool = SafeTool;
    let mut args = HashMap::new();
    args.insert("should_error".to_string(), json!(false));
    let result = safe_tool.call(args).await?;
    println!("SafeTool: {:?}", result);

    // Test MathTool
    let math_tool = MathTool;
    let mut args = HashMap::new();
    args.insert("a".to_string(), json!(10));
    args.insert("b".to_string(), json!(5));
    args.insert("operation".to_string(), json!("add"));
    let result = math_tool.call(args).await?;
    println!("MathTool: {:?}", result);

    // Test DataProcessor
    let processor = DataProcessor;
    let mut args = HashMap::new();
    args.insert("data".to_string(), json!(["hello", "world", "rust"]));
    let result = processor.call(args).await?;
    println!("DataProcessor: {:?}", result);

    println!("\nAll examples work correctly!");

    Ok(())
}

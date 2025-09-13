//! Example: Tool Implementation Without Macros
//! Since mcp_tool macro doesn't exist, we implement tools directly

use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
struct CalculatorTool;

#[async_trait]
impl ToolHandler for CalculatorTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("add");

        let a = arguments.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let b = arguments.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Ok(ToolResult {
                        content: vec![ContentBlock::text("Error: Division by zero")],
                        is_error: Some(true),
                        meta: None,
                        structured_content: None,
                    });
                }
                a / b
            }
            _ => {
                return Ok(ToolResult {
                    content: vec![ContentBlock::text("Error: Invalid operation")],
                    is_error: Some(true),
                    meta: None,
                    structured_content: None,
                });
            }
        };

        Ok(ToolResult {
            content: vec![ContentBlock::text(&format!("Result: {}", result))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServer::new("tool-example".to_string(), "1.0.0".to_string());

    // Add calculator tool
    server
        .add_tool(
            "calculator".to_string(),
            Some("Perform arithmetic operations".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"]
                    },
                    "a": { "type": "number" },
                    "b": { "type": "number" }
                },
                "required": ["operation", "a", "b"]
            }),
            CalculatorTool,
        )
        .await?;

    println!("Tool example server created with calculator tool");
    Ok(())
}

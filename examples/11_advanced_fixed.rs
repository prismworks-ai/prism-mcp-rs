//! Example 11: Advanced Tools (Fixed)
//! Demonstrates advanced tool patterns

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

struct MultiTool;

#[async_trait]
impl ToolHandler for MultiTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        let operation = arguments
            .get("op")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let content = match operation {
            "calc" => vec![ContentBlock::text("Result: 42")],
            "data" => vec![ContentBlock::text("Data processed")],
            _ => vec![ContentBlock::text("Unknown operation")],
        };

        Ok(CallToolResult {
            content,
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Advanced Tools Example");
    println!("=====================");

    let tool = MultiTool;

    let mut args = HashMap::new();
    args.insert("op".to_string(), json!("calc"));
    let result = tool.call(args).await?;
    println!("Calc result: {:?}", result.content);

    let mut args = HashMap::new();
    args.insert("op".to_string(), json!("data"));
    let result = tool.call(args).await?;
    println!("Data result: {:?}", result.content);

    Ok(())
}

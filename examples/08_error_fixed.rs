//! Example 08: Error Handling (Fixed)
//! Demonstrates error handling patterns

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

struct RobustTool;

#[async_trait]
impl ToolHandler for RobustTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        if arguments.is_empty() {
            return Err(McpError::Validation("Parameters required".to_string()));
        }
        Ok(CallToolResult {
            content: vec![ContentBlock::text("Success")],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Error Handling Example");
    println!("=====================");

    let tool = RobustTool;

    // Test error handling
    match tool.call(HashMap::new()).await {
        Err(e) => println!("Handled error: {}", e),
        Ok(_) => println!("Unexpected success"),
    }

    // Test success
    let mut args = HashMap::new();
    args.insert("data".to_string(), json!("test"));
    let result = tool.call(args).await?;
    println!("Success: {:?}", result.content);

    Ok(())
}

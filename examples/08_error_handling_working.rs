//! Example 08: Error Handling (Working Fixed Version)
//! Demonstrates proper error handling in MCP

use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

/// Tool that demonstrates various error conditions
struct ErrorDemoTool;

#[async_trait]
impl ToolHandler for ErrorDemoTool {
    async fn call(&self, arguments: HashMap<String, serde_json::Value>) -> McpResult<ToolResult> {
        let error_type = arguments
            .get("error_type")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        
        match error_type {
            "validation" => Err(McpError::validation("Invalid input provided")),
            "internal" => Err(McpError::internal("Internal server error")),
            "protocol" => Err(McpError::protocol("Protocol violation")),
            "transport" => Err(McpError::transport("Transport error")),
            "none" => Ok(ToolResult {
                content: vec![ContentBlock::Text {
                    text: "No error - success!".to_string(),
                    annotations: None,
                    meta: None,
                }],
                is_error: Some(false),
                structured_content: None,
                meta: None,
            }),
            _ => Err(McpError::validation(format!(
                "Unknown error type: {}", 
                error_type
            ))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Error Handling Working Example");
    println!("==============================");
    
    let tool = ErrorDemoTool;
    
    // Test successful call
    let mut args = HashMap::new();
    args.insert("error_type".to_string(), serde_json::json!("none"));
    
    match tool.call(args).await {
        Ok(result) => println!("Success: {:?}", result),
        Err(e) => println!("Error: {:?}", e),
    }
    
    // Test validation error
    let mut args = HashMap::new();
    args.insert("error_type".to_string(), serde_json::json!("validation"));
    
    match tool.call(args).await {
        Ok(result) => println!("Success: {:?}", result),
        Err(e) => println!("Validation Error: {:?}", e),
    }
    
    println!("Error handling working example completed");
    
    Ok(())
}

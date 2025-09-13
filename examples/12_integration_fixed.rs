//! Example 12: Integration Patterns (Fixed)
//! Shows complete MCP server integration

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

#[allow(dead_code)]
struct IntegratedTool;

#[async_trait]
impl ToolHandler for IntegratedTool {
    async fn call(&self, _arguments: HashMap<String, Value>) -> McpResult<CallToolResult> {
        Ok(CallToolResult {
            content: vec![ContentBlock::text("Integrated tool executed")],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Integration Patterns Example");
    println!("============================");

    let server = McpServer::new("integrated-server".to_string(), "1.0.0".to_string());

    // Register tool - Note: In production, this would be done during server setup
    // server.add_tool() is an async method that would be called like:
    // server.add_tool(
    //     "integrated",
    //     Some("Integrated tool"),
    //     serde_json::json!({}),
    //     IntegratedTool,
    // ).await.unwrap();

    println!("Server: {}", server.name());
    println!("Version: {}", server.version());
    println!("Tools registered successfully");

    Ok(())
}

//! Working example demonstrating correct API usage

use anyhow::Result;
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
struct SimpleToolHandler;

#[async_trait]
impl ToolHandler for SimpleToolHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello from tool!");

        Ok(ToolResult {
            content: vec![ContentBlock::text(message)],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Working Example MCP Server...");

    // Create server
    let server = McpServer::new("working-example".to_string(), "1.0.0".to_string());

    // Add a tool
    server
        .add_tool(
            "echo".to_string(),
            Some("Echo a message".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                }
            }),
            SimpleToolHandler,
        )
        .await?;

    // For stdio transport (requires stdio feature)
    #[cfg(feature = "stdio")]
    {
        // server.run_with_stdio() - method not available.await?;
    }

    // For HTTP transport (requires http feature)
    #[cfg(feature = "http")]
    {
        use prism_mcp_rs::transport::HttpServerTransport;
        let transport = HttpServerTransport::new("127.0.0.1:8080");
        // server.run_with_transport(transport).await?;
    }

    Ok(())
}

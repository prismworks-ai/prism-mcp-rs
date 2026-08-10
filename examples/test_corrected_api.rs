use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
struct SystemToolHandler;

#[async_trait]
impl ToolHandler for SystemToolHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");

        Ok(ToolResult {
            content: vec![ContentBlock::text(format!(
                "Hello, {}! System: {}",
                name,
                std::env::consts::OS
            ))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create server
    let server = McpServer::new("corrected-example-server".to_string(), "1.0.0".to_string());

    // Add the system_info tool with corrected API - using the async method

    server
        .add_tool(
            "hello_system".to_string(),
            Some("Say hello and show system info".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name to greet"
                    }
                }
            }),
            SystemToolHandler,
        )
        .await?;

    println!("Starting MCP server with corrected API patterns...");
    #[cfg(feature = "stdio")]
    {
        let mut server = server;
        let transport = StdioServerTransport::new();
        return server.start(transport).await;
    }

    #[cfg(not(feature = "stdio"))]
    {
        eprintln!("stdio feature is disabled; skipping server start.");
        Ok(())
    }
}

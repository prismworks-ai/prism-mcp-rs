//! Fixed examples that compile with the actual library API
//!
//! This example shows the correct way to use the prism-mcp-rs library

use anyhow::Result;
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

// Simple tool handler implementation
#[derive(Clone)]
struct EchoHandler;

#[async_trait]
impl ToolHandler for EchoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello!");

        Ok(ToolResult {
            content: vec![ContentBlock::text(message)],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

// Simple resource handler
#[derive(Clone)]
struct FileResourceHandler;

#[async_trait]
impl ResourceHandler for FileResourceHandler {
    async fn read(&self, uri: String) -> McpResult<ResourceContents> {
        // Simple example - just return static content
        Ok(ResourceContents::Text(TextResourceContents {
            text: format!("Content of resource: {}", uri),
            uri: uri.clone(),
            mime_type: Some("text/plain".to_string()),
        }))
    }

    async fn list(&self) -> McpResult<Vec<Resource>> {
        Ok(vec![
            Resource { uri: "file://example.txt".to_string(), name: "example.txt".to_string(), description: Some("Example file".to_string()), mime_type: Some("text/plain".to_string()),
                annotations: None,
                meta: None,
                size: None,
                title: None,
            , annotations: None, meta: None, size: None, title: None }
        ])
    }
}

// Simple prompt handler
#[derive(Clone)]
struct GreetingPromptHandler;

#[async_trait]
impl PromptHandler for GreetingPromptHandler {
    async fn get_prompt(&self, arguments: HashMap<String, String>) -> McpResult<GetPromptResult> {
        let name = arguments.get("name").unwrap_or(&"World".to_string());

        Ok(GetPromptResult {
            description: Some("A greeting prompt".to_string()),
            messages: vec![PromptMessage {
                role: Role::User,
                content: Content::Text(TextContent {
                    text: format!("Hello, {}!", name),
                    annotations: None,
                }),
            }],
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Fixed Example MCP Server...");

    // Create server using the correct API
    let mut server = McpServer::new("fixed-example".to_string(), "1.0.0".to_string());

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
            EchoHandler,
        )
        .await?;

    // Add a resource
    server
        .add_resource_handler("file".to_string(), FileResourceHandler)
        .await?;

    // Add a prompt
    server.add_prompt(
        Prompt { name: "greeting".to_string(), description: Some("Generate a greeting".to_string()), arguments: Some(vec![
                PromptArgument { name: "name".to_string(), description: Some("Name to greet".to_string()), required: Some(false),
                    title: None,
                , meta: None, title: None , title: None }
            ]),
            meta: None,
            title: None,
        },
        GreetingPromptHandler,
    ).await?;

    println!("Server created successfully!");
    println!("Note: To run the server, you would need to:");
    println!("1. Create a transport (StdioServerTransport, HttpServerTransport, etc.)");
    println!("2. Start the server with the transport");
    println!("3. Handle the server lifecycle");

    // Example with HTTP transport (requires 'http' feature)
    #[cfg(feature = "http")]
    {
        use prism_mcp_rs::transport::HttpServerTransport;
        let transport = HttpServerTransport::new("127.0.0.1:8080");
        println!("HTTP transport created on 127.0.0.1:8080");
        // To actually run: server would need a method to accept transport
    }

    // Example with stdio transport (requires 'stdio' feature)
    #[cfg(feature = "stdio")]
    {
        use prism_mcp_rs::transport::StdioServerTransport;
        let transport = StdioServerTransport::new();
        println!("STDIO transport created");
        // To actually run: server would need a method to accept transport
    }

    Ok(())
}

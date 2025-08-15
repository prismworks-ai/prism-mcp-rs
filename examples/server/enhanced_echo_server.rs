// ! improved Echo Server Example
// !
// ! This example demonstrates the improved API with convenience methods
// ! and the improved prelude module. This is a working example that shows
// ! how the fixes make the SDK much more ergonomic to use.

use prism_mcp_rs::prelude::*;

/// Simple echo tool handler using the improved prelude
struct EchoHandler;

#[async_trait]
impl ToolHandler for EchoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello, World!");

        let repeat_count = arguments
            .get("repeat")
            .and_then(|v| v.as_u64())
            .unwrap_or(1)
            .min(10); // Limit to prevent spam

        let separator = arguments
            .get("separator")
            .and_then(|v| v.as_str())
            .unwrap_or(" ");

        let mut responses = Vec::new();
        for _ in 0..repeat_count {
            responses.push(message.to_string());
        }

        let result = responses.join(separator);

        Ok(ToolResult {
            content: vec![ContentBlock::text(result)],
            is_error: None,
            structured_content: Some(json!({
                "original_message": message,
                "repeat_count": repeat_count,
                "separator": separator
            })),
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging (only if tracing-subscriber feature is enabled)
    #[cfg(feature = "tracing-subscriber")]
    tracing_subscriber::fmt::init();

    let server = McpServer::new("improved-echo-server".to_string(), "1.0.0".to_string());

    // Add the echo tool using the improved API
    server
        .add_tool(
            "echo".to_string(),
            Some("Echo a message with optional repetition".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to echo"
                    },
                    "repeat": {
                        "type": "integer",
                        "description": "Number of times to repeat the message (max 10)",
                        "minimum": 1,
                        "maximum": 10,
                        "default": 1
                    },
                    "separator": {
                        "type": "string",
                        "description": "Separator between repeated messages",
                        "default": " "
                    }
                },
                "required": ["message"]
            }),
            EchoHandler,
        )
        .await?;

    // Use the new convenience method to run the server
    // This automatically:
    // 1. Creates a STDIO transport
    // 2. Starts the server
    // 3. Waits for Ctrl+C
    // 4. smoothly shuts down
    server.run_with_stdio().await
}

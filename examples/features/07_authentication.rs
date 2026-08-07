//! Illustrative tool-argument validation; this is not transport authentication.
//!
//! Production services must validate credentials before MCP dispatch, construct
//! a trusted `RequestContext`, and enforce authorization centrally. Never pass a
//! bearer token as a tool argument as this demonstration does.

use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
struct AuthenticatedTool;

#[async_trait]
impl ToolHandler for AuthenticatedTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Deliberately simplified demonstration. Do not copy this token pattern
        // into a production MCP service.
        let token = arguments
            .get("auth_token")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if token.is_empty() {
            return Ok(ToolResult {
                content: vec![ContentBlock::text("Error: Authentication required")],
                is_error: Some(true),
                meta: None,
                structured_content: None,
            });
        }

        // Mock validation
        if token != "valid-token-123" {
            return Ok(ToolResult {
                content: vec![ContentBlock::text("Error: Invalid authentication token")],
                is_error: Some(true),
                meta: None,
                structured_content: None,
            });
        }

        Ok(ToolResult {
            content: vec![ContentBlock::text(
                "Successfully authenticated and executed",
            )],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServer::new("auth-example".to_string(), "1.0.0".to_string());

    server
        .add_tool(
            "authenticated_operation".to_string(),
            Some("Operation requiring authentication".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "auth_token": { "type": "string" },
                    "operation": { "type": "string" }
                },
                "required": ["auth_token"]
            }),
            AuthenticatedTool,
        )
        .await?;

    println!("Authentication example server created");
    Ok(())
}

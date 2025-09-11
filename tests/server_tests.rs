// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! Tests for server components

use prism_mcp_rs::{
    core::tool::{EchoTool, ToolHandler},
    protocol::types::ServerCapabilities,
    server::McpServer,
};
use serde_json::json;
use std::collections::HashMap;

#[cfg(test)]
mod server_tests {
    use super::*;

    #[tokio::test]
    async fn test_server_with_tool() {
        let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());

        // Test that tool can be added and validates functionality
        let result = server
            .add_tool(
                "echo".to_string(),
                Some("Echo a message".to_string()),
                json!({
                    "type": "object",
                    "properties": {
                        "message": {"type": "string"}
                    }
                }),
                EchoTool,
            )
            .await;

        assert!(result.is_ok(), "Tool should be added successfully");
    }

    #[tokio::test]
    async fn test_echo_tool() {
        let tool = EchoTool;
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("Hello, World!"));

        let result = tool.call(args).await.unwrap();
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.is_error, None);
    }

    #[test]
    fn test_server_capabilities_with_tools() {
        let capabilities = ServerCapabilities {
            tools: Some(prism_mcp_rs::protocol::types::ToolsCapability {
                list_changed: Some(true),
            }),
            ..Default::default()
        };

        assert!(capabilities.tools.is_some());
        assert_eq!(capabilities.tools.unwrap().list_changed, Some(true));
    }
}

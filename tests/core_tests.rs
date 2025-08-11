// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! complete tests for core modules and utilities

use async_trait::async_trait;
use prism_mcp_rs::{
    core::{
        error::{McpError, McpResult},
        tool::{AdditionTool, EchoTool, Tool, ToolHandler},
    },
    protocol::types::{Content, ToolResult},
    utils::uri::*,
};
use serde_json::{Value, json};
use std::collections::HashMap;

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let errors = vec![
            McpError::Protocol("Invalid protocol version".to_string()),
            McpError::Transport("Connection failed".to_string()),
            McpError::ToolNotFound("unknown_tool".to_string()),
            McpError::Validation("Missing required parameter".to_string()),
            McpError::Internal("Server error".to_string()),
            McpError::Timeout("Operation timed out".to_string()),
            McpError::ResourceNotFound("Resource not found".to_string()),
            McpError::PromptNotFound("Prompt not found".to_string()),
            McpError::Serialization("JSON serialization failed".to_string()),
            McpError::Io("File operation failed".to_string()),
        ];

        for error in errors {
            match error {
                McpError::Protocol(msg) => assert!(!msg.is_empty()),
                McpError::Transport(msg) => assert!(!msg.is_empty()),
                McpError::ToolNotFound(msg) => assert!(!msg.is_empty()),
                McpError::Validation(msg) => assert!(!msg.is_empty()),
                McpError::Internal(msg) => assert!(!msg.is_empty()),
                McpError::Timeout(msg) => assert!(!msg.is_empty()),
                McpError::ResourceNotFound(msg) => assert!(!msg.is_empty()),
                McpError::PromptNotFound(msg) => assert!(!msg.is_empty()),
                McpError::Serialization(msg) => assert!(!msg.is_empty()),
                McpError::Io(msg) => assert!(!msg.is_empty()),
                _ => {}
            }
        }
    }

    #[test]
    fn test_error_display() {
        let error = McpError::Protocol("Test error message".to_string());
        let display_str = format!("{error}");
        assert!(display_str.contains("Test error message"));
    }

    #[test]
    fn test_error_categories() {
        let protocol_errors = vec![
            McpError::Protocol("Protocol error".to_string()),
            McpError::ToolNotFound("Tool not found".to_string()),
            McpError::Validation("Invalid params".to_string()),
        ];

        for error in protocol_errors {
            assert!(matches!(
                error,
                McpError::Protocol(_) | McpError::ToolNotFound(_) | McpError::Validation(_)
            ));
        }
    }

    #[test]
    fn test_error_recovery() {
        assert!(McpError::connection("timeout").is_recoverable());
        assert!(!McpError::validation("invalid input").is_recoverable());
        assert!(McpError::timeout("request timeout").is_recoverable());
    }
}

#[cfg(test)]
mod tool_tests {
    use super::*;

    struct TestTool;

    #[async_trait]
    impl ToolHandler for TestTool {
        async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
            let action = arguments
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            match action {
                "success" => Ok(ToolResult {
                    content: vec![Content::text("Operation successful")],
                    is_error: Some(false),
                    structured_content: None,
                    meta: None,
                }),
                "error" => Ok(ToolResult {
                    content: vec![Content::text("Operation failed")],
                    is_error: Some(true),
                    structured_content: None,
                    meta: None,
                }),
                "timeout" => Err(McpError::Timeout("Operation timed out".to_string())),
                _ => Ok(ToolResult {
                    content: vec![Content::text("Default response")],
                    is_error: Some(false),
                    structured_content: None,
                    meta: None,
                }),
            }
        }
    }

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new(
            "test_tool".to_string(),
            Some("A test tool".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string"}
                }
            }),
            TestTool,
        );

        assert_eq!(tool.info.name, "test_tool");
        assert_eq!(tool.info.description.as_ref().unwrap(), "A test tool");
        assert!(tool.is_enabled());
    }

    #[tokio::test]
    async fn test_tool_handler_success() {
        let handler = TestTool;
        let mut args = HashMap::new();
        args.insert("action".to_string(), json!("success"));

        let result = handler.call(args).await.unwrap();
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_echo_tool() {
        let tool = EchoTool;
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("test message"));

        let result = tool.call(args).await.unwrap();
        match &result.content[0] {
            Content::Text { text, .. } => assert_eq!(text, "test message"),
            _ => panic!("Expected text content"),
        }
    }

    #[tokio::test]
    async fn test_addition_tool() {
        let tool = AdditionTool;
        let mut args = HashMap::new();
        args.insert("a".to_string(), json!(5.0));
        args.insert("b".to_string(), json!(3.0));

        let result = tool.call(args).await.unwrap();
        match &result.content[0] {
            Content::Text { text, .. } => assert_eq!(text, "8"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_tool_enable_disable() {
        let mut tool = Tool::new(
            "test_tool".to_string(),
            None,
            json!({"type": "object"}),
            EchoTool,
        );

        assert!(tool.is_enabled());

        tool.disable();
        assert!(!tool.is_enabled());

        tool.enable();
        assert!(tool.is_enabled());
    }

    #[tokio::test]
    async fn test_disabled_tool() {
        let mut tool = Tool::new(
            "test_tool".to_string(),
            None,
            json!({"type": "object"}),
            EchoTool,
        );

        tool.disable();

        let result = tool.call(HashMap::new()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::Validation(msg) => assert!(msg.contains("disabled")),
            _ => panic!("Expected validation error"),
        }
    }
}

#[cfg(test)]
mod uri_tests {
    use super::*;

    #[test]
    fn test_validate_uri_valid() {
        let valid_uris = vec![
            "file:///path/to/file.txt",
            "https://example.com/resource",
            "http://localhost:8080/api",
        ];

        for uri in valid_uris {
            let result = validate_uri(uri);
            assert!(result.is_ok(), "URI should be valid: {uri}");
        }
    }

    #[test]
    fn test_guess_mime_type() {
        let test_cases = vec![
            ("file.txt", Some("text/plain".to_string())),
            ("data.json", Some("application/json".to_string())),
            ("image.png", Some("image/png".to_string())),
        ];

        for (filename, expected) in test_cases {
            let result = guess_mime_type(filename);
            assert_eq!(result, expected, "Failed for: {filename}");
        }
    }

    #[test]
    fn test_get_uri_extension() {
        let test_cases = vec![
            ("file.txt", Some("txt".to_string())),
            ("data.json", Some("json".to_string())),
            ("image.png", Some("png".to_string())),
            ("no_extension", None),
        ];

        for (filename, expected) in test_cases {
            let result = get_uri_extension(filename);
            assert_eq!(result, expected, "Failed for: {filename}");
        }
    }
}

#[cfg(test)]
mod content_tests {
    use super::*;

    #[test]
    fn test_text_content() {
        let content = Content::text("Hello, world!");
        match content {
            Content::Text {
                text, annotations, ..
            } => {
                assert_eq!(text, "Hello, world!");
                assert!(annotations.is_none());
            }
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_image_content() {
        let content = Content::image("base64data", "image/png");
        match content {
            Content::Image {
                data,
                mime_type,
                annotations,
                ..
            } => {
                assert_eq!(data, "base64data");
                assert_eq!(mime_type, "image/png");
                assert!(annotations.is_none());
            }
            _ => panic!("Expected image content"),
        }
    }

    #[test]
    fn test_audio_content() {
        let content = Content::audio("base64audiodata", "audio/wav");
        match content {
            Content::Audio {
                data,
                mime_type,
                annotations,
                ..
            } => {
                assert_eq!(data, "base64audiodata");
                assert_eq!(mime_type, "audio/wav");
                assert!(annotations.is_none());
            }
            _ => panic!("Expected audio content"),
        }
    }

    #[test]
    fn test_resource_content() {
        let content = Content::resource_link("file:///test.txt", "Test File");
        match content {
            Content::ResourceLink {
                uri,
                name,
                annotations,
                ..
            } => {
                assert_eq!(uri, "file:///test.txt");
                assert_eq!(name, "Test File");
                assert!(annotations.is_none());
            }
            _ => panic!("Expected resource link content"),
        }
    }
}

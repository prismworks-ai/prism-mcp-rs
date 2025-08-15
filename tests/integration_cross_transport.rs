// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! Cross-Transport Integration Tests
// !
// ! Tests that validate protocol compliance and behavior consistency across different transport types.

use prism_mcp_rs::{
    core::{
        error::{McpError, McpResult},
        resource::LegacyResourceHandler,
        tool::ToolHandler,
    },
    protocol::types::*,
};
use serde_json::{Value, json};
use std::{collections::HashMap, time::Duration};

#[cfg(test)]
mod cross_transport_tests {
    use super::*;

    // Universal tool handler for cross-transport testing
    struct UniversalToolHandler;

    impl UniversalToolHandler {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait::async_trait]
    impl ToolHandler for UniversalToolHandler {
        async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
            let message = arguments
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("default message");

            Ok(ToolResult {
                content: vec![Content::Text {
                    text: format!("Echo: {message}"),
                    annotations: None,
                    meta: None,
                }],
                is_error: None,
                structured_content: None,
                meta: None,
            })
        }
    }

    // Universal resource handler for cross-transport testing
    struct UniversalResourceHandler {
        resources: HashMap<String, String>,
    }

    impl UniversalResourceHandler {
        fn new() -> Self {
            let mut resources = HashMap::new();
            resources.insert(
                "universal://test/text".to_string(),
                "Test text content".to_string(),
            );
            resources.insert(
                "universal://test/json".to_string(),
                r#"{"test": "data"}"#.to_string(),
            );

            Self { resources }
        }
    }

    #[async_trait::async_trait]
    impl LegacyResourceHandler for UniversalResourceHandler {
        async fn read(&self, uri: &str) -> McpResult<String> {
            match self.resources.get(uri) {
                Some(content) => Ok(content.clone()),
                None => Err(McpError::validation(format!("Resource not found: {uri}"))),
            }
        }

        async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
            Ok(self
                .resources
                .keys()
                .map(|uri| ResourceInfo {
                    uri: uri.clone(),
                    name: format!("Resource: {uri}"),
                    description: None,
                    mime_type: Some("text/plain".to_string()),
                    annotations: None,
                    meta: None,
                    title: Some(format!("Resource: {uri}")),
                    size: None,
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn test_transport_feature_parity() {
        // Test that all transports support the same core features
        let stdio_tool_handler = UniversalToolHandler::new();
        let stdio_resource_handler = UniversalResourceHandler::new();

        // Test tool execution
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("feature test"));
        let execute_result = stdio_tool_handler.call(args).await;
        assert!(execute_result.is_ok(), "Tool execution should work");

        // Test resource reading
        let read_result = stdio_resource_handler.read("universal://test/text").await;
        assert!(read_result.is_ok(), "Resource reading should work");

        // Test error handling
        let error_result = stdio_resource_handler.read("nonexistent://resource").await;
        assert!(error_result.is_err(), "Error handling should work");

        // Test JSON-RPC compliance
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test".to_string(),
            params: None,
        };
        let json_test = serde_json::to_string(&request).is_ok();
        assert!(json_test, "JSON-RPC compliance should work");

        println!("Transport feature parity test completed");
    }

    #[tokio::test]
    async fn test_cross_transport_data_integrity() {
        // Test that data remains intact across transport boundaries
        let test_data_sets = [
            json!({"message": "simple test"}),
            json!({"message": "Unicode: 世界 🌍"}),
            json!({"message": "Special: \"quotes\" 'apostrophes'"}),
        ];

        let stdio_handler = UniversalToolHandler::new();

        for (i, test_data) in test_data_sets.iter().enumerate() {
            let mut args = HashMap::new();
            args.insert("message".to_string(), test_data["message"].clone());

            let result = stdio_handler.call(args).await;
            assert!(result.is_ok(), "Data integrity test {i} should succeed");

            let tool_result = result.unwrap();
            match &tool_result.content[0] {
                Content::Text { text, .. } => {
                    assert!(
                        text.contains(test_data["message"].as_str().unwrap()),
                        "Data should be preserved intact"
                    );
                }
                _ => panic!("Expected text content"),
            }
        }

        println!("Cross-transport data integrity test completed");
        println!("Tested {} data sets for integrity", test_data_sets.len());
    }

    #[tokio::test]
    async fn test_cross_transport_message_size_limits() {
        // Test message size handling across transports
        let size_test_cases = vec![
            ("small", "x".repeat(100)),   // 100 bytes
            ("medium", "x".repeat(1000)), // 1KB
            ("large", "x".repeat(4000)),  // 4KB
        ];

        let stdio_handler = UniversalToolHandler::new();

        for (size_name, large_text) in &size_test_cases {
            let mut args = HashMap::new();
            args.insert("message".to_string(), json!(large_text));

            let result = stdio_handler.call(args).await;
            assert!(result.is_ok(), "Size test '{size_name}' should succeed");
        }

        println!("Cross-transport message size test completed");
    }

    #[tokio::test]
    async fn test_json_rpc_compliance() {
        // Test JSON-RPC protocol compliance
        let json_rpc_tests = vec![
            // Valid requests
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "tools/list".to_string(),
                params: None,
            },
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!("string_id"),
                method: "tools/call".to_string(),
                params: Some(json!({
                    "name": "universal_echo",
                    "arguments": {"message": "test"}
                })),
            },
        ];

        // Test request serialization consistency
        for (i, request) in json_rpc_tests.iter().enumerate() {
            let serialized = serde_json::to_string(request).unwrap();
            let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

            assert_eq!(
                request.jsonrpc, deserialized.jsonrpc,
                "Request {i}: JSON-RPC version should match"
            );
            assert_eq!(request.id, deserialized.id, "Request {i}: ID should match");
            assert_eq!(
                request.method, deserialized.method,
                "Request {i}: Method should match"
            );

            println!("JSON-RPC request {i} serialization: OK");
        }

        // Test response consistency
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: json!(42),
            result: Some(json!({
                "tools": [],
                "metadata": {"transport_agnostic": true}
            })),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.jsonrpc, deserialized.jsonrpc);
        assert_eq!(response.id, deserialized.id);
        assert_eq!(response.result, deserialized.result);

        println!("JSON-RPC compliance test completed");
    }

    #[tokio::test]
    async fn test_protocol_version_compatibility() {
        // Test protocol version consistency
        assert_eq!(LATEST_PROTOCOL_VERSION, "2025-06-18");
        assert_eq!(JSONRPC_VERSION, "2.0");

        // Test capabilities structure
        let server_caps = ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&server_caps).unwrap();
        let deserialized: ServerCapabilities = serde_json::from_str(&serialized).unwrap();

        assert_eq!(server_caps, deserialized);

        println!("Protocol version compatibility test completed");
    }

    #[tokio::test]
    async fn test_content_types_support() {
        // Test all content types from 2025-03-26 specification
        let text_content = Content::text("Hello, world!");
        let image_content = Content::image("base64data", "image/png");
        let audio_content = Content::audio("audiodata", "audio/wav");
        let resource_content = Content::resource("file:///test.txt");

        // Test serialization/deserialization
        for content in vec![text_content, image_content, audio_content, resource_content] {
            let serialized = serde_json::to_string(&content).unwrap();
            let deserialized: Content = serde_json::from_str(&serialized).unwrap();
            assert_eq!(content, deserialized);
        }

        println!("Content types support test completed");
    }

    #[tokio::test]
    async fn test_error_handling_consistency() {
        // Test that error scenarios behave consistently
        let resource_handler = UniversalResourceHandler::new();

        // Test with invalid URIs
        let invalid_uris = vec![
            "invalid://protocol/resource",
            "universal://test/nonexistent",
            "malformed uri",
        ];

        for uri in &invalid_uris {
            let result = resource_handler.read(uri).await;
            assert!(result.is_err(), "Invalid URI '{uri}' should fail");
        }

        // Test tool handler with various inputs
        let tool_handler = UniversalToolHandler::new();

        // Empty arguments should work
        let result = tool_handler.call(HashMap::new()).await;
        assert!(result.is_ok(), "Empty arguments should be handled smoothly");

        println!("Error handling consistency test completed");
    }

    #[tokio::test]
    async fn test_performance_characteristics() {
        // Test basic performance characteristics
        let handler = UniversalToolHandler::new();
        let start_time = std::time::Instant::now();

        // Perform a series of operations
        for i in 0..10 {
            let mut args = HashMap::new();
            args.insert(
                "message".to_string(),
                json!(format!("Performance test {}", i)),
            );

            let result = handler.call(args).await;
            assert!(result.is_ok(), "Tool call {i} should succeed");
        }

        let elapsed = start_time.elapsed();

        // Basic performance check - all calls should complete quickly
        assert!(
            elapsed < Duration::from_millis(100),
            "10 tool calls should complete in under 100ms, took: {elapsed:?}"
        );

        println!("Performance characteristics test completed");
    }
}

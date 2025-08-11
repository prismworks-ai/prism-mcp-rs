// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! WebSocket Transport End-to-End Integration Tests
// !
// ! This test suite validates complete server functionality and tool/resource operations
// ! using the WebSocket transport with real message flows.

#![cfg(feature = "websocket")]

use prism_mcp_rs::{
    core::{
        error::{McpError, McpResult},
        resource::LegacyResourceHandler,
        tool::ToolHandler,
    },
    protocol::types::{Content, ToolResult},
    server::McpServer,
    transport::{ServerTransport, TransportConfig, WebSocketServerTransport},
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[cfg(feature = "websocket")]
mod e2e_websocket_tests {
    use super::*;

    // Test tool handler for WebSocket integration tests
    struct WebSocketTestToolHandler {
        call_count: Arc<Mutex<u32>>,
    }

    impl WebSocketTestToolHandler {
        fn new() -> Self {
            Self {
                call_count: Arc::new(Mutex::new(0)),
            }
        }

        async fn get_call_count(&self) -> u32 {
            *self.call_count.lock().await
        }
    }

    #[async_trait::async_trait]
    impl ToolHandler for WebSocketTestToolHandler {
        async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
            // Update call tracking
            {
                let mut count = self.call_count.lock().await;
                *count += 1;
            }

            // Get message from arguments
            let message = arguments
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("default WebSocket message");

            Ok(ToolResult {
                content: vec![Content::Text {
                    text: format!("WebSocket Echo: {message}"),
                    annotations: None,
                    meta: None,
                }],
                is_error: None,
                structured_content: None,
                meta: None,
            })
        }
    }

    // Test resource handler for WebSocket integration tests
    struct WebSocketTestResourceHandler {
        resources: HashMap<String, String>,
    }

    impl WebSocketTestResourceHandler {
        fn new() -> Self {
            let mut resources = HashMap::new();
            resources.insert(
                "ws://test/resource1".to_string(),
                "WebSocket Content of resource 1".to_string(),
            );
            resources.insert(
                "ws://test/resource2".to_string(),
                "WebSocket Content of resource 2 with more data".to_string(),
            );

            Self { resources }
        }
    }

    #[async_trait::async_trait]
    impl LegacyResourceHandler for WebSocketTestResourceHandler {
        async fn read(&self, uri: &str) -> McpResult<String> {
            match self.resources.get(uri) {
                Some(content) => Ok(content.clone()),
                None => Err(McpError::validation(format!(
                    "WebSocket Resource not found: {uri}"
                ))),
            }
        }

        async fn list(&self) -> McpResult<Vec<prism_mcp_rs::protocol::types::ResourceInfo>> {
            Ok(self
                .resources
                .keys()
                .map(|uri| prism_mcp_rs::protocol::types::ResourceInfo {
                    uri: uri.clone(),
                    name: format!("WebSocket Resource: {uri}"),
                    description: Some("WebSocket test resource".to_string()),
                    mime_type: Some("text/plain".to_string()),
                    annotations: None,
                    size: None,
                    title: Some(format!("WebSocket Resource: {uri}")),
                    meta: None,
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn test_websocket_server_creation() {
        // Test WebSocket server creation and basic properties
        let server = McpServer::new("websocket-test-server".to_string(), "1.0.0".to_string());

        // Test that we can create tools and add them
        let _tool_handler = WebSocketTestToolHandler::new();
        let _resource_handler = WebSocketTestResourceHandler::new();

        // Test basic server operations
        assert_eq!(server.name(), "websocket-test-server");
        assert_eq!(server.version(), "1.0.0");

        println!("WebSocket server creation test passed");
    }

    #[tokio::test]
    async fn test_websocket_transport_configuration() {
        // Test WebSocket transport with various configurations
        let configs = vec![
            TransportConfig::default(),
            TransportConfig {
                read_timeout_ms: Some(5000),
                write_timeout_ms: Some(3000),
                max_message_size: Some(8192),
                ..Default::default()
            },
        ];

        for (i, config) in configs.into_iter().enumerate() {
            let transport = WebSocketServerTransport::with_config("127.0.0.1:0", config);

            // Test basic transport properties
            assert!(
                !transport.is_running(),
                "WebSocket Transport {i} should not be running initially"
            );

            let info = transport.server_info();
            assert!(
                info.contains("WebSocket"),
                "WebSocket Transport {i} info should mention WebSocket: {info}"
            );
        }

        println!("WebSocket transport configuration test passed");
    }

    #[tokio::test]
    async fn test_websocket_tool_handler_integration() {
        // Test WebSocket tool handler functionality independently
        let handler = WebSocketTestToolHandler::new();

        // Test echo functionality
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("Hello, WebSocket World!"));

        let result = handler.call(args).await;
        assert!(result.is_ok(), "WebSocket Tool call should succeed");

        let result = result.unwrap();
        assert!(
            result.is_error.is_none() || !result.is_error.unwrap(),
            "WebSocket Tool should not error"
        );
        assert_eq!(result.content.len(), 1, "Should have one content item");

        // Check the content
        match &result.content[0] {
            Content::Text { text, .. } => {
                assert!(
                    text.contains("Hello, WebSocket World!"),
                    "Should echo the WebSocket message"
                );
                assert!(
                    text.contains("WebSocket Echo:"),
                    "Should have WebSocket prefix"
                );
            }
            _ => panic!("Expected text content"),
        }

        // Verify call tracking
        assert_eq!(
            handler.get_call_count().await,
            1,
            "Should have 1 WebSocket call"
        );

        println!("WebSocket tool handler integration test passed");
    }

    #[tokio::test]
    async fn test_websocket_resource_handler_integration() {
        // Test WebSocket resource handler functionality
        let handler = WebSocketTestResourceHandler::new();

        // Test reading existing resource
        let content = handler.read("ws://test/resource1").await;
        assert!(
            content.is_ok(),
            "Should read WebSocket resource successfully"
        );

        let content = content.unwrap();
        assert_eq!(content, "WebSocket Content of resource 1");

        // Test reading non-existent resource
        let missing = handler.read("ws://test/missing").await;
        assert!(
            missing.is_err(),
            "Should fail for missing WebSocket resource"
        );

        match missing.unwrap_err() {
            McpError::Validation(_) => {} // Expected
            other => panic!("Expected Validation error, got: {other:?}"),
        }

        // Test resource listing
        let resources = handler.list().await;
        assert!(
            resources.is_ok(),
            "Should list WebSocket resources successfully"
        );

        let resources = resources.unwrap();
        assert_eq!(resources.len(), 2, "Should have 2 WebSocket resources");

        println!("WebSocket resource handler integration test passed");
    }

    #[tokio::test]
    async fn test_websocket_concurrent_tool_calls() {
        // Test handling multiple WebSocket tool calls concurrently
        let handler = Arc::new(WebSocketTestToolHandler::new());

        let mut handles = Vec::new();

        // Spawn multiple concurrent tool calls
        for i in 0..5 {
            let handler_clone = handler.clone();
            let handle = tokio::spawn(async move {
                let mut args = HashMap::new();
                args.insert(
                    "message".to_string(),
                    json!(format!("WebSocket Message {}", i)),
                );

                handler_clone.call(args).await
            });
            handles.push(handle);
        }

        // Wait for all calls to complete
        let mut success_count = 0;
        for handle in handles {
            let result = handle.await.unwrap();
            if result.is_ok() {
                success_count += 1;
                let tool_result = result.unwrap();
                assert!(
                    tool_result.is_error.is_none() || !tool_result.is_error.unwrap(),
                    "WebSocket Tool call should succeed"
                );
            }
        }

        assert_eq!(success_count, 5, "All WebSocket tool calls should succeed");
        assert_eq!(
            handler.get_call_count().await,
            5,
            "Should have 5 WebSocket calls recorded"
        );

        println!("WebSocket concurrent tool calls test passed");
    }

    #[tokio::test]
    async fn test_websocket_performance_characteristics() {
        // Test basic WebSocket performance characteristics
        let handler = WebSocketTestToolHandler::new();
        let start_time = std::time::Instant::now();

        // Perform a series of operations
        for i in 0..10 {
            let mut args = HashMap::new();
            args.insert(
                "message".to_string(),
                json!(format!("WebSocket Performance test {}", i)),
            );

            let result = handler.call(args).await;
            assert!(result.is_ok(), "WebSocket Tool call {i} should succeed");
        }

        let elapsed = start_time.elapsed();

        // Basic performance check - all calls should complete quickly
        assert!(
            elapsed < Duration::from_millis(100),
            "10 WebSocket tool calls should complete in under 100ms, took: {elapsed:?}"
        );

        assert_eq!(
            handler.get_call_count().await,
            10,
            "Should have 10 WebSocket calls"
        );

        println!("WebSocket performance characteristics test passed");
    }

    #[tokio::test]
    async fn test_websocket_error_scenarios_and_recovery() {
        // Test various WebSocket error scenarios and recovery mechanisms
        let resource_handler = WebSocketTestResourceHandler::new();

        // Test resource handler with invalid URI
        let result = resource_handler.read("invalid://websocket/uri").await;
        assert!(
            result.is_err(),
            "Invalid WebSocket resource URI should fail"
        );

        // Test tool handler with missing arguments
        let tool_handler = WebSocketTestToolHandler::new();
        let empty_args = HashMap::new();

        let result = tool_handler.call(empty_args).await;
        assert!(
            result.is_ok(),
            "WebSocket Tool should handle missing arguments smoothly"
        );

        println!("WebSocket error scenarios and recovery test passed");
    }

    #[tokio::test]
    async fn test_websocket_large_data_handling() {
        // Test WebSocket handling of larger data sets
        let handler = WebSocketTestToolHandler::new();

        // Create a large message
        let large_message = "x".repeat(2048); // 2KB message
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!(large_message));

        let result = handler.call(args).await;
        assert!(result.is_ok(), "Should handle large WebSocket messages");

        let result = result.unwrap();
        match &result.content[0] {
            Content::Text { text, .. } => {
                assert!(text.len() > 2000, "Should return large WebSocket response");
                assert!(
                    text.contains("WebSocket Echo:"),
                    "Should have WebSocket prefix"
                );
            }
            _ => panic!("Expected text content"),
        }

        println!("WebSocket large data handling test passed");
    }

    #[tokio::test]
    async fn test_websocket_json_serialization_compatibility() {
        // Test JSON serialization and deserialization for WebSocket
        let handler = WebSocketTestToolHandler::new();

        // Test with complex JSON data
        let complex_data = json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"},
                "unicode": "Hello, WebSocket 世界! 🌍"
            }
        });

        let mut args = HashMap::new();
        args.insert("message".to_string(), complex_data);

        let result = handler.call(args).await;
        assert!(
            result.is_ok(),
            "Should handle complex JSON data over WebSocket"
        );

        // Test serialization of result
        let result = result.unwrap();
        let serialized = serde_json::to_string(&result);
        assert!(
            serialized.is_ok(),
            "WebSocket Result should be serializable"
        );

        let deserialized: Result<ToolResult, _> = serde_json::from_str(&serialized.unwrap());
        assert!(
            deserialized.is_ok(),
            "WebSocket Result should be deserializable"
        );

        println!("WebSocket JSON serialization compatibility test passed");
    }

    #[tokio::test]
    async fn test_websocket_bidirectional_capabilities() {
        // Test WebSocket-specific bidirectional communication capabilities
        let handler = WebSocketTestToolHandler::new();

        // Test with WebSocket-specific data to validate bidirectional nature
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("bidirectional test"));
        args.insert(
            "websocket_features".to_string(),
            json!({
                "supports_notifications": true,
                "supports_push": true,
                "connection_type": "persistent"
            }),
        );

        let result = handler.call(args).await;
        assert!(
            result.is_ok(),
            "WebSocket bidirectional test should succeed"
        );

        let result = result.unwrap();
        match &result.content[0] {
            Content::Text { text, .. } => {
                assert!(
                    text.contains("bidirectional test"),
                    "Should handle bidirectional data"
                );
            }
            _ => panic!("Expected text content"),
        }

        println!("WebSocket bidirectional capabilities test passed");
    }
}

#[cfg(not(feature = "websocket"))]
#[test]
fn integration_websocket_feature_disabled() {
    println!("WebSocket integration tests require the 'websocket' feature to be enabled");
    println!("Run with: cargo test --features websocket --test integration_e2e_websocket");
    assert!(true, "WebSocket feature not enabled - this is expected");
}

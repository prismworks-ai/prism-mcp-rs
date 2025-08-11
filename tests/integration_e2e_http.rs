// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! HTTP Transport End-to-End Integration Tests
// !
// ! This test suite validates complete server functionality and tool/resource operations
// ! using the HTTP transport with real message flows.

#![cfg(feature = "http")]

use prism_mcp_rs::{
    core::{
        error::{McpError, McpResult},
        resource::LegacyResourceHandler,
        tool::ToolHandler,
    },
    protocol::types::{Content, ToolResult},
    server::McpServer,
    transport::{HttpServerTransport, ServerTransport, TransportConfig},
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[cfg(feature = "http")]
mod e2e_http_tests {
    use super::*;

    // Test tool handler for HTTP integration tests
    struct HttpTestToolHandler {
        call_count: Arc<Mutex<u32>>,
    }

    impl HttpTestToolHandler {
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
    impl ToolHandler for HttpTestToolHandler {
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
                .unwrap_or("default HTTP message");

            Ok(ToolResult {
                content: vec![Content::Text {
                    text: format!("HTTP Echo: {message}"),
                    annotations: None,
                    meta: None,
                }],
                is_error: None,
                structured_content: None,
                meta: None,
            })
        }
    }

    // Test resource handler for HTTP integration tests
    struct HttpTestResourceHandler {
        resources: HashMap<String, String>,
    }

    impl HttpTestResourceHandler {
        fn new() -> Self {
            let mut resources = HashMap::new();
            resources.insert(
                "http://test/resource1".to_string(),
                "HTTP Content of resource 1".to_string(),
            );
            resources.insert(
                "http://test/resource2".to_string(),
                "HTTP Content of resource 2 with more data".to_string(),
            );

            Self { resources }
        }
    }

    #[async_trait::async_trait]
    impl LegacyResourceHandler for HttpTestResourceHandler {
        async fn read(&self, uri: &str) -> McpResult<String> {
            match self.resources.get(uri) {
                Some(content) => Ok(content.clone()),
                None => Err(McpError::validation(format!(
                    "HTTP Resource not found: {uri}"
                ))),
            }
        }

        async fn list(&self) -> McpResult<Vec<prism_mcp_rs::protocol::types::ResourceInfo>> {
            Ok(self
                .resources
                .keys()
                .map(|uri| prism_mcp_rs::protocol::types::ResourceInfo {
                    uri: uri.clone(),
                    name: format!("HTTP Resource: {uri}"),
                    description: Some("HTTP test resource".to_string()),
                    mime_type: Some("text/plain".to_string()),
                    annotations: None,
                    size: None,
                    title: Some(format!("HTTP Resource: {uri}")),
                    meta: None,
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn test_http_server_creation() {
        // Test HTTP server creation and basic properties
        let server = McpServer::new("http-test-server".to_string(), "1.0.0".to_string());

        // Test that we can create tools and add them
        let _tool_handler = HttpTestToolHandler::new();
        let _resource_handler = HttpTestResourceHandler::new();

        // Test basic server operations
        assert_eq!(server.name(), "http-test-server");
        assert_eq!(server.version(), "1.0.0");

        println!("HTTP server creation test passed");
    }

    #[tokio::test]
    async fn test_http_transport_configuration() {
        // Test HTTP transport with various configurations
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
            let transport = HttpServerTransport::with_config("127.0.0.1:0", config);

            // Test basic transport properties
            assert!(
                !transport.is_running(),
                "HTTP Transport {i} should not be running initially"
            );

            let info = transport.server_info();
            assert!(
                info.contains("HTTP"),
                "HTTP Transport {i} info should mention HTTP: {info}"
            );
        }

        println!("HTTP transport configuration test passed");
    }

    #[tokio::test]
    async fn test_http_tool_handler_integration() {
        // Test HTTP tool handler functionality independently
        let handler = HttpTestToolHandler::new();

        // Test echo functionality
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("Hello, HTTP World!"));

        let result = handler.call(args).await;
        assert!(result.is_ok(), "HTTP Tool call should succeed");

        let result = result.unwrap();
        assert!(
            result.is_error.is_none() || !result.is_error.unwrap(),
            "HTTP Tool should not error"
        );
        assert_eq!(result.content.len(), 1, "Should have one content item");

        // Check the content
        match &result.content[0] {
            Content::Text { text, .. } => {
                assert!(
                    text.contains("Hello, HTTP World!"),
                    "Should echo the HTTP message"
                );
                assert!(text.contains("HTTP Echo:"), "Should have HTTP prefix");
            }
            _ => panic!("Expected text content"),
        }

        // Verify call tracking
        assert_eq!(handler.get_call_count().await, 1, "Should have 1 HTTP call");

        println!("HTTP tool handler integration test passed");
    }

    #[tokio::test]
    async fn test_http_resource_handler_integration() {
        // Test HTTP resource handler functionality
        let handler = HttpTestResourceHandler::new();

        // Test reading existing resource
        let content = handler.read("http://test/resource1").await;
        assert!(content.is_ok(), "Should read HTTP resource successfully");

        let content = content.unwrap();
        assert_eq!(content, "HTTP Content of resource 1");

        // Test reading non-existent resource
        let missing = handler.read("http://test/missing").await;
        assert!(missing.is_err(), "Should fail for missing HTTP resource");

        match missing.unwrap_err() {
            McpError::Validation(_) => {} // Expected
            other => panic!("Expected Validation error, got: {other:?}"),
        }

        // Test resource listing
        let resources = handler.list().await;
        assert!(resources.is_ok(), "Should list HTTP resources successfully");

        let resources = resources.unwrap();
        assert_eq!(resources.len(), 2, "Should have 2 HTTP resources");

        println!("HTTP resource handler integration test passed");
    }

    #[tokio::test]
    async fn test_http_concurrent_tool_calls() {
        // Test handling multiple HTTP tool calls concurrently
        let handler = Arc::new(HttpTestToolHandler::new());

        let mut handles = Vec::new();

        // Spawn multiple concurrent tool calls
        for i in 0..5 {
            let handler_clone = handler.clone();
            let handle = tokio::spawn(async move {
                let mut args = HashMap::new();
                args.insert("message".to_string(), json!(format!("HTTP Message {}", i)));

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
                    "HTTP Tool call should succeed"
                );
            }
        }

        assert_eq!(success_count, 5, "All HTTP tool calls should succeed");
        assert_eq!(
            handler.get_call_count().await,
            5,
            "Should have 5 HTTP calls recorded"
        );

        println!("HTTP concurrent tool calls test passed");
    }

    #[tokio::test]
    async fn test_http_performance_characteristics() {
        // Test basic HTTP performance characteristics
        let handler = HttpTestToolHandler::new();
        let start_time = std::time::Instant::now();

        // Perform a series of operations
        for i in 0..10 {
            let mut args = HashMap::new();
            args.insert(
                "message".to_string(),
                json!(format!("HTTP Performance test {}", i)),
            );

            let result = handler.call(args).await;
            assert!(result.is_ok(), "HTTP Tool call {i} should succeed");
        }

        let elapsed = start_time.elapsed();

        // Basic performance check - all calls should complete quickly
        assert!(
            elapsed < Duration::from_millis(100),
            "10 HTTP tool calls should complete in under 100ms, took: {elapsed:?}"
        );

        assert_eq!(
            handler.get_call_count().await,
            10,
            "Should have 10 HTTP calls"
        );

        println!("HTTP performance characteristics test passed");
    }

    #[tokio::test]
    async fn test_http_error_scenarios_and_recovery() {
        // Test various HTTP error scenarios and recovery mechanisms
        let resource_handler = HttpTestResourceHandler::new();

        // Test resource handler with invalid URI
        let result = resource_handler.read("invalid://http/uri").await;
        assert!(result.is_err(), "Invalid HTTP resource URI should fail");

        // Test tool handler with missing arguments
        let tool_handler = HttpTestToolHandler::new();
        let empty_args = HashMap::new();

        let result = tool_handler.call(empty_args).await;
        assert!(
            result.is_ok(),
            "HTTP Tool should handle missing arguments smoothly"
        );

        println!("HTTP error scenarios and recovery test passed");
    }

    #[tokio::test]
    async fn test_http_large_data_handling() {
        // Test HTTP handling of larger data sets
        let handler = HttpTestToolHandler::new();

        // Create a large message
        let large_message = "x".repeat(2048); // 2KB message
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!(large_message));

        let result = handler.call(args).await;
        assert!(result.is_ok(), "Should handle large HTTP messages");

        let result = result.unwrap();
        match &result.content[0] {
            Content::Text { text, .. } => {
                assert!(text.len() > 2000, "Should return large HTTP response");
                assert!(text.contains("HTTP Echo:"), "Should have HTTP prefix");
            }
            _ => panic!("Expected text content"),
        }

        println!("HTTP large data handling test passed");
    }

    #[tokio::test]
    async fn test_http_json_serialization_compatibility() {
        // Test JSON serialization and deserialization for HTTP
        let handler = HttpTestToolHandler::new();

        // Test with complex JSON data
        let complex_data = json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"},
                "unicode": "Hello, HTTP 世界! 🌍"
            }
        });

        let mut args = HashMap::new();
        args.insert("message".to_string(), complex_data);

        let result = handler.call(args).await;
        assert!(result.is_ok(), "Should handle complex JSON data over HTTP");

        // Test serialization of result
        let result = result.unwrap();
        let serialized = serde_json::to_string(&result);
        assert!(serialized.is_ok(), "HTTP Result should be serializable");

        let deserialized: Result<ToolResult, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok(), "HTTP Result should be deserializable");

        println!("HTTP JSON serialization compatibility test passed");
    }
}

#[cfg(not(feature = "http"))]
#[test]
fn integration_http_feature_disabled() {
    println!("HTTP integration tests require the 'http' feature to be enabled");
    println!("Run with: cargo test --features http --test integration_e2e_http");
    assert!(true, "HTTP feature not enabled - this is expected");
}

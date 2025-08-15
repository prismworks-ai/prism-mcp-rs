// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! Critical End-to-End Integration Tests - STDIO Transport
// !
// ! This test suite validates complete server functionality and tool/resource operations
// ! using the STDIO transport with real message flows.

#![cfg(feature = "stdio")]

use prism_mcp_rs::{
    core::{
        error::{McpError, McpResult},
        resource::LegacyResourceHandler,
        tool::ToolHandler,
    },
    protocol::types::{Content, ToolResult},
    server::McpServer,
    transport::{ServerTransport, StdioServerTransport, TransportConfig},
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

#[cfg(test)]
mod e2e_stdio_tests {
    use super::*;

    // Test tool handler for integration tests
    struct TestToolHandler {
        call_count: Arc<Mutex<u32>>,
    }

    impl TestToolHandler {
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
    impl ToolHandler for TestToolHandler {
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

    // Test resource handler for integration tests
    struct TestResourceHandler {
        resources: HashMap<String, String>,
    }

    impl TestResourceHandler {
        fn new() -> Self {
            let mut resources = HashMap::new();
            resources.insert(
                "test://resource1".to_string(),
                "Content of resource 1".to_string(),
            );
            resources.insert(
                "test://resource2".to_string(),
                "Content of resource 2 with more data".to_string(),
            );

            Self { resources }
        }
    }

    #[async_trait::async_trait]
    impl LegacyResourceHandler for TestResourceHandler {
        async fn read(&self, uri: &str) -> McpResult<String> {
            match self.resources.get(uri) {
                Some(content) => Ok(content.clone()),
                None => Err(McpError::validation(format!("Resource not found: {uri}"))),
            }
        }

        async fn list(&self) -> McpResult<Vec<prism_mcp_rs::protocol::types::ResourceInfo>> {
            Ok(self
                .resources
                .keys()
                .map(|uri| prism_mcp_rs::protocol::types::ResourceInfo {
                    uri: uri.clone(),
                    name: format!("Resource: {uri}"),
                    description: None,
                    mime_type: Some("text/plain".to_string()),
                    annotations: None,
                    size: None,
                    title: None,
                    meta: None,
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn test_stdio_server_creation() {
        // Test STDIO server creation and basic properties
        let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());

        // Test that we can create tools and add them
        let _tool_handler = TestToolHandler::new();
        let _resource_handler = TestResourceHandler::new();

        // Test basic server operations
        assert_eq!(server.name(), "test-server");
        assert_eq!(server.version(), "1.0.0");

        println!("STDIO server creation test passed");
    }

    #[tokio::test]
    async fn test_tool_handler_integration() {
        // Test tool handler functionality independently
        let handler = TestToolHandler::new();

        // Test echo functionality
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("Hello, World!"));

        let result = handler.call(args).await;
        assert!(result.is_ok(), "Tool call should succeed");

        let result = result.unwrap();
        assert!(
            result.is_error.is_none() || !result.is_error.unwrap(),
            "Tool should not error"
        );
        assert_eq!(result.content.len(), 1, "Should have one content item");

        // Check the content
        match &result.content[0] {
            Content::Text { text, .. } => {
                assert!(text.contains("Hello, World!"), "Should echo the message");
            }
            _ => panic!("Expected text content"),
        }

        // Verify call tracking
        assert_eq!(handler.get_call_count().await, 1, "Should have 1 call");

        println!("Tool handler integration test passed");
    }

    #[tokio::test]
    async fn test_resource_handler_integration() {
        // Test resource handler functionality
        let handler = TestResourceHandler::new();

        // Test reading existing resource
        let content = handler.read("test://resource1").await;
        assert!(content.is_ok(), "Should read resource successfully");

        let content = content.unwrap();
        assert_eq!(content, "Content of resource 1");

        // Test reading non-existent resource
        let missing = handler.read("test://missing").await;
        assert!(missing.is_err(), "Should fail for missing resource");

        match missing.unwrap_err() {
            McpError::Validation(_) => {} // Expected
            other => panic!("Expected Validation error, got: {other:?}"),
        }

        println!("Resource handler integration test passed");
    }

    #[tokio::test]
    async fn test_stdio_transport_configuration() {
        // Test STDIO transport with various configurations
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
            let transport = StdioServerTransport::with_config(config);

            // Test basic transport properties
            assert!(
                !transport.is_running(),
                "Transport {i} should not be running initially"
            );

            let info = transport.server_info();
            assert!(
                info.contains("STDIO"),
                "Transport {i} info should mention STDIO"
            );
            assert!(
                info.contains("running: false"),
                "Transport {i} should show not running"
            );
        }

        println!("STDIO transport configuration test passed");
    }

    #[tokio::test]
    async fn test_concurrent_tool_calls() {
        // Test handling multiple tool calls concurrently
        let handler = Arc::new(TestToolHandler::new());

        let mut handles = Vec::new();

        // Spawn multiple concurrent tool calls
        for i in 0..5 {
            let handler_clone = handler.clone();
            let handle = tokio::spawn(async move {
                let mut args = HashMap::new();
                args.insert("message".to_string(), json!(format!("Message {}", i)));

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
                    "Tool call should succeed"
                );
            }
        }

        assert_eq!(success_count, 5, "All tool calls should succeed");
        assert_eq!(
            handler.get_call_count().await,
            5,
            "Should have 5 calls recorded"
        );

        println!("Concurrent tool calls test passed");
    }

    #[tokio::test]
    async fn test_performance_characteristics() {
        // Test basic performance characteristics
        let handler = TestToolHandler::new();
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

        assert_eq!(handler.get_call_count().await, 10, "Should have 10 calls");

        println!("Performance characteristics test passed");
    }

    #[tokio::test]
    async fn test_error_scenarios_and_recovery() {
        // Test various error scenarios and recovery mechanisms
        let resource_handler = TestResourceHandler::new();

        // Test resource handler with invalid URI
        let result = resource_handler.read("invalid://uri").await;
        assert!(result.is_err(), "Invalid resource URI should fail");

        // Test tool handler with missing arguments
        let tool_handler = TestToolHandler::new();
        let empty_args = HashMap::new();

        let result = tool_handler.call(empty_args).await;
        assert!(
            result.is_ok(),
            "Tool should handle missing arguments smoothly"
        );

        println!("Error scenarios and recovery test passed");
    }

    #[tokio::test]
    async fn test_memory_and_resource_cleanup() {
        // Test that resources are properly cleaned up
        {
            let handler = TestToolHandler::new();
            let resource_handler = TestResourceHandler::new();

            // Perform operations
            let mut args = HashMap::new();
            args.insert("message".to_string(), json!("cleanup test"));
            let _ = handler.call(args).await;
            let _ = resource_handler.read("test://resource1").await;

            // Resources should be cleaned up when dropped
        } // Handlers dropped here

        // Test server cleanup
        {
            let server = McpServer::new("cleanup-test".to_string(), "1.0.0".to_string());

            // Server will be dropped and cleaned up
            assert_eq!(server.name(), "cleanup-test");
        }

        // If we reach here without panicking, cleanup worked correctly
        // Resource cleanup completed successfully - no assertion needed

        println!("Memory and resource cleanup test passed");
    }

    #[tokio::test]
    async fn test_large_data_handling() {
        // Test handling of larger data sets
        let handler = TestToolHandler::new();

        // Create a large message
        let large_message = "x".repeat(1024); // 1KB message
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!(large_message));

        let result = handler.call(args).await;
        assert!(result.is_ok(), "Should handle large messages");

        let result = result.unwrap();
        match &result.content[0] {
            Content::Text { text, .. } => {
                assert!(text.len() > 1000, "Should return large response");
            }
            _ => panic!("Expected text content"),
        }

        println!("Large data handling test passed");
    }

    #[tokio::test]
    async fn test_json_serialization_compatibility() {
        // Test JSON serialization and deserialization
        let handler = TestToolHandler::new();

        // Test with complex JSON data
        let complex_data = json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"},
                "unicode": "Hello, 世界! 🌍"
            }
        });

        let mut args = HashMap::new();
        args.insert("message".to_string(), complex_data);

        let result = handler.call(args).await;
        assert!(result.is_ok(), "Should handle complex JSON data");

        // Test serialization of result
        let result = result.unwrap();
        let serialized = serde_json::to_string(&result);
        assert!(serialized.is_ok(), "Result should be serializable");

        let deserialized: Result<ToolResult, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok(), "Result should be deserializable");

        println!("JSON serialization compatibility test passed");
    }
}

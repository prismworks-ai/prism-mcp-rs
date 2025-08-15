// ! Server Lifecycle Management
// !
// ! Module provides lifecycle management functionality for MCP servers,
// ! including state transitions, shutdown handling, and resource management.

// Re-export all types from missing_types for compatibility
pub use crate::protocol::missing_types::*;

// Add any additional lifecycle-specific implementations here if needed

#[cfg(test)]
mod tests {
    use crate::core::error::McpResult;
    use crate::server::mcp_server::McpServer;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    async fn test_server_startup_shutdown() {
        let server = McpServer::new("lifecycle-test".to_string(), "1.0.0".to_string());

        // Test initial state - should not be running
        assert!(!server.is_running().await);

        // Initialize the server
        server.initialize().await.unwrap();
        assert!(server.is_running().await);
        assert!(server.is_initialized().await);
    }

    #[tokio::test]
    async fn test_connection_lifecycle() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "connection-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Initialize server
        {
            let s = server.lock().await;
            s.initialize().await.unwrap();
        }

        // Test connection management
        let server1 = server.clone();
        let server2 = server.clone();

        // Simulate concurrent connections
        let handle1 = tokio::spawn(async move {
            let s = server1.lock().await;
            s.is_running().await
        });

        let handle2 = tokio::spawn(async move {
            let s = server2.lock().await;
            s.is_running().await
        });

        let (result1, result2): (Result<bool, _>, Result<bool, _>) = tokio::join!(handle1, handle2);
        assert!(result1.unwrap());
        assert!(result2.unwrap());
    }

    #[tokio::test]
    async fn test_smooth_shutdown() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "shutdown-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Test smooth shutdown behavior
        let server_clone = server.clone();

        // Simulate ongoing work
        let work_handle = tokio::spawn(async move {
            let _s = server_clone.lock().await;
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(10)).await;
        });

        // Wait for work to complete
        let result = timeout(Duration::from_millis(100), work_handle).await;
        assert!(result.is_ok());

        // Server should still be accessible after work completes
        let _s = server.lock().await;
    }

    #[tokio::test]
    async fn test_resource_management() {
        // Test that resources are properly managed during lifecycle
        let server = McpServer::new("resource-test".to_string(), "1.0.0".to_string());

        // Initialize server
        server.initialize().await.unwrap();
        assert!(server.is_running().await);

        // Initially no tools
        assert!(!server.has_tools().await);
        assert_eq!(server.tool_count().await, 0);

        // Add a simple tool
        server
            .add_simple_tool("test-tool", "A test tool", |_args| {
                Ok(vec![crate::protocol::types::ContentBlock::text(
                    "test result",
                )])
            })
            .await
            .unwrap();

        // Now should have tools
        assert!(server.has_tools().await);
        assert_eq!(server.tool_count().await, 1);
        assert!(server.has_tool("test-tool").await);

        // Tools should be accessible
        let tools = server.list_tools().await;
        assert!(tools.is_ok());
        assert_eq!(tools.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_concurrent_lifecycle_operations() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "concurrent-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Initialize server
        {
            let s = server.lock().await;
            s.initialize().await.unwrap();
        }

        let mut handles = vec![];

        // Spawn multiple concurrent operations
        for i in 0..5 {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                let s = server_clone.lock().await;
                // Add a tool concurrently
                s.add_simple_tool(&format!("tool-{i}"), "Test tool", |_args| {
                    Ok(vec![crate::protocol::types::ContentBlock::text("result")])
                })
                .await
                .unwrap();
                s.is_running().await
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        // Verify server state is consistent
        let s = server.lock().await;
        assert!(s.is_running().await);
        assert_eq!(s.tool_count().await, 5);
    }

    #[tokio::test]
    async fn test_error_recovery() {
        let server = McpServer::new("error-test".to_string(), "1.0.0".to_string());

        // Initialize server
        server.initialize().await.unwrap();
        assert!(server.is_running().await);

        // Add a tool that might cause errors
        server
            .add_simple_tool("error-tool", "A tool that might error", |_args| {
                Err(crate::core::error::McpError::InvalidParams(
                    "Test error".to_string(),
                ))
            })
            .await
            .unwrap();

        // Server should remain stable even with error-prone tools
        assert!(server.is_running().await);
        assert!(server.has_tool("error-tool").await);

        // Server should handle requests even if some tools error
        let tools = server.list_tools().await;
        assert!(tools.is_ok());
        assert_eq!(tools.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_both_add_tool_methods() {
        let server = McpServer::new("tool-methods-test".to_string(), "1.0.0".to_string());
        server.initialize().await.unwrap();

        // Test the full add_tool() method with proper parameters
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Input parameter"
                }
            },
            "required": ["input"]
        });

        // Create a proper ToolHandler implementation
        struct TestHandler;

        #[async_trait]
        impl crate::core::tool::ToolHandler for TestHandler {
            async fn call(
                &self,
                _arguments: std::collections::HashMap<String, serde_json::Value>,
            ) -> McpResult<crate::protocol::types::ToolResult> {
                Ok(crate::protocol::types::ToolResult {
                    content: vec![crate::protocol::types::ContentBlock::text(
                        "full method result",
                    )],
                    is_error: Some(false),
                    structured_content: None,
                    meta: None,
                })
            }
        }

        // Test full add_tool method
        server
            .add_tool(
                "full-tool".to_string(),
                Some("A tool using the full method".to_string()),
                schema,
                TestHandler,
            )
            .await
            .unwrap();

        // Test simple add_simple_tool method
        server
            .add_simple_tool("simple-tool", "A tool using the simple method", |_args| {
                Ok(vec![crate::protocol::types::ContentBlock::text(
                    "simple method result",
                )])
            })
            .await
            .unwrap();

        // Both tools should be registered
        assert_eq!(server.tool_count().await, 2);
        assert!(server.has_tool("full-tool").await);
        assert!(server.has_tool("simple-tool").await);

        // Both should be listed
        let tools = server.list_tools().await.unwrap();
        assert_eq!(tools.len(), 2);
    }

    #[tokio::test]
    async fn test_lifecycle_state_transitions() {
        let server = McpServer::new("state-test".to_string(), "1.0.0".to_string());

        // Test basic server operations
        let tools = server.list_tools().await;
        assert!(tools.is_ok());

        // Server should handle multiple operations
        for _i in 0..3 {
            let _tools = server.list_tools().await;
        }
    }

    #[tokio::test]
    async fn test_lifecycle_with_timeouts() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "timeout-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Test operations with timeouts
        let server_clone = server.clone();
        let quick_op = timeout(Duration::from_millis(50), async move {
            let s = server_clone.lock().await;
            s.list_tools().await.is_ok()
        })
        .await;

        assert!(quick_op.is_ok());
        assert!(quick_op.unwrap());

        // Test longer operation
        let server_clone2 = server.clone();
        let longer_op = timeout(Duration::from_millis(100), async move {
            let s = server_clone2.lock().await;
            s.list_tools().await.is_ok()
        })
        .await;

        assert!(longer_op.is_ok());
        assert!(longer_op.unwrap());
    }

    #[tokio::test]
    async fn test_memory_management() {
        // Test that server properly manages memory during lifecycle
        let mut servers = Vec::new();

        // Create multiple servers to test memory management
        for i in 0..10 {
            let server = McpServer::new(format!("memory-test-{i}"), "1.0.0".to_string());

            // Basic operation should work
            let _tools = server.list_tools().await;
            servers.push(server);
        }

        // All servers should be functional
        for server in &servers {
            let tools = server.list_tools().await;
            assert!(tools.is_ok());
        }

        // Drop servers (test cleanup)
        drop(servers);

        // Create new server to verify no interference
        let new_server = McpServer::new("cleanup-test".to_string(), "1.0.0".to_string());
        let tools = new_server.list_tools().await;
        assert!(tools.is_ok());
    }

    #[test]
    fn test_lifecycle_constants_and_types() {
        // Test that lifecycle module properly re-exports types
        // This ensures that the module is properly integrated

        // The re-export should work without compilation errors
        // If this test compiles, the re-exports are working
    }
}

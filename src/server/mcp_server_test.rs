// ! complete tests for MCP Server

#[cfg(test)]
mod tests {
    use super::super::mcp_server::*;
    use crate::core::error::{McpError, McpResult};
    use crate::core::prompt::{Prompt, PromptHandler};
    use crate::core::resource::{Resource, ResourceHandler};
    use crate::core::tool::{Tool, ToolHandler};
    use crate::protocol::types::*;
    use crate::transport::traits::*;
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::time::Duration;

    // Mock Tool Handler
    struct TestToolHandler {
        name: String,
        calls: Arc<RwLock<Vec<Value>>>,
    }

    #[async_trait]
    impl ToolHandler for TestToolHandler {
        async fn call(&self, arguments: Value) -> McpResult<CallToolResult> {
            let mut calls = self.calls.write().await;
            calls.push(arguments.clone());
            
            Ok(CallToolResult {
                content: vec![ContentBlock::text(format!("Tool {} called", self.name))],
                is_error: None,
                meta: None,
            })
        }
    }

    // Mock Resource Handler
    struct TestResourceHandler {
        resources: Vec<Resource>,
        reads: Arc<RwLock<Vec<String>>>,
    }

    #[async_trait]
    impl ResourceHandler for TestResourceHandler {
        async fn read(&self, uri: &str) -> McpResult<ResourceContents> {
            let mut reads = self.reads.write().await;
            reads.push(uri.to_string());
            
            Ok(ResourceContents::Text {
                text: format!("Content of {}", uri),
                meta: None,
            })
        }

        async fn list(&self) -> McpResult<Vec<Resource>> {
            Ok(self.resources.clone())
        }
    }

    // Mock Prompt Handler
    struct TestPromptHandler {
        prompts: Arc<RwLock<Vec<String>>>,
    }

    #[async_trait]
    impl PromptHandler for TestPromptHandler {
        async fn get_prompt(&self, arguments: Option<Value>) -> McpResult<GetPromptResult> {
            let mut prompts = self.prompts.write().await;
            let name = arguments
                .and_then(|a| a.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("default");
            prompts.push(name.to_string());
            
            Ok(GetPromptResult {
                description: Some(format!("Prompt: {}", name)),
                messages: vec![
                    PromptMessage {
                        role: Role::User,
                        content: ContentBlock::text(format!("Hello from {}", name)),
                    },
                ],
                meta: None,
            })
        }
    }

    // Test Server Creation
    #[tokio::test]
    async fn test_server_creation() {
        let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        assert_eq!(server.info().name, "test-server");
        assert_eq!(server.info().version, "1.0.0");
    }

    #[tokio::test]
    async fn test_server_with_capabilities() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        
        server.set_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: Some(true) }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            prompts: Some(PromptsCapability { list_changed: Some(true) }),
            logging: None,
            experimental: None,
        }).await;
        
        let caps = server.capabilities();
        assert!(caps.tools.is_some());
        assert!(caps.resources.is_some());
        assert!(caps.prompts.is_some());
    }

    // Test Tool Management
    #[tokio::test]
    async fn test_add_tool() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        let calls = Arc::new(RwLock::new(Vec::new()));
        
        let handler = TestToolHandler {
            name: "calculator".to_string(),
            calls: calls.clone(),
        };
        
        let tool = Tool::new(
            "calculator".to_string(),
            Some("A calculator tool".to_string()),
            json!({}),
            Box::new(handler),
        );
        
        server.add_tool(tool).await.unwrap();
        
        let tools = server.list_tools().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "calculator");
    }

    #[tokio::test]
    async fn test_add_multiple_tools() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        
        for i in 0..5 {
            let handler = TestToolHandler {
                name: format!("tool_{}", i),
                calls: Arc::new(RwLock::new(Vec::new())),
            };
            
            let tool = Tool::new(
                format!("tool_{}", i),
                Some(format!("Tool {}", i)),
                json!({}),
                Box::new(handler),
            );
            
            server.add_tool(tool).await.unwrap();
        }
        
        let tools = server.list_tools().await;
        assert_eq!(tools.len(), 5);
    }

    #[tokio::test]
    async fn test_remove_tool() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        
        let handler = TestToolHandler {
            name: "temp_tool".to_string(),
            calls: Arc::new(RwLock::new(Vec::new())),
        };
        
        let tool = Tool::new(
            "temp_tool".to_string(),
            Some("Temporary tool".to_string()),
            json!({}),
            Box::new(handler),
        );
        
        server.add_tool(tool).await.unwrap();
        assert_eq!(server.list_tools().await.len(), 1);
        
        server.remove_tool("temp_tool").await.unwrap();
        assert_eq!(server.list_tools().await.len(), 0);
    }

    #[tokio::test]
    async fn test_call_tool() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        let calls = Arc::new(RwLock::new(Vec::new()));
        
        let handler = TestToolHandler {
            name: "echo".to_string(),
            calls: calls.clone(),
        };
        
        let tool = Tool::new(
            "echo".to_string(),
            Some("Echo tool".to_string()),
            json!({}),
            Box::new(handler),
        );
        
        server.add_tool(tool).await.unwrap();
        
        let result = server.call_tool("echo", Some(json!({"message": "test"}))).await;
        assert!(result.is_ok());
        
        let calls_made = calls.read().await;
        assert_eq!(calls_made.len(), 1);
        assert_eq!(calls_made[0]["message"], "test");
    }

    #[tokio::test]
    async fn test_call_nonexistent_tool() {
        let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        let result = server.call_tool("nonexistent", None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), McpError::ToolNotFound(_)));
    }

    // Test Resource Management
    #[tokio::test]
    async fn test_add_resource() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        let reads = Arc::new(RwLock::new(Vec::new()));
        
        let handler = TestResourceHandler {
            resources: vec![
                Resource {
                    uri: "file:///test.txt".to_string(),
                    name: Some("test.txt".to_string()),
                    description: Some("Test file".to_string()),
                    mime_type: Some("text/plain".to_string()),
                    meta: None,
                },
            ],
            reads: reads.clone(),
        };
        
        server.add_resource_handler(Box::new(handler)).await.unwrap();
        
        let resources = server.list_resources().await.unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "file:///test.txt");
    }

    #[tokio::test]
    async fn test_read_resource() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        let reads = Arc::new(RwLock::new(Vec::new()));
        
        let handler = TestResourceHandler {
            resources: vec![],
            reads: reads.clone(),
        };
        
        server.add_resource_handler(Box::new(handler)).await.unwrap();
        
        let result = server.read_resource("file:///test.txt").await;
        assert!(result.is_ok());
        
        let reads_made = reads.read().await;
        assert_eq!(reads_made.len(), 1);
        assert_eq!(reads_made[0], "file:///test.txt");
    }

    // Test Prompt Management
    #[tokio::test]
    async fn test_add_prompt() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        let prompts_called = Arc::new(RwLock::new(Vec::new()));
        
        let handler = TestPromptHandler {
            prompts: prompts_called.clone(),
        };
        
        let prompt = Prompt::new(
            "greeting".to_string(),
            Some("Greeting prompt".to_string()),
            Some(vec![]),
            Box::new(handler),
        );
        
        server.add_prompt(prompt).await.unwrap();
        
        let prompts = server.list_prompts().await;
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "greeting");
    }

    #[tokio::test]
    async fn test_get_prompt() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        let prompts_called = Arc::new(RwLock::new(Vec::new()));
        
        let handler = TestPromptHandler {
            prompts: prompts_called.clone(),
        };
        
        let prompt = Prompt::new(
            "test_prompt".to_string(),
            Some("Test prompt".to_string()),
            None,
            Box::new(handler),
        );
        
        server.add_prompt(prompt).await.unwrap();
        
        let result = server.get_prompt("test_prompt", Some(json!({"name": "Alice"}))).await;
        assert!(result.is_ok());
        
        let prompts_made = prompts_called.read().await;
        assert_eq!(prompts_made.len(), 1);
        assert_eq!(prompts_made[0], "Alice");
    }

    // Test Server State
    #[tokio::test]
    async fn test_server_state_management() {
        let server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        
        // Initial state
        assert!(!server.is_initialized().await);
        
        // Initialize
        server.set_initialized(true).await;
        assert!(server.is_initialized().await);
        
        // Reset
        server.set_initialized(false).await;
        assert!(!server.is_initialized().await);
    }

    // Test Concurrent Operations
    #[tokio::test]
    async fn test_concurrent_tool_calls() {
        let server = Arc::new(McpServer::new("test-server".to_string(), "1.0.0".to_string()));
        let calls = Arc::new(RwLock::new(Vec::new()));
        
        let handler = TestToolHandler {
            name: "concurrent".to_string(),
            calls: calls.clone(),
        };
        
        let tool = Tool::new(
            "concurrent".to_string(),
            Some("Concurrent tool".to_string()),
            json!({}),
            Box::new(handler),
        );
        
        server.add_tool(tool).await.unwrap();
        
        let mut handles = vec![];
        
        for i in 0..10 {
            let srv = Arc::clone(&server);
            let handle = tokio::spawn(async move {
                srv.call_tool("concurrent", Some(json!({"id": i}))).await
            });
            handles.push(handle);
        }
        
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
        
        let calls_made = calls.read().await;
        assert_eq!(calls_made.len(), 10);
    }

    #[tokio::test]
    async fn test_concurrent_resource_operations() {
        let server = Arc::new(McpServer::new("test-server".to_string(), "1.0.0".to_string()));
        let reads = Arc::new(RwLock::new(Vec::new()));
        
        let handler = TestResourceHandler {
            resources: vec![],
            reads: reads.clone(),
        };
        
        server.add_resource_handler(Box::new(handler)).await.unwrap();
        
        let mut handles = vec![];
        
        for i in 0..10 {
            let srv = Arc::clone(&server);
            let handle = tokio::spawn(async move {
                srv.read_resource(&format!("file:///test{}.txt", i)).await
            });
            handles.push(handle);
        }
        
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
        
        let reads_made = reads.read().await;
        assert_eq!(reads_made.len(), 10);
    }

    // Test Error Scenarios
    #[tokio::test]
    async fn test_duplicate_tool_name() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        
        let handler1 = TestToolHandler {
            name: "duplicate".to_string(),
            calls: Arc::new(RwLock::new(Vec::new())),
        };
        
        let tool1 = Tool::new(
            "duplicate".to_string(),
            Some("First tool".to_string()),
            json!({}),
            Box::new(handler1),
        );
        
        server.add_tool(tool1).await.unwrap();
        
        let handler2 = TestToolHandler {
            name: "duplicate".to_string(),
            calls: Arc::new(RwLock::new(Vec::new())),
        };
        
        let tool2 = Tool::new(
            "duplicate".to_string(),
            Some("Second tool".to_string()),
            json!({}),
            Box::new(handler2),
        );
        
        let result = server.add_tool(tool2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_tool() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        let result = server.remove_tool("nonexistent").await;
        assert!(result.is_err());
    }

    // Test Server Info
    #[tokio::test]
    async fn test_server_info() {
        let server = McpServer::new("info-server".to_string(), "2.0.0".to_string());
        let info = server.info();
        
        assert_eq!(info.name, "info-server");
        assert_eq!(info.version, "2.0.0");
    }

    // Test Statistics
    #[tokio::test]
    async fn test_server_stats() {
        let mut server = McpServer::new("test-server".to_string(), "1.0.0".to_string());
        
        // Add some tools
        for i in 0..3 {
            let handler = TestToolHandler {
                name: format!("tool_{}", i),
                calls: Arc::new(RwLock::new(Vec::new())),
            };
            
            let tool = Tool::new(
                format!("tool_{}", i),
                None,
                json!({}),
                Box::new(handler),
            );
            
            server.add_tool(tool).await.unwrap();
        }
        
        // Add some prompts
        for i in 0..2 {
            let handler = TestPromptHandler {
                prompts: Arc::new(RwLock::new(Vec::new())),
            };
            
            let prompt = Prompt::new(
                format!("prompt_{}", i),
                None,
                None,
                Box::new(handler),
            );
            
            server.add_prompt(prompt).await.unwrap();
        }
        
        assert_eq!(server.list_tools().await.len(), 3);
        assert_eq!(server.list_prompts().await.len(), 2);
    }
}
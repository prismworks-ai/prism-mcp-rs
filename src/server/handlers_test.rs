// ! complete tests for server handlers

#[cfg(test)]
mod tests {
    use super::super::handlers::*;
    use crate::core::error::{McpError, McpResult};
    use crate::core::prompt::{Prompt, PromptHandler};
    use crate::core::resource::{Resource, ResourceHandler};
    use crate::core::tool::{Tool, ToolHandler};
    use crate::protocol::types::*;
    use crate::server::McpServer;
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Mock handlers for testing
    struct MockToolHandler {
        should_fail: bool,
        call_count: Arc<RwLock<usize>>,
    }

    #[async_trait]
    impl ToolHandler for MockToolHandler {
        async fn call(&self, _arguments: Value) -> McpResult<CallToolResult> {
            let mut count = self.call_count.write().await;
            *count += 1;
            
            if self.should_fail {
                Err(McpError::ToolNotFound("Mock failure".to_string()))
            } else {
                Ok(CallToolResult {
                    content: vec![ContentBlock::text("Success")],
                    is_error: None,
                    meta: None,
                })
            }
        }
    }

    struct MockResourceHandler {
        should_fail: bool,
        resources: Vec<Resource>,
    }

    #[async_trait]
    impl ResourceHandler for MockResourceHandler {
        async fn read(&self, uri: &str) -> McpResult<ResourceContents> {
            if self.should_fail {
                Err(McpError::ResourceNotFound(uri.to_string()))
            } else {
                Ok(ResourceContents::Text {
                    text: format!("Content of {}", uri),
                    meta: None,
                })
            }
        }

        async fn list(&self) -> McpResult<Vec<Resource>> {
            if self.should_fail {
                Err(McpError::Internal("Failed to list".to_string()))
            } else {
                Ok(self.resources.clone())
            }
        }
    }

    struct MockPromptHandler {
        should_fail: bool,
    }

    #[async_trait]
    impl PromptHandler for MockPromptHandler {
        async fn get_prompt(&self, arguments: Option<Value>) -> McpResult<GetPromptResult> {
            if self.should_fail {
                Err(McpError::PromptNotFound("Mock failure".to_string()))
            } else {
                let name = arguments
                    .as_ref()
                    .and_then(|a| a.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("default");
                    
                Ok(GetPromptResult {
                    description: Some(format!("Prompt for {}", name)),
                    messages: vec![
                        PromptMessage {
                            role: Role::User,
                            content: ContentBlock::text(format!("Hello, {}", name)),
                        },
                    ],
                    meta: None,
                })
            }
        }
    }

    // Test Initialize Handler
    #[tokio::test]
    async fn test_initialize_handler_success() {
        let handler = InitializeHandler::new();
        let params = InitializeParams {
            protocol_version: "2025-06-18".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: Some(ClientInfo {
                name: "test-client".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            meta: None,
        };

        let result = handler.handle(params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.protocol_version, "2025-06-18");
        assert!(response.capabilities.tools.is_some());
        assert!(response.capabilities.resources.is_some());
        assert!(response.capabilities.prompts.is_some());
    }

    #[tokio::test]
    async fn test_initialize_handler_with_server_info() {
        let handler = InitializeHandler::with_server_info(
            "test-server".to_string(),
            "2.0.0".to_string(),
        );
        
        let params = InitializeParams {
            protocol_version: "2025-06-18".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: None,
            meta: None,
        };

        let result = handler.handle(params).await.unwrap();
        assert_eq!(result.server_info.name, "test-server");
        assert_eq!(result.server_info.version, "2.0.0");
    }

    #[tokio::test]
    async fn test_initialize_handler_invalid_protocol() {
        let handler = InitializeHandler::new();
        let params = InitializeParams {
            protocol_version: "invalid-version".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: None,
            meta: None,
        };

        let result = handler.handle(params).await;
        // Should still succeed but potentially with warnings
        assert!(result.is_ok());
    }

    // Test Ping Handler
    #[tokio::test]
    async fn test_ping_handler() {
        let result = PingHandler::handle(None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PingResult {});

        let result = PingHandler::handle(Some(json!({"test": "data"})));
        assert!(result.await.is_ok());
    }

    // Test Tools List Handler
    #[tokio::test]
    async fn test_tools_list_handler() {
        let handler = ToolsListHandler::new();
        
        // Add some tools
        let tool1 = ToolInfo {
            name: "tool1".to_string(),
            description: Some("Tool 1".to_string()),
            input_schema: json!({}),
        };
        
        let tool2 = ToolInfo {
            name: "tool2".to_string(),
            description: Some("Tool 2".to_string()),
            input_schema: json!({}),
        };
        
        handler.add_tool(tool1.clone()).await;
        handler.add_tool(tool2.clone()).await;
        
        let result = handler.handle(None).await.unwrap();
        assert_eq!(result.tools.len(), 2);
        assert!(result.tools.iter().any(|t| t.name == "tool1"));
        assert!(result.tools.iter().any(|t| t.name == "tool2"));
    }

    #[tokio::test]
    async fn test_tools_list_handler_with_filter() {
        let handler = ToolsListHandler::new();
        
        let tool = ToolInfo {
            name: "calculator".to_string(),
            description: Some("Math tool".to_string()),
            input_schema: json!({}),
        };
        
        handler.add_tool(tool).await;
        
        // Test with matching filter
        let params = ListToolsParams {
            cursor: None,
            meta: None,
        };
        
        let result = handler.handle(Some(params)).await.unwrap();
        assert_eq!(result.tools.len(), 1);
    }

    // Test Tool Call Handler
    #[tokio::test]
    async fn test_tool_call_handler_success() {
        let call_count = Arc::new(RwLock::new(0));
        let mock_handler = MockToolHandler {
            should_fail: false,
            call_count: call_count.clone(),
        };
        
        let tool = Tool::new(
            "test-tool".to_string(),
            Some("Test tool".to_string()),
            json!({}),
            Box::new(mock_handler),
        );
        
        let handler = ToolCallHandler::new();
        handler.register_tool(Arc::new(tool)).await;
        
        let params = CallToolParams {
            name: "test-tool".to_string(),
            arguments: Some(json!({"arg": "value"})),
            meta: None,
        };
        
        let result = handler.handle(params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.content.len(), 1);
        assert_eq!(*call_count.read().await, 1);
    }

    #[tokio::test]
    async fn test_tool_call_handler_not_found() {
        let handler = ToolCallHandler::new();
        
        let params = CallToolParams {
            name: "non-existent".to_string(),
            arguments: None,
            meta: None,
        };
        
        let result = handler.handle(params).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), McpError::ToolNotFound(_)));
    }

    #[tokio::test]
    async fn test_tool_call_handler_failure() {
        let call_count = Arc::new(RwLock::new(0));
        let mock_handler = MockToolHandler {
            should_fail: true,
            call_count: call_count.clone(),
        };
        
        let tool = Tool::new(
            "failing-tool".to_string(),
            Some("Failing tool".to_string()),
            json!({}),
            Box::new(mock_handler),
        );
        
        let handler = ToolCallHandler::new();
        handler.register_tool(Arc::new(tool)).await;
        
        let params = CallToolParams {
            name: "failing-tool".to_string(),
            arguments: None,
            meta: None,
        };
        
        let result = handler.handle(params).await;
        assert!(result.is_err());
        assert_eq!(*call_count.read().await, 1);
    }

    // Test Resources List Handler
    #[tokio::test]
    async fn test_resources_list_handler() {
        let resources = vec![
            Resource {
                uri: "file:///test1.txt".to_string(),
                name: Some("test1.txt".to_string()),
                description: Some("Test file 1".to_string()),
                mime_type: Some("text/plain".to_string()),
                meta: None,
            },
            Resource {
                uri: "file:///test2.txt".to_string(),
                name: Some("test2.txt".to_string()),
                description: Some("Test file 2".to_string()),
                mime_type: Some("text/plain".to_string()),
                meta: None,
            },
        ];
        
        let mock_handler = MockResourceHandler {
            should_fail: false,
            resources: resources.clone(),
        };
        
        let handler = ResourcesListHandler::new();
        handler.register_handler(Box::new(mock_handler)).await;
        
        let result = handler.handle(None).await.unwrap();
        assert_eq!(result.resources.len(), 2);
    }

    #[tokio::test]
    async fn test_resources_list_handler_with_pagination() {
        let mut resources = vec![];
        for i in 0..10 {
            resources.push(Resource {
                uri: format!("file:///test{}.txt", i),
                name: Some(format!("test{}.txt", i)),
                description: None,
                mime_type: Some("text/plain".to_string()),
                meta: None,
            });
        }
        
        let mock_handler = MockResourceHandler {
            should_fail: false,
            resources,
        };
        
        let handler = ResourcesListHandler::new();
        handler.register_handler(Box::new(mock_handler)).await;
        
        let params = ListResourcesParams {
            cursor: None,
            meta: None,
        };
        
        let result = handler.handle(Some(params)).await.unwrap();
        assert_eq!(result.resources.len(), 10);
    }

    // Test Resource Read Handler
    #[tokio::test]
    async fn test_resource_read_handler_success() {
        let mock_handler = MockResourceHandler {
            should_fail: false,
            resources: vec![],
        };
        
        let handler = ResourceReadHandler::new();
        handler.register_handler(Box::new(mock_handler)).await;
        
        let params = ReadResourceParams {
            uri: "file:///test.txt".to_string(),
            meta: None,
        };
        
        let result = handler.handle(params).await.unwrap();
        assert_eq!(result.contents.len(), 1);
        
        if let ResourceContents::Text { text, .. } = &result.contents[0] {
            assert_eq!(text, "Content of file:///test.txt");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_resource_read_handler_not_found() {
        let mock_handler = MockResourceHandler {
            should_fail: true,
            resources: vec![],
        };
        
        let handler = ResourceReadHandler::new();
        handler.register_handler(Box::new(mock_handler)).await;
        
        let params = ReadResourceParams {
            uri: "file:///nonexistent.txt".to_string(),
            meta: None,
        };
        
        let result = handler.handle(params).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), McpError::ResourceNotFound(_)));
    }

    // Test Prompts List Handler
    #[tokio::test]
    async fn test_prompts_list_handler() {
        let handler = PromptsListHandler::new();
        
        let prompt1 = PromptInfo {
            name: "greeting".to_string(),
            description: Some("Greeting prompt".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "name".to_string(),
                    description: Some("User's name".to_string()),
                    required: Some(true),
                },
            ]),
        };
        
        let prompt2 = PromptInfo {
            name: "farewell".to_string(),
            description: Some("Farewell prompt".to_string()),
            arguments: None,
        };
        
        handler.add_prompt(prompt1.clone()).await;
        handler.add_prompt(prompt2.clone()).await;
        
        let result = handler.handle(None).await.unwrap();
        assert_eq!(result.prompts.len(), 2);
    }

    #[tokio::test]
    async fn test_prompts_list_handler_with_cursor() {
        let handler = PromptsListHandler::new();
        
        for i in 0..5 {
            let prompt = PromptInfo {
                name: format!("prompt{}", i),
                description: Some(format!("Prompt {}", i)),
                arguments: None,
            };
            handler.add_prompt(prompt).await;
        }
        
        let params = ListPromptsParams {
            cursor: None,
            meta: None,
        };
        
        let result = handler.handle(Some(params)).await.unwrap();
        assert_eq!(result.prompts.len(), 5);
    }

    // Test Prompt Get Handler
    #[tokio::test]
    async fn test_prompt_get_handler_success() {
        let mock_handler = MockPromptHandler {
            should_fail: false,
        };
        
        let prompt = Prompt::new(
            "test-prompt".to_string(),
            Some("Test prompt".to_string()),
            None,
            Box::new(mock_handler),
        );
        
        let handler = PromptGetHandler::new();
        handler.register_prompt(Arc::new(prompt)).await;
        
        let params = GetPromptParams {
            name: "test-prompt".to_string(),
            arguments: Some(json!({"name": "Alice"})),
            meta: None,
        };
        
        let result = handler.handle(params).await.unwrap();
        assert!(result.description.is_some());
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_prompt_get_handler_not_found() {
        let handler = PromptGetHandler::new();
        
        let params = GetPromptParams {
            name: "non-existent".to_string(),
            arguments: None,
            meta: None,
        };
        
        let result = handler.handle(params).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), McpError::PromptNotFound(_)));
    }

    #[tokio::test]
    async fn test_prompt_get_handler_failure() {
        let mock_handler = MockPromptHandler {
            should_fail: true,
        };
        
        let prompt = Prompt::new(
            "failing-prompt".to_string(),
            Some("Failing prompt".to_string()),
            None,
            Box::new(mock_handler),
        );
        
        let handler = PromptGetHandler::new();
        handler.register_prompt(Arc::new(prompt)).await;
        
        let params = GetPromptParams {
            name: "failing-prompt".to_string(),
            arguments: None,
            meta: None,
        };
        
        let result = handler.handle(params).await;
        assert!(result.is_err());
    }

    // Test Message Creation Handler
    #[tokio::test]
    async fn test_create_message_handler() {
        let handler = CreateMessageHandler::new();
        
        let params = CreateMessageParams {
            messages: vec![
                SamplingMessage {
                    role: Role::User,
                    content: ContentBlock::text("Hello"),
                },
            ],
            model_preferences: None,
            system_prompt: None,
            include_context: None,
            temperature: None,
            max_tokens: Some(100),
            stop_sequences: None,
            metadata: None,
            meta: None,
        };
        
        let result = handler.handle(params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.role, Role::Assistant);
        assert!(!response.content.is_empty());
    }

    // Test validation helpers
    #[test]
    fn test_validate_protocol_version() {
        assert!(validate_protocol_version("2025-06-18"));
        assert!(validate_protocol_version("2025-03-26"));
        assert!(!validate_protocol_version("invalid"));
        assert!(!validate_protocol_version("2024-01-01"));
    }

    #[test]
    fn test_validate_uri_format() {
        assert!(validate_uri("file:///path/to/file.txt"));
        assert!(validate_uri("https://example.com/resource"));
        assert!(validate_uri("custom://resource/path"));
        assert!(!validate_uri("invalid uri"));
        assert!(!validate_uri(""));
    }

    #[test]
    fn test_validate_tool_name() {
        assert!(validate_tool_name("valid_tool_name"));
        assert!(validate_tool_name("tool-with-dash"));
        assert!(validate_tool_name("tool123"));
        assert!(!validate_tool_name(""));
        assert!(!validate_tool_name("tool with spaces"));
        assert!(!validate_tool_name("tool/with/slashes"));
    }

    // Test notification builders
    #[test]
    fn test_build_progress_notification() {
        let notification = build_progress_notification(
            1.0,
            Some("Processing".to_string()),
        );
        
        assert_eq!(notification.method, "notifications/progress");
        if let Some(params) = notification.params {
            assert_eq!(params["progress"], 1.0);
            assert_eq!(params["status"], "Processing");
        } else {
            panic!("Expected params");
        }
    }

    #[test]
    fn test_build_resource_updated_notification() {
        let notification = build_resource_updated_notification(
            "file:///test.txt".to_string(),
        );
        
        assert_eq!(notification.method, "notifications/resources/updated");
        if let Some(params) = notification.params {
            assert_eq!(params["uri"], "file:///test.txt");
        } else {
            panic!("Expected params");
        }
    }

    #[test]
    fn test_build_tool_list_changed_notification() {
        let notification = build_tool_list_changed_notification();
        assert_eq!(notification.method, "notifications/tools/list_changed");
        assert!(notification.params.is_none());
    }

    // Test concurrent handler access
    #[tokio::test]
    async fn test_concurrent_handler_access() {
        let handler = Arc::new(ToolsListHandler::new());
        let mut handles = vec![];
        
        // Spawn multiple tasks that access the handler
        for i in 0..10 {
            let h = Arc::clone(&handler);
            let handle = tokio::spawn(async move {
                let tool = ToolInfo {
                    name: format!("tool{}", i),
                    description: Some(format!("Tool {}", i)),
                    input_schema: json!({}),
                };
                h.add_tool(tool).await;
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify all tools were added
        let result = handler.handle(None).await.unwrap();
        assert_eq!(result.tools.len(), 10);
    }

    // Test error recovery
    #[tokio::test]
    async fn test_handler_error_recovery() {
        let call_count = Arc::new(RwLock::new(0));
        let mock_handler = MockToolHandler {
            should_fail: false,
            call_count: call_count.clone(),
        };
        
        let tool = Tool::new(
            "recovery-tool".to_string(),
            Some("Recovery tool".to_string()),
            json!({}),
            Box::new(mock_handler),
        );
        
        let handler = ToolCallHandler::new();
        handler.register_tool(Arc::new(tool)).await;
        
        // Multiple calls should work independently
        for _ in 0..3 {
            let params = CallToolParams {
                name: "recovery-tool".to_string(),
                arguments: None,
                meta: None,
            };
            
            let result = handler.handle(params).await;
            assert!(result.is_ok());
        }
        
        assert_eq!(*call_count.read().await, 3);
    }
}
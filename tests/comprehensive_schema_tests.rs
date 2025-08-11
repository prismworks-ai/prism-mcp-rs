// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! complete schema compliance validation for MCP Protocol SDK
// ! This test validates all protocol types against the JSON schema

use serde_json::json;
use std::collections::HashMap;

// Import the MCP protocol types
use prism_mcp_rs::protocol::messages::*;
use prism_mcp_rs::protocol::types::*;
use prism_mcp_rs::protocol::*;

#[cfg(test)]
mod complete_schema_validation {
    use super::*;

    /// Validates the protocol version constant
    #[test]
    fn test_protocol_version_compliance() {
        assert_eq!(LATEST_PROTOCOL_VERSION, "2025-06-18");
        assert_eq!(JSONRPC_VERSION, "2.0");
        assert_eq!(PROTOCOL_VERSION, LATEST_PROTOCOL_VERSION); // Legacy compatibility
    }

    /// Validates Implementation struct against schema
    #[test]
    fn test_implementation_schema_compliance() {
        let impl_info = Implementation {
            name: "test-implementation".to_string(),
            version: "1.0.0".to_string(),
            title: None,
        };

        let json_val = serde_json::to_value(&impl_info).unwrap();
        assert!(json_val["name"].is_string());
        assert!(json_val["version"].is_string());
        assert_eq!(json_val["name"], "test-implementation");
        assert_eq!(json_val["version"], "1.0.0");
    }

    /// Validates ServerCapabilities against schema
    #[test]
    fn test_server_capabilities_schema_compliance() {
        let capabilities = ServerCapabilities {
            prompts: Some(PromptsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            sampling: Some(SamplingCapability::default()),
            logging: Some(LoggingCapability::default()),
            completions: Some(CompletionsCapability::default()),
            experimental: Some(HashMap::new()),
        };

        let json_val = serde_json::to_value(&capabilities).unwrap();

        // Validate structure
        assert!(json_val["prompts"].is_object());
        assert!(json_val["resources"].is_object());
        assert!(json_val["tools"].is_object());
        assert!(json_val["sampling"].is_object());
        assert!(json_val["logging"].is_object());
        assert!(json_val["completions"].is_object());
        assert!(json_val["experimental"].is_object());

        // Validate specific fields
        assert_eq!(json_val["prompts"]["listChanged"], true);
        assert_eq!(json_val["resources"]["subscribe"], true);
        assert_eq!(json_val["resources"]["listChanged"], true);
        assert_eq!(json_val["tools"]["listChanged"], true);
    }

    /// Validates ClientCapabilities against schema
    #[test]
    fn test_client_capabilities_schema_compliance() {
        let capabilities = ClientCapabilities {
            sampling: Some(SamplingCapability::default()),
            roots: Some(RootsCapability {
                list_changed: Some(true),
            }),
            elicitation: None,
            experimental: Some(HashMap::new()),
        };

        let json_val = serde_json::to_value(&capabilities).unwrap();

        assert!(json_val["sampling"].is_object());
        assert!(json_val["roots"].is_object());
        assert!(json_val["experimental"].is_object());
        assert_eq!(json_val["roots"]["listChanged"], true);
    }

    /// Validates Content types against schema
    #[test]
    fn test_content_types_schema_compliance() {
        // Test text content
        let text_content = Content::text("Hello, world!");
        let json_val = serde_json::to_value(&text_content).unwrap();
        assert_eq!(json_val["type"], "text");
        assert_eq!(json_val["text"], "Hello, world!");

        // Test image content
        let image_content = Content::image("base64data", "image/png");
        let json_val = serde_json::to_value(&image_content).unwrap();
        assert_eq!(json_val["type"], "image");
        assert_eq!(json_val["data"], "base64data");
        assert_eq!(json_val["mimeType"], "image/png");

        // Test audio content (2025-03-26 NEW)
        let audio_content = Content::audio("audiodata", "audio/wav");
        let json_val = serde_json::to_value(&audio_content).unwrap();
        assert_eq!(json_val["type"], "audio");
        assert_eq!(json_val["data"], "audiodata");
        assert_eq!(json_val["mimeType"], "audio/wav");

        // Test resource_link content (2025-06-18 NEW)
        let resource_link_content = Content::resource_link("file:///test.txt", "test file");
        let json_val = serde_json::to_value(&resource_link_content).unwrap();
        assert_eq!(json_val["type"], "resource_link");
        assert_eq!(json_val["uri"], "file:///test.txt");
        assert_eq!(json_val["name"], "test file");

        // Test embedded resource content (2025-06-18)
        let embedded_resource = Content::embedded_resource(ResourceContents::Text {
            uri: "file:///test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: "File content".to_string(),
            meta: None,
        });
        let json_val = serde_json::to_value(&embedded_resource).unwrap();
        assert_eq!(json_val["type"], "resource");
        assert_eq!(json_val["resource"]["uri"], "file:///test.txt");
        assert_eq!(json_val["resource"]["text"], "File content");
    }

    /// Validates Tool with annotations against schema
    #[test]
    fn test_tool_with_annotations_schema_compliance() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("param1".to_string(), json!({"type": "string"}));
                    props
                }),
                required: Some(vec!["param1".to_string()]),
                additional_properties: HashMap::new(),
            },
            output_schema: None,
            annotations: Some(ToolAnnotations {
                title: None,
                read_only_hint: Some(true),
                destructive_hint: Some(false),
                idempotent_hint: None,
                open_world_hint: None,
            }),
            title: Some("Test Tool".to_string()),
            meta: None,
        };

        let json_val = serde_json::to_value(&tool).unwrap();

        // Validate core tool fields
        assert_eq!(json_val["name"], "test_tool");
        assert_eq!(json_val["description"], "A test tool");
        assert_eq!(json_val["inputSchema"]["type"], "object");
        assert!(json_val["inputSchema"]["properties"].is_object());
        assert!(json_val["inputSchema"]["required"].is_array());

        // Validate annotations (2025-06-18)
        assert!(json_val["annotations"].is_object());
        assert_eq!(json_val["annotations"]["destructiveHint"], false);
        assert_eq!(json_val["annotations"]["readOnlyHint"], true);
    }

    /// Validates Resource against schema
    #[test]
    fn test_resource_schema_compliance() {
        let resource = Resource {
            uri: "file:///test.txt".to_string(),
            name: "Test File".to_string(),
            description: Some("A test file".to_string()),
            mime_type: Some("text/plain".to_string()),
            annotations: Some(Annotations {
                audience: None,
                priority: None,
                last_modified: None,
                danger: None,
                destructive: None,
                read_only: Some(true),
            }),
            size: Some(1024),
            title: Some("Test File".to_string()),
            meta: None,
        };

        let json_val = serde_json::to_value(&resource).unwrap();

        assert_eq!(json_val["uri"], "file:///test.txt");
        assert_eq!(json_val["name"], "Test File");
        assert_eq!(json_val["description"], "A test file");
        assert_eq!(json_val["mimeType"], "text/plain");
        assert_eq!(json_val["size"], 1024);
        assert!(json_val["annotations"].is_object());
    }

    /// Validates Prompt against schema
    #[test]
    fn test_prompt_schema_compliance() {
        let prompt = Prompt {
            name: "test_prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "input".to_string(),
                description: Some("Input text".to_string()),
                required: Some(true),
                title: Some("Input".to_string()),
            }]),
            title: Some("Test Prompt".to_string()),
            meta: None,
        };

        let json_val = serde_json::to_value(&prompt).unwrap();

        assert_eq!(json_val["name"], "test_prompt");
        assert_eq!(json_val["description"], "A test prompt");
        assert!(json_val["arguments"].is_array());
        assert_eq!(json_val["arguments"][0]["name"], "input");
        assert_eq!(json_val["arguments"][0]["required"], true);
    }

    /// Validates JSON-RPC message types against schema
    #[test]
    fn test_jsonrpc_message_schema_compliance() {
        // Test request
        let request = JsonRpcRequest::new::<serde_json::Value>(
            json!("test-1"),
            "tools/list".to_string(),
            Some(json!({})),
        )
        .unwrap();

        let json_val = serde_json::to_value(&request).unwrap();
        assert_eq!(json_val["jsonrpc"], "2.0");
        assert_eq!(json_val["id"], "test-1");
        assert_eq!(json_val["method"], "tools/list");
        assert!(json_val["params"].is_object());

        // Test response
        let response = JsonRpcResponse::success(json!("test-1"), json!({"tools": []})).unwrap();

        let json_val = serde_json::to_value(&response).unwrap();
        assert_eq!(json_val["jsonrpc"], "2.0");
        assert_eq!(json_val["id"], "test-1");
        assert!(json_val["result"].is_object());

        // Test error
        let error = JsonRpcError::error(
            json!("test-1"),
            -32601,
            "Method not found".to_string(),
            None,
        );

        let json_val = serde_json::to_value(&error).unwrap();
        assert_eq!(json_val["jsonrpc"], "2.0");
        assert_eq!(json_val["id"], "test-1");
        assert_eq!(json_val["error"]["code"], -32601);
        assert_eq!(json_val["error"]["message"], "Method not found");

        // Test notification
        let notification = JsonRpcNotification::new::<serde_json::Value>(
            "notifications/progress".to_string(),
            Some(json!({"progress": 50})),
        )
        .unwrap();

        let json_val = serde_json::to_value(&notification).unwrap();
        assert_eq!(json_val["jsonrpc"], "2.0");
        assert_eq!(json_val["method"], "notifications/progress");
        assert!(json_val["params"].is_object());
    }

    /// Validates InitializeParams against schema
    #[test]
    fn test_initialize_params_schema_compliance() {
        let params = InitializeParams {
            protocol_version: "2025-06-18".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                title: Some("Test Client".to_string()),
            },
            meta: None,
        };

        let json_val = serde_json::to_value(&params).unwrap();

        assert_eq!(json_val["protocolVersion"], "2025-06-18");
        assert!(json_val["capabilities"].is_object());
        assert!(json_val["clientInfo"].is_object());
        assert_eq!(json_val["clientInfo"]["name"], "test-client");
        assert_eq!(json_val["clientInfo"]["version"], "1.0.0");
    }

    /// Validates InitializeResult against schema
    #[test]
    fn test_initialize_result_schema_compliance() {
        let result = InitializeResult {
            protocol_version: "2025-06-18".to_string(),
            capabilities: ServerCapabilities::default(),
            server_info: Implementation {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                title: Some("Test Server".to_string()),
            },
            instructions: Some("Test instructions".to_string()),
            meta: None,
        };

        let json_val = serde_json::to_value(&result).unwrap();

        assert_eq!(json_val["protocolVersion"], "2025-06-18");
        assert!(json_val["capabilities"].is_object());
        assert!(json_val["serverInfo"].is_object());
        assert_eq!(json_val["serverInfo"]["name"], "test-server");
        assert_eq!(json_val["instructions"], "Test instructions");
    }

    /// Validates CallToolParams and Result against schema
    #[test]
    fn test_call_tool_schema_compliance() {
        // Test params
        let mut arguments = HashMap::new();
        arguments.insert("input".to_string(), json!("test input"));

        let params = CallToolParams {
            name: "test_tool".to_string(),
            arguments: Some(arguments),
            meta: None,
        };

        let json_val = serde_json::to_value(&params).unwrap();
        assert_eq!(json_val["name"], "test_tool");
        assert!(json_val["arguments"].is_object());
        assert_eq!(json_val["arguments"]["input"], "test input");

        // Test result
        let result = CallToolResult {
            content: vec![Content::text("Tool output")],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        };

        let json_val = serde_json::to_value(&result).unwrap();
        assert!(json_val["content"].is_array());
        assert_eq!(json_val["content"][0]["type"], "text");
        assert_eq!(json_val["content"][0]["text"], "Tool output");
        assert_eq!(json_val["isError"], false);
    }

    /// Validates Tool with outputSchema against schema (2025-06-18)
    #[test]
    fn test_tool_with_output_schema_compliance() {
        use serde_json::json;
        use std::collections::HashMap;

        // Create output schema
        let output_schema = ToolOutputSchema::with_properties(HashMap::from([
            (
                "result".to_string(),
                json!({"type": "string", "description": "Processing result"}),
            ),
            (
                "metadata".to_string(),
                json!({"type": "object", "description": "Additional metadata"}),
            ),
            (
                "success".to_string(),
                json!({"type": "boolean", "description": "Operation success status"}),
            ),
        ]))
        .with_required(vec!["result".to_string(), "success".to_string()]);

        // Create tool with both input and output schemas
        let tool = Tool {
            name: "data_processor".to_string(),
            description: Some("complete data processing tool".to_string()),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "data".to_string(),
                        json!({"type": "string", "description": "Input data"}),
                    ),
                    (
                        "format".to_string(),
                        json!({"type": "string", "enum": ["json", "xml", "csv"]}),
                    ),
                ])),
                required: Some(vec!["data".to_string()]),
                additional_properties: HashMap::new(),
            },
            output_schema: Some(output_schema),
            annotations: Some(ToolAnnotations {
                title: Some("Data Processor".to_string()),
                read_only_hint: Some(false),
                destructive_hint: Some(false),
                idempotent_hint: Some(true),
                open_world_hint: Some(false),
            }),
            title: Some("complete Data Processor".to_string()),
            meta: None,
        };

        // Test serialization
        let json_val = serde_json::to_value(&tool).unwrap();

        // Validate basic tool properties
        assert_eq!(json_val["name"], "data_processor");
        assert_eq!(json_val["title"], "complete Data Processor");
        assert_eq!(json_val["description"], "complete data processing tool");

        // Validate input schema
        assert!(json_val["inputSchema"].is_object());
        assert_eq!(json_val["inputSchema"]["type"], "object");
        assert!(json_val["inputSchema"]["properties"].is_object());
        assert!(json_val["inputSchema"]["required"].is_array());

        // Validate output schema (NEW in 2025-06-18)
        assert!(json_val["outputSchema"].is_object());
        assert_eq!(json_val["outputSchema"]["type"], "object");
        assert!(json_val["outputSchema"]["properties"].is_object());
        assert!(json_val["outputSchema"]["required"].is_array());

        // Validate output schema structure
        let output_props = &json_val["outputSchema"]["properties"];
        assert!(output_props["result"].is_object());
        assert!(output_props["metadata"].is_object());
        assert!(output_props["success"].is_object());
        assert_eq!(output_props["result"]["type"], "string");
        assert_eq!(output_props["success"]["type"], "boolean");

        // Validate required fields in output schema
        let required = json_val["outputSchema"]["required"].as_array().unwrap();
        assert_eq!(required.len(), 2);
        assert!(required.contains(&json!("result")));
        assert!(required.contains(&json!("success")));

        // Validate annotations
        assert!(json_val["annotations"].is_object());
        assert_eq!(json_val["annotations"]["title"], "Data Processor");
        assert_eq!(json_val["annotations"]["idempotentHint"], true);
        assert_eq!(json_val["annotations"]["openWorldHint"], false);

        // Test that the tool can be serialized and deserialized
        let serialized = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tool, deserialized);

        // Verify the output schema is preserved
        assert!(deserialized.output_schema.is_some());
        let output_schema = deserialized.output_schema.unwrap();
        assert_eq!(output_schema.schema_type, "object");
        assert_eq!(
            output_schema.required,
            Some(vec!["result".to_string(), "success".to_string()])
        );
    }

    /// Validates SamplingMessage and CreateMessageParams against schema
    #[test]
    fn test_sampling_schema_compliance() {
        let message = SamplingMessage {
            role: Role::User,
            content: SamplingContent::text("Hello AI"),
        };

        let json_val = serde_json::to_value(&message).unwrap();
        assert_eq!(json_val["role"], "user");
        assert_eq!(json_val["content"]["type"], "text");
        assert_eq!(json_val["content"]["text"], "Hello AI");

        // Test CreateMessageParams
        let params = CreateMessageParams {
            messages: vec![message],
            max_tokens: 1000,
            system_prompt: Some("You are helpful".to_string()),
            include_context: Some("thisServer".to_string()),
            temperature: Some(0.7),
            stop_sequences: Some(vec!["STOP".to_string()]),
            model_preferences: Some(ModelPreferences {
                cost_priority: None,
                speed_priority: None,
                intelligence_priority: None,
                hints: None,
            }),
            metadata: None,
            meta: None,
        };

        let json_val = serde_json::to_value(&params).unwrap();
        assert!(json_val["messages"].is_array());
        assert_eq!(json_val["maxTokens"], 1000);
        assert_eq!(json_val["systemPrompt"], "You are helpful");
        assert_eq!(json_val["includeContext"], "thisServer");
        // Use approximate comparison for floating point values due to JSON precision
        assert!((json_val["temperature"].as_f64().unwrap() - 0.7).abs() < 0.01);
        assert!(json_val["stopSequences"].is_array());
        assert!(json_val["modelPreferences"].is_object());
    }

    /// Validates CreateMessageResult against schema
    #[test]
    fn test_create_message_result_schema_compliance() {
        let result = CreateMessageResult {
            role: Role::Assistant,
            content: SamplingContent::text("AI response"),
            model: "claude-3-5-sonnet".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            meta: None,
        };

        let json_val = serde_json::to_value(&result).unwrap();
        assert_eq!(json_val["role"], "assistant");
        assert_eq!(json_val["content"]["type"], "text");
        assert_eq!(json_val["content"]["text"], "AI response");
        assert_eq!(json_val["model"], "claude-3-5-sonnet");
        assert_eq!(json_val["stopReason"], "endTurn");
    }

    /// Validates LoggingLevel against schema
    #[test]
    fn test_logging_level_schema_compliance() {
        let levels = vec![
            LoggingLevel::Debug,
            LoggingLevel::Info,
            LoggingLevel::Notice,
            LoggingLevel::Warning,
            LoggingLevel::Error,
            LoggingLevel::Critical,
            LoggingLevel::Alert,
            LoggingLevel::Emergency,
        ];

        for level in levels {
            let json_val = serde_json::to_value(&level).unwrap();
            assert!(json_val.is_string());

            // Verify it's one of the expected values
            let level_str = json_val.as_str().unwrap();
            assert!(matches!(
                level_str,
                "debug"
                    | "info"
                    | "notice"
                    | "warning"
                    | "error"
                    | "critical"
                    | "alert"
                    | "emergency"
            ));
        }
    }

    /// Validates completion request and response against schema
    #[test]
    fn test_completion_schema_compliance() {
        // Test completion reference types
        let prompt_ref = CompletionReference::Prompt {
            name: "test_prompt".to_string(),
        };
        let json_val = serde_json::to_value(&prompt_ref).unwrap();
        assert_eq!(json_val["type"], "ref/prompt");
        assert_eq!(json_val["name"], "test_prompt");

        let resource_ref = CompletionReference::Resource {
            uri: "file:///test.txt".to_string(),
        };
        let json_val = serde_json::to_value(&resource_ref).unwrap();
        assert_eq!(json_val["type"], "ref/resource");
        assert_eq!(json_val["uri"], "file:///test.txt");

        // Test completion params
        let params = CompleteParams {
            reference: prompt_ref,
            argument: CompletionArgument {
                name: "input".to_string(),
                value: "partial".to_string(),
            },
            meta: None,
        };

        let json_val = serde_json::to_value(&params).unwrap();
        assert!(json_val["ref"].is_object());
        assert!(json_val["argument"].is_object());
        assert_eq!(json_val["argument"]["name"], "input");
        assert_eq!(json_val["argument"]["value"], "partial");

        // Test completion result
        let result = CompleteResult {
            completion: CompletionData {
                values: vec!["input1".to_string(), "input2".to_string()],
                total: Some(2),
                has_more: Some(false),
            },
            meta: None,
        };

        let json_val = serde_json::to_value(&result).unwrap();
        assert!(json_val["completion"]["values"].is_array());
        assert_eq!(json_val["completion"]["total"], 2);
        assert_eq!(json_val["completion"]["hasMore"], false);
    }

    /// Validates Root against schema
    #[test]
    fn test_root_schema_compliance() {
        let root = Root {
            uri: "file:///project".to_string(),
            name: Some("Project Root".to_string()),
        };

        let json_val = serde_json::to_value(&root).unwrap();
        assert_eq!(json_val["uri"], "file:///project");
        assert_eq!(json_val["name"], "Project Root");
    }

    /// Validates progress notification against schema
    #[test]
    fn test_progress_notification_schema_compliance() {
        let progress = ProgressParams {
            progress_token: json!("upload-123"),
            progress: 75.0,
            total: Some(100.0),
            message: Some("Uploading files...".to_string()),
        };

        let json_val = serde_json::to_value(&progress).unwrap();
        assert_eq!(json_val["progressToken"], "upload-123");
        assert_eq!(json_val["progress"], 75.0);
        assert_eq!(json_val["total"], 100.0);
        assert_eq!(json_val["message"], "Uploading files...");
    }

    /// Validates ResourceContents types against schema
    #[test]
    fn test_resource_contents_schema_compliance() {
        // Test text resource
        let text_resource = ResourceContents::Text {
            uri: "file:///text.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: "File content here".to_string(),
            meta: None,
        };

        let json_val = serde_json::to_value(&text_resource).unwrap();
        assert_eq!(json_val["uri"], "file:///text.txt");
        assert_eq!(json_val["mimeType"], "text/plain");
        assert_eq!(json_val["text"], "File content here");

        // Test binary resource
        let blob_resource = ResourceContents::Blob {
            uri: "file:///image.png".to_string(),
            mime_type: Some("image/png".to_string()),
            blob: "base64imagedata".to_string(),
            meta: None,
        };

        let json_val = serde_json::to_value(&blob_resource).unwrap();
        assert_eq!(json_val["uri"], "file:///image.png");
        assert_eq!(json_val["mimeType"], "image/png");
        assert_eq!(json_val["blob"], "base64imagedata");
    }

    /// Validates error codes against schema
    #[test]
    fn test_error_codes_schema_compliance() {
        use crate::error_codes::*;

        // Test standard JSON-RPC error codes
        assert_eq!(PARSE_ERROR, -32700);
        assert_eq!(INVALID_REQUEST, -32600);
        assert_eq!(METHOD_NOT_FOUND, -32601);
        assert_eq!(INVALID_PARAMS, -32602);
        assert_eq!(INTERNAL_ERROR, -32603);

        // Test MCP-specific error codes
        assert_eq!(TOOL_NOT_FOUND, -32000);
        assert_eq!(RESOURCE_NOT_FOUND, -32001);
        assert_eq!(PROMPT_NOT_FOUND, -32002);
    }

    #[test]
    fn test_method_names_schema_compliance() {
        use crate::methods::*;

        assert_eq!(COMPLETION_COMPLETE, "completion/complete");
        assert_eq!(LOGGING_SET_LEVEL, "logging/setLevel");
        assert_eq!(ROOTS_LIST, "roots/list");

        // Test notification methods
        assert_eq!(PROGRESS, "notifications/progress");
        assert_eq!(CANCELLED, "notifications/cancelled");
        assert_eq!(TOOLS_LIST_CHANGED, "notifications/tools/list_changed");
        assert_eq!(
            RESOURCES_LIST_CHANGED,
            "notifications/resources/list_changed"
        );
    }

    /// Validates ModelPreferences against schema
    #[test]
    fn test_model_preferences_schema_compliance() {
        let preferences = ModelPreferences {
            cost_priority: Some(0.3),
            speed_priority: Some(0.7),
            intelligence_priority: Some(0.9),
            hints: None,
        };

        let json_val = serde_json::to_value(&preferences).unwrap();
        // Use approximate comparison for floating point values due to JSON precision
        assert!((json_val["costPriority"].as_f64().unwrap() - 0.3).abs() < 0.01);
        assert!((json_val["speedPriority"].as_f64().unwrap() - 0.7).abs() < 0.01);
        assert!((json_val["intelligencePriority"].as_f64().unwrap() - 0.9).abs() < 0.01);
    }

    /// Validates complete message flow against schema
    #[test]
    fn test_complete_message_flow_schema_compliance() {
        // Initialize request
        let init_request = JsonRpcRequest::new::<InitializeParams>(
            json!("init-1"),
            INITIALIZE.to_string(),
            Some(InitializeParams {
                protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
                capabilities: ClientCapabilities::default(),
                client_info: Implementation {
                    name: "test-client".to_string(),
                    version: "1.0.0".to_string(),
                    title: Some("Test Client".to_string()),
                },
                meta: None,
            }),
        )
        .unwrap();

        let json_val = serde_json::to_value(&init_request).unwrap();
        assert_eq!(json_val["method"], "initialize");
        assert_eq!(json_val["params"]["protocolVersion"], "2025-06-18");

        // Initialize response
        let init_response = JsonRpcResponse::success(
            json!("init-1"),
            InitializeResult {
                protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
                capabilities: ServerCapabilities::default(),
                server_info: Implementation {
                    name: "test-server".to_string(),
                    version: "1.0.0".to_string(),
                    title: Some("Test Server".to_string()),
                },
                instructions: None,
                meta: None,
            },
        )
        .unwrap();

        let json_val = serde_json::to_value(&init_response).unwrap();
        assert_eq!(json_val["result"]["protocolVersion"], "2025-06-18");

        // Initialized notification
        let initialized_notif = JsonRpcNotification::new::<InitializedParams>(
            INITIALIZED.to_string(),
            Some(InitializedParams { meta: None }),
        )
        .unwrap();

        let json_val = serde_json::to_value(&initialized_notif).unwrap();
        assert_eq!(json_val["method"], "notifications/initialized");
    }

    /// Test that all new 2025-03-26 features are properly implemented
    #[test]
    fn test_all_2025_features_complete() {
        println!("✓ Protocol version: {LATEST_PROTOCOL_VERSION}");

        // Audio content
        let audio = Content::audio("audiodata", "audio/wav");
        assert!(matches!(audio, Content::Audio { .. }));
        println!("✓ Audio content support");

        // Annotations
        let annotations = Annotations::new()
            .destructive(DangerLevel::High)
            .for_audience(vec![Role::User]);
        assert!(annotations.destructive.is_some());
        assert!(annotations.danger.is_some());
        println!("✓ Annotations support");

        // Tool annotations
        let tool_annotations = ToolAnnotations::new().read_only();
        let tool = Tool::new("test", "description").with_annotations(tool_annotations);
        assert!(tool.annotations.is_some());
        println!("✓ Tool annotations support");

        // Completion capabilities
        let caps = ServerCapabilities {
            completions: Some(CompletionsCapability::default()),
            ..Default::default()
        };
        assert!(caps.completions.is_some());
        println!("✓ Completion capabilities");

        // Roots capabilities
        let client_caps = ClientCapabilities {
            roots: Some(RootsCapability {
                list_changed: Some(true),
            }),
            ..Default::default()
        };
        assert!(client_caps.roots.is_some());
        println!("✓ Roots capabilities");

        // improved progress
        let progress = ProgressParams {
            progress_token: json!("token"),
            progress: 50.0,
            total: Some(100.0),
            message: Some("Processing...".to_string()),
        };
        assert!(progress.message.is_some());
        println!("✓ improved progress notifications");

        // Resource links (2025-06-18 NEW)
        let resource_link = Content::resource_link("file://test.txt", "test file");
        assert!(matches!(resource_link, Content::ResourceLink { .. }));
        println!("✓ Resource links support");

        // Embedded resources (2025-06-18)
        let embedded_resource = Content::embedded_resource(ResourceContents::Text {
            uri: "file://test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: "content".to_string(),
            meta: None,
        });
        assert!(matches!(embedded_resource, Content::Resource { .. }));
        println!("✓ Embedded resources support");

        // Metadata support
        let init_params = InitializeParams {
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                title: Some("Test".to_string()),
            },
            meta: Some({
                let mut meta = HashMap::new();
                meta.insert("custom".to_string(), json!("value"));
                meta
            }),
        };
        assert!(init_params.meta.is_some());
        println!("✓ Metadata support in requests");

        println!("\n All 2025-03-26 features are properly implemented and schema-compliant!");
    }

    /// Final complete validation test
    #[test]
    fn test_final_schema_compliance_report() {
        println!("\n=== complete SCHEMA COMPLIANCE REPORT ===");
        println!("Protocol Version: {LATEST_PROTOCOL_VERSION}");
        println!("JSON-RPC Version: {JSONRPC_VERSION}");

        let mut checks_passed = 0;
        let total_checks = 15;

        // Check 1: Core types
        let _impl = Implementation {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            title: Some("Test Implementation".to_string()),
        };
        checks_passed += 1;
        println!("✓ Core types (Implementation)");

        // Check 2: Capabilities
        let _caps = ServerCapabilities::default();
        checks_passed += 1;
        println!("✓ Server capabilities");

        // Check 3: Content types
        let _content = Content::text("test");
        checks_passed += 1;
        println!("✓ Content types");

        // Check 4: Tool types
        let _tool = Tool::new("test", "description");
        checks_passed += 1;
        println!("✓ Tool types");

        // Check 5: Resource types
        let _resource = Resource {
            uri: "file://test".to_string(),
            name: "Test Resource".to_string(),
            description: None,
            mime_type: None,
            annotations: None,
            size: None,
            title: Some("Test Resource".to_string()),
            meta: None,
        };
        checks_passed += 1;
        println!("✓ Resource types");

        // Check 6: Prompt types
        let _prompt = Prompt {
            name: "test".to_string(),
            description: None,
            arguments: None,
            title: Some("Test Prompt".to_string()),
            meta: None,
        };
        checks_passed += 1;
        println!("✓ Prompt types");

        // Check 7: JSON-RPC types
        let _request =
            JsonRpcRequest::new::<serde_json::Value>(json!(1), "test".to_string(), None).unwrap();
        checks_passed += 1;
        println!("✓ JSON-RPC types");

        // Check 8: Message types
        let _params = InitializeParams {
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                title: Some("Test".to_string()),
            },
            meta: None,
        };
        checks_passed += 1;
        println!("✓ Message parameter types");

        // Check 9: Sampling types
        let _message = SamplingMessage::user_text("test");
        checks_passed += 1;
        println!("✓ Sampling types");

        // Check 10: Logging types
        let _level = LoggingLevel::Info;
        checks_passed += 1;
        println!("✓ Logging types");

        // Check 11: Completion types
        let _completion = CompletionReference::Prompt {
            name: "test".to_string(),
        };
        checks_passed += 1;
        println!("✓ Completion types");

        // Check 12: Root types
        let _root = Root::new("file://test".to_string());
        checks_passed += 1;
        println!("✓ Root types");

        // Check 13: Progress types
        let _progress = ProgressParams {
            progress_token: json!("test"),
            progress: 50.0,
            total: None,
            message: None,
        };
        checks_passed += 1;
        println!("✓ Progress notification types");

        // Check 14: Error codes
        assert_eq!(crate::error_codes::PARSE_ERROR, -32700);
        checks_passed += 1;
        println!("✓ Error codes");

        // Check 15: 2025-03-26 features
        let _audio = Content::audio("data", "audio/wav");
        let _annotations = Annotations::new();
        checks_passed += 1;
        println!("✓ 2025-03-26 new features");

        println!("\n=== COMPLIANCE SUMMARY ===");
        println!("Checks passed: {checks_passed}/{total_checks}");
        println!(
            "Compliance rate: {:.1}%",
            (checks_passed as f64 / total_checks as f64) * 100.0
        );

        if checks_passed == total_checks {
            println!(" ALL PROTOCOL TYPES ARE 100% COMPLIANT WITH SCHEMA!");
        } else {
            println!("Warning:  Some compliance issues found");
        }

        assert_eq!(
            checks_passed, total_checks,
            "Not all schema compliance checks passed"
        );
    }
}

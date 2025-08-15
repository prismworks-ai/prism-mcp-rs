// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! Tests for protocol types and validation

use prism_mcp_rs::protocol::{types::*, validation::*};
use serde_json::json;
use std::collections::HashMap;

#[cfg(test)]
mod protocol_types_tests {
    use super::*;

    #[test]
    fn test_content_creation() {
        let text_content = Content::text("Hello, world!");
        match text_content {
            Content::Text {
                text,
                annotations,
                meta: _,
            } => {
                assert_eq!(text, "Hello, world!");
                assert!(annotations.is_none());
            }
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_jsonrpc_message_creation() {
        let request = JsonRpcRequest::new(
            json!("test-123"),
            "tools/list".to_string(),
            Some(json!({"cursor": null})),
        )
        .unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, json!("test-123"));
        assert_eq!(request.method, "tools/list");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_jsonrpc_response_creation() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: json!("test-456"),
            result: Some(json!({
                "tools": []
            })),
        };

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
    }

    #[test]
    fn test_tool_creation() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("param".to_string(), json!({"type": "string"}));
                    props
                }),
                required: None,
                additional_properties: HashMap::new(),
            },
            output_schema: None,
            annotations: None,
            title: None,
            meta: None,
        };

        assert_eq!(tool.name, "test_tool");
        assert!(tool.description.is_some());
    }

    #[test]
    fn test_resource_creation() {
        let resource = Resource {
            uri: "file:///test.txt".to_string(),
            name: "Test File".to_string(),
            description: Some("A test file resource".to_string()),
            mime_type: Some("text/plain".to_string()),
            annotations: None,
            size: Some(1024),
            title: None,
            meta: None,
        };

        assert_eq!(resource.uri, "file:///test.txt");
        assert_eq!(resource.name, "Test File");
        assert_eq!(resource.size.unwrap(), 1024);
    }

    #[test]
    fn test_audio_content() {
        let audio = Content::audio("base64audiodata", "audio/wav");
        match audio {
            Content::Audio {
                data, mime_type, ..
            } => {
                assert_eq!(data, "base64audiodata");
                assert_eq!(mime_type, "audio/wav");
            }
            _ => panic!("Expected audio content"),
        }
    }
}

#[cfg(test)]
mod protocol_validation_tests {
    use super::*;

    #[test]
    fn test_validate_uri() {
        let valid_uris = vec![
            "file:///path/to/file.txt",
            "https://example.com/resource",
            "mcp://server/resource",
        ];

        for uri in valid_uris {
            let result = validate_uri(uri);
            assert!(result.is_ok(), "URI should be valid: {uri}");
        }
    }

    #[test]
    fn test_validate_method_name() {
        let valid_methods = vec!["initialize", "tools/list", "resources/read"];

        for method in valid_methods {
            let result = validate_method_name(method);
            assert!(result.is_ok(), "Method should be valid: {method}");
        }
    }

    #[test]
    fn test_validate_content() {
        let valid_content = Content::Text {
            text: "Valid text content".to_string(),
            annotations: None,
            meta: None,
        };

        let result = validate_content(&valid_content);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod protocol_constants_tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(LATEST_PROTOCOL_VERSION, "2025-06-18");
        assert_eq!(JSONRPC_VERSION, "2.0");
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(error_codes::PARSE_ERROR, -32700);
        assert_eq!(error_codes::INVALID_REQUEST, -32600);
        assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_codes::TOOL_NOT_FOUND, -32000);
        assert_eq!(error_codes::RESOURCE_NOT_FOUND, -32001);
        assert_eq!(error_codes::PROMPT_NOT_FOUND, -32002);
    }
}

#[cfg(test)]
mod annotations_tests {
    use super::*;

    #[test]
    fn test_annotations_creation() {
        let annotations = Annotations::new()
            .read_only()
            .for_audience(vec![Role::User])
            .with_danger_level(DangerLevel::Safe);

        assert_eq!(annotations.read_only, Some(true));
        assert_eq!(annotations.destructive, Some(false));
        assert_eq!(annotations.danger, Some(DangerLevel::Safe));
        assert_eq!(annotations.audience, Some(vec![Role::User]));
    }

    #[test]
    fn test_destructive_annotations() {
        let annotations = Annotations::new().destructive(DangerLevel::High);

        assert_eq!(annotations.destructive, Some(true));
        assert_eq!(annotations.read_only, Some(false));
        assert_eq!(annotations.danger, Some(DangerLevel::High));
    }

    #[test]
    fn test_tool_with_annotations() {
        let tool = Tool::new("safe_reader", "Read file safely")
            .with_annotations(ToolAnnotations::new().read_only());

        assert_eq!(tool.name, "safe_reader");
        assert!(tool.annotations.is_some());
        assert_eq!(tool.annotations.unwrap().read_only_hint, Some(true));
    }
}

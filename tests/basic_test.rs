use prism_mcp_rs::protocol::types::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_basic_protocol_types() {
    // Test that we can create and serialize basic types

    // Test Content
    let text_content = Content::text("Hello, world!");
    let serialized = serde_json::to_string(&text_content).unwrap();
    assert!(serialized.contains("Hello, world!"));

    // Test Tool with required fields
    let tool = Tool {
        name: "test_tool".to_string(),
        description: Some("A test tool".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: None,
            additional_properties: HashMap::new(),
        },
        output_schema: None,
        annotations: None,
        title: Some("Test Tool".to_string()),
        meta: None,
    };
    assert_eq!(tool.name, "test_tool");

    // Test JSON-RPC Request
    let request = JsonRpcRequest {
        jsonrpc: JSONRPC_VERSION.to_string(),
        id: json!(1),
        method: "test_method".to_string(),
        params: Some(json!({"test": "value"})),
    };
    assert_eq!(request.method, "test_method");
    assert_eq!(request.id, json!(1));

    // Test JSON-RPC Response
    let response = JsonRpcResponse {
        jsonrpc: JSONRPC_VERSION.to_string(),
        id: json!(1),
        result: Some(json!({"result": "success"})),
    };
    assert_eq!(response.id, json!(1));

    // Test that serialization works
    let request_json = serde_json::to_string(&request).unwrap();
    assert!(request_json.contains("test_method"));

    let response_json = serde_json::to_string(&response).unwrap();
    assert!(response_json.contains("success"));

    println!("[x] All basic protocol types work correctly!");
}

#[test]
fn test_2025_features() {
    // Test new 2025-06-18 features

    // Test Audio content
    let audio_content = Content::audio("base64audiodata", "audio/wav");
    let serialized = serde_json::to_value(&audio_content).unwrap();
    assert_eq!(serialized["type"], "audio");
    assert_eq!(serialized["data"], "base64audiodata");
    assert_eq!(serialized["mimeType"], "audio/wav");

    // Test Resource link content (resource method renamed to resource_link)
    let resource_content = Content::resource_link("file:///test.txt", "Test File");
    let serialized = serde_json::to_value(&resource_content).unwrap();
    assert_eq!(serialized["type"], "resource_link");
    assert_eq!(serialized["uri"], "file:///test.txt");
    assert_eq!(serialized["name"], "Test File");

    // Test Annotations (updated API)
    let annotations = Annotations::new().with_priority(0.8);

    assert_eq!(annotations.priority, Some(0.8));

    // Test Tool with annotations and required fields
    let tool = Tool {
        name: "safe_tool".to_string(),
        description: Some("A safe tool".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: None,
            additional_properties: HashMap::new(),
        },
        output_schema: None,
        annotations: None, // ToolAnnotations are different from regular Annotations
        title: Some("Safe Tool".to_string()),
        meta: None,
    };

    assert_eq!(tool.name, "safe_tool");

    println!("[x] All 2025-06-18 features work correctly!");
}

#[test]
fn test_server_capabilities() {
    // Test that we can create and work with server capabilities
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        // Note: completions renamed in 2025-06-18 spec
        completions: None,
        ..Default::default()
    };

    let serialized = serde_json::to_value(&capabilities).unwrap();
    assert_eq!(serialized["tools"]["listChanged"], true);
    assert_eq!(serialized["resources"]["subscribe"], true);

    println!("[x] Server capabilities work correctly!");
}

#[test]
fn test_constants() {
    // Test that protocol constants are correct
    assert_eq!(LATEST_PROTOCOL_VERSION, "2025-06-18");
    assert_eq!(JSONRPC_VERSION, "2.0");
    assert_eq!(PROTOCOL_VERSION, LATEST_PROTOCOL_VERSION);

    println!("[x] Protocol constants are correct!");
}

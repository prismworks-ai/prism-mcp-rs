//! Test to verify the client API fixes

use prism_mcp_rs::client::McpClientBuilder;
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::protocol::types::ClientInfo;
use std::collections::HashMap;

#[test]
fn test_client_builder_with_client_info() {
    // Test that we can build with ClientInfo
    let client_info = ClientInfo {
        name: "test-client".to_string(),
        version: "1.0.0".to_string(),
        title: Some("Test Client".to_string()),
    };

    let builder = McpClientBuilder::new().with_client_info(client_info.clone());

    let client = builder.build().expect("Failed to build client");
    assert_eq!(client.info().name, "test-client");
    assert_eq!(client.info().version, "1.0.0");
}

#[cfg(feature = "stdio")]
#[tokio::test]
async fn test_client_builder_connect_stdio() {
    use prism_mcp_rs::protocol::types::ClientCapabilities;

    let client_info = ClientInfo {
        name: "test-client".to_string(),
        version: "1.0.0".to_string(),
        title: None,
    };

    let builder = McpClientBuilder::new()
        .with_client_info(client_info)
        .with_capabilities(ClientCapabilities::default());

    // This should compile (won't run successfully without a real server)
    let _result = builder
        .connect_stdio("echo", &["test".to_string()], Some(HashMap::new()))
        .await;
}

#[tokio::test]
async fn test_client_session_methods() {
    use prism_mcp_rs::client::{ClientSession, McpClient};
    use prism_mcp_rs::protocol::messages::*;

    // Create client and session
    let client = McpClient::new("test".to_string(), "1.0.0".to_string());
    let session = ClientSession::new(client);

    // These methods should exist and compile
    // (they'll fail without connection but that's expected)
    let _ = session.list_tools(None).await;
    let _ = session.list_resources(None).await;
    let _ = session.list_prompts(None).await;

    let call_params = CallToolParams {
        name: "test-tool".to_string(),
        arguments: Some(HashMap::new()),
        meta: None,
    };
    let _ = session.call_tool(call_params).await;

    let read_params = ReadResourceParams {
        uri: "test://resource".to_string(),
        meta: None,
    };
    let _ = session.read_resource(read_params).await;

    let prompt_params = GetPromptParams {
        name: "test-prompt".to_string(),
        arguments: Some(HashMap::new()),
        meta: None,
    };
    let _ = session.get_prompt(prompt_params).await;
}

#[test]
fn test_protocol_types_export() {
    // These imports should work
    use prism_mcp_rs::protocol::messages::*;
    use prism_mcp_rs::protocol::types::*;

    // Test that types are accessible
    let _info = ClientInfo {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        title: None,
    };

    let _tool_result = CallToolResult {
        content: vec![],
        is_error: Some(false),
        structured_content: None,
        meta: None,
    };

    let _read_result = ReadResourceResult {
        contents: vec![],
        meta: None,
    };
}

#[cfg(feature = "stdio")]
#[test]
fn test_stdio_transport_with_env() {
    use prism_mcp_rs::transport::StdioClientTransport;
    use std::collections::HashMap;

    // Test that with_env method exists
    let env_vars: HashMap<String, String> =
        HashMap::from([("TEST_VAR".to_string(), "test_value".to_string())]);

    // This should compile (actual transport creation would fail without valid command)
    let _future = StdioClientTransport::with_env("echo", vec!["test"], env_vars);
}

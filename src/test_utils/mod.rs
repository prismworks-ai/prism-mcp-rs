//! Testing utilities for MCP implementations
//!
//! This module provides helper functions and utilities for testing MCP servers and clients.
//!
//! # Usage
//!
//! These utilities are only available when running tests:
//!
//! ```
//! #[cfg(test)]
//! mod tests {
//!     use prism_mcp_rs::test_utils::*;
//!     use prism_mcp_rs::protocol::JsonRpcMessage;
//!     
//!     fn example_error_response_assertion() {
//!         let request = mock_request("unknown_method");
//!         // Process request...
//!         let response = JsonRpcMessage::Error(
//!             JsonRpcError::method_not_found(request.id.clone())
//!         );
//!         
//!         assert_error_response(&response, -32601);
//!     }
//! }
//! ```

use crate::protocol::*;
use serde_json::{json, Value};

/// Create a mock JSON-RPC request for testing
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::mock_request;
/// let request = mock_request("initialize");
/// assert_eq!(request.method, "initialize");
/// ```
pub fn mock_request(method: &str) -> JsonRpcRequest {
    JsonRpcRequest::new(json!("test-123"), method.to_string(), None::<()>)
        .expect("Failed to create mock request")
}

/// Create a mock JSON-RPC request with parameters
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::mock_request_with_params;
/// # use serde_json::json;
/// let request = mock_request_with_params(
///     "tools/call",
///     json!({"name": "calculator", "arguments": {}})
/// );
/// ```
pub fn mock_request_with_params<T: serde::Serialize>(method: &str, params: T) -> JsonRpcRequest {
    JsonRpcRequest::new(json!("test-123"), method.to_string(), Some(params))
        .expect("Failed to create mock request with params")
}

/// Assert that a JsonRpcMessage is an error with specific code
///
/// # Panics
///
/// Panics if the message is not an error or has a different error code
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::assert_error_response;
/// # use prism_mcp_rs::protocol::{JsonRpcMessage, JsonRpcError};
/// # use serde_json::json;
/// let error = JsonRpcMessage::Error(JsonRpcError::parse_error());
/// assert_error_response(&error, -32700);
/// ```
pub fn assert_error_response(response: &JsonRpcMessage, expected_code: i32) {
    match response {
        JsonRpcMessage::Error(err) => {
            assert_eq!(
                err.error.code, expected_code,
                "Expected error code {}, got {}",
                expected_code, err.error.code
            );
        }
        _ => panic!("Expected error response, got {:?}", response),
    }
}

/// Assert that a JsonRpcMessage is a successful response
///
/// # Panics
///
/// Panics if the message is not a successful response
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::assert_success_response;
/// # use prism_mcp_rs::protocol::{JsonRpcMessage, JsonRpcResponse};
/// # use serde_json::json;
/// let response = JsonRpcMessage::Response(
///     JsonRpcResponse::success_unchecked(json!("1"), json!({}))
/// );
/// assert_success_response(&response);
/// ```
pub fn assert_success_response(response: &JsonRpcMessage) {
    match response {
        JsonRpcMessage::Response(_) => {}
        _ => panic!("Expected success response, got {:?}", response),
    }
}

/// Create a mock tool call request
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::mock_tool_call;
/// # use serde_json::json;
/// let request = mock_tool_call("calculator", json!({"expression": "2+2"}));
/// assert_eq!(request.method, "tools/call");
/// ```
pub fn mock_tool_call(tool_name: &str, args: Value) -> JsonRpcRequest {
    JsonRpcRequest::new(
        json!("test-tool-123"),
        "tools/call".to_string(),
        Some(json!({
            "name": tool_name,
            "arguments": args
        })),
    )
    .expect("Failed to create mock tool call")
}

/// Create a mock resource read request
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::mock_resource_read;
/// let request = mock_resource_read("file:///example.txt");
/// assert_eq!(request.method, "resources/read");
/// ```
pub fn mock_resource_read(uri: &str) -> JsonRpcRequest {
    JsonRpcRequest::new(
        json!("test-resource-123"),
        "resources/read".to_string(),
        Some(json!({
            "uri": uri
        })),
    )
    .expect("Failed to create mock resource read")
}

/// Create a mock prompt get request
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::mock_prompt_get;
/// # use serde_json::json;
/// let request = mock_prompt_get("greeting", json!({"name": "Alice"}));
/// assert_eq!(request.method, "prompts/get");
/// ```
pub fn mock_prompt_get(prompt_name: &str, args: Value) -> JsonRpcRequest {
    JsonRpcRequest::new(
        json!("test-prompt-123"),
        "prompts/get".to_string(),
        Some(json!({
            "name": prompt_name,
            "arguments": args
        })),
    )
    .expect("Failed to create mock prompt get")
}

/// Create a mock initialize request
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::mock_initialize;
/// let request = mock_initialize("test-client", "1.0.0");
/// assert_eq!(request.method, "initialize");
/// ```
pub fn mock_initialize(client_name: &str, client_version: &str) -> JsonRpcRequest {
    JsonRpcRequest::new(
        json!("init-123"),
        "initialize".to_string(),
        Some(json!({
            "protocolVersion": crate::protocol::LEGACY_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": client_name,
                "version": client_version
            }
        })),
    )
    .expect("Failed to create mock initialize")
}

/// Create a mock notification
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::mock_notification;
/// # use serde_json::json;
/// let notification = mock_notification("initialized", json!({}));
/// assert_eq!(notification.method, "initialized");
/// ```
pub fn mock_notification(method: &str, params: Value) -> JsonRpcNotification {
    JsonRpcNotification {
        jsonrpc: JSONRPC_VERSION.to_string(),
        method: method.to_string(),
        params: Some(params),
    }
}

/// Create a mock successful response with result
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::mock_success;
/// # use serde_json::json;
/// let response = mock_success(json!({"tools": []}));
/// ```
pub fn mock_success(result: Value) -> JsonRpcResponse {
    JsonRpcResponse::success_unchecked(json!("test-123"), result)
}

/// Create a mock error response
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::mock_error;
/// let error = mock_error(-32601, "Method not found");
/// ```
pub fn mock_error(code: i32, message: &str) -> JsonRpcError {
    JsonRpcError::new(json!("test-123"), code, message.to_string())
}

/// Test helper to verify a response contains expected fields
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::{mock_success, assert_response_contains};
/// # use serde_json::json;
/// let response = mock_success(json!({"tools": [], "version": "1.0"}));
/// assert_response_contains(&response, &["tools", "version"]);
/// ```
pub fn assert_response_contains(response: &JsonRpcResponse, expected_fields: &[&str]) {
    if let Some(ref result) = response.result {
        if let Some(result_obj) = result.as_object() {
            for field in expected_fields {
                assert!(
                    result_obj.contains_key(*field),
                    "Response missing expected field: {}",
                    field
                );
            }
        }
    } else {
        panic!("Response result is not an object");
    }
}

/// Test helper to create a batch request
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::{create_batch_request, mock_request};
/// let batch = create_batch_request(vec![
///     mock_request("tools/list"),
///     mock_request("resources/list"),
/// ]);
/// assert_eq!(batch.len(), 2);
/// ```
pub fn create_batch_request(requests: Vec<JsonRpcRequest>) -> JsonRpcBatchRequest {
    requests // JsonRpcBatchRequest is a type alias for Vec<JsonRpcRequest>
}

/// Test helper to create a batch response
///
/// # Example
/// ```
/// # use prism_mcp_rs::test_utils::{create_batch_response, mock_success};
/// # use prism_mcp_rs::protocol::JsonRpcResponseOrError;
/// # use serde_json::json;
/// let batch = create_batch_response(vec![
///     JsonRpcResponseOrError::Response(mock_success(json!({}))),
/// ]);
/// assert_eq!(batch.len(), 1);
/// ```
pub fn create_batch_response(
    responses: Vec<JsonRpcResponseOrError>,
) -> Vec<JsonRpcResponseOrError> {
    responses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_request() {
        let request = mock_request("test_method");
        assert_eq!(request.method, "test_method");
        assert_eq!(request.jsonrpc, JSONRPC_VERSION);
    }

    #[test]
    fn test_mock_request_with_params() {
        let request = mock_request_with_params("test", json!({"key": "value"}));
        assert_eq!(request.method, "test");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_assert_error_response() {
        let error = JsonRpcMessage::Error(JsonRpcError::parse_error(json!(1)));
        assert_error_response(&error, -32700);
    }

    #[test]
    #[should_panic(expected = "Expected error response")]
    fn test_assert_error_response_panic() {
        let response =
            JsonRpcMessage::Response(JsonRpcResponse::success_unchecked(json!("1"), json!({})));
        assert_error_response(&response, -32700);
    }

    #[test]
    fn test_mock_tool_call() {
        let request = mock_tool_call("calc", json!({"x": 1}));
        assert_eq!(request.method, "tools/call");
        let params = request.params.unwrap();
        assert_eq!(params["name"], "calc");
    }

    #[test]
    fn test_mock_initialize() {
        let request = mock_initialize("client", "1.0.0");
        assert_eq!(request.method, "initialize");
        let params = request.params.unwrap();
        assert_eq!(
            params["protocolVersion"],
            crate::protocol::LEGACY_PROTOCOL_VERSION
        );
    }

    #[test]
    fn test_assert_response_contains() {
        let response = mock_success(json!({
            "field1": "value1",
            "field2": "value2"
        }));
        assert_response_contains(&response, &["field1", "field2"]);
    }

    #[test]
    #[should_panic(expected = "Response missing expected field")]
    fn test_assert_response_contains_missing() {
        let response = mock_success(json!({"field1": "value1"}));
        assert_response_contains(&response, &["field1", "field2"]);
    }
}

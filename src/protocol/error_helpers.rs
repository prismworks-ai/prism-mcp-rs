//! JSON-RPC error response helper functions
//!
//! This module provides convenience methods for creating JSON-RPC error responses
//! according to the MCP protocol specification.

use serde_json::Value;

use crate::protocol::{
    error_codes::*,
    types::{ErrorObject, JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, 
            JsonRpcNotification, RequestId, JSONRPC_VERSION},
};

impl JsonRpcError {
    /// Create a new JSON-RPC error with the given parameters
    /// 
    /// # Examples
    /// 
    /// ```rust,ignore
    /// use prism_mcp_rs::protocol::JsonRpcError;
    /// 
    /// let error = JsonRpcError::new(
    ///     json!("req-123"),
    ///     -32601,
    ///     "Method not found"
    /// );
    /// ```
    pub fn new<S: Into<String>>(id: RequestId, code: i32, message: S) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            error: ErrorObject {
                code,
                message: message.into(),
                data: None,
            },
        }
    }

    /// Create a new JSON-RPC error with additional data
    /// 
    /// # Examples
    /// 
    /// ```rust,ignore
    /// use prism_mcp_rs::protocol::JsonRpcError;
    /// use serde_json::json;
    /// 
    /// let error = JsonRpcError::with_data(
    ///     json!("req-123"),
    ///     -32602,
    ///     "Invalid params",
    ///     Some(json!({"expected": "string", "got": "number"}))
    /// );
    /// ```
    pub fn with_data<S: Into<String>>(
        id: RequestId,
        code: i32,
        message: S,
        data: Option<Value>,
    ) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            error: ErrorObject {
                code,
                message: message.into(),
                data,
            },
        }
    }

    /// Create a "Parse error" response (-32700)
    /// 
    /// Invalid JSON was received by the server
    pub fn parse_error(id: RequestId) -> Self {
        Self::new(id, PARSE_ERROR, "Parse error")
    }

    /// Create an "Invalid Request" response (-32600)
    /// 
    /// The JSON sent is not a valid Request object
    pub fn invalid_request(id: RequestId) -> Self {
        Self::new(id, INVALID_REQUEST, "Invalid Request")
    }

    /// Create a "Method not found" response (-32601)
    /// 
    /// The method does not exist or is not available
    pub fn method_not_found(id: RequestId) -> Self {
        Self::new(id, METHOD_NOT_FOUND, "Method not found")
    }

    /// Create a "Method not found" response with the method name
    pub fn method_not_found_with_name<S: Into<String>>(id: RequestId, method: S) -> Self {
        let method = method.into();
        Self::with_data(
            id,
            METHOD_NOT_FOUND,
            format!("Method '{}' not found", method),
            Some(serde_json::json!({ "method": method })),
        )
    }

    /// Create an "Invalid params" response (-32602)
    /// 
    /// Invalid method parameter(s)
    pub fn invalid_params(id: RequestId) -> Self {
        Self::new(id, INVALID_PARAMS, "Invalid params")
    }

    /// Create an "Invalid params" response with details
    pub fn invalid_params_with_message<S: Into<String>>(id: RequestId, details: S) -> Self {
        Self::new(id, INVALID_PARAMS, details)
    }

    /// Create an "Internal error" response (-32603)
    /// 
    /// Internal JSON-RPC error
    pub fn internal_error(id: RequestId) -> Self {
        Self::new(id, INTERNAL_ERROR, "Internal error")
    }

    /// Create an "Internal error" response with details
    pub fn internal_error_with_message<S: Into<String>>(id: RequestId, details: S) -> Self {
        Self::new(id, INTERNAL_ERROR, details)
    }

    /// Create a "Tool not found" error (-32000)
    /// 
    /// MCP-specific: The requested tool does not exist
    pub fn tool_not_found<S: Into<String>>(id: RequestId, tool_name: S) -> Self {
        let name = tool_name.into();
        Self::with_data(
            id,
            TOOL_NOT_FOUND,
            format!("Tool '{}' not found", name),
            Some(serde_json::json!({ "tool": name })),
        )
    }

    /// Create a "Resource not found" error (-32001)
    /// 
    /// MCP-specific: The requested resource does not exist
    pub fn resource_not_found<S: Into<String>>(id: RequestId, uri: S) -> Self {
        let uri = uri.into();
        Self::with_data(
            id,
            RESOURCE_NOT_FOUND,
            format!("Resource '{}' not found", uri),
            Some(serde_json::json!({ "uri": uri })),
        )
    }

    /// Create a "Prompt not found" error (-32002)
    /// 
    /// MCP-specific: The requested prompt does not exist
    pub fn prompt_not_found<S: Into<String>>(id: RequestId, prompt_name: S) -> Self {
        let name = prompt_name.into();
        Self::with_data(
            id,
            PROMPT_NOT_FOUND,
            format!("Prompt '{}' not found", name),
            Some(serde_json::json!({ "prompt": name })),
        )
    }

    /// Create a custom error with a specific code and message
    pub fn custom<S: Into<String>>(id: RequestId, code: i32, message: S) -> Self {
        Self::new(id, code, message)
    }

    /// Create a custom error with a specific code, message, and data
    pub fn custom_with_data<S: Into<String>>(
        id: RequestId,
        code: i32,
        message: S,
        data: Value,
    ) -> Self {
        Self::with_data(id, code, message, Some(data))
    }
}

// Conversion from JsonRpcError to JsonRpcMessage for easier use
impl From<JsonRpcError> for JsonRpcMessage {
    fn from(error: JsonRpcError) -> Self {
        JsonRpcMessage::Error(error)
    }
}

// Helper trait for converting errors to JsonRpcMessage
pub trait IntoJsonRpcMessage {
    fn into_message(self) -> JsonRpcMessage;
}

impl IntoJsonRpcMessage for JsonRpcError {
    fn into_message(self) -> JsonRpcMessage {
        JsonRpcMessage::Error(self)
    }
}

// Additional type conversions for ergonomic API
impl From<JsonRpcResponse> for JsonRpcMessage {
    fn from(response: JsonRpcResponse) -> Self {
        JsonRpcMessage::Response(response)
    }
}

impl From<JsonRpcNotification> for JsonRpcMessage {
    fn from(notification: JsonRpcNotification) -> Self {
        JsonRpcMessage::Notification(notification)
    }
}

impl From<JsonRpcRequest> for JsonRpcMessage {
    fn from(request: JsonRpcRequest) -> Self {
        JsonRpcMessage::Request(request)
    }
}

impl TryFrom<JsonRpcMessage> for JsonRpcRequest {
    type Error = crate::core::error::McpError;
    
    fn try_from(msg: JsonRpcMessage) -> Result<Self, Self::Error> {
        match msg {
            JsonRpcMessage::Request(req) => Ok(req),
            _ => Err(crate::core::error::McpError::Protocol(
                "Not a JSON-RPC request".to_string()
            )),
        }
    }
}

impl TryFrom<JsonRpcMessage> for JsonRpcResponse {
    type Error = crate::core::error::McpError;
    
    fn try_from(msg: JsonRpcMessage) -> Result<Self, Self::Error> {
        match msg {
            JsonRpcMessage::Response(resp) => Ok(resp),
            _ => Err(crate::core::error::McpError::Protocol(
                "Not a JSON-RPC response".to_string()
            )),
        }
    }
}

impl TryFrom<JsonRpcMessage> for JsonRpcError {
    type Error = crate::core::error::McpError;
    
    fn try_from(msg: JsonRpcMessage) -> Result<Self, Self::Error> {
        match msg {
            JsonRpcMessage::Error(err) => Ok(err),
            _ => Err(crate::core::error::McpError::Protocol(
                "Not a JSON-RPC error".to_string()
            )),
        }
    }
}

impl TryFrom<JsonRpcMessage> for JsonRpcNotification {
    type Error = crate::core::error::McpError;
    
    fn try_from(msg: JsonRpcMessage) -> Result<Self, Self::Error> {
        match msg {
            JsonRpcMessage::Notification(notif) => Ok(notif),
            _ => Err(crate::core::error::McpError::Protocol(
                "Not a JSON-RPC notification".to_string()
            )),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_error_creation() {
        let id = json!("test-123");

        // Test basic error creation
        let error = JsonRpcError::new(id.clone(), -32601, "Method not found");
        assert_eq!(error.jsonrpc, "2.0");
        assert_eq!(error.id, id);
        assert_eq!(error.error.code, -32601);
        assert_eq!(error.error.message, "Method not found");
        assert!(error.error.data.is_none());

        // Test error with data
        let data = json!({"detail": "extra info"});
        let error_with_data =
            JsonRpcError::with_data(id.clone(), -32602, "Invalid params", Some(data.clone()));
        assert_eq!(error_with_data.error.data, Some(data));
    }

    #[test]
    fn test_standard_errors() {
        let id = json!(123);

        // Parse error
        let parse_err = JsonRpcError::parse_error(id.clone());
        assert_eq!(parse_err.error.code, PARSE_ERROR);
        assert_eq!(parse_err.error.message, "Parse error");

        // Invalid request
        let invalid_req = JsonRpcError::invalid_request(id.clone());
        assert_eq!(invalid_req.error.code, INVALID_REQUEST);
        assert_eq!(invalid_req.error.message, "Invalid Request");

        // Method not found
        let method_err = JsonRpcError::method_not_found(id.clone());
        assert_eq!(method_err.error.code, METHOD_NOT_FOUND);
        assert_eq!(method_err.error.message, "Method not found");

        // Method not found with name
        let method_err_with_name = JsonRpcError::method_not_found_with_name(id.clone(), "test_method");
        assert_eq!(method_err_with_name.error.code, METHOD_NOT_FOUND);
        assert!(method_err_with_name.error.message.contains("test_method"));
        assert!(method_err_with_name.error.data.is_some());

        // Invalid params
        let params_err = JsonRpcError::invalid_params(id.clone());
        assert_eq!(params_err.error.code, INVALID_PARAMS);
        assert_eq!(params_err.error.message, "Invalid params");

        // Invalid params with message
        let params_err_msg = JsonRpcError::invalid_params_with_message(id.clone(), "Missing required field 'name'");
        assert_eq!(params_err_msg.error.code, INVALID_PARAMS);
        assert_eq!(params_err_msg.error.message, "Missing required field 'name'");

        // Internal error
        let internal_err = JsonRpcError::internal_error(id.clone());
        assert_eq!(internal_err.error.code, INTERNAL_ERROR);
        assert_eq!(internal_err.error.message, "Internal error");

        // Internal error with message
        let internal_err_msg = JsonRpcError::internal_error_with_message(id.clone(), "Database connection failed");
        assert_eq!(internal_err_msg.error.code, INTERNAL_ERROR);
        assert_eq!(internal_err_msg.error.message, "Database connection failed");
    }

    #[test]
    fn test_mcp_specific_errors() {
        let id = json!("req-456");

        // Tool not found
        let tool_err = JsonRpcError::tool_not_found(id.clone(), "my_tool");
        assert_eq!(tool_err.error.code, TOOL_NOT_FOUND);
        assert!(tool_err.error.message.contains("my_tool"));
        assert_eq!(tool_err.error.data, Some(json!({"tool": "my_tool"})));

        // Resource not found
        let resource_err = JsonRpcError::resource_not_found(id.clone(), "file:///test.txt");
        assert_eq!(resource_err.error.code, RESOURCE_NOT_FOUND);
        assert!(resource_err.error.message.contains("file:///test.txt"));
        assert_eq!(
            resource_err.error.data,
            Some(json!({"uri": "file:///test.txt"}))
        );

        // Prompt not found
        let prompt_err = JsonRpcError::prompt_not_found(id.clone(), "test_prompt");
        assert_eq!(prompt_err.error.code, PROMPT_NOT_FOUND);
        assert!(prompt_err.error.message.contains("test_prompt"));
        assert_eq!(
            prompt_err.error.data,
            Some(json!({"prompt": "test_prompt"}))
        );
    }

    #[test]
    fn test_custom_errors() {
        let id = json!(789);

        // Custom error without data
        let custom = JsonRpcError::custom(id.clone(), -32099, "Custom error message");
        assert_eq!(custom.error.code, -32099);
        assert_eq!(custom.error.message, "Custom error message");
        assert!(custom.error.data.is_none());

        // Custom error with data
        let custom_data = json!({"field": "value", "count": 42});
        let custom_with_data = JsonRpcError::custom_with_data(
            id.clone(),
            -32098,
            "Another custom error",
            custom_data.clone(),
        );
        assert_eq!(custom_with_data.error.code, -32098);
        assert_eq!(custom_with_data.error.message, "Another custom error");
        assert_eq!(custom_with_data.error.data, Some(custom_data));
    }

    #[test]
    fn test_conversion_to_message() {
        let id = json!("msg-001");
        let error = JsonRpcError::method_not_found(id);

        // Test From trait
        let message: JsonRpcMessage = error.clone().into();
        assert!(matches!(message, JsonRpcMessage::Error(_)));

        // Test IntoJsonRpcMessage trait
        let message2 = error.into_message();
        assert!(matches!(message2, JsonRpcMessage::Error(_)));
    }

    #[test]
    fn test_error_serialization() {
        let id = json!(42);
        let error = JsonRpcError::invalid_params_with_message(id, "Field 'name' is required");

        let serialized = serde_json::to_value(&error).unwrap();
        assert_eq!(serialized["jsonrpc"], "2.0");
        assert_eq!(serialized["id"], 42);
        assert_eq!(serialized["error"]["code"], INVALID_PARAMS);
        assert_eq!(serialized["error"]["message"], "Field 'name' is required");

        // Test round-trip
        let deserialized: JsonRpcError = serde_json::from_value(serialized).unwrap();
        assert_eq!(deserialized, error);
    }
}
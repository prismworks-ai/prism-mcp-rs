// ! MCP protocol validation utilities (2025-03-26)
// !
// ! Module provides validation functions for MCP protocol messages and types,
// ! ensuring that requests and responses conform to the 2025-03-26 protocol specification,
// ! including support for audio content, annotations, and improved capabilities.

use crate::core::error::{McpError, McpResult};
use crate::protocol::{messages::*, methods, types::*};
use serde_json::Value;

/// Validates that a JSON-RPC message conforms to the specification
pub fn validate_jsonrpc_message(message: &Value) -> McpResult<()> {
    let obj = message
        .as_object()
        .ok_or_else(|| McpError::Validation("Message must be a JSON object".to_string()))?;

    // Check required jsonrpc field
    let jsonrpc = obj
        .get("jsonrpc")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::Validation("Missing or invalid 'jsonrpc' field".to_string()))?;

    if jsonrpc != "2.0" {
        return Err(McpError::Validation("jsonrpc must be '2.0'".to_string()));
    }

    // Check that it has either 'method' (request/notification) or 'result'/'error' (response)
    let has_method = obj.contains_key("method");
    let has_result = obj.contains_key("result");
    let has_error = obj.contains_key("error");
    let has_id = obj.contains_key("id");

    if has_method {
        // Request or notification
        if has_result || has_error {
            return Err(McpError::Validation(
                "Request/notification cannot have 'result' or 'error' fields".to_string(),
            ));
        }

        // Requests must have an id, notifications must not
        // We allow both for flexibility in parsing
    } else if has_result || has_error {
        // Response
        if !has_id {
            return Err(McpError::Validation(
                "Response must have an 'id' field".to_string(),
            ));
        }

        if has_result && has_error {
            return Err(McpError::Validation(
                "Response cannot have both 'result' and 'error' fields".to_string(),
            ));
        }
    } else {
        return Err(McpError::Validation(
            "Message must be a request, response, or notification".to_string(),
        ));
    }

    Ok(())
}

/// Validates a JSON-RPC request
pub fn validate_jsonrpc_request(request: &JsonRpcRequest) -> McpResult<()> {
    if request.jsonrpc != "2.0" {
        return Err(McpError::Validation("jsonrpc must be '2.0'".to_string()));
    }

    if request.method.is_empty() {
        return Err(McpError::Validation(
            "Method name cannot be empty".to_string(),
        ));
    }

    // Method names starting with "rpc." are reserved for JSON-RPC internal methods
    if request.method.starts_with("rpc.") && !request.method.starts_with("rpc.discover") {
        return Err(McpError::Validation(
            "Method names starting with 'rpc.' are reserved".to_string(),
        ));
    }

    Ok(())
}

/// Validates a JSON-RPC response
pub fn validate_jsonrpc_response(response: &JsonRpcResponse) -> McpResult<()> {
    if response.jsonrpc != "2.0" {
        return Err(McpError::Validation("jsonrpc must be '2.0'".to_string()));
    }

    // JsonRpcResponse only has result field, not error
    // Error responses use JsonRpcError type instead
    Ok(())
}

/// Validates a JSON-RPC notification
pub fn validate_jsonrpc_notification(notification: &JsonRpcNotification) -> McpResult<()> {
    if notification.jsonrpc != "2.0" {
        return Err(McpError::Validation("jsonrpc must be '2.0'".to_string()));
    }

    if notification.method.is_empty() {
        return Err(McpError::Validation(
            "Method name cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Validates initialization parameters
pub fn validate_initialize_params(params: &InitializeParams) -> McpResult<()> {
    if params.client_info.name.is_empty() {
        return Err(McpError::Validation(
            "Client name cannot be empty".to_string(),
        ));
    }

    if params.client_info.version.is_empty() {
        return Err(McpError::Validation(
            "Client version cannot be empty".to_string(),
        ));
    }

    if params.protocol_version.is_empty() {
        return Err(McpError::Validation(
            "Protocol version cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Validates tool information (2025-03-26 with annotations)
pub fn validate_tool_info(tool: &Tool) -> McpResult<()> {
    if tool.name.is_empty() {
        return Err(McpError::Validation(
            "Tool name cannot be empty".to_string(),
        ));
    }

    // Validate that input_schema is a valid JSON Schema object
    if tool.input_schema.schema_type != "object" {
        return Err(McpError::Validation(
            "Tool input_schema type must be 'object'".to_string(),
        ));
    }

    // Validate annotations if present
    if let Some(annotations) = &tool.annotations {
        validate_tool_annotations(annotations)?;
    }

    Ok(())
}

/// Validates tool call parameters
pub fn validate_call_tool_params(params: &CallToolParams) -> McpResult<()> {
    if params.name.is_empty() {
        return Err(McpError::Validation(
            "Tool name cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Validates resource information (2025-03-26 with annotations)
pub fn validate_resource_info(resource: &Resource) -> McpResult<()> {
    if resource.uri.is_empty() {
        return Err(McpError::Validation(
            "Resource URI cannot be empty".to_string(),
        ));
    }

    if resource.name.is_empty() {
        return Err(McpError::Validation(
            "Resource name cannot be empty".to_string(),
        ));
    }

    // Basic URI validation - check if it looks like a valid URI
    validate_uri(&resource.uri)?;

    // Validate annotations if present
    if let Some(annotations) = &resource.annotations {
        validate_annotations(annotations)?;
    }

    Ok(())
}

/// Validates resource read parameters
pub fn validate_read_resource_params(params: &ReadResourceParams) -> McpResult<()> {
    if params.uri.is_empty() {
        return Err(McpError::Validation(
            "Resource URI cannot be empty".to_string(),
        ));
    }

    validate_uri(&params.uri)?;

    Ok(())
}

/// Validates resource content (2025-03-26)
pub fn validate_resource_content(content: &ResourceContents) -> McpResult<()> {
    match content {
        ResourceContents::Text { uri, text, .. } => {
            if uri.is_empty() {
                return Err(McpError::Validation(
                    "Resource content URI cannot be empty".to_string(),
                ));
            }
            if text.is_empty() {
                return Err(McpError::Validation(
                    "Text resource content cannot be empty".to_string(),
                ));
            }
        }
        ResourceContents::Blob { uri, blob, .. } => {
            if uri.is_empty() {
                return Err(McpError::Validation(
                    "Resource content URI cannot be empty".to_string(),
                ));
            }
            if blob.is_empty() {
                return Err(McpError::Validation(
                    "Blob resource content cannot be empty".to_string(),
                ));
            }
        }
    }

    Ok(())
}

/// Validates prompt information (2025-03-26)
pub fn validate_prompt_info(prompt: &Prompt) -> McpResult<()> {
    if prompt.name.is_empty() {
        return Err(McpError::Validation(
            "Prompt name cannot be empty".to_string(),
        ));
    }

    if let Some(args) = &prompt.arguments {
        for arg in args {
            if arg.name.is_empty() {
                return Err(McpError::Validation(
                    "Prompt argument name cannot be empty".to_string(),
                ));
            }
        }
    }

    Ok(())
}

/// Validates prompt get parameters
pub fn validate_get_prompt_params(params: &GetPromptParams) -> McpResult<()> {
    if params.name.is_empty() {
        return Err(McpError::Validation(
            "Prompt name cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Validates prompt messages
pub fn validate_prompt_messages(messages: &[PromptMessage]) -> McpResult<()> {
    if messages.is_empty() {
        return Err(McpError::Validation(
            "Prompt must have at least one message".to_string(),
        ));
    }

    for message in messages {
        // Role is an enum, so it can't be empty - validate content instead
        validate_content(&message.content)?;
    }

    Ok(())
}

/// Validates sampling messages
pub fn validate_sampling_messages(messages: &[SamplingMessage]) -> McpResult<()> {
    if messages.is_empty() {
        return Err(McpError::Validation(
            "Sampling request must have at least one message".to_string(),
        ));
    }

    for message in messages {
        // Role is an enum, so it can't be empty - validate content instead
        validate_sampling_content(&message.content)?;
    }

    Ok(())
}

/// Validates create message parameters (2025-03-26)
pub fn validate_create_message_params(params: &CreateMessageParams) -> McpResult<()> {
    validate_sampling_messages(&params.messages)?;

    // max_tokens validation
    if params.max_tokens == 0 {
        return Err(McpError::Validation(
            "max_tokens must be greater than 0".to_string(),
        ));
    }

    // Validate model preferences if present
    if let Some(prefs) = &params.model_preferences {
        validate_model_preferences(prefs)?;
    }

    Ok(())
}

/// Validates sampling content (2025-06-18)
pub fn validate_sampling_content(content: &SamplingContent) -> McpResult<()> {
    match content {
        SamplingContent::Text {
            text, annotations, ..
        } => {
            if text.is_empty() {
                return Err(McpError::Validation(
                    "Text content cannot be empty".to_string(),
                ));
            }
            if let Some(annotations) = annotations {
                validate_annotations(annotations)?;
            }
        }
        SamplingContent::Image {
            data,
            mime_type,
            annotations,
            ..
        } => {
            if data.is_empty() {
                return Err(McpError::Validation(
                    "Image data cannot be empty".to_string(),
                ));
            }
            if !mime_type.starts_with("image/") {
                return Err(McpError::Validation(
                    "Image MIME type must start with 'image/'".to_string(),
                ));
            }
            if let Some(annotations) = annotations {
                validate_annotations(annotations)?;
            }
        }
        SamplingContent::Audio {
            data,
            mime_type,
            annotations,
            ..
        } => {
            if data.is_empty() {
                return Err(McpError::Validation(
                    "Audio data cannot be empty".to_string(),
                ));
            }
            if !mime_type.starts_with("audio/") {
                return Err(McpError::Validation(
                    "Audio MIME type must start with 'audio/'".to_string(),
                ));
            }
            if let Some(annotations) = annotations {
                validate_annotations(annotations)?;
            }
        }
    }
    Ok(())
}

/// Validates content (2025-06-18 with ContentBlock)
pub fn validate_content(content: &ContentBlock) -> McpResult<()> {
    match content {
        ContentBlock::Text {
            text, annotations, ..
        } => {
            if text.is_empty() {
                return Err(McpError::Validation(
                    "Text content cannot be empty".to_string(),
                ));
            }
            if let Some(annotations) = annotations {
                validate_annotations(annotations)?;
            }
        }
        ContentBlock::Image {
            data,
            mime_type,
            annotations,
            ..
        } => {
            if data.is_empty() {
                return Err(McpError::Validation(
                    "Image data cannot be empty".to_string(),
                ));
            }
            if mime_type.is_empty() {
                return Err(McpError::Validation(
                    "Image MIME type cannot be empty".to_string(),
                ));
            }
            if !mime_type.starts_with("image/") {
                return Err(McpError::Validation(
                    "Image MIME type must start with 'image/'".to_string(),
                ));
            }
            if let Some(annotations) = annotations {
                validate_annotations(annotations)?;
            }
        }
        ContentBlock::Audio {
            data,
            mime_type,
            annotations,
            ..
        } => {
            if data.is_empty() {
                return Err(McpError::Validation(
                    "Audio data cannot be empty".to_string(),
                ));
            }
            if mime_type.is_empty() {
                return Err(McpError::Validation(
                    "Audio MIME type cannot be empty".to_string(),
                ));
            }
            if !mime_type.starts_with("audio/") {
                return Err(McpError::Validation(
                    "Audio MIME type must start with 'audio/'".to_string(),
                ));
            }
            if let Some(annotations) = annotations {
                validate_annotations(annotations)?;
            }
        }
        ContentBlock::Resource {
            resource,
            annotations,
            ..
        } => {
            // For embedded resource, validate the ResourceContents
            match resource {
                ResourceContents::Text { uri, text, .. } => {
                    if uri.is_empty() {
                        return Err(McpError::Validation(
                            "Resource URI cannot be empty".to_string(),
                        ));
                    }
                    if text.is_empty() {
                        return Err(McpError::Validation(
                            "Text resource content cannot be empty".to_string(),
                        ));
                    }
                    validate_uri(uri)?;
                }
                ResourceContents::Blob { uri, blob, .. } => {
                    if uri.is_empty() {
                        return Err(McpError::Validation(
                            "Resource URI cannot be empty".to_string(),
                        ));
                    }
                    if blob.is_empty() {
                        return Err(McpError::Validation(
                            "Blob resource content cannot be empty".to_string(),
                        ));
                    }
                    validate_uri(uri)?;
                }
            }
            if let Some(annotations) = annotations {
                validate_annotations(annotations)?;
            }
        }
        ContentBlock::ResourceLink {
            uri,
            name,
            annotations,
            ..
        } => {
            if uri.is_empty() {
                return Err(McpError::Validation(
                    "Resource link URI cannot be empty".to_string(),
                ));
            }
            if name.is_empty() {
                return Err(McpError::Validation(
                    "Resource link name cannot be empty".to_string(),
                ));
            }
            validate_uri(uri)?;
            if let Some(annotations) = annotations {
                validate_annotations(annotations)?;
            }
        }
    }

    Ok(())
}

/// Validates annotations (2025-06-18)
pub fn validate_annotations(annotations: &Annotations) -> McpResult<()> {
    // Validate priority is in valid range
    if let Some(priority) = annotations.priority {
        if !(0.0..=1.0).contains(&priority) {
            return Err(McpError::Validation(
                "Annotation priority must be between 0.0 and 1.0".to_string(),
            ));
        }
    }

    // Validate lastModified is a valid ISO 8601 timestamp (basic check)
    if let Some(last_modified) = &annotations.last_modified {
        if last_modified.is_empty() {
            return Err(McpError::Validation(
                "Annotation lastModified cannot be empty".to_string(),
            ));
        }
        // Could add more complete ISO 8601 validation here
    }

    // Audience validation - all Role enum values are valid
    Ok(())
}

/// Validates tool annotations (2025-06-18 Updated for ToolAnnotations)
pub fn validate_tool_annotations(
    _annotations: &crate::protocol::types::ToolAnnotations,
) -> McpResult<()> {
    // All tool annotation fields are optional hints, so any values are valid
    // Future versions might add specific validation rules
    Ok(())
}

/// Validates completion reference (2025-03-26 NEW)
pub fn validate_completion_reference(reference: &CompletionReference) -> McpResult<()> {
    match reference {
        CompletionReference::Prompt { name } => {
            if name.is_empty() {
                return Err(McpError::Validation(
                    "Completion prompt name cannot be empty".to_string(),
                ));
            }
        }
        CompletionReference::Resource { uri } => {
            if uri.is_empty() {
                return Err(McpError::Validation(
                    "Completion resource URI cannot be empty".to_string(),
                ));
            }
            validate_uri(uri)?;
        }
        CompletionReference::Tool { name } => {
            if name.is_empty() {
                return Err(McpError::Validation(
                    "Completion tool name cannot be empty".to_string(),
                ));
            }
        }
    }

    Ok(())
}

/// Validates completion argument (2025-03-26 NEW)
pub fn validate_completion_argument(argument: &CompletionArgument) -> McpResult<()> {
    if argument.name.is_empty() {
        return Err(McpError::Validation(
            "Completion argument name cannot be empty".to_string(),
        ));
    }

    // Value can be empty (partial input)
    Ok(())
}

/// Validates complete parameters (2025-03-26 NEW)
pub fn validate_complete_params(params: &CompleteParams) -> McpResult<()> {
    validate_completion_reference(&params.reference)?;
    validate_completion_argument(&params.argument)?;

    Ok(())
}

/// Validates root definition (2025-03-26 NEW)
pub fn validate_root(root: &Root) -> McpResult<()> {
    if root.uri.is_empty() {
        return Err(McpError::Validation("Root URI cannot be empty".to_string()));
    }

    // Root URIs must start with file://for now
    if !root.uri.starts_with("file://") {
        return Err(McpError::Validation(
            "Root URI must start with 'file://'".to_string(),
        ));
    }

    Ok(())
}

/// Validates model preferences (2025-03-26 improved)
pub fn validate_model_preferences(preferences: &ModelPreferences) -> McpResult<()> {
    if let Some(cost) = preferences.cost_priority {
        if !(0.0..=1.0).contains(&cost) {
            return Err(McpError::Validation(
                "Cost priority must be between 0.0 and 1.0".to_string(),
            ));
        }
    }

    if let Some(speed) = preferences.speed_priority {
        if !(0.0..=1.0).contains(&speed) {
            return Err(McpError::Validation(
                "Speed priority must be between 0.0 and 1.0".to_string(),
            ));
        }
    }

    if let Some(intelligence) = preferences.intelligence_priority {
        if !(0.0..=1.0).contains(&intelligence) {
            return Err(McpError::Validation(
                "Intelligence priority must be between 0.0 and 1.0".to_string(),
            ));
        }
    }

    Ok(())
}
pub fn validate_uri(uri: &str) -> McpResult<()> {
    if uri.is_empty() {
        return Err(McpError::Validation("URI cannot be empty".to_string()));
    }

    // Basic check for scheme
    if !uri.contains("://") && !uri.starts_with('/') && !uri.starts_with("file:") {
        return Err(McpError::Validation(
            "URI must have a scheme or be an absolute path".to_string(),
        ));
    }

    Ok(())
}

/// Validates method name against MCP specification (2025-03-26)
pub fn validate_method_name(method: &str) -> McpResult<()> {
    if method.is_empty() {
        return Err(McpError::Validation(
            "Method name cannot be empty".to_string(),
        ));
    }

    // Check for valid MCP method patterns (2025-03-26)
    match method {
        methods::INITIALIZE
        | methods::INITIALIZED
        | methods::PING
        | methods::TOOLS_LIST
        | methods::TOOLS_CALL
        | methods::TOOLS_LIST_CHANGED
        | methods::RESOURCES_LIST
        | methods::RESOURCES_TEMPLATES_LIST  // New in 2025-03-26
        | methods::RESOURCES_READ
        | methods::RESOURCES_SUBSCRIBE
        | methods::RESOURCES_UNSUBSCRIBE
        | methods::RESOURCES_UPDATED
        | methods::RESOURCES_LIST_CHANGED
        | methods::PROMPTS_LIST
        | methods::PROMPTS_GET
        | methods::PROMPTS_LIST_CHANGED
        | methods::SAMPLING_CREATE_MESSAGE
        | methods::ROOTS_LIST  // New in 2025-03-26
        | methods::ROOTS_LIST_CHANGED  // New in 2025-03-26
        | methods::COMPLETION_COMPLETE  // New in 2025-03-26
        | methods::LOGGING_SET_LEVEL
        | methods::LOGGING_MESSAGE
        | methods::PROGRESS
        | methods::CANCELLED => Ok(()),  // New in 2025-03-26
        _ => {
            // Allow custom methods if they follow naming conventions
            if method.contains('/') || method.contains('.') {
                Ok(())
            } else {
                Err(McpError::Validation(format!(
                    "Unknown or invalid method name: {method}"
                )))
            }
        }
    }
}

/// Validates server capabilities
pub fn validate_server_capabilities(_capabilities: &ServerCapabilities) -> McpResult<()> {
    // All capability structures are currently valid if they exist
    // Future versions might add validation for specific capability values
    Ok(())
}

/// Validates client capabilities
pub fn validate_client_capabilities(_capabilities: &ClientCapabilities) -> McpResult<()> {
    // All capability structures are currently valid if they exist
    // Future versions might add validation for specific capability values
    Ok(())
}

/// Validates progress parameters (2025-03-26 improved)
pub fn validate_progress_params(params: &ProgressNotificationParams) -> McpResult<()> {
    if !(0.0..=1.0).contains(&params.progress) {
        return Err(McpError::Validation(
            "Progress must be between 0.0 and 1.0".to_string(),
        ));
    }

    Ok(())
}

/// Validates logging message parameters (2025-03-26)
pub fn validate_logging_message_params(params: &LoggingMessageNotificationParams) -> McpResult<()> {
    // Logger name can be empty (optional), but data cannot be null
    if params.data.is_null() {
        return Err(McpError::Validation(
            "Log message data cannot be null".to_string(),
        ));
    }

    Ok(())
}

/// complete validation for any MCP request (2025-03-26)
pub fn validate_mcp_request(method: &str, params: Option<&Value>) -> McpResult<()> {
    validate_method_name(method)?;

    if let Some(params_value) = params {
        match method {
            methods::INITIALIZE => {
                let params: InitializeParams = serde_json::from_value(params_value.clone())
                    .map_err(|e| McpError::Validation(format!("Invalid initialize params: {e}")))?;
                validate_initialize_params(&params)?;
            }
            methods::TOOLS_CALL => {
                let params: CallToolParams = serde_json::from_value(params_value.clone())
                    .map_err(|e| McpError::Validation(format!("Invalid call tool params: {e}")))?;
                validate_call_tool_params(&params)?;
            }
            methods::RESOURCES_READ => {
                let params: ReadResourceParams = serde_json::from_value(params_value.clone())
                    .map_err(|e| {
                        McpError::Validation(format!("Invalid read resource params: {e}"))
                    })?;
                validate_read_resource_params(&params)?;
            }
            methods::PROMPTS_GET => {
                let params: GetPromptParams = serde_json::from_value(params_value.clone())
                    .map_err(|e| McpError::Validation(format!("Invalid get prompt params: {e}")))?;
                validate_get_prompt_params(&params)?;
            }
            methods::SAMPLING_CREATE_MESSAGE => {
                let params: CreateMessageParams = serde_json::from_value(params_value.clone())
                    .map_err(|e| {
                        McpError::Validation(format!("Invalid create message params: {e}"))
                    })?;
                validate_create_message_params(&params)?;
            }
            methods::COMPLETION_COMPLETE => {
                // New in 2025-03-26
                let params: CompleteParams = serde_json::from_value(params_value.clone())
                    .map_err(|e| McpError::Validation(format!("Invalid complete params: {e}")))?;
                validate_complete_params(&params)?;
            }
            methods::PROGRESS => {
                let params: ProgressNotificationParams =
                    serde_json::from_value(params_value.clone()).map_err(|e| {
                        McpError::Validation(format!("Invalid progress params: {e}"))
                    })?;
                validate_progress_params(&params)?;
            }
            methods::LOGGING_MESSAGE => {
                let params: LoggingMessageNotificationParams =
                    serde_json::from_value(params_value.clone()).map_err(|e| {
                        McpError::Validation(format!("Invalid logging message params: {e}"))
                    })?;
                validate_logging_message_params(&params)?;
            }
            _ => {
                // For other methods, we just validate that params is a valid JSON object if present
                if !params_value.is_object() && !params_value.is_null() {
                    return Err(McpError::Validation(
                        "Parameters must be a JSON object or null".to_string(),
                    ));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_jsonrpc_request() {
        let valid_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test_method".to_string(),
            params: None,
        };
        assert!(validate_jsonrpc_request(&valid_request).is_ok());

        let invalid_request = JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            id: json!(1),
            method: "test_method".to_string(),
            params: None,
        };
        assert!(validate_jsonrpc_request(&invalid_request).is_err());
    }

    #[test]
    fn test_validate_uri() {
        assert!(validate_uri("https://example.com").is_ok());
        assert!(validate_uri("file:///path/to/file").is_ok());
        assert!(validate_uri("/absolute/path").is_ok());
        assert!(validate_uri("").is_err());
        assert!(validate_uri("invalid").is_err());
    }

    #[test]
    fn test_validate_tool_info() {
        let valid_tool = Tool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(
                    json!({
                        "param": {"type": "string"}
                    })
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
                ),
                required: None,
                additional_properties: std::collections::HashMap::new(),
            },
            output_schema: None,
            annotations: None,
            title: Some("Test Tool".to_string()),
            meta: None,
        };
        assert!(validate_tool_info(&valid_tool).is_ok());

        let invalid_tool = Tool {
            name: "".to_string(),
            description: None,
            input_schema: ToolInputSchema {
                schema_type: "string".to_string(), // Invalid type
                properties: None,
                required: None,
                additional_properties: std::collections::HashMap::new(),
            },
            output_schema: None,
            annotations: None,
            title: None,
            meta: None,
        };
        assert!(validate_tool_info(&invalid_tool).is_err());
    }

    #[test]
    fn test_validate_create_message_params() {
        let valid_params = CreateMessageParams {
            messages: vec![SamplingMessage::user_text("Hello")],
            model_preferences: None,
            system_prompt: None,
            include_context: None,
            max_tokens: 100,
            temperature: None,
            stop_sequences: None,
            metadata: None,
            meta: None,
        };
        assert!(validate_create_message_params(&valid_params).is_ok());

        let invalid_params = CreateMessageParams {
            messages: vec![],
            model_preferences: None,
            system_prompt: None,
            include_context: None,
            max_tokens: 0, // Invalid max_tokens
            temperature: None,
            stop_sequences: None,
            metadata: None,
            meta: None,
        };
        assert!(validate_create_message_params(&invalid_params).is_err());
    }

    #[test]
    fn test_validate_content() {
        let valid_text = Content::text("Hello, world!");
        assert!(validate_content(&valid_text).is_ok());

        let valid_image = Content::image("base64data", "image/png");
        assert!(validate_content(&valid_image).is_ok());

        // Test new audio content (2025-03-26)
        let valid_audio = Content::audio("base64data", "audio/wav");
        assert!(validate_content(&valid_audio).is_ok());

        let invalid_text = Content::Text {
            text: "".to_string(),
            annotations: None,
            meta: None,
        };
        assert!(validate_content(&invalid_text).is_err());

        let invalid_image = Content::Image {
            data: "data".to_string(),
            mime_type: "text/plain".to_string(), // Invalid MIME type for image
            annotations: None,
            meta: None,
        };
        assert!(validate_content(&invalid_image).is_err());

        let invalid_audio = Content::Audio {
            data: "data".to_string(),
            mime_type: "image/png".to_string(), // Invalid MIME type for audio
            annotations: None,
            meta: None,
        };
        assert!(validate_content(&invalid_audio).is_err());
    }

    #[test]
    fn test_validate_method_name() {
        assert!(validate_method_name(methods::INITIALIZE).is_ok());
        assert!(validate_method_name(methods::TOOLS_LIST).is_ok());
        assert!(validate_method_name("custom/method").is_ok());
        assert!(validate_method_name("custom.method").is_ok());
        assert!(validate_method_name("").is_err());
    }

    #[test]
    fn test_validate_mcp_request() {
        let init_params = json!({
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            },
            "capabilities": {},
            "protocolVersion": "2025-03-26"
        });

        assert!(validate_mcp_request(methods::INITIALIZE, Some(&init_params)).is_ok());
        assert!(validate_mcp_request(methods::PING, None).is_ok());
        assert!(validate_mcp_request("", None).is_err());

        // Test new 2025-03-26 methods
        assert!(validate_mcp_request(methods::ROOTS_LIST, None).is_ok());
        assert!(validate_mcp_request(methods::COMPLETION_COMPLETE, None).is_ok());
        assert!(validate_mcp_request(methods::RESOURCES_TEMPLATES_LIST, None).is_ok());
    }
}

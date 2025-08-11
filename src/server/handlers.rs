// ! MCP server request handlers
// !
// ! Module provides specialized handlers for different types of MCP requests,
// ! implementing the business logic for each protocol operation.

use serde_json::Value;
use std::collections::HashMap;

use crate::core::error::{McpError, McpResult};
use crate::protocol::{LATEST_PROTOCOL_VERSION, messages::*, methods, types::*};

/// Handler for initialization requests
pub struct InitializeHandler;

impl InitializeHandler {
    /// Process an initialize request
    pub async fn handle(
        server_info: &ServerInfo,
        capabilities: &ServerCapabilities,
        params: Option<Value>,
    ) -> McpResult<InitializeResult> {
        let params: InitializeParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::Validation(format!("Invalid initialize params: {e}")))?,
            None => {
                return Err(McpError::Validation(
                    "Missing initialize parameters".to_string(),
                ));
            }
        };

        // Validate protocol version compatibility
        if params.protocol_version != LATEST_PROTOCOL_VERSION {
            let protocol_version = params.protocol_version;
            let expected = LATEST_PROTOCOL_VERSION;
            return Err(McpError::Protocol(format!(
                "Unsupported protocol version: {protocol_version}. Expected: {expected}"
            )));
        }

        // Validate client info
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

        Ok(InitializeResult::new(
            LATEST_PROTOCOL_VERSION.to_string(),
            capabilities.clone(),
            server_info.clone(),
        ))
    }
}

/// Handler for tool-related requests
pub struct ToolHandler;

impl ToolHandler {
    /// Handle tools/list request
    pub async fn handle_list(
        tools: &HashMap<String, crate::core::tool::Tool>,
        params: Option<Value>,
    ) -> McpResult<ListToolsResult> {
        let _params: ListToolsParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::Validation(format!("Invalid list tools params: {e}")))?,
            None => ListToolsParams::default(),
        };

        // Pagination support will be added in future versions
        let tools: Vec<ToolInfo> = tools
            .values()
            .filter(|tool| tool.enabled)
            .map(|tool| {
                // Convert from core::tool::ToolInfo to protocol::types::ToolInfo
                ToolInfo {
                    name: tool.info.name.clone(),
                    description: tool.info.description.clone(),
                    input_schema: tool.info.input_schema.clone(),
                    output_schema: tool.info.output_schema.clone(),
                    annotations: None,
                    title: None,
                    meta: None,
                }
            })
            .collect();

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    /// Handle tools/call request
    pub async fn handle_call(
        tools: &HashMap<String, crate::core::tool::Tool>,
        params: Option<Value>,
    ) -> McpResult<CallToolResult> {
        let params: CallToolParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::Validation(format!("Invalid call tool params: {e}")))?,
            None => {
                return Err(McpError::Validation(
                    "Missing tool call parameters".to_string(),
                ));
            }
        };

        if params.name.is_empty() {
            return Err(McpError::Validation(
                "Tool name cannot be empty".to_string(),
            ));
        }

        let tool = tools
            .get(&params.name)
            .ok_or_else(|| McpError::ToolNotFound(params.name.clone()))?;

        if !tool.enabled {
            let name = &params.name;
            return Err(McpError::ToolNotFound(format!("Tool '{name}' is disabled")));
        }

        let arguments = params.arguments.unwrap_or_default();
        let result = tool.handler.call(arguments).await?;

        Ok(CallToolResult {
            content: result.content,
            is_error: result.is_error,
            structured_content: None,
            meta: None,
        })
    }
}

/// Handler for resource-related requests
pub struct ResourceHandler;

impl ResourceHandler {
    /// Handle resources/list request
    pub async fn handle_list(
        resources: &HashMap<String, crate::core::resource::Resource>,
        params: Option<Value>,
    ) -> McpResult<ListResourcesResult> {
        let _params: ListResourcesParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::Validation(format!("Invalid list resources params: {e}")))?,
            None => ListResourcesParams::default(),
        };

        // Pagination support will be added in future versions
        let resources: Vec<ResourceInfo> = resources
            .values()
            .map(|resource| {
                // Convert from core::resource::ResourceInfo to protocol::types::ResourceInfo
                ResourceInfo {
                    uri: resource.info.uri.clone(),
                    name: resource.info.name.clone(),
                    description: resource.info.description.clone(),
                    mime_type: resource.info.mime_type.clone(),
                    annotations: None,
                    size: None,
                    title: None,
                    meta: None,
                }
            })
            .collect();

        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
        })
    }

    /// Handle resources/read request
    pub async fn handle_read(
        resources: &HashMap<String, crate::core::resource::Resource>,
        params: Option<Value>,
    ) -> McpResult<ReadResourceResult> {
        let params: ReadResourceParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::Validation(format!("Invalid read resource params: {e}")))?,
            None => {
                return Err(McpError::Validation(
                    "Missing resource read parameters".to_string(),
                ));
            }
        };

        if params.uri.is_empty() {
            return Err(McpError::Validation(
                "Resource URI cannot be empty".to_string(),
            ));
        }

        let resource = resources
            .get(&params.uri)
            .ok_or_else(|| McpError::ResourceNotFound(params.uri.clone()))?;

        // Query parameter extraction from URI will be implemented in future versions
        let query_params = HashMap::new();
        let contents = resource.handler.read(&params.uri, &query_params).await?;

        Ok(ReadResourceResult {
            contents,
            meta: None,
        })
    }

    /// Handle resources/subscribe request
    pub async fn handle_subscribe(
        resources: &HashMap<String, crate::core::resource::Resource>,
        params: Option<Value>,
    ) -> McpResult<SubscribeResourceResult> {
        let params: SubscribeResourceParams = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                McpError::Validation(format!("Invalid subscribe resource params: {e}"))
            })?,
            None => {
                return Err(McpError::Validation(
                    "Missing resource subscribe parameters".to_string(),
                ));
            }
        };

        if params.uri.is_empty() {
            return Err(McpError::Validation(
                "Resource URI cannot be empty".to_string(),
            ));
        }

        let resource = resources
            .get(&params.uri)
            .ok_or_else(|| McpError::ResourceNotFound(params.uri.clone()))?;

        resource.handler.subscribe(&params.uri).await?;

        Ok(SubscribeResourceResult { meta: None })
    }

    /// Handle resources/unsubscribe request
    pub async fn handle_unsubscribe(
        resources: &HashMap<String, crate::core::resource::Resource>,
        params: Option<Value>,
    ) -> McpResult<UnsubscribeResourceResult> {
        let params: UnsubscribeResourceParams = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                McpError::Validation(format!("Invalid unsubscribe resource params: {e}"))
            })?,
            None => {
                return Err(McpError::Validation(
                    "Missing resource unsubscribe parameters".to_string(),
                ));
            }
        };

        if params.uri.is_empty() {
            return Err(McpError::Validation(
                "Resource URI cannot be empty".to_string(),
            ));
        }

        let resource = resources
            .get(&params.uri)
            .ok_or_else(|| McpError::ResourceNotFound(params.uri.clone()))?;

        resource.handler.unsubscribe(&params.uri).await?;

        Ok(UnsubscribeResourceResult { meta: None })
    }
}

/// Handler for prompt-related requests
pub struct PromptHandler;

impl PromptHandler {
    /// Handle prompts/list request
    pub async fn handle_list(
        prompts: &HashMap<String, crate::core::prompt::Prompt>,
        params: Option<Value>,
    ) -> McpResult<ListPromptsResult> {
        let _params: ListPromptsParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::Validation(format!("Invalid list prompts params: {e}")))?,
            None => ListPromptsParams::default(),
        };

        // Pagination support will be added in future versions
        let prompts: Vec<PromptInfo> = prompts
            .values()
            .map(|prompt| {
                // Convert from core::prompt::PromptInfo to protocol::types::PromptInfo
                PromptInfo {
                    name: prompt.info.name.clone(),
                    description: prompt.info.description.clone(),
                    arguments: prompt.info.arguments.as_ref().map(|args| {
                        args.iter()
                            .map(|arg| PromptArgument {
                                name: arg.name.clone(),
                                description: arg.description.clone(),
                                required: arg.required,
                                title: None,
                            })
                            .collect()
                    }),
                    title: None,
                    meta: None,
                }
            })
            .collect();

        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
            meta: None,
        })
    }

    /// Handle prompts/get request
    pub async fn handle_get(
        prompts: &HashMap<String, crate::core::prompt::Prompt>,
        params: Option<Value>,
    ) -> McpResult<GetPromptResult> {
        let params: GetPromptParams = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::Validation(format!("Invalid get prompt params: {e}")))?,
            None => {
                return Err(McpError::Validation(
                    "Missing prompt get parameters".to_string(),
                ));
            }
        };

        if params.name.is_empty() {
            return Err(McpError::Validation(
                "Prompt name cannot be empty".to_string(),
            ));
        }

        let prompt = prompts
            .get(&params.name)
            .ok_or_else(|| McpError::PromptNotFound(params.name.clone()))?;

        let arguments = params
            .arguments
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        let result = prompt.handler.get(arguments).await?;

        Ok(GetPromptResult {
            description: result.description,
            messages: result
                .messages
                .into_iter()
                .map(|msg| {
                    // Convert from core::prompt::PromptMessage to protocol::types::PromptMessage
                    PromptMessage {
                        role: msg.role,
                        content: match msg.content {
                            ContentBlock::Text { text, .. } => ContentBlock::Text {
                                text,
                                annotations: None,
                                meta: None,
                            },
                            ContentBlock::Image {
                                data, mime_type, ..
                            } => ContentBlock::Image {
                                data,
                                mime_type,
                                annotations: None,
                                meta: None,
                            },
                            other => other,
                        },
                    }
                })
                .collect(),
            meta: None,
        })
    }
}

/// Handler for sampling requests
pub struct SamplingHandler;

impl SamplingHandler {
    /// Handle sampling/createMessage request
    pub async fn handle_create_message(_params: Option<Value>) -> McpResult<CreateMessageResult> {
        // Note: Sampling is typically handled by the client side (LLM),
        // but servers can provide sampling capabilities if they have access to LLMs
        Err(McpError::Protocol(
            "Sampling not implemented on server side".to_string(),
        ))
    }
}

/// Handler for logging requests
pub struct LoggingHandler;

impl LoggingHandler {
    /// Handle logging/setLevel request
    pub async fn handle_set_level(params: Option<Value>) -> McpResult<SetLoggingLevelResult> {
        let _params: SetLoggingLevelParams = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                McpError::Validation(format!("Invalid set logging level params: {e}"))
            })?,
            None => {
                return Err(McpError::Validation(
                    "Missing logging level parameters".to_string(),
                ));
            }
        };

        // Logging level management feature planned for future implementation
        // This would typically integrate with a logging framework like tracing

        Ok(SetLoggingLevelResult { meta: None })
    }
}

/// Handler for ping requests
pub struct PingHandler;

impl PingHandler {
    /// Handle ping request
    pub async fn handle(_params: Option<Value>) -> McpResult<PingResult> {
        Ok(PingResult { meta: None })
    }
}

/// Helper functions for common validation patterns
pub mod validation {
    use super::*;

    /// Validate that required parameters are present
    pub fn require_params<T>(params: Option<Value>, error_msg: &str) -> McpResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::Validation(format!("{error_msg}: {e}"))),
            None => Err(McpError::Validation(error_msg.to_string())),
        }
    }

    /// Validate that a string parameter is not empty
    pub fn require_non_empty_string(value: &str, field_name: &str) -> McpResult<()> {
        if value.is_empty() {
            Err(McpError::Validation(format!(
                "{field_name} cannot be empty"
            )))
        } else {
            Ok(())
        }
    }

    /// Validate URI format
    pub fn validate_uri_format(uri: &str) -> McpResult<()> {
        if uri.is_empty() {
            return Err(McpError::Validation("URI cannot be empty".to_string()));
        }

        // Basic URI validation - check for scheme or absolute path
        if !uri.contains("://") && !uri.starts_with('/') && !uri.starts_with("file:") {
            return Err(McpError::Validation(
                "URI must have a scheme or be an absolute path".to_string(),
            ));
        }

        Ok(())
    }
}

/// Notification builders for common server events
pub mod notifications {
    use super::*;

    /// Create a tools list changed notification
    pub fn tools_list_changed() -> McpResult<JsonRpcNotification> {
        Ok(JsonRpcNotification::new(
            methods::TOOLS_LIST_CHANGED.to_string(),
            Some(ToolListChangedParams { meta: None }),
        )?)
    }

    /// Create a resources list changed notification
    pub fn resources_list_changed() -> McpResult<JsonRpcNotification> {
        Ok(JsonRpcNotification::new(
            methods::RESOURCES_LIST_CHANGED.to_string(),
            Some(ResourceListChangedParams { meta: None }),
        )?)
    }

    /// Create a prompts list changed notification
    pub fn prompts_list_changed() -> McpResult<JsonRpcNotification> {
        Ok(JsonRpcNotification::new(
            methods::PROMPTS_LIST_CHANGED.to_string(),
            Some(PromptListChangedParams { meta: None }),
        )?)
    }

    /// Create a resource updated notification
    pub fn resource_updated(uri: String) -> McpResult<JsonRpcNotification> {
        Ok(JsonRpcNotification::new(
            methods::RESOURCES_UPDATED.to_string(),
            Some(ResourceUpdatedParams { uri }),
        )?)
    }

    /// Create a progress notification
    pub fn progress(
        progress_token: String,
        progress: f32,
        total: Option<f32>,
    ) -> McpResult<JsonRpcNotification> {
        Ok(JsonRpcNotification::new(
            methods::PROGRESS.to_string(),
            Some(ProgressParams {
                progress_token: serde_json::Value::String(progress_token),
                progress,
                total,
                message: None,
            }),
        )?)
    }

    /// Create a logging message notification
    pub fn log_message(
        level: LoggingLevel,
        logger: Option<String>,
        data: Value,
    ) -> McpResult<JsonRpcNotification> {
        Ok(JsonRpcNotification::new(
            methods::LOGGING_MESSAGE.to_string(),
            Some(LoggingMessageParams {
                level,
                logger,
                data,
            }),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_initialize_handler() {
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            title: Some("Test Server".to_string()),
        };
        let capabilities = ServerCapabilities::default();

        let params = json!({
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            },
            "capabilities": {},
            "protocolVersion": LATEST_PROTOCOL_VERSION
        });

        let result = InitializeHandler::handle(&server_info, &capabilities, Some(params)).await;
        assert!(result.is_ok());

        let init_result = result.unwrap();
        assert_eq!(init_result.server_info.name, "test-server");
        assert_eq!(init_result.protocol_version, LATEST_PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn test_ping_handler() {
        let result = PingHandler::handle(None).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_helpers() {
        // Test require_non_empty_string
        assert!(validation::require_non_empty_string("test", "field").is_ok());
        assert!(validation::require_non_empty_string("", "field").is_err());

        // Test validate_uri_format
        assert!(validation::validate_uri_format("https://example.com").is_ok());
        assert!(validation::validate_uri_format("file:///path").is_ok());
        assert!(validation::validate_uri_format("/absolute/path").is_ok());
        assert!(validation::validate_uri_format("").is_err());
        assert!(validation::validate_uri_format("invalid").is_err());
    }

    #[test]
    fn test_notification_builders() {
        assert!(notifications::tools_list_changed().is_ok());
        assert!(notifications::resources_list_changed().is_ok());
        assert!(notifications::prompts_list_changed().is_ok());
        assert!(notifications::resource_updated("file:///test".to_string()).is_ok());
        assert!(notifications::progress("token".to_string(), 0.5, Some(100.0)).is_ok());
        assert!(
            notifications::log_message(
                LoggingLevel::Info,
                Some("test".to_string()),
                json!({"message": "test log"})
            )
            .is_ok()
        );
    }
}

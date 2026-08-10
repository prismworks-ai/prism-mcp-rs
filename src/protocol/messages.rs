//! MCP Protocol Messages
//!
//! Module defines all protocol message types used in MCP communication,
//! aligned with the 2025-03-26 specification.

use crate::protocol::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Request Parameter Types
// ============================================================================

/// Parameters for initialize request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeParams {
    /// Protocol version client supports
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Client implementation info
    #[serde(rename = "clientInfo")]
    pub client_info: Implementation,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for tool call request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallToolParams {
    /// Name of the tool to call
    pub name: String,
    /// Arguments to pass to the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, serde_json::Value>>,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for resource read request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadResourceParams {
    /// URI of the resource to read
    pub uri: String,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for resource subscription request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscribeResourceParams {
    /// URI of the resource to subscribe to
    pub uri: String,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for resource unsubscription request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnsubscribeResourceParams {
    /// URI of the resource to unsubscribe from
    pub uri: String,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for prompt get request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptParams {
    /// Name of the prompt
    pub name: String,
    /// Arguments for prompt templating
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, String>>,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for list requests (with pagination)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListParams {
    /// Pagination cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for list tools request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ListToolsParams {
    /// Pagination cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for list resources request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ListResourcesParams {
    /// Pagination cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for list prompts request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ListPromptsParams {
    /// Pagination cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for ping request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PingParams {
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for list resource templates request (New in 2025-11-25)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ListResourceTemplatesParams {
    /// Pagination cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for list roots request (New in 2025-11-25)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ListRootsParams {
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for completion request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteParams {
    /// Reference to the item being completed
    #[serde(rename = "ref")]
    pub reference: CompletionReference,
    /// Argument being completed
    pub argument: CompletionArgument,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Reference for completion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum CompletionReference {
    #[serde(rename = "ref/prompt")]
    Prompt { name: String },
    #[serde(rename = "ref/resource")]
    Resource { uri: String },
    #[serde(rename = "ref/tool")]
    Tool { name: String },
}

/// Argument for completion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionArgument {
    /// Name of the argument
    pub name: String,
    /// Current value for completion
    pub value: String,
}

/// Parameters for sampling/createMessage request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateMessageParams {
    /// Messages in the conversation
    pub messages: Vec<SamplingMessage>,
    /// Maximum tokens to generate
    #[serde(rename = "maxTokens")]
    pub max_tokens: u32,
    /// Optional system prompt
    #[serde(rename = "systemPrompt", skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Include context from servers
    #[serde(rename = "includeContext", skip_serializing_if = "Option::is_none")]
    pub include_context: Option<String>,
    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Stop sequences
    #[serde(rename = "stopSequences", skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Model preferences
    #[serde(rename = "modelPreferences", skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<ModelPreferences>,
    /// Optional tools that the model may call while sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Optional strategy for choosing tools during sampling.
    #[serde(rename = "toolChoice", skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<SamplingToolChoice>,
    /// Provider-specific metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for logging level set request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLoggingLevelParams {
    /// Logging level to set
    pub level: LoggingLevel,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for elicitation request (2025-11-25 NEW)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElicitParams {
    /// Message to present to the user
    pub message: String,
    /// Interaction mode used by the client UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ElicitationMode>,
    /// URL to open when mode is `url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Schema describing the requested form fields
    #[serde(rename = "requestedSchema", skip_serializing_if = "Option::is_none")]
    pub requested_schema: Option<ElicitationSchema>,
    /// Request metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Elicitation interaction mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationMode {
    Form,
    Url,
}

// ============================================================================
// Response Result Types
// ============================================================================

/// Result for initialize request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeResult {
    /// Protocol version server supports
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Server implementation info
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,
    /// Optional instructions for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for list tools request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListToolsResult {
    /// Available tools
    pub tools: Vec<Tool>,
    /// Next cursor for pagination
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for list resources request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResourcesResult {
    /// Available resources
    pub resources: Vec<Resource>,
    /// Next cursor for pagination
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for list resource templates request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResourceTemplatesResult {
    /// Available resource templates
    #[serde(rename = "resourceTemplates")]
    pub resource_templates: Vec<ResourceTemplate>,
    /// Next cursor for pagination
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for read resource request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadResourceResult {
    /// Resource contents
    pub contents: Vec<ResourceContents>,
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for list prompts request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListPromptsResult {
    /// Available prompts
    pub prompts: Vec<Prompt>,
    /// Next cursor for pagination
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for completion request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteResult {
    /// Completion information
    pub completion: CompletionData,
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Completion data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionData {
    /// Completion values
    pub values: Vec<String>,
    /// Total number of completions available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
    /// Whether there are more completions available
    #[serde(rename = "hasMore", skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

/// Result for list roots request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListRootsResult {
    /// Available roots
    pub roots: Vec<Root>,
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for ping request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PingResult {
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for set logging level request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLoggingLevelResult {
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for elicitation request (2025-11-25 NEW)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElicitResult {
    /// User action in response to elicitation
    pub action: ElicitationAction,
    /// Submitted form data (only present when action is "accept")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, serde_json::Value>>,
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for subscribe resource request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscribeResourceResult {
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Result for unsubscribe resource request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnsubscribeResourceResult {
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Root definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Root {
    /// URI of the root
    pub uri: String,
    /// Optional name for the root
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// ============================================================================
// Notification Parameter Types
// ============================================================================

/// Parameters for progress notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressParams {
    /// Progress token from original request
    #[serde(rename = "progressToken")]
    pub progress_token: ProgressToken,
    /// Current progress value
    pub progress: f32,
    /// Total progress expected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f32>,
    /// Optional progress message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Parameters for resource updated notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceUpdatedParams {
    /// URI of the updated resource
    pub uri: String,
}

/// Parameters for cancelled notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CancelledParams {
    /// ID of the request being cancelled
    #[serde(rename = "requestId")]
    pub request_id: RequestId,
    /// Optional reason for cancellation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Parameters for initialized notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializedParams {
    /// Notification metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for logging message notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingMessageParams {
    /// Logging level
    pub level: LoggingLevel,
    /// Logger name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
    /// Log data
    pub data: serde_json::Value,
}

/// Parameters for tool list changed notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolListChangedParams {
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for resource list changed notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceListChangedParams {
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for prompt list changed notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptListChangedParams {
    /// Response metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for elicitation completion notification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElicitationCompleteParams {
    /// Final elicitation result payload.
    pub result: ElicitResult,
}

/// Parameters for tasks/send.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskSendParams {
    /// Task identifier.
    #[serde(rename = "taskId")]
    pub task_id: String,
    /// Task payload.
    pub task: serde_json::Value,
    /// Request metadata.
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for tasks/cancel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCancelParams {
    /// Task identifier.
    #[serde(rename = "taskId")]
    pub task_id: String,
    /// Request metadata.
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Parameters for notifications/tasks/status/update.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskStatusUpdateParams {
    /// Task identifier.
    #[serde(rename = "taskId")]
    pub task_id: String,
    /// Task status value.
    pub status: String,
    /// Optional status detail payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

/// Parameters for progress notification (alias for better naming)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressNotificationParams {
    /// Progress token from original request
    #[serde(rename = "progressToken")]
    pub progress_token: ProgressToken,
    /// Current progress value
    pub progress: f32,
    /// Total progress expected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f32>,
    /// Optional progress message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Parameters for logging message notification (alias for better naming)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingMessageNotificationParams {
    /// Logging level
    pub level: LoggingLevel,
    /// Logger name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
    /// Log data
    pub data: serde_json::Value,
}

// ============================================================================
// Helper Constructors
// ============================================================================

impl CallToolParams {
    pub fn new(name: String) -> Self {
        Self {
            name,
            arguments: None,
            meta: None,
        }
    }

    pub fn new_with_arguments(name: String, arguments: HashMap<String, serde_json::Value>) -> Self {
        Self {
            name,
            arguments: Some(arguments),
            meta: None,
        }
    }

    pub fn with_arguments(mut self, arguments: HashMap<String, serde_json::Value>) -> Self {
        self.arguments = Some(arguments);
        self
    }
}

impl ReadResourceParams {
    pub fn new(uri: String) -> Self {
        Self { uri, meta: None }
    }
}

impl GetPromptParams {
    pub fn new(name: String) -> Self {
        Self {
            name,
            arguments: None,
            meta: None,
        }
    }

    pub fn new_with_arguments(name: String, arguments: HashMap<String, String>) -> Self {
        Self {
            name,
            arguments: Some(arguments),
            meta: None,
        }
    }

    pub fn with_arguments(mut self, arguments: HashMap<String, String>) -> Self {
        self.arguments = Some(arguments);
        self
    }
}

impl InitializeParams {
    pub fn new(
        protocol_version: String,
        capabilities: ClientCapabilities,
        client_info: Implementation,
    ) -> Self {
        Self {
            protocol_version,
            capabilities,
            client_info,
            meta: None,
        }
    }
}

impl InitializeResult {
    pub fn new(
        protocol_version: String,
        capabilities: ServerCapabilities,
        server_info: Implementation,
    ) -> Self {
        Self {
            protocol_version,
            capabilities,
            server_info,
            instructions: None,
            meta: None,
        }
    }
}

impl CreateMessageParams {
    /// Create minimal sampling/createMessage params.
    pub fn new(messages: Vec<SamplingMessage>, max_tokens: u32) -> Self {
        Self {
            messages,
            max_tokens,
            system_prompt: None,
            include_context: None,
            temperature: None,
            stop_sequences: None,
            model_preferences: None,
            tools: None,
            tool_choice: None,
            metadata: None,
            meta: None,
        }
    }

    /// Set system prompt.
    pub fn with_system_prompt<S: Into<String>>(mut self, system_prompt: S) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    /// Set context inclusion strategy.
    pub fn with_include_context<S: Into<String>>(mut self, include_context: S) -> Self {
        self.include_context = Some(include_context.into());
        self
    }

    /// Set sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set stop sequences.
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Set model preferences.
    pub fn with_model_preferences(mut self, model_preferences: ModelPreferences) -> Self {
        self.model_preferences = Some(model_preferences);
        self
    }

    /// Set available tools for sampling.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Add a single available tool for sampling.
    pub fn add_tool(mut self, tool: Tool) -> Self {
        self.tools.get_or_insert_with(Vec::new).push(tool);
        self
    }

    /// Set tool choice behavior for sampling.
    pub fn with_tool_choice(mut self, tool_choice: SamplingToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Set provider metadata.
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set request metadata.
    pub fn with_meta(mut self, meta: HashMap<String, serde_json::Value>) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl ElicitParams {
    /// Create a minimal elicitation request.
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
            mode: None,
            url: None,
            requested_schema: None,
            meta: None,
        }
    }

    /// Create a form-based elicitation request.
    pub fn form<S: Into<String>>(message: S, requested_schema: ElicitationSchema) -> Self {
        Self {
            message: message.into(),
            mode: Some(ElicitationMode::Form),
            url: None,
            requested_schema: Some(requested_schema),
            meta: None,
        }
    }

    /// Create a URL-based elicitation request.
    pub fn url<M: Into<String>, U: Into<String>>(message: M, url: U) -> Self {
        Self {
            message: message.into(),
            mode: Some(ElicitationMode::Url),
            url: Some(url.into()),
            requested_schema: None,
            meta: None,
        }
    }

    /// Set elicitation mode.
    pub fn with_mode(mut self, mode: ElicitationMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Set URL for URL-mode elicitation.
    pub fn with_url<S: Into<String>>(mut self, url: S) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set requested form schema for form-mode elicitation.
    pub fn with_requested_schema(mut self, requested_schema: ElicitationSchema) -> Self {
        self.requested_schema = Some(requested_schema);
        self
    }

    /// Set request metadata.
    pub fn with_meta(mut self, meta: HashMap<String, serde_json::Value>) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl ElicitResult {
    /// Create an accepted elicitation result.
    pub fn accepted(content: HashMap<String, serde_json::Value>) -> Self {
        Self {
            action: ElicitationAction::Accept,
            content: Some(content),
            meta: None,
        }
    }

    /// Create a declined elicitation result.
    pub fn declined() -> Self {
        Self {
            action: ElicitationAction::Decline,
            content: None,
            meta: None,
        }
    }

    /// Create a cancelled elicitation result.
    pub fn cancelled() -> Self {
        Self {
            action: ElicitationAction::Cancel,
            content: None,
            meta: None,
        }
    }

    /// Set response metadata.
    pub fn with_meta(mut self, meta: HashMap<String, serde_json::Value>) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl ElicitationCompleteParams {
    /// Create a new elicitation completion notification payload.
    pub fn new(result: ElicitResult) -> Self {
        Self { result }
    }
}

impl TaskSendParams {
    /// Create a new task send request payload.
    pub fn new<S: Into<String>>(task_id: S, task: serde_json::Value) -> Self {
        Self {
            task_id: task_id.into(),
            task,
            meta: None,
        }
    }

    /// Set request metadata.
    pub fn with_meta(mut self, meta: HashMap<String, serde_json::Value>) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl TaskCancelParams {
    /// Create a new task cancel request payload.
    pub fn new<S: Into<String>>(task_id: S) -> Self {
        Self {
            task_id: task_id.into(),
            meta: None,
        }
    }

    /// Set request metadata.
    pub fn with_meta(mut self, meta: HashMap<String, serde_json::Value>) -> Self {
        self.meta = Some(meta);
        self
    }
}

impl TaskStatusUpdateParams {
    /// Create a new task status update notification payload.
    pub fn new<T: Into<String>, S: Into<String>>(task_id: T, status: S) -> Self {
        Self {
            task_id: task_id.into(),
            status: status.into(),
            detail: None,
        }
    }

    /// Set status detail payload.
    pub fn with_detail(mut self, detail: serde_json::Value) -> Self {
        self.detail = Some(detail);
        self
    }
}

impl Root {
    pub fn new(uri: String) -> Self {
        Self { uri, name: None }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

// ============================================================================
// Default Implementations
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_initialize_params_serialization() {
        let params = InitializeParams {
            protocol_version: "2025-11-25".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                title: Some("Test Client".to_string()),
                website_url: None,
                icons: None,
            },
            meta: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["protocolVersion"], "2025-11-25");
        assert_eq!(json["clientInfo"]["name"], "test-client");

        // Test round-trip
        let deserialized: InitializeParams = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.protocol_version, "2025-11-25");
    }

    #[test]
    fn test_initialize_result_serialization() {
        let result = InitializeResult {
            protocol_version: "2025-11-25".to_string(),
            capabilities: ServerCapabilities::default(),
            server_info: Implementation {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                title: Some("Test Server".to_string()),
                website_url: None,
                icons: None,
            },
            instructions: Some("Test instructions".to_string()),
            meta: None,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["protocolVersion"], "2025-11-25");
        assert_eq!(json["serverInfo"]["name"], "test-server");
        assert_eq!(json["instructions"], "Test instructions");
    }

    #[test]
    fn test_call_tool_params_serialization() {
        let mut args = HashMap::new();
        args.insert("input".to_string(), json!("test value"));

        let params = CallToolParams {
            name: "test_tool".to_string(),
            arguments: Some(args),
            meta: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["name"], "test_tool");
        assert_eq!(json["arguments"]["input"], "test value");
    }

    #[test]
    fn test_call_tool_result_serialization() {
        let result = CallToolResult {
            content: vec![ContentBlock::text("Tool executed successfully")],
            is_error: Some(false),
            structured_content: Some(json!({"status": "success", "data": 42})),
            meta: None,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["isError"], false);
        assert_eq!(json["structuredContent"]["status"], "success");
        assert_eq!(json["structuredContent"]["data"], 42);
    }

    #[test]
    fn test_list_tools_params_serialization() {
        let params = ListToolsParams {
            cursor: Some("page2".to_string()),
            meta: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["cursor"], "page2");
    }

    #[test]
    fn test_list_tools_result_serialization() {
        let tool = Tool::new("test_tool", "A test tool").with_title("Test Tool");

        let result = ListToolsResult {
            tools: vec![tool],
            next_cursor: Some("page3".to_string()),
            meta: None,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert!(json["tools"].is_array());
        assert_eq!(json["tools"][0]["name"], "test_tool");
        assert_eq!(json["nextCursor"], "page3");
    }

    #[test]
    fn test_create_message_params_serialization() {
        let message = SamplingMessage::user_text("Hello, AI!");

        let params = CreateMessageParams {
            messages: vec![message],
            max_tokens: 1000,
            system_prompt: Some("You are helpful".to_string()),
            include_context: Some("thisServer".to_string()),
            temperature: Some(0.7),
            stop_sequences: None,
            model_preferences: None,
            tools: None,
            tool_choice: None,
            metadata: None,
            meta: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["maxTokens"], 1000);
        assert_eq!(json["systemPrompt"], "You are helpful");
        assert_eq!(json["includeContext"], "thisServer");
        assert!((json["temperature"].as_f64().unwrap() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_create_message_result_serialization() {
        let result = CreateMessageResult {
            role: Role::Assistant,
            content: SamplingContent::text("AI response here"),
            model: "claude-3-5-sonnet".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            meta: None,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["role"], "assistant");
        assert_eq!(json["content"]["text"], "AI response here");
        assert_eq!(json["model"], "claude-3-5-sonnet");
        assert_eq!(json["stopReason"], "endTurn");
    }

    #[test]
    fn test_list_resources_params_serialization() {
        let params = ListResourcesParams {
            cursor: None,
            meta: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert!(json.is_object());
    }

    #[test]
    fn test_read_resource_params_serialization() {
        let params = ReadResourceParams {
            uri: "file:///test.txt".to_string(),
            meta: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["uri"], "file:///test.txt");
    }

    #[test]
    fn test_list_prompts_result_serialization() {
        let prompt = Prompt::new("test_prompt")
            .with_title("Test Prompt")
            .with_description("A test prompt");

        let result = ListPromptsResult {
            prompts: vec![prompt],
            next_cursor: None,
            meta: None,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert!(json["prompts"].is_array());
        assert_eq!(json["prompts"][0]["name"], "test_prompt");
    }

    #[test]
    fn test_get_prompt_params_serialization() {
        let mut arguments = HashMap::new();
        arguments.insert("input".to_string(), "test input".to_string());

        let params = GetPromptParams {
            name: "test_prompt".to_string(),
            arguments: Some(arguments),
            meta: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["name"], "test_prompt");
        assert_eq!(json["arguments"]["input"], "test input");
    }

    #[test]
    fn test_progress_params_serialization() {
        let params = ProgressParams {
            progress_token: json!("upload-123"),
            progress: 75.5,
            total: Some(100.0),
            message: Some("Uploading files...".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["progressToken"], "upload-123");
        assert_eq!(json["progress"], 75.5);
        assert_eq!(json["total"], 100.0);
        assert_eq!(json["message"], "Uploading files...");
    }

    #[test]
    fn test_complete_params_serialization() {
        let reference = CompletionReference::Prompt {
            name: "test_prompt".to_string(),
        };

        let params = CompleteParams {
            reference,
            argument: CompletionArgument {
                name: "input".to_string(),
                value: "partial".to_string(),
            },
            meta: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["ref"]["type"], "ref/prompt");
        assert_eq!(json["ref"]["name"], "test_prompt");
        assert_eq!(json["argument"]["name"], "input");
        assert_eq!(json["argument"]["value"], "partial");
    }

    #[test]
    fn test_all_message_roundtrip_compatibility() {
        // Test that all message types can be serialized and deserialized
        let test_cases = vec![
            (
                "InitializeParams",
                json!({
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test",
                        "version": "1.0.0"
                    }
                }),
            ),
            ("ListToolsParams", json!({})),
            (
                "CallToolParams",
                json!({
                    "name": "test_tool",
                    "arguments": {"input": "test"}
                }),
            ),
        ];

        for (type_name, test_json) in test_cases {
            match type_name {
                "InitializeParams" => {
                    let _: InitializeParams = serde_json::from_value(test_json).unwrap();
                }
                "ListToolsParams" => {
                    let _: ListToolsParams = serde_json::from_value(test_json).unwrap();
                }
                "CallToolParams" => {
                    let _: CallToolParams = serde_json::from_value(test_json).unwrap();
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_message_metadata_handling() {
        let mut meta = HashMap::new();
        meta.insert("custom".to_string(), json!("value"));

        let params = InitializeParams {
            protocol_version: "2025-11-25".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation::new("test", "1.0.0"),
            meta: Some(meta),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["_meta"]["custom"], "value");
    }

    #[test]
    fn test_constructor_methods() {
        // Test CallToolParams constructors
        let tool_params = CallToolParams::new("test_tool".to_string());
        assert_eq!(tool_params.name, "test_tool");
        assert!(tool_params.arguments.is_none());

        let mut args = HashMap::new();
        args.insert("key".to_string(), json!("value"));
        let tool_params_with_args =
            CallToolParams::new_with_arguments("test_tool".to_string(), args.clone());
        assert_eq!(tool_params_with_args.name, "test_tool");
        assert_eq!(tool_params_with_args.arguments, Some(args));

        // Test ReadResourceParams constructor
        let resource_params = ReadResourceParams::new("file:///test.txt".to_string());
        assert_eq!(resource_params.uri, "file:///test.txt");

        // Test GetPromptParams constructors
        let prompt_params = GetPromptParams::new("test_prompt".to_string());
        assert_eq!(prompt_params.name, "test_prompt");
        assert!(prompt_params.arguments.is_none());

        let mut str_args = HashMap::new();
        str_args.insert("key".to_string(), "value".to_string());
        let prompt_params_with_args =
            GetPromptParams::new_with_arguments("test_prompt".to_string(), str_args.clone());
        assert_eq!(prompt_params_with_args.name, "test_prompt");
        assert_eq!(prompt_params_with_args.arguments, Some(str_args));

        // Test Root constructor
        let root = Root::new("file:///root".to_string());
        assert_eq!(root.uri, "file:///root");
        assert!(root.name.is_none());

        let root_with_name = root.with_name("Root Name".to_string());
        assert_eq!(root_with_name.name, Some("Root Name".to_string()));

        // Test CreateMessageParams constructors/builders
        let create_params =
            CreateMessageParams::new(vec![SamplingMessage::user_text("Hello")], 256)
                .with_system_prompt("You are helpful")
                .with_include_context("thisServer")
                .with_temperature(0.5)
                .with_stop_sequences(vec!["STOP".to_string()])
                .with_tool_choice(SamplingToolChoice::Auto)
                .add_tool(Tool::new("echo", "Echo tool"));
        assert_eq!(create_params.max_tokens, 256);
        assert_eq!(
            create_params.system_prompt,
            Some("You are helpful".to_string())
        );
        assert_eq!(create_params.tools.as_ref().map(Vec::len), Some(1));
        assert_eq!(create_params.tool_choice, Some(SamplingToolChoice::Auto));

        // Test ElicitParams constructors/builders
        let mut properties = HashMap::new();
        properties.insert(
            "name".to_string(),
            PrimitiveSchemaDefinition::String {
                title: Some("Name".to_string()),
                description: None,
                min_length: None,
                max_length: None,
                format: None,
                enum_values: None,
                enum_names: None,
            },
        );
        let schema = ElicitationSchema {
            schema_type: "object".to_string(),
            properties,
            required: Some(vec!["name".to_string()]),
        };
        let form_elicit = ElicitParams::form("Need your name", schema.clone());
        assert_eq!(form_elicit.mode, Some(ElicitationMode::Form));
        assert!(form_elicit.url.is_none());
        assert!(form_elicit.requested_schema.is_some());

        let url_elicit = ElicitParams::url("Continue setup", "https://example.com/setup");
        assert_eq!(url_elicit.mode, Some(ElicitationMode::Url));
        assert_eq!(
            url_elicit.url,
            Some("https://example.com/setup".to_string())
        );
        assert!(url_elicit.requested_schema.is_none());

        // Test ElicitResult and completion params constructors
        let accepted_result =
            ElicitResult::accepted(HashMap::from([("name".to_string(), json!("Alice"))]));
        assert_eq!(accepted_result.action, ElicitationAction::Accept);
        assert!(accepted_result.content.is_some());

        let completion = ElicitationCompleteParams::new(ElicitResult::declined());
        assert_eq!(completion.result.action, ElicitationAction::Decline);

        // Test task message constructors
        let task_send = TaskSendParams::new("task-123", json!({"kind":"review"}));
        assert_eq!(task_send.task_id, "task-123");

        let task_cancel = TaskCancelParams::new("task-123");
        assert_eq!(task_cancel.task_id, "task-123");

        let task_status =
            TaskStatusUpdateParams::new("task-123", "running").with_detail(json!({"progress": 42}));
        assert_eq!(task_status.status, "running");
        assert!(task_status.detail.is_some());
    }

    #[test]
    fn test_new_2025_06_18_message_types() {
        // Test ListResourceTemplatesParams
        let templates_params = ListResourceTemplatesParams {
            cursor: Some("cursor123".to_string()),
            meta: None,
        };
        let json = serde_json::to_value(&templates_params).unwrap();
        assert_eq!(json["cursor"], "cursor123");

        // Test ListRootsParams
        let roots_params = ListRootsParams { meta: None };
        let json = serde_json::to_value(&roots_params).unwrap();
        assert!(json.is_object());

        // Test ElicitParams
        let mut properties = HashMap::new();
        properties.insert(
            "name".to_string(),
            PrimitiveSchemaDefinition::String {
                title: Some("Your Name".to_string()),
                description: Some("Enter your full name".to_string()),
                min_length: None,
                max_length: None,
                format: None,
                enum_values: None,
                enum_names: None,
            },
        );

        let elicit_params = ElicitParams {
            message: "Please fill out the form".to_string(),
            mode: Some(ElicitationMode::Form),
            url: None,
            requested_schema: Some(ElicitationSchema {
                schema_type: "object".to_string(),
                properties,
                required: Some(vec!["name".to_string()]),
            }),
            meta: None,
        };
        let json = serde_json::to_value(&elicit_params).unwrap();
        assert_eq!(json["message"], "Please fill out the form");
        assert_eq!(json["requestedSchema"]["type"], "object");
    }

    #[test]
    fn test_notification_params_serialization() {
        // Test ResourceUpdatedParams
        let resource_updated = ResourceUpdatedParams {
            uri: "file:///updated.txt".to_string(),
        };
        let json = serde_json::to_value(&resource_updated).unwrap();
        assert_eq!(json["uri"], "file:///updated.txt");

        // Test CancelledParams
        let cancelled = CancelledParams {
            request_id: json!("req-123"),
            reason: Some("User cancelled".to_string()),
        };
        let json = serde_json::to_value(&cancelled).unwrap();
        assert_eq!(json["requestId"], "req-123");
        assert_eq!(json["reason"], "User cancelled");

        // Test InitializedParams
        let initialized = InitializedParams { meta: None };
        let json = serde_json::to_value(&initialized).unwrap();
        assert!(json.is_object());

        // Test LoggingMessageParams
        let logging_msg = LoggingMessageParams {
            level: LoggingLevel::Info,
            logger: Some("test.logger".to_string()),
            data: json!({"message": "Test log message"}),
        };
        let json = serde_json::to_value(&logging_msg).unwrap();
        assert_eq!(json["level"], "info");
        assert_eq!(json["logger"], "test.logger");
        assert_eq!(json["data"]["message"], "Test log message");

        // Test list changed notification params
        let tool_list_changed = ToolListChangedParams { meta: None };
        let json = serde_json::to_value(&tool_list_changed).unwrap();
        assert!(json.is_object());

        let resource_list_changed = ResourceListChangedParams { meta: None };
        let json = serde_json::to_value(&resource_list_changed).unwrap();
        assert!(json.is_object());

        let prompt_list_changed = PromptListChangedParams { meta: None };
        let json = serde_json::to_value(&prompt_list_changed).unwrap();
        assert!(json.is_object());
    }

    #[test]
    fn test_result_types_serialization() {
        // Test ListResourcesResult
        let resources_result = ListResourcesResult {
            resources: vec![Resource {
                uri: "file:///test.txt".to_string(),
                name: "test.txt".to_string(),
                description: Some("Test file".to_string()),
                mime_type: Some("text/plain".to_string()),
                annotations: None,
                size: None,
                icons: None,
                title: None,
                meta: None,
            }],
            next_cursor: Some("next-page".to_string()),
            meta: None,
        };
        let json = serde_json::to_value(&resources_result).unwrap();
        assert!(json["resources"].is_array());
        assert_eq!(json["resources"][0]["uri"], "file:///test.txt");
        assert_eq!(json["nextCursor"], "next-page");

        // Test ReadResourceResult
        let read_result = ReadResourceResult {
            contents: vec![ResourceContents::Text {
                uri: "file:///test.txt".to_string(),
                mime_type: Some("text/plain".to_string()),
                text: "file content".to_string(),
                meta: None,
            }],
            meta: None,
        };
        let json = serde_json::to_value(&read_result).unwrap();
        assert!(json["contents"].is_array());

        // Test CompleteResult
        let complete_result = CompleteResult {
            completion: CompletionData {
                values: vec!["option1".to_string(), "option2".to_string()],
                total: Some(2),
                has_more: Some(false),
            },
            meta: None,
        };
        let json = serde_json::to_value(&complete_result).unwrap();
        assert_eq!(json["completion"]["values"][0], "option1");
        assert_eq!(json["completion"]["total"], 2);
        assert_eq!(json["completion"]["hasMore"], false);

        // Test ListRootsResult
        let roots_result = ListRootsResult {
            roots: vec![Root::new("file:///root".to_string()).with_name("Root".to_string())],
            meta: None,
        };
        let json = serde_json::to_value(&roots_result).unwrap();
        assert!(json["roots"].is_array());
        assert_eq!(json["roots"][0]["uri"], "file:///root");
        assert_eq!(json["roots"][0]["name"], "Root");

        // Test simple result types
        let ping_result = PingResult { meta: None };
        let json = serde_json::to_value(&ping_result).unwrap();
        assert!(json.is_object());

        let logging_result = SetLoggingLevelResult { meta: None };
        let json = serde_json::to_value(&logging_result).unwrap();
        assert!(json.is_object());

        let subscribe_result = SubscribeResourceResult { meta: None };
        let json = serde_json::to_value(&subscribe_result).unwrap();
        assert!(json.is_object());

        let unsubscribe_result = UnsubscribeResourceResult { meta: None };
        let json = serde_json::to_value(&unsubscribe_result).unwrap();
        assert!(json.is_object());
    }

    #[test]
    fn test_edge_cases_and_empty_values() {
        // Test empty ListToolsParams
        let empty_list_tools = ListToolsParams::default();
        let json = serde_json::to_value(&empty_list_tools).unwrap();
        assert!(json.is_object());
        assert!(json.get("cursor").is_none());

        // Test CallToolParams without arguments
        let tool_no_args = CallToolParams::new("simple_tool".to_string());
        let json = serde_json::to_value(&tool_no_args).unwrap();
        assert_eq!(json["name"], "simple_tool");
        assert!(json.get("arguments").is_none());

        // Test with empty arguments map
        let empty_args = HashMap::new();
        let tool_empty_args = CallToolParams::new_with_arguments("tool".to_string(), empty_args);
        let json = serde_json::to_value(&tool_empty_args).unwrap();
        assert_eq!(json["name"], "tool");
        assert!(json["arguments"].is_object());
        assert_eq!(json["arguments"].as_object().unwrap().len(), 0);

        // Test progress with no optional fields
        let minimal_progress = ProgressParams {
            progress_token: json!("token"),
            progress: 50.0,
            total: None,
            message: None,
        };
        let json = serde_json::to_value(&minimal_progress).unwrap();
        assert_eq!(json["progress"], 50.0);
        assert!(json.get("total").is_none());
        assert!(json.get("message").is_none());
    }
}

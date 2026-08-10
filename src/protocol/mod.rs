//! Dual-era MCP protocol implementation.
//!
//! This module contains the core protocol types and message handling for the
//! Model Context Protocol revisions 2026-07-28 and 2025-11-25, including
//! JSON-RPC serialization, validation, stateless request envelopes, safe
//! revision selection, and typed protocol objects.
//!
//! # Protocol Structure
//!
//! The MCP protocol is built on JSON-RPC 2.0 with extensions for:
//! - **Bidirectional Communication**: Both client and server can send requests
//! - **Capability Negotiation**: Dynamic feature discovery
//! - **Content Types**: Rich content including text, images, and resources
//! - **Batch Operations**: Efficient bulk request processing
//!
//! # Core Types
//!
//! ## JSON-RPC Messages
//! ```
//! use prism_mcp_rs::protocol::{
//!     JsonRpcRequest, JsonRpcResponse, JsonRpcError, JsonRpcMessage
//! };
//! use serde_json::json;
//!
//! // Create a request
//! let request = JsonRpcRequest::new(
//!     json!("1"),
//!     "tools/list".to_string(),
//!     None::<()>,
//! ).unwrap();
//!
//! // Create a success response
//! let response = JsonRpcResponse::success_unchecked(
//!     json!("1"),
//!     json!({"tools": []}),
//! );
//!
//! // Create an error response
//! let error = JsonRpcError::method_not_found(json!("1"));
//! ```
//!
//! ## Error Handling
//! ```
//! use prism_mcp_rs::protocol::{JsonRpcError, error_codes};
//! use serde_json::json;
//!
//! // Standard JSON-RPC errors
//! let parse_err = JsonRpcError::parse_error(json!(null));
//! let method_err = JsonRpcError::method_not_found(json!("1"));
//! let invalid_params = JsonRpcError::invalid_params(json!("1"));
//!
//! // MCP-specific errors
//! let tool_err = JsonRpcError::tool_not_found(json!("1"), "unknown-tool");
//! let resource_err = JsonRpcError::resource_not_found(json!("1"), "missing.txt");
//!
//! // Custom errors with error codes
//! let custom_err = JsonRpcError::new(
//!     json!("1"),
//!     error_codes::INTERNAL_ERROR,
//!     "Internal server error".to_string()
//! );
//! ```
//!
//! # Protocol Flow
//!
//! - **2026-07-28**: the client sends `server/discover`, then includes its
//!   revision, identity, and capabilities in every request `_meta` object.
//! - **2025-11-25**: the client performs `initialize`, retains negotiated
//!   connection state, and sends the initialized notification.
//!
//! [`ProtocolMode::Auto`] prefers 2026 and permits a 2025 fallback only when
//! `server/discover` is explicitly rejected as an unknown method.
//!
//! # Features
//!
//! - **Type Safety**: Strongly typed protocol messages
//! - **Validation**: Request and response validation
//! - **Extensibility**: Support for custom methods and capabilities
//! - **Batch Processing**: Efficient bulk operations
//! - **Metadata**: Rich metadata for all protocol objects

pub mod batch;
pub mod discovery;
pub mod error_helpers;
pub mod messages;
pub mod metadata;
pub mod methods;
pub mod missing_types;
pub mod roots_types;
pub mod schema_introspection;
pub mod subscriptions;
pub mod tasks;
pub mod types;
pub mod validation;
pub mod version;

// Re-export commonly used types and constants
pub use batch::*;
pub use discovery::*;
pub use error_helpers::IntoJsonRpcMessage;
pub use messages::*;

// Re-export metadata module types (Implementation is now only in types module)
pub use metadata::{MetadataBuilder, ProtocolCapabilities};

pub use missing_types::*;
// Re-export roots_types items except those that conflict with messages
pub use roots_types::{
    ListRootsRequest,
    RootsListChangedNotification,
    // Explicitly exclude Root and ListRootsResult which are already in messages
};
pub use schema_introspection::*;
pub use subscriptions::*;
pub use tasks::*;
// Re-export all types module items
pub use types::{
    error_codes, AnnotationAudience, Annotations, AudioContent, BaseMetadata, CallToolResult,
    ClientCapabilities, ClientInfo, CompletionsCapability, Content, ContentBlock,
    CreateMessageResult, Cursor, DangerLevel, ElicitationAction, ElicitationCapability,
    ElicitationSchema, EmbeddedResource, ErrorObject, GetPromptResult, Icon, IconTheme,
    ImageContent, Implementation, JsonRpcBatchRequest, JsonRpcBatchResponse, JsonRpcError,
    JsonRpcId, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcRequestOrNotification,
    JsonRpcResponse, JsonRpcResponseOrError, LoggingCapability, LoggingLevel, ModelHint,
    ModelPreferences, Notification, NotificationParams, PaginatedRequest, PaginatedResult,
    PrimitiveSchemaDefinition, ProgressToken, Prompt, PromptArgument, PromptInfo, PromptMessage,
    PromptResult, PromptsCapability, Request, RequestId, RequestMeta, RequestParams, Resource,
    ResourceContents, ResourceInfo, ResourceLink, ResourceTemplate, ResourcesCapability, Role,
    RootsCapability, SamplingCapability, SamplingContent, SamplingMessage, SamplingToolChoice,
    ServerCapabilities, ServerInfo, StopReason, TextContent, Tool, ToolAnnotations, ToolInfo,
    ToolInputSchema, ToolOutputSchema, ToolResult, ToolsCapability, JSONRPC_VERSION,
    LATEST_PROTOCOL_VERSION, PROTOCOL_VERSION,
};

pub use validation::*;
pub use version::{
    decode_http_header_value, decorate_modern_request, decorate_modern_result,
    encode_http_header_value, is_cacheable_method, is_legacy_only_method, is_method_not_found,
    json_rpc_error_details, modern_request_context, request_protocol_version, request_routing_name,
    tool_call_headers, tool_header_mappings, validate_http_headers, validate_tool_call_headers,
    CacheScope, ConnectResult, DiscoverParams, DiscoverResult as ModernDiscoverResult,
    InputRequiredResult, ModernRequestContext, NegotiatedProtocol, OperationResult, ProtocolEra,
    ProtocolMode, RequestMetaObject, ResultType, ToolHeaderMapping, CLIENT_CAPABILITIES_META_KEY,
    CLIENT_INFO_META_KEY, HEADER_MISMATCH, LEGACY_PROTOCOL_VERSION, MCP_METHOD_HEADER,
    MCP_NAME_HEADER, MCP_PROTOCOL_VERSION_HEADER, MISSING_REQUIRED_CLIENT_CAPABILITY,
    MODERN_PROTOCOL_VERSION, PROTOCOL_VERSION_META_KEY, SERVER_INFO_META_KEY,
    SUPPORTED_PROTOCOL_VERSIONS, UNSUPPORTED_PROTOCOL_VERSION,
};

// Re-export method constants for convenience
pub use methods::{
    CANCELLED, COMPLETION_COMPLETE, ELICITATION_COMPLETE, INITIALIZE, INITIALIZED, LOGGING_MESSAGE,
    LOGGING_SET_LEVEL, PING, PROGRESS, PROMPTS_GET, PROMPTS_LIST, PROMPTS_LIST_CHANGED,
    RESOURCES_LIST, RESOURCES_LIST_CHANGED, RESOURCES_READ, RESOURCES_SUBSCRIBE,
    RESOURCES_TEMPLATES_LIST, RESOURCES_UNSUBSCRIBE, RESOURCES_UPDATED, ROOTS_LIST,
    ROOTS_LIST_CHANGED, RPC_DISCOVER, SAMPLING_CREATE_MESSAGE, SERVER_DISCOVER,
    SUBSCRIPTIONS_ACKNOWLEDGED, SUBSCRIPTIONS_LISTEN, TASKS_CANCEL, TASKS_GET, TASKS_SEND,
    TASKS_STATUS, TASKS_STATUS_UPDATE, TASKS_UPDATE, TOOLS_CALL, TOOLS_LIST, TOOLS_LIST_CHANGED,
};

// Legacy constant for compatibility
pub const MCP_PROTOCOL_VERSION: &str = LATEST_PROTOCOL_VERSION;

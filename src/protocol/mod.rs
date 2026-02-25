//! MCP protocol implementation (2025-11-25)
//!
//! This module contains the core protocol types and message handling for the
//! Model Context Protocol version 2025-11-25, including JSON-RPC message
//! serialization, validation, and new features like improved content system,
//! annotations, improved capabilities, full metadata support, batch operations,
//! and complete schema introspection.
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
//! 1. **Initialization**: Client sends `initialize` request
//! 2. **Capability Exchange**: Server responds with supported capabilities
//! 3. **Operation Phase**: Normal request/response exchange
//! 4. **Shutdown**: Clean termination with proper cleanup
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
pub mod types;
// NOTE: types_2025 is temporarily disabled to resolve ContentBlock duplication conflicts
// during schema upgrade to 2025-11-25. Will be removed after consolidation.
// pub mod types_2025;
pub mod validation;

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
// Re-export all types module items
pub use types::{
    error_codes, AnnotationAudience, Annotations, AudioContent, BaseMetadata, CallToolResult,
    ClientCapabilities, ClientInfo, CompletionsCapability, Content, ContentBlock,
    CreateMessageResult, Cursor, DangerLevel, ElicitationAction, ElicitationCapability,
    ElicitationSchema, EmbeddedResource, ErrorObject, GetPromptResult, Icon, ImageContent,
    Implementation, JsonRpcBatchRequest, JsonRpcBatchResponse, JsonRpcError, JsonRpcId,
    JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcRequestOrNotification,
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

// Re-export method constants for convenience
pub use methods::{
    CANCELLED, COMPLETION_COMPLETE, ELICITATION_COMPLETE, INITIALIZE, INITIALIZED, LOGGING_MESSAGE,
    LOGGING_SET_LEVEL, PING, PROGRESS, PROMPTS_GET, PROMPTS_LIST, PROMPTS_LIST_CHANGED,
    RESOURCES_LIST, RESOURCES_LIST_CHANGED, RESOURCES_READ, RESOURCES_SUBSCRIBE,
    RESOURCES_TEMPLATES_LIST, RESOURCES_UNSUBSCRIBE, RESOURCES_UPDATED, ROOTS_LIST,
    ROOTS_LIST_CHANGED, RPC_DISCOVER, SAMPLING_CREATE_MESSAGE, TASKS_CANCEL, TASKS_SEND,
    TASKS_STATUS_UPDATE, TOOLS_CALL, TOOLS_LIST, TOOLS_LIST_CHANGED,
};

// Legacy constant for compatibility
pub const MCP_PROTOCOL_VERSION: &str = LATEST_PROTOCOL_VERSION;

// NOTE: types_2025 re-export disabled during consolidation
// Export types_2025 for complete tests
// pub use types_2025 as types_2025_complete;

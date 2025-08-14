//! MCP protocol implementation (2025-06-18)
//!
//! This module contains the core protocol types and message handling for the
//! Model Context Protocol version 2025-06-18, including JSON-RPC message
//! serialization, validation, and new features like improved content system,
//! annotations, improved capabilities, full metadata support, batch operations,
//! and complete schema introspection.

pub mod batch;
pub mod discovery;
pub mod messages;
pub mod metadata;
pub mod methods;
pub mod missing_types;
pub mod roots_types;
pub mod schema_introspection;
pub mod types;
// NOTE: types_2025 is temporarily disabled to resolve ContentBlock duplication conflicts
// during schema upgrade to 2025-06-18. Will be removed after consolidation.
// pub mod types_2025;
pub mod validation;

// Re-export commonly used types and constants
pub use batch::*;
pub use discovery::*;
pub use messages::*;

// Re-export metadata module types explicitly to avoid conflicts with types module
// These types (Implementation, ServerInfo, ClientInfo) are the canonical versions
pub use metadata::{
    ClientInfo, Implementation, MetadataBuilder, ProtocolCapabilities, ServerInfo,
};

pub use missing_types::*;
// Re-export roots_types items except those that conflict with messages
pub use roots_types::{
    ListRootsRequest,
    RootsListChangedNotification,
    // Explicitly exclude Root and ListRootsResult which are already in messages
};
pub use schema_introspection::*;

// Re-export all types module items EXCEPT Implementation, ServerInfo, and ClientInfo
// which are already exported from metadata module to avoid ambiguity
pub use types::{
    AnnotationAudience, Annotations, AudioContent, BaseMetadata, CallToolResult,
    ClientCapabilities, CompletionsCapability, Content, ContentBlock, CreateMessageResult,
    Cursor, DangerLevel, ElicitationAction, ElicitationCapability, ElicitationSchema,
    EmbeddedResource, ErrorObject, GetPromptResult, ImageContent, JsonRpcBatchRequest,
    JsonRpcBatchResponse, JsonRpcError, JsonRpcId, JsonRpcMessage, JsonRpcNotification,
    JsonRpcRequest, JsonRpcRequestOrNotification, JsonRpcResponse, JsonRpcResponseOrError,
    LoggingCapability, LoggingLevel, ModelHint, ModelPreferences, Notification,
    NotificationParams, PaginatedRequest, PaginatedResult, PrimitiveSchemaDefinition,
    ProgressToken, Prompt, PromptArgument, PromptInfo, PromptMessage, PromptResult,
    PromptsCapability, Request, RequestId, RequestMeta, RequestParams, Resource,
    ResourceContents, ResourceInfo, ResourceLink, ResourceTemplate, ResourcesCapability,
    Role, RootsCapability, SamplingCapability, SamplingContent, SamplingMessage,
    ServerCapabilities, StopReason, TextContent, Tool, ToolAnnotations, ToolInfo,
    ToolInputSchema, ToolOutputSchema, ToolResult, ToolsCapability, error_codes,
    JSONRPC_VERSION, LATEST_PROTOCOL_VERSION, PROTOCOL_VERSION,
};

pub use validation::*;

// Re-export method constants for convenience
pub use methods::{
    CANCELLED, COMPLETION_COMPLETE, INITIALIZE, INITIALIZED, LOGGING_MESSAGE, LOGGING_SET_LEVEL,
    PING, PROGRESS, PROMPTS_GET, PROMPTS_LIST, PROMPTS_LIST_CHANGED, RESOURCES_LIST,
    RESOURCES_LIST_CHANGED, RESOURCES_READ, RESOURCES_SUBSCRIBE, RESOURCES_TEMPLATES_LIST,
    RESOURCES_UNSUBSCRIBE, RESOURCES_UPDATED, ROOTS_LIST, ROOTS_LIST_CHANGED, RPC_DISCOVER,
    SAMPLING_CREATE_MESSAGE, TOOLS_CALL, TOOLS_LIST, TOOLS_LIST_CHANGED,
};

// Legacy constant for compatibility
pub const MCP_PROTOCOL_VERSION: &str = LATEST_PROTOCOL_VERSION;

// NOTE: types_2025 re-export disabled during consolidation
// Export types_2025 for complete tests
// pub use types_2025 as types_2025_complete;
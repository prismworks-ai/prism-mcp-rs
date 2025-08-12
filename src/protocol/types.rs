//! Complete MCP Protocol Types for 2025-06-18 Specification
//!
//! This module contains all the core types defined by the Model Context Protocol
//! specification version 2025-06-18, with simplified JSON-RPC (no batching) and
//! improved metadata handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Core Protocol Constants
// ============================================================================

/// MCP Protocol version (2025-06-18)
pub const LATEST_PROTOCOL_VERSION: &str = "2025-06-18";
pub const JSONRPC_VERSION: &str = "2.0";

// Legacy constant for compatibility
pub const PROTOCOL_VERSION: &str = LATEST_PROTOCOL_VERSION;

// ============================================================================
// Type Aliases
// ============================================================================

/// Progress token for associating notifications with requests
pub type ProgressToken = serde_json::Value; // string | number

/// Cursor for pagination
pub type Cursor = String;

/// Request ID for JSON-RPC correlation
pub type RequestId = serde_json::Value; // string | number | null

/// JSON-RPC ID type for better type safety
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JsonRpcId {
    String(String),
    Number(i64),
    Null,
}

impl From<i64> for JsonRpcId {
    fn from(value: i64) -> Self {
        JsonRpcId::Number(value)
    }
}

impl From<String> for JsonRpcId {
    fn from(value: String) -> Self {
        JsonRpcId::String(value)
    }
}

impl From<&str> for JsonRpcId {
    fn from(value: &str) -> Self {
        JsonRpcId::String(value.to_string())
    }
}

// ============================================================================
// BaseMetadata Interface (2025-06-18)
// ============================================================================

/// Base interface for metadata with name (identifier) and title (display name) properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseMetadata {
    /// Intended for programmatic or logical use, but used as a display name in past specs or fallback (if title isn't present).
    pub name: String,
    /// Intended for UI and end-user contexts — improved to be human-readable and easily understood,
    /// even by those unfamiliar with domain-specific terminology.
    ///
    /// If not provided, the name should be used for display (except for Tool,
    /// where `annotations.title` should be given precedence over using `name`, if present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

// ============================================================================
// Core Implementation Info
// ============================================================================

/// Information about an MCP implementation (2025-06-18 with title support)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Implementation {
    /// Intended for programmatic or logical use, but used as a display name in past specs or fallback (if title isn't present).
    pub name: String,
    /// Version of the implementation
    pub version: String,
    /// Intended for UI and end-user contexts — improved to be human-readable and easily understood,
    /// even by those unfamiliar with domain-specific terminology.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Implementation {
    /// Create a new implementation with name and version
    pub fn new<S: Into<String>>(name: S, version: S) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            title: None,
        }
    }

    /// Create implementation with title
    pub fn with_title<S: Into<String>>(name: S, version: S, title: S) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            title: Some(title.into()),
        }
    }
}

// Type aliases for compatibility
pub type ServerInfo = Implementation;
pub type ClientInfo = Implementation;

// ============================================================================
// Capabilities (2025-06-18)
// ============================================================================

/// Server capabilities for 2025-06-18
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ServerCapabilities {
    /// Prompt-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    /// Resource-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// Tool-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    /// Sampling-related capabilities (client to server)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapability>,
    /// Logging capabilities (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapability>,
    /// Autocompletion capabilities (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completions: Option<CompletionsCapability>,
    /// Experimental capabilities (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
}

/// Client capabilities for 2025-06-18
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ClientCapabilities {
    /// Sampling-related capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapability>,
    /// Roots listing capabilities (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
    /// Elicitation support (2025-06-18 NEW)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elicitation: Option<ElicitationCapability>,
    /// Experimental capabilities (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,
}

/// Prompt-related server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PromptsCapability {
    /// Whether the server supports prompt list change notifications
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resource-related server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResourcesCapability {
    /// Whether the server supports resource subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    /// Whether the server supports resource list change notifications
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Tool-related server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolsCapability {
    /// Whether the server supports tool list change notifications
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Sampling-related capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SamplingCapability {
    /// Additional properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// Logging capabilities (2025-03-26)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LoggingCapability {
    /// Additional properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// Autocompletion capabilities (2025-03-26)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CompletionsCapability {
    /// Additional properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// Roots capability for clients (2025-06-18)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RootsCapability {
    /// Whether the client supports notifications for changes to the roots list
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Elicitation capabilities (2025-06-18 NEW)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ElicitationCapability {
    /// Additional properties for elicitation capability
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Annotations (2025-06-18 improved)
// ============================================================================

/// Optional annotations for the client. The client can use annotations to inform how objects are used or displayed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Annotations {
    /// Describes who the intended customer of this object or data is.
    ///
    /// It can include multiple entries to indicate content useful for multiple audiences (e.g., `["user", "assistant"]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<Role>>,
    /// Describes how important this data is for operating the server.
    ///
    /// A value of 1 means "most important," and indicates that the data is
    /// effectively required, while 0 means "least important," and indicates that
    /// the data is fully optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    /// The moment the resource was last modified, as an ISO 8601 formatted string.
    ///
    /// Should be an ISO 8601 formatted string (e.g., "2025-01-12T15:00:58Z").
    ///
    /// Examples: last activity timestamp in an open file, timestamp when the resource
    /// was attached, etc.
    #[serde(rename = "lastModified", skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    /// Legacy danger level field for test compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danger: Option<DangerLevel>,
    /// Legacy destructive field for test compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive: Option<bool>,
    /// Legacy read_only field for test compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
}

// ============================================================================
// Content Types (2025-06-18 with ResourceLink)
// ============================================================================

/// Text content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextContent {
    /// Content type identifier
    #[serde(rename = "type")]
    pub content_type: String, // "text"
    /// The text content
    pub text: String,
    /// Content annotations (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Image content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageContent {
    /// Content type identifier
    #[serde(rename = "type")]
    pub content_type: String, // "image"
    /// Base64-encoded image data
    pub data: String,
    /// MIME type of the image
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    /// Content annotations (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Audio content (2025-06-18)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioContent {
    /// Content type identifier
    #[serde(rename = "type")]
    pub content_type: String, // "audio"
    /// Base64-encoded audio data
    pub data: String,
    /// MIME type of the audio
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    /// Content annotations (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// ResourceLink content (2025-06-18 NEW)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceLink {
    /// Content type identifier
    #[serde(rename = "type")]
    pub content_type: String, // "resource_link"
    /// URI of the resource
    pub uri: String,
    /// Human-readable name of the resource
    pub name: String,
    /// Description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the resource
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Size of the resource in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Title for UI display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Content annotations (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Embedded resource content (2025-06-18)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddedResource {
    /// Content type identifier
    #[serde(rename = "type")]
    pub content_type: String, // "resource"
    /// Resource contents
    pub resource: ResourceContents,
    /// Content annotations (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// ContentBlock union type (2025-06-18 including ResourceLink)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
        /// Content annotations (2025-06-18)
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Annotations>,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
    /// Image content
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded image data
        data: String,
        /// MIME type of the image
        #[serde(rename = "mimeType")]
        mime_type: String,
        /// Content annotations (2025-06-18)
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Annotations>,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
    /// Audio content (2025-06-18)
    #[serde(rename = "audio")]
    Audio {
        /// Base64-encoded audio data
        data: String,
        /// MIME type of the audio
        #[serde(rename = "mimeType")]
        mime_type: String,
        /// Content annotations (2025-06-18)
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Annotations>,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
    /// ResourceLink content (2025-06-18 NEW)
    #[serde(rename = "resource_link")]
    ResourceLink {
        /// URI of the resource
        uri: String,
        /// Human-readable name of the resource
        name: String,
        /// Description of the resource
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// MIME type of the resource
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        /// Size of the resource in bytes
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<u64>,
        /// Title for UI display
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// Content annotations (2025-06-18)
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Annotations>,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
    /// Embedded resource content (2025-06-18)
    #[serde(rename = "resource")]
    Resource {
        /// Resource contents
        resource: ResourceContents,
        /// Content annotations (2025-06-18)
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Annotations>,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
}

// Legacy alias for backwards compatibility
pub type Content = ContentBlock;

// ============================================================================
// Tool Types (2025-06-18 with Title and Structured Content)
// ============================================================================

/// Tool-specific annotations (2025-06-18 Schema Compliance)
///
/// NOTE: all properties in ToolAnnotations are **hints**.
/// They are not guaranteed to provide a faithful description of
/// tool behavior (including descriptive properties like `title`).
///
/// Clients should never make tool use decisions based on ToolAnnotations
/// received from untrusted servers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolAnnotations {
    /// A human-readable title for the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// If true, the tool does not modify its environment
    /// Default: false
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,

    /// If true, the tool may perform destructive updates to its environment
    /// If false, the tool performs only additive updates
    /// (This property is meaningful only when `readOnlyHint == false`)
    /// Default: true
    #[serde(rename = "destructiveHint", skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,

    /// If true, calling the tool repeatedly with the same arguments
    /// will have no additional effect on its environment
    /// (This property is meaningful only when `readOnlyHint == false`)
    /// Default: false
    #[serde(rename = "idempotentHint", skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,

    /// If true, this tool may interact with an "open world" of external entities
    /// If false, the tool's domain of interaction is closed
    /// For example, the world of a web search tool is open, whereas that
    /// of a memory tool is not
    /// Default: true
    #[serde(rename = "openWorldHint", skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

impl ToolAnnotations {
    /// Create new empty tool annotations
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the human-readable title for the tool
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Mark tool as read-only (does not modify environment)
    pub fn read_only(mut self) -> Self {
        self.read_only_hint = Some(true);
        self
    }

    /// Mark tool as destructive (may perform destructive updates)
    pub fn destructive(mut self) -> Self {
        self.destructive_hint = Some(true);
        self
    }

    /// Mark tool as idempotent (same input produces same result)
    pub fn idempotent(mut self) -> Self {
        self.idempotent_hint = Some(true);
        self
    }

    /// Mark tool as interacting with open world of external entities
    pub fn open_world(mut self) -> Self {
        self.open_world_hint = Some(true);
        self
    }

    /// Mark tool as interacting with closed world (limited domain)
    pub fn closed_world(mut self) -> Self {
        self.open_world_hint = Some(false);
        self
    }
}

// ============================================================================
// Tool Annotations Integration with improved Metadata
// ============================================================================

impl From<&crate::core::tool_metadata::ToolBehaviorHints> for ToolAnnotations {
    fn from(hints: &crate::core::tool_metadata::ToolBehaviorHints) -> Self {
        Self {
            title: None, // Title should be set separately at tool level
            read_only_hint: hints.read_only,
            destructive_hint: hints.destructive,
            idempotent_hint: hints.idempotent,
            // Map open_world_hint: if requires_auth or resource_intensive, likely open world
            open_world_hint: if hints.requires_auth.unwrap_or(false)
                || hints.resource_intensive.unwrap_or(false)
            {
                Some(true)
            } else {
                None
            },
        }
    }
}

impl From<&crate::core::tool_metadata::ImprovedToolMetadata> for ToolAnnotations {
    fn from(metadata: &crate::core::tool_metadata::ImprovedToolMetadata) -> Self {
        ToolAnnotations::from(&metadata.behavior_hints)
    }
}

impl ToolAnnotations {
    /// Create ToolAnnotations from improved metadata with explicit title override
    pub fn from_improved_metadata(
        metadata: &crate::core::tool_metadata::ImprovedToolMetadata,
        title_override: Option<String>,
    ) -> Self {
        let mut annotations = Self::from(metadata);
        if let Some(title) = title_override {
            annotations.title = Some(title);
        }
        annotations
    }

    /// Create minimal ToolAnnotations from behavior hints
    pub fn from_behavior_hints(hints: &crate::core::tool_metadata::ToolBehaviorHints) -> Self {
        Self::from(hints)
    }
}

/// Tool definition with annotations and title (2025-06-18)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    /// Intended for programmatic or logical use
    pub name: String,
    /// Description of what the tool does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema describing the tool's input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,
    /// Optional JSON Schema object defining the structure of the tool's output returned in
    /// the structuredContent field of a CallToolResult (2025-06-18 NEW)
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<ToolOutputSchema>,
    /// Tool behavior annotations (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,
    /// Intended for UI and end-user contexts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Tool input schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInputSchema {
    /// Schema type (always "object")
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Schema properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
    /// Required properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Additional schema properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// Tool output schema (2025-06-18 NEW)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolOutputSchema {
    /// Schema type (always "object")
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Schema properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
    /// Required properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl ToolOutputSchema {
    /// Create a new tool output schema
    pub fn new() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
        }
    }

    /// Create a tool output schema with properties
    pub fn with_properties(properties: HashMap<String, serde_json::Value>) -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: Some(properties),
            required: None,
        }
    }

    /// Add required fields to the schema
    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }

    /// Add properties to the schema
    pub fn with_properties_map(mut self, properties: HashMap<String, serde_json::Value>) -> Self {
        self.properties = Some(properties);
        self
    }
}

impl Default for ToolOutputSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a tool execution (2025-06-18 with structured content)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallToolResult {
    /// Content returned by the tool
    pub content: Vec<ContentBlock>,
    /// Whether this result represents an error
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    /// An optional JSON object that represents the structured result of the tool call
    #[serde(rename = "structuredContent", skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<serde_json::Value>,
    /// Result metadata (2025-06-18)
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

// Re-export types with legacy names for compatibility
pub type ToolInfo = Tool;
pub type ToolResult = CallToolResult;

// Additional compatibility aliases for documentation examples
// Note: Content type alias is defined earlier in the file

// ============================================================================
// Resource Types (2025-06-18)
// ============================================================================

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    /// URI of the resource
    pub uri: String,
    /// Intended for programmatic or logical use, but used as a display name in past specs or fallback (if title isn't present).
    pub name: String,
    /// Description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the resource
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Resource annotations (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// Resource size in bytes (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Intended for UI and end-user contexts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Resource template for URI patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceTemplate {
    /// URI template with variables
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,
    /// Intended for programmatic or logical use
    pub name: String,
    /// Description of the resource template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of resources from this template
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Resource annotations (2025-06-18)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
    /// Intended for UI and end-user contexts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Content of a resource (2025-06-18)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ResourceContents {
    /// Text resource content
    Text {
        /// URI of the resource
        uri: String,
        /// MIME type
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        /// Text content
        text: String,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
    /// Binary resource content
    Blob {
        /// URI of the resource
        uri: String,
        /// MIME type
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        /// Base64-encoded binary data
        blob: String,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
}

impl ResourceContents {
    /// Get the URI of the resource
    pub fn uri(&self) -> &str {
        match self {
            ResourceContents::Text { uri, .. } => uri,
            ResourceContents::Blob { uri, .. } => uri,
        }
    }
}

// Legacy type aliases for compatibility
pub type ResourceInfo = Resource;

// ============================================================================
// Prompt Types (2025-06-18)
// ============================================================================

/// Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    /// Intended for programmatic or logical use
    pub name: String,
    /// Description of what the prompt does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Arguments that the prompt accepts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
    /// Intended for UI and end-user contexts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Metadata field for future extensions
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Argument for a prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptArgument {
    /// Intended for programmatic or logical use
    pub name: String,
    /// Description of the argument
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this argument is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Intended for UI and end-user contexts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// Message in a prompt result (2025-06-18 with ContentBlock support)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptMessage {
    /// Role of the message
    pub role: Role,
    /// Content of the message (supports all content types including resource_link)
    pub content: ContentBlock,
}

/// Result of prompt execution (2025-06-18 with metadata)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptResult {
    /// Description of the prompt result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Messages generated by the prompt
    pub messages: Vec<PromptMessage>,
    /// Result metadata (2025-06-18)
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

// Legacy type aliases for compatibility
pub type PromptInfo = Prompt;
pub type PromptResult = GetPromptResult;

// ============================================================================
// Sampling Types (2025-06-18)
// ============================================================================

/// A message in a sampling conversation (2025-06-18 with ContentBlock)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamplingMessage {
    /// Role of the message
    pub role: Role,
    /// Content of the message (text, image, or audio only - no resource_link in sampling)
    pub content: SamplingContent,
}

/// Content types allowed in sampling (subset of ContentBlock)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SamplingContent {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
        /// Content annotations (2025-06-18)
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Annotations>,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
    /// Image content
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded image data
        data: String,
        /// MIME type of the image
        #[serde(rename = "mimeType")]
        mime_type: String,
        /// Content annotations (2025-06-18)
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Annotations>,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
    /// Audio content (2025-06-18)
    #[serde(rename = "audio")]
    Audio {
        /// Base64-encoded audio data
        data: String,
        /// MIME type of the audio
        #[serde(rename = "mimeType")]
        mime_type: String,
        /// Content annotations (2025-06-18)
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<Annotations>,
        /// Metadata field for future extensions
        #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
        meta: Option<HashMap<String, serde_json::Value>>,
    },
}

/// Model hint for model selection (2025-06-18)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelHint {
    /// A hint for a model name.
    ///
    /// The client SHOULD treat this as a substring of a model name; for example:
    /// - `claude-3-5-sonnet` should match `claude-3-5-sonnet-20241022`
    /// - `sonnet` should match `claude-3-5-sonnet-20241022`, `claude-3-sonnet-20240229`, etc.
    /// - `claude` should match any Claude model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Additional provider-specific hints for model selection.
    /// Keys not declared here are currently left unspecified by the spec and are up
    /// to the client to interpret. This allows provider-specific extensions.
    #[serde(flatten)]
    pub additional_hints: Option<HashMap<String, serde_json::Value>>,
}

/// Model preferences for sampling (2025-06-18 improved)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelPreferences {
    /// How much to prioritize cost when selecting a model
    #[serde(rename = "costPriority", skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f64>,
    /// How much to prioritize sampling speed (latency) when selecting a model
    #[serde(rename = "speedPriority", skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f64>,
    /// How much to prioritize intelligence and capabilities when selecting a model
    #[serde(
        rename = "intelligencePriority",
        skip_serializing_if = "Option::is_none"
    )]
    pub intelligence_priority: Option<f64>,
    /// Optional hints to use for model selection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<ModelHint>>,
}

/// Result of sampling/createMessage (2025-06-18)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateMessageResult {
    /// Role of the generated message
    pub role: Role,
    /// Content of the generated message
    pub content: SamplingContent,
    /// Model used for generation
    pub model: String,
    /// Stop reason
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    /// Result metadata (2025-06-18)
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
}

/// Reasons why sampling stopped
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    EndTurn,
    StopSequence,
    MaxTokens,
    #[serde(untagged)]
    Other(String),
}

// ============================================================================
// Elicitation Types (2025-06-18 NEW)
// ============================================================================

/// Primitive schema definition for elicitation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PrimitiveSchemaDefinition {
    #[serde(rename = "string")]
    String {
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
        min_length: Option<u32>,
        #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
        max_length: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
        enum_values: Option<Vec<String>>,
        #[serde(rename = "enumNames", skip_serializing_if = "Option::is_none")]
        enum_names: Option<Vec<String>>,
    },
    #[serde(rename = "number")]
    Number {
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        minimum: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        maximum: Option<i32>,
    },
    #[serde(rename = "integer")]
    Integer {
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        minimum: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        maximum: Option<i32>,
    },
    #[serde(rename = "boolean")]
    Boolean {
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<bool>,
    },
}

/// Restricted schema for elicitation (only top-level properties allowed)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElicitationSchema {
    /// Schema type (always "object")
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Top-level properties
    pub properties: HashMap<String, PrimitiveSchemaDefinition>,
    /// Required properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

/// Elicitation user action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationAction {
    /// User submitted the form/confirmed the action
    Accept,
    /// User explicitly declined the action
    Decline,
    /// User dismissed without making an explicit choice
    Cancel,
}

// ============================================================================
// Logging Types (2025-06-18)
// ============================================================================

/// Logging level enumeration (2025-06-18)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

// ============================================================================
// JSON-RPC Types (2025-03-26 with Batching)
// ============================================================================

/// JSON-RPC request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request ID for correlation
    pub id: RequestId,
    /// Method name being called
    pub method: String,
    /// Method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC response message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request ID for correlation
    pub id: RequestId,
    /// Result of the method call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// JSON-RPC error message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request ID for correlation
    pub id: RequestId,
    /// Error information
    pub error: ErrorObject,
}

/// Error object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorObject {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC notification message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Method name being called
    pub method: String,
    /// Method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC message types (2025-06-18)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Error(JsonRpcError),
    Notification(JsonRpcNotification),
}

// ============================================================================
// Request/Response Metadata (2025-03-26 NEW)
// ============================================================================

/// Base request with metadata support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    /// Method name
    pub method: String,
    /// Parameters with metadata support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<RequestParams>,
}

/// Request parameters with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestParams {
    /// Request metadata (2025-03-26 NEW)
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
    /// Additional parameters
    #[serde(flatten)]
    pub params: HashMap<String, serde_json::Value>,
}

/// Request metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestMeta {
    /// Progress token for out-of-band progress notifications
    #[serde(rename = "progressToken", skip_serializing_if = "Option::is_none")]
    pub progress_token: Option<ProgressToken>,
}

/// Base notification with metadata support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    /// Method name
    pub method: String,
    /// Parameters with metadata support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<NotificationParams>,
}

/// Notification parameters with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationParams {
    /// Notification metadata (2025-03-26 NEW)
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, serde_json::Value>>,
    /// Additional parameters
    #[serde(flatten)]
    pub params: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Pagination Support
// ============================================================================

/// Base for paginated requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginatedRequest {
    /// Cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

/// Base for paginated results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginatedResult {
    /// Cursor for next page
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
}

// ============================================================================
// Helper Constructors
// ============================================================================

impl ContentBlock {
    /// Create text content
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self::Text {
            text: text.into(),
            annotations: None,
            meta: None,
        }
    }

    /// Create image content
    pub fn image<S: Into<String>>(data: S, mime_type: S) -> Self {
        Self::Image {
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
            meta: None,
        }
    }

    /// Create audio content (2025-06-18)
    pub fn audio<S: Into<String>>(data: S, mime_type: S) -> Self {
        Self::Audio {
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
            meta: None,
        }
    }

    /// Create resource link content (2025-06-18 NEW)
    pub fn resource_link<S: Into<String>>(uri: S, name: S) -> Self {
        Self::ResourceLink {
            uri: uri.into(),
            name: name.into(),
            description: None,
            mime_type: None,
            size: None,
            title: None,
            annotations: None,
            meta: None,
        }
    }

    /// Create embedded resource content (2025-06-18)
    pub fn embedded_resource(resource: ResourceContents) -> Self {
        Self::Resource {
            resource,
            annotations: None,
            meta: None,
        }
    }

    /// Create resource content (legacy compatibility)
    pub fn resource<S: Into<String>>(uri: S) -> Self {
        let uri_str = uri.into();
        Self::resource_link(uri_str.clone(), uri_str)
    }
}

impl SamplingContent {
    /// Create text content for sampling
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self::Text {
            text: text.into(),
            annotations: None,
            meta: None,
        }
    }

    /// Create image content for sampling
    pub fn image<S: Into<String>>(data: S, mime_type: S) -> Self {
        Self::Image {
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
            meta: None,
        }
    }

    /// Create audio content for sampling
    pub fn audio<S: Into<String>>(data: S, mime_type: S) -> Self {
        Self::Audio {
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
            meta: None,
        }
    }
}

impl Annotations {
    /// Create new annotations
    pub fn new() -> Self {
        Self {
            audience: None,
            priority: None,
            last_modified: None,
            danger: None,
            destructive: None,
            read_only: None,
        }
    }

    /// Set priority (0.0 = least important, 1.0 = most important)
    pub fn with_priority(mut self, priority: f64) -> Self {
        self.priority = Some(priority.clamp(0.0, 1.0));
        self
    }

    /// Set audience
    pub fn for_audience(mut self, audience: Vec<Role>) -> Self {
        self.audience = Some(audience);
        self
    }

    /// Set last modified timestamp (ISO 8601 format)
    pub fn with_last_modified<S: Into<String>>(mut self, timestamp: S) -> Self {
        self.last_modified = Some(timestamp.into());
        self
    }

    /// Set audience (legacy compatibility)
    pub fn for_audience_legacy(self, _audience: Vec<AnnotationAudience>) -> Self {
        // Legacy compatibility - ignore audience in new API
        self
    }

    /// Set danger level (legacy compatibility)
    pub fn with_danger_level(mut self, level: DangerLevel) -> Self {
        // Legacy compatibility - set danger field for backward compatibility
        self.danger = Some(level);
        self
    }

    /// Legacy danger field (always returns None for compatibility)
    pub fn danger(&self) -> Option<DangerLevel> {
        None
    }

    /// Legacy audience field (always returns None for compatibility)
    pub fn audience(&self) -> Option<Vec<AnnotationAudience>> {
        None
    }

    /// Set as read-only (legacy compatibility)
    pub fn read_only(mut self) -> Self {
        self.read_only = Some(true);
        self.destructive = Some(false);
        self
    }

    /// Set as destructive (legacy compatibility)
    pub fn destructive(mut self, level: DangerLevel) -> Self {
        self.destructive = Some(true);
        self.read_only = Some(false);
        self.danger = Some(level);
        self
    }
}

impl Tool {
    /// Create a new tool
    pub fn new<S: Into<String>>(name: S, description: S) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: None,
                required: None,
                additional_properties: HashMap::new(),
            },
            output_schema: None,
            annotations: None,
            title: None,
            meta: None,
        }
    }

    /// Add title to the tool
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add annotations to the tool
    pub fn with_annotations(mut self, annotations: ToolAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    /// Add output schema to the tool (2025-06-18 NEW)
    pub fn with_output_schema(mut self, output_schema: ToolOutputSchema) -> Self {
        self.output_schema = Some(output_schema);
        self
    }
}

impl Resource {
    /// Create a new resource
    pub fn new<S: Into<String>>(uri: S, name: S) -> Self {
        Self {
            uri: uri.into(),
            name: name.into(),
            description: None,
            mime_type: None,
            annotations: None,
            size: None,
            title: None,
            meta: None,
        }
    }

    /// Create a resource from legacy format (name was optional)
    pub fn from_legacy<S: Into<String>>(uri: S, name: Option<S>) -> Self {
        Self {
            uri: uri.into(),
            name: name
                .map(|n| n.into())
                .unwrap_or_else(|| "Unnamed Resource".to_string()),
            description: None,
            mime_type: None,
            annotations: None,
            size: None,
            title: None,
            meta: None,
        }
    }

    /// Add title to the resource
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add description to the resource
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl ResourceTemplate {
    /// Create a new resource template
    pub fn new<S: Into<String>>(uri_template: S, name: S) -> Self {
        Self {
            uri_template: uri_template.into(),
            name: name.into(),
            description: None,
            mime_type: None,
            annotations: None,
            title: None,
            meta: None,
        }
    }

    /// Create a resource template from legacy format (name was optional)
    pub fn from_legacy<S: Into<String>>(uri_template: S, name: Option<S>) -> Self {
        Self {
            uri_template: uri_template.into(),
            name: name
                .map(|n| n.into())
                .unwrap_or_else(|| "Unnamed Template".to_string()),
            description: None,
            mime_type: None,
            annotations: None,
            title: None,
            meta: None,
        }
    }

    /// Add title to the resource template
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }
}

impl Prompt {
    /// Create a new prompt
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            description: None,
            arguments: None,
            title: None,
            meta: None,
        }
    }

    /// Add title to the prompt
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add description to the prompt
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl PromptArgument {
    /// Create a new prompt argument
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            description: None,
            required: None,
            title: None,
        }
    }

    /// Add title to the prompt argument
    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Mark as required
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new<T: Serialize>(
        id: RequestId,
        method: String,
        params: Option<T>,
    ) -> std::result::Result<Self, serde_json::Error> {
        let params = match params {
            Some(p) => Some(serde_json::to_value(p)?),
            None => None,
        };

        Ok(Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method,
            params,
        })
    }
}

impl JsonRpcResponse {
    /// Create a successful JSON-RPC response
    pub fn success<T: Serialize>(
        id: RequestId,
        result: T,
    ) -> std::result::Result<Self, serde_json::Error> {
        Ok(Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(serde_json::to_value(result)?),
        })
    }
}

impl JsonRpcError {
    /// Create an error JSON-RPC response
    pub fn error(
        id: RequestId,
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            error: ErrorObject {
                code,
                message,
                data,
            },
        }
    }
}

impl JsonRpcNotification {
    /// Create a new JSON-RPC notification
    pub fn new<T: Serialize>(
        method: String,
        params: Option<T>,
    ) -> std::result::Result<Self, serde_json::Error> {
        let params = match params {
            Some(p) => Some(serde_json::to_value(p)?),
            None => None,
        };

        Ok(Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method,
            params,
        })
    }
}

impl SamplingMessage {
    /// Create a user text message
    pub fn user_text<S: Into<String>>(text: S) -> Self {
        Self {
            role: Role::User,
            content: SamplingContent::text(text),
        }
    }

    /// Create an assistant text message
    pub fn assistant_text<S: Into<String>>(text: S) -> Self {
        Self {
            role: Role::Assistant,
            content: SamplingContent::text(text),
        }
    }

    /// Create a user image message
    pub fn user_image<S: Into<String>>(data: S, mime_type: S) -> Self {
        Self {
            role: Role::User,
            content: SamplingContent::image(data, mime_type),
        }
    }

    /// Create a user audio message
    pub fn user_audio<S: Into<String>>(data: S, mime_type: S) -> Self {
        Self {
            role: Role::User,
            content: SamplingContent::audio(data, mime_type),
        }
    }
}

// ============================================================================
// Error Codes
// ============================================================================

/// Standard JSON-RPC error codes
pub mod error_codes {
    /// Invalid JSON was received
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist / is not available
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s)
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error
    pub const INTERNAL_ERROR: i32 = -32603;

    /// MCP-specific error codes
    pub const TOOL_NOT_FOUND: i32 = -32000;
    pub const RESOURCE_NOT_FOUND: i32 = -32001;
    pub const PROMPT_NOT_FOUND: i32 = -32002;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_protocol_version() {
        assert_eq!(LATEST_PROTOCOL_VERSION, "2025-06-18");
        assert_eq!(JSONRPC_VERSION, "2.0");
    }

    #[test]
    fn test_content_block_types() {
        // Test text content
        let text = ContentBlock::text("Hello, world!");
        let json = serde_json::to_value(&text).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello, world!");

        // Test audio content (2025-06-18)
        let audio = ContentBlock::audio("base64data", "audio/wav");
        let json = serde_json::to_value(&audio).unwrap();
        assert_eq!(json["type"], "audio");
        assert_eq!(json["data"], "base64data");
        assert_eq!(json["mimeType"], "audio/wav");

        // Test resource link content (new in 2025-06-18)
        let resource_link = ContentBlock::resource_link("file:///test.txt", "test file");
        let json = serde_json::to_value(&resource_link).unwrap();
        assert_eq!(json["type"], "resource_link");
        assert_eq!(json["uri"], "file:///test.txt");
        assert_eq!(json["name"], "test file");
    }

    #[test]
    fn test_annotations() {
        let annotations = Annotations::new()
            .with_priority(0.8)
            .for_audience(vec![Role::User, Role::Assistant])
            .with_last_modified("2025-01-12T15:00:58Z");

        assert_eq!(annotations.priority, Some(0.8));
        assert_eq!(
            annotations.audience,
            Some(vec![Role::User, Role::Assistant])
        );
        assert_eq!(
            annotations.last_modified,
            Some("2025-01-12T15:00:58Z".to_string())
        );
    }

    #[test]
    fn test_tool_with_title() {
        let tool = Tool::new("file_reader", "Read files safely")
            .with_title("File Reader Tool")
            .with_annotations(ToolAnnotations::new().with_title("File Reader"));

        assert_eq!(tool.name, "file_reader");
        assert_eq!(tool.title, Some("File Reader Tool".to_string()));
        assert!(tool.annotations.is_some());
        assert_eq!(
            tool.annotations.unwrap().title,
            Some("File Reader".to_string())
        );
    }

    #[test]
    fn test_tool_with_output_schema() {
        use serde_json::json;

        // Create output schema
        let output_schema = ToolOutputSchema::with_properties(HashMap::from([
            ("result".to_string(), json!({"type": "string"})),
            ("count".to_string(), json!({"type": "number"})),
        ]))
        .with_required(vec!["result".to_string()]);

        // Create tool with output schema
        let tool = Tool::new(
            "data_processor",
            "Processes data and returns structured output",
        )
        .with_title("Data Processor")
        .with_output_schema(output_schema);

        // Verify fields
        assert_eq!(tool.name, "data_processor");
        assert_eq!(tool.title, Some("Data Processor".to_string()));
        assert!(tool.output_schema.is_some());

        let schema = tool.output_schema.as_ref().unwrap();
        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties.is_some());
        assert_eq!(schema.required, Some(vec!["result".to_string()]));

        // Test serialization
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["name"], "data_processor");
        assert!(json["inputSchema"].is_object());
        assert!(json["outputSchema"].is_object());
        assert_eq!(json["outputSchema"]["type"], "object");
        assert!(json["outputSchema"]["properties"].is_object());
        assert!(json["outputSchema"]["required"].is_array());
    }

    #[test]
    fn test_server_capabilities_2025_06_18() {
        let caps = ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            completions: Some(CompletionsCapability::default()),
            logging: Some(LoggingCapability::default()),
            experimental: Some(HashMap::new()),
            ..Default::default()
        };

        let json = serde_json::to_value(&caps).unwrap();
        assert!(json["tools"]["listChanged"].as_bool().unwrap());
        assert!(json["completions"].is_object());
        assert!(json["logging"].is_object());
        assert!(json["experimental"].is_object());
    }

    #[test]
    fn test_client_capabilities_with_elicitation() {
        let caps = ClientCapabilities {
            elicitation: Some(ElicitationCapability::default()),
            roots: Some(RootsCapability {
                list_changed: Some(true),
            }),
            ..Default::default()
        };

        let json = serde_json::to_value(&caps).unwrap();
        assert!(json["elicitation"].is_object());
        assert!(json["roots"]["listChanged"].as_bool().unwrap());
    }

    #[test]
    fn test_implementation_with_title() {
        let impl_info = Implementation::with_title("my-server", "1.0.0", "My Awesome Server");

        assert_eq!(impl_info.name, "my-server");
        assert_eq!(impl_info.version, "1.0.0");
        assert_eq!(impl_info.title, Some("My Awesome Server".to_string()));
    }

    #[test]
    fn test_model_preferences_improved() {
        let prefs = ModelPreferences {
            cost_priority: Some(0.3),
            speed_priority: Some(0.7),
            intelligence_priority: Some(0.9),
            hints: Some(vec![ModelHint {
                name: Some("claude".to_string()),
                additional_hints: None,
            }]),
        };

        let json = serde_json::to_value(&prefs).unwrap();
        assert_eq!(json["costPriority"], 0.3);
        assert_eq!(json["speedPriority"], 0.7);
        assert_eq!(json["intelligencePriority"], 0.9);
        assert!(json["hints"].is_array());
    }

    #[test]
    fn test_call_tool_result_with_structured_content() {
        let result = CallToolResult {
            content: vec![ContentBlock::text("Operation completed")],
            is_error: Some(false),
            structured_content: Some(json!({"status": "success", "count": 42})),
            meta: None,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert!(json["content"].is_array());
        assert_eq!(json["isError"], false);
        assert_eq!(json["structuredContent"]["status"], "success");
        assert_eq!(json["structuredContent"]["count"], 42);
    }

    #[test]
    fn test_sampling_content_types() {
        // Test that SamplingContent doesn't include resource_link
        let text = SamplingContent::text("Hello");
        let image = SamplingContent::image("data", "image/png");
        let audio = SamplingContent::audio("data", "audio/wav");

        let text_json = serde_json::to_value(&text).unwrap();
        let image_json = serde_json::to_value(&image).unwrap();
        let audio_json = serde_json::to_value(&audio).unwrap();

        assert_eq!(text_json["type"], "text");
        assert_eq!(image_json["type"], "image");
        assert_eq!(audio_json["type"], "audio");
    }
}

// ============================================================================
// Legacy/Compatibility Types for Tests
// ============================================================================

/// Batch request type alias for compatibility
pub type JsonRpcBatchRequest = Vec<JsonRpcRequest>;

/// Batch response type alias for compatibility
pub type JsonRpcBatchResponse = Vec<JsonRpcResponse>;

/// Request or notification union for compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JsonRpcRequestOrNotification {
    Request(JsonRpcRequest),
    Notification(JsonRpcNotification),
}

/// Response or error union for compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JsonRpcResponseOrError {
    Response(JsonRpcResponse),
    Error(JsonRpcError),
}

/// Annotation audience for content targeting (legacy)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnnotationAudience {
    User,
    Developer,
    System,
}

/// Danger level for tool safety annotations (legacy)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DangerLevel {
    Safe,
    Low,
    Medium,
    High,
}
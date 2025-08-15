// ! improved Schema Introspection for MCP Protocol (2025-06-18)
// !
// ! Module provides complete schema introspection capabilities,
// ! allowing clients to discover the full structure and capabilities of
// ! the MCP server at runtime

use crate::protocol::discovery::*;
use crate::protocol::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Schema Introspection Types
// ============================================================================

/// Introspection result with schema information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntrospectionResult {
    /// Protocol version and compatibility information
    pub protocol: ProtocolInfo,

    /// Method schemas
    pub methods: MethodSchemas,

    /// Type definitions used across the protocol
    pub types: TypeDefinitions,

    /// Capability schemas
    pub capabilities: CapabilitySchemas,

    /// Transport information
    pub transports: Vec<TransportInfo>,

    /// Extensions and experimental features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<ExtensionInfo>,
}

/// Protocol version and compatibility information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolInfo {
    /// Current protocol version
    pub version: String,

    /// Minimum compatible version
    pub min_version: String,

    /// Maximum compatible version
    pub max_version: String,

    /// List of all supported versions
    pub supported_versions: Vec<String>,

    /// Protocol features by version
    pub version_features: HashMap<String, Vec<String>>,
}

/// Method schemas with documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodSchemas {
    /// Request methods (client to server)
    pub requests: Vec<MethodSchema>,

    /// Server-initiated methods (server to client)
    pub server_requests: Vec<MethodSchema>,

    /// Notification methods
    pub notifications: Vec<MethodSchema>,

    /// Subscription methods
    pub subscriptions: Vec<MethodSchema>,
}

/// Schema for a single method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodSchema {
    /// Method name
    pub name: String,

    /// Human-readable title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Method category
    pub category: String,

    /// JSON Schema for parameters
    pub params: serde_json::Value,

    /// JSON Schema for result
    pub result: serde_json::Value,

    /// Error schemas Method can return
    pub errors: Vec<ErrorSchema>,

    /// Examples of usage
    pub examples: Vec<MethodExample>,

    /// Method-specific metadata
    pub metadata: MethodMetadata,
}

/// Error schema definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorSchema {
    /// Error code
    pub code: i32,

    /// Error name
    pub name: String,

    /// Error description
    pub description: String,

    /// Schema for error data field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_schema: Option<serde_json::Value>,
}

/// Example of method usage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodExample {
    /// Example title
    pub title: String,

    /// Example description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Example request
    pub request: serde_json::Value,

    /// Example response
    pub response: serde_json::Value,
}

/// Method-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodMetadata {
    /// Whether method requires authentication
    pub requires_auth: bool,

    /// Whether method supports progress notifications
    pub supports_progress: bool,

    /// Whether method supports cancellation
    pub supports_cancellation: bool,

    /// Whether method supports batching
    pub supports_batching: bool,

    /// Rate limiting information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitInfo>,

    /// Deprecation information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<DeprecationInfo>,

    /// Version when method was introduced
    pub since_version: String,

    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Deprecation information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeprecationInfo {
    /// Whether method is deprecated
    pub deprecated: bool,

    /// Version when deprecated
    pub since: String,

    /// Version when it will be removed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_version: Option<String>,

    /// Replacement method or alternative
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,

    /// Deprecation message
    pub message: String,
}

/// Type definitions used across the protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypeDefinitions {
    /// Core types (ContentBlock, etc.)
    pub core: HashMap<String, serde_json::Value>,

    /// Request parameter types
    pub requests: HashMap<String, serde_json::Value>,

    /// Response result types
    pub responses: HashMap<String, serde_json::Value>,

    /// Notification types
    pub notifications: HashMap<String, serde_json::Value>,

    /// Capability types
    pub capabilities: HashMap<String, serde_json::Value>,

    /// Custom/extension types
    pub custom: HashMap<String, serde_json::Value>,
}

/// Capability schemas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilitySchemas {
    /// Server capability schema
    pub server: serde_json::Value,

    /// Client capability schema
    pub client: serde_json::Value,

    /// Individual capability details
    pub capabilities: Vec<CapabilityDetail>,
}

/// Detailed information about a capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityDetail {
    /// Capability name
    pub name: String,

    /// Capability type (server/client)
    pub capability_type: String,

    /// Description
    pub description: String,

    /// Schema for capability configuration
    pub schema: serde_json::Value,

    /// Methods enabled by this capability
    pub enabled_methods: Vec<String>,

    /// Dependencies on other capabilities
    pub dependencies: Vec<String>,

    /// Version when introduced
    pub since_version: String,
}

/// Transport information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransportInfo {
    /// Transport name
    pub name: String,

    /// Transport type (stdio, http, websocket, etc.)
    pub transport_type: String,

    /// Description
    pub description: String,

    /// Configuration schema
    pub config_schema: serde_json::Value,

    /// Supported features
    pub features: Vec<String>,

    /// Performance characteristics
    pub performance: TransportPerformance,
}

/// Transport performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransportPerformance {
    /// Latency characteristics
    pub latency: String,

    /// Throughput characteristics
    pub throughput: String,

    /// Whether transport supports streaming
    pub streaming: bool,

    /// Whether transport supports multiplexing
    pub multiplexing: bool,

    /// Whether transport supports compression
    pub compression: bool,
}

/// Extension information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtensionInfo {
    /// Available extensions
    pub extensions: Vec<Extension>,

    /// Experimental features
    pub experimental: Vec<ExperimentalFeature>,
}

/// Extension definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Extension {
    /// Extension name
    pub name: String,

    /// Extension version
    pub version: String,

    /// Description
    pub description: String,

    /// Methods added by extension
    pub methods: Vec<String>,

    /// Types added by extension
    pub types: Vec<String>,

    /// Configuration schema
    pub config_schema: serde_json::Value,
}

/// Experimental feature definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentalFeature {
    /// Feature name
    pub name: String,

    /// Description
    pub description: String,

    /// Stability level (alpha, beta, rc)
    pub stability: String,

    /// Feature flag to enable
    pub flag: String,

    /// Expected stable version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stable_version: Option<String>,
}

// ============================================================================
// Schema Builder
// ============================================================================

/// Builder for creating introspection schemas
pub struct SchemaBuilder {
    protocol: ProtocolInfo,
    methods: MethodSchemas,
    types: TypeDefinitions,
    capabilities: CapabilitySchemas,
    transports: Vec<TransportInfo>,
    extensions: Option<ExtensionInfo>,
}

impl SchemaBuilder {
    /// Create a new schema builder for MCP 2025-06-18
    pub fn new() -> Self {
        Self {
            protocol: ProtocolInfo {
                version: LATEST_PROTOCOL_VERSION.to_string(),
                min_version: "2024-11-05".to_string(),
                max_version: LATEST_PROTOCOL_VERSION.to_string(),
                supported_versions: vec![
                    "2024-11-05".to_string(),
                    "2025-03-26".to_string(),
                    "2025-06-18".to_string(),
                ],
                version_features: Self::build_version_features(),
            },
            methods: MethodSchemas {
                requests: Vec::new(),
                server_requests: Vec::new(),
                notifications: Vec::new(),
                subscriptions: Vec::new(),
            },
            types: TypeDefinitions {
                core: HashMap::new(),
                requests: HashMap::new(),
                responses: HashMap::new(),
                notifications: HashMap::new(),
                capabilities: HashMap::new(),
                custom: HashMap::new(),
            },
            capabilities: CapabilitySchemas {
                server: serde_json::json!({}),
                client: serde_json::json!({}),
                capabilities: Vec::new(),
            },
            transports: Vec::new(),
            extensions: None,
        }
    }

    /// Build version features map
    fn build_version_features() -> HashMap<String, Vec<String>> {
        let mut features = HashMap::new();

        features.insert(
            "2024-11-05".to_string(),
            vec![
                "core-protocol".to_string(),
                "tools".to_string(),
                "resources".to_string(),
                "prompts".to_string(),
                "sampling".to_string(),
            ],
        );

        features.insert(
            "2025-03-26".to_string(),
            vec![
                "streamable-http".to_string(),
                "json-rpc-batching".to_string(),
                "improved-metadata".to_string(),
            ],
        );

        features.insert(
            "2025-06-18".to_string(),
            vec![
                "elicitation".to_string(),
                "audio-content".to_string(),
                "resource-links".to_string(),
                "structured-tool-output".to_string(),
                "oauth-2.1".to_string(),
                "improved-annotations".to_string(),
            ],
        );

        features
    }

    /// Add a method schema
    pub fn add_method(mut self, schema: MethodSchema, category: &str) -> Self {
        match category {
            "request" => self.methods.requests.push(schema),
            "server_request" => self.methods.server_requests.push(schema),
            "notification" => self.methods.notifications.push(schema),
            "subscription" => self.methods.subscriptions.push(schema),
            _ => {}
        }
        self
    }

    /// Add a type definition
    pub fn add_type(mut self, category: &str, name: String, schema: serde_json::Value) -> Self {
        match category {
            "core" => {
                self.types.core.insert(name, schema);
            }
            "request" => {
                self.types.requests.insert(name, schema);
            }
            "response" => {
                self.types.responses.insert(name, schema);
            }
            "notification" => {
                self.types.notifications.insert(name, schema);
            }
            "capability" => {
                self.types.capabilities.insert(name, schema);
            }
            "custom" => {
                self.types.custom.insert(name, schema);
            }
            _ => {}
        }
        self
    }

    /// Add a transport
    pub fn add_transport(mut self, transport: TransportInfo) -> Self {
        self.transports.push(transport);
        self
    }

    /// Add a capability
    pub fn add_capability(mut self, capability: CapabilityDetail) -> Self {
        self.capabilities.capabilities.push(capability);
        self
    }

    /// Build the introspection result
    pub fn build(self) -> IntrospectionResult {
        IntrospectionResult {
            protocol: self.protocol,
            methods: self.methods,
            types: self.types,
            capabilities: self.capabilities,
            transports: self.transports,
            extensions: self.extensions,
        }
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Introspection Provider
// ============================================================================

/// Provider for schema introspection
pub struct IntrospectionProvider {
    #[allow(dead_code)]
    builder: SchemaBuilder,
}

impl IntrospectionProvider {
    /// Create a new introspection provider
    pub fn new() -> Self {
        Self {
            builder: SchemaBuilder::new(),
        }
    }

    /// Build introspection for MCP 2025-06-18
    pub fn build_complete_introspection(&self) -> IntrospectionResult {
        let mut builder = SchemaBuilder::new();

        // Add standard transports
        builder = builder
            .add_transport(TransportInfo {
                name: "stdio".to_string(),
                transport_type: "stdio".to_string(),
                description: "Standard input/output transport for local processes".to_string(),
                config_schema: serde_json::json!({}),
                features: vec!["bidirectional".to_string(), "low-latency".to_string()],
                performance: TransportPerformance {
                    latency: "microseconds".to_string(),
                    throughput: "high".to_string(),
                    streaming: true,
                    multiplexing: false,
                    compression: false,
                },
            })
            .add_transport(TransportInfo {
                name: "http-sse".to_string(),
                transport_type: "http".to_string(),
                description: "HTTP with Server-Sent Events for web-based communication".to_string(),
                config_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {"type": "string"},
                        "headers": {"type": "object"}
                    }
                }),
                features: vec![
                    "web-compatible".to_string(),
                    "firewall-friendly".to_string(),
                ],
                performance: TransportPerformance {
                    latency: "milliseconds".to_string(),
                    throughput: "medium".to_string(),
                    streaming: true,
                    multiplexing: false,
                    compression: true,
                },
            })
            .add_transport(TransportInfo {
                name: "websocket".to_string(),
                transport_type: "websocket".to_string(),
                description: "WebSocket transport for real-time bidirectional communication"
                    .to_string(),
                config_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {"type": "string"},
                        "protocols": {"type": "array"}
                    }
                }),
                features: vec!["real-time".to_string(), "bidirectional".to_string()],
                performance: TransportPerformance {
                    latency: "low-milliseconds".to_string(),
                    throughput: "high".to_string(),
                    streaming: true,
                    multiplexing: true,
                    compression: true,
                },
            });

        // Add core capabilities
        builder = builder
            .add_capability(CapabilityDetail {
                name: "tools".to_string(),
                capability_type: "server".to_string(),
                description: "Ability to expose and execute tools".to_string(),
                schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "listChanged": {"type": "boolean"}
                    }
                }),
                enabled_methods: vec!["tools/list".to_string(), "tools/call".to_string()],
                dependencies: vec![],
                since_version: "2024-11-05".to_string(),
            })
            .add_capability(CapabilityDetail {
                name: "elicitation".to_string(),
                capability_type: "client".to_string(),
                description: "Ability to collect user input through forms".to_string(),
                schema: serde_json::json!({}),
                enabled_methods: vec!["elicitation/create".to_string()],
                dependencies: vec![],
                since_version: "2025-06-18".to_string(),
            });

        builder.build()
    }
}

impl Default for IntrospectionProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builder() {
        let builder = SchemaBuilder::new();
        let result = builder.build();

        assert_eq!(result.protocol.version, "2025-06-18");
        assert!(
            result
                .protocol
                .supported_versions
                .contains(&"2025-06-18".to_string())
        );
    }

    #[test]
    fn test_introspection_provider() {
        let provider = IntrospectionProvider::new();
        let introspection = provider.build_complete_introspection();

        assert!(!introspection.transports.is_empty());
        assert!(!introspection.capabilities.capabilities.is_empty());

        // Check for specific transports
        assert!(introspection.transports.iter().any(|t| t.name == "stdio"));
        assert!(
            introspection
                .transports
                .iter()
                .any(|t| t.name == "websocket")
        );

        // Check for specific capabilities
        assert!(
            introspection
                .capabilities
                .capabilities
                .iter()
                .any(|c| c.name == "tools")
        );
        assert!(
            introspection
                .capabilities
                .capabilities
                .iter()
                .any(|c| c.name == "elicitation")
        );
    }

    #[test]
    fn test_version_features() {
        let features = SchemaBuilder::build_version_features();

        assert!(features.contains_key("2025-06-18"));
        let v2025_features = &features["2025-06-18"];
        assert!(v2025_features.contains(&"elicitation".to_string()));
        assert!(v2025_features.contains(&"audio-content".to_string()));
        assert!(v2025_features.contains(&"oauth-2.1".to_string()));
    }
}

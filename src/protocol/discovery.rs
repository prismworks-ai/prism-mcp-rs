// ! RPC Discovery Module for MCP Protocol
// !
// ! Module implements the optional `rpc.discover` mechanism that allows clients
// ! to dynamically discover available methods, their parameters, and capabilities.
// ! This enables introspection of the MCP server's capabilities at runtime.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Discovery Types
// ============================================================================

/// Request for discovering available RPC methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoverRequest {
    /// Optional filter to limit discovery to specific categories
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<DiscoveryFilter>,

    /// Whether to include detailed parameter schemas
    #[serde(default = "default_include_schemas")]
    pub include_schemas: bool,

    /// Whether to include capability information
    #[serde(default = "default_include_capabilities")]
    pub include_capabilities: bool,
}

fn default_include_schemas() -> bool {
    true
}

fn default_include_capabilities() -> bool {
    true
}

/// Filter for discovery requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DiscoveryFilter {
    /// Discover only client methods (methods the client can call on the server)
    Client,
    /// Discover only server methods (methods the server can call on the client)
    Server,
    /// Discover only notification methods
    Notifications,
    /// Discover methods by category
    Category(String),
    /// Discover all methods (default)
    All,
}

/// Response containing discovered RPC methods and capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoverResult {
    /// Protocol version
    pub protocol_version: String,

    /// Available methods grouped by category
    pub methods: HashMap<String, Vec<MethodInfo>>,

    /// Server capabilities (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<DiscoveredCapabilities>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DiscoveryMetadata>,
}

/// Information about a single RPC method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodInfo {
    /// Method name (e.g., "tools/list")
    pub name: String,

    /// Human-readable description of the method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Method type
    pub method_type: MethodType,

    /// Direction of the method call
    pub direction: MethodDirection,

    /// JSON Schema for request parameters (if include_schemas is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params_schema: Option<serde_json::Value>,

    /// JSON Schema for response result (if include_schemas is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_schema: Option<serde_json::Value>,

    /// Whether Method requires authentication
    #[serde(default)]
    pub requires_auth: bool,

    /// Whether Method supports progress notifications
    #[serde(default)]
    pub supports_progress: bool,

    /// Whether Method supports cancellation
    #[serde(default)]
    pub supports_cancellation: bool,

    /// Tags for categorization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Type of RPC method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MethodType {
    /// Request-response method
    Request,
    /// One-way notification
    Notification,
    /// Subscription method
    Subscription,
}

/// Direction of method invocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MethodDirection {
    /// Client calls server
    ClientToServer,
    /// Server calls client
    ServerToClient,
    /// Can be called in either direction
    Bidirectional,
}

/// Discovered capabilities information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredCapabilities {
    /// Server capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerCapabilityInfo>,

    /// Required client capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_client: Option<ClientCapabilityInfo>,

    /// Optional client capabilities that enhance functionality
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_client: Option<ClientCapabilityInfo>,
}

/// Server capability information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerCapabilityInfo {
    /// Whether the server supports tools
    pub tools: bool,

    /// Whether the server supports resources
    pub resources: bool,

    /// Whether the server supports prompts
    pub prompts: bool,

    /// Whether the server supports logging
    pub logging: bool,

    /// Whether the server supports completions
    pub completions: bool,

    /// List of experimental capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Vec<String>>,
}

/// Client capability information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientCapabilityInfo {
    /// Whether the client should support sampling
    pub sampling: bool,

    /// Whether the client should support roots
    pub roots: bool,

    /// Whether the client should support elicitation
    pub elicitation: bool,

    /// List of experimental capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Vec<String>>,
}

/// Additional metadata for discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveryMetadata {
    /// Server implementation name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,

    /// Server implementation version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,

    /// API documentation URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,

    /// Support contact information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_contact: Option<String>,

    /// Rate limiting information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<HashMap<String, RateLimitInfo>>,
}

/// Rate limiting information for methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateLimitInfo {
    /// Maximum requests per time window
    pub max_requests: u32,

    /// Time window in seconds
    pub window_seconds: u32,

    /// Whether rate limiting is per-method or global
    #[serde(default)]
    pub per_method: bool,
}

// ============================================================================
// Discovery Implementation
// ============================================================================

/// Registry of available RPC methods for discovery
pub struct MethodRegistry {
    methods: Vec<MethodInfo>,
}

impl MethodRegistry {
    /// Create a new method registry
    pub fn new() -> Self {
        Self {
            methods: Vec::new(),
        }
    }

    /// Register a new method
    pub fn register(&mut self, method: MethodInfo) {
        self.methods.push(method);
    }

    /// Get all registered methods
    pub fn get_methods(&self) -> &[MethodInfo] {
        &self.methods
    }

    /// Filter methods by category
    pub fn filter_by_category(&self, category: &str) -> Vec<&MethodInfo> {
        self.methods
            .iter()
            .filter(|m| {
                m.tags
                    .as_ref()
                    .is_some_and(|tags| tags.contains(&category.to_string()))
            })
            .collect()
    }

    /// Filter methods by direction
    pub fn filter_by_direction(&self, direction: MethodDirection) -> Vec<&MethodInfo> {
        self.methods
            .iter()
            .filter(|m| m.direction == direction)
            .collect()
    }

    /// Filter methods by type
    pub fn filter_by_type(&self, method_type: MethodType) -> Vec<&MethodInfo> {
        self.methods
            .iter()
            .filter(|m| m.method_type == method_type)
            .collect()
    }

    /// Build the standard MCP method registry
    pub fn build_standard_registry() -> Self {
        let mut registry = Self::new();

        // Core protocol methods
        registry.register(MethodInfo {
            name: "initialize".to_string(),
            description: Some("Initialize the MCP connection".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: false,
            tags: Some(vec!["core".to_string(), "initialization".to_string()]),
        });

        registry.register(MethodInfo {
            name: "ping".to_string(),
            description: Some("Check connection liveness".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::Bidirectional,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: false,
            tags: Some(vec!["core".to_string(), "health".to_string()]),
        });

        // Tool methods
        registry.register(MethodInfo {
            name: "tools/list".to_string(),
            description: Some("List available tools".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: true,
            tags: Some(vec!["tools".to_string()]),
        });

        registry.register(MethodInfo {
            name: "tools/call".to_string(),
            description: Some("Call a tool with arguments".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: true,
            supports_cancellation: true,
            tags: Some(vec!["tools".to_string()]),
        });

        // Resource methods
        registry.register(MethodInfo {
            name: "resources/list".to_string(),
            description: Some("List available resources".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: true,
            tags: Some(vec!["resources".to_string()]),
        });

        registry.register(MethodInfo {
            name: "resources/read".to_string(),
            description: Some("Read a resource by URI".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: true,
            supports_cancellation: true,
            tags: Some(vec!["resources".to_string()]),
        });

        // Prompt methods
        registry.register(MethodInfo {
            name: "prompts/list".to_string(),
            description: Some("List available prompts".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: true,
            tags: Some(vec!["prompts".to_string()]),
        });

        registry.register(MethodInfo {
            name: "prompts/get".to_string(),
            description: Some("Get a prompt by name".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: true,
            tags: Some(vec!["prompts".to_string()]),
        });

        // Sampling methods (server to client)
        registry.register(MethodInfo {
            name: "sampling/createMessage".to_string(),
            description: Some("Request message generation from client's LLM".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ServerToClient,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: true,
            supports_cancellation: true,
            tags: Some(vec!["sampling".to_string(), "llm".to_string()]),
        });

        // Roots methods (server to client)
        registry.register(MethodInfo {
            name: "roots/list".to_string(),
            description: Some("List client's root directories".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ServerToClient,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: false,
            tags: Some(vec!["roots".to_string(), "filesystem".to_string()]),
        });

        // Elicitation methods (server to client)
        registry.register(MethodInfo {
            name: "elicitation/create".to_string(),
            description: Some("Request user input through a form".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ServerToClient,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: true,
            tags: Some(vec!["elicitation".to_string(), "user-input".to_string()]),
        });

        // Completion methods
        registry.register(MethodInfo {
            name: "completion/complete".to_string(),
            description: Some("Get completion suggestions".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: true,
            tags: Some(vec!["completion".to_string(), "autocomplete".to_string()]),
        });

        // Logging methods
        registry.register(MethodInfo {
            name: "logging/setLevel".to_string(),
            description: Some("Set logging level".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: false,
            tags: Some(vec!["logging".to_string()]),
        });

        // Discovery method itself
        registry.register(MethodInfo {
            name: "rpc.discover".to_string(),
            description: Some("Discover available RPC methods and capabilities".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: false,
            tags: Some(vec!["discovery".to_string(), "meta".to_string()]),
        });

        // Notification methods
        registry.register(MethodInfo {
            name: "notifications/initialized".to_string(),
            description: Some("Client initialization complete notification".to_string()),
            method_type: MethodType::Notification,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: false,
            tags: Some(vec![
                "notifications".to_string(),
                "initialization".to_string(),
            ]),
        });

        registry.register(MethodInfo {
            name: "notifications/cancelled".to_string(),
            description: Some("Request cancellation notification".to_string()),
            method_type: MethodType::Notification,
            direction: MethodDirection::Bidirectional,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: false,
            tags: Some(vec!["notifications".to_string(), "control".to_string()]),
        });

        registry.register(MethodInfo {
            name: "notifications/progress".to_string(),
            description: Some("Progress update notification".to_string()),
            method_type: MethodType::Notification,
            direction: MethodDirection::Bidirectional,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: false,
            tags: Some(vec!["notifications".to_string(), "progress".to_string()]),
        });

        registry
    }
}

impl Default for MethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_registry_creation() {
        let registry = MethodRegistry::build_standard_registry();
        let methods = registry.get_methods();

        // Check that we have registered methods
        assert!(!methods.is_empty());

        // Check for specific core methods
        assert!(methods.iter().any(|m| m.name == "initialize"));
        assert!(methods.iter().any(|m| m.name == "ping"));
        assert!(methods.iter().any(|m| m.name == "rpc.discover"));
    }

    #[test]
    fn test_method_filtering() {
        let registry = MethodRegistry::build_standard_registry();

        // Test filtering by direction
        let client_to_server = registry.filter_by_direction(MethodDirection::ClientToServer);
        assert!(!client_to_server.is_empty());

        let server_to_client = registry.filter_by_direction(MethodDirection::ServerToClient);
        assert!(!server_to_client.is_empty());

        // Test filtering by type
        let requests = registry.filter_by_type(MethodType::Request);
        assert!(!requests.is_empty());

        let notifications = registry.filter_by_type(MethodType::Notification);
        assert!(!notifications.is_empty());

        // Test filtering by category
        let tool_methods = registry.filter_by_category("tools");
        assert!(!tool_methods.is_empty());
        assert!(tool_methods.iter().all(|m| m.name.starts_with("tools/")));
    }

    #[test]
    fn test_discover_request_serialization() {
        let request = DiscoverRequest {
            filter: Some(DiscoveryFilter::Client),
            include_schemas: true,
            include_capabilities: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: DiscoverRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request, deserialized);
    }
}

// ! Discovery handler for the RPC discovery mechanism
// !
// ! Module provides the handler for the `rpc.discover` method, allowing
// ! clients to introspect server capabilities and available methods at runtime.

use serde_json::Value;
use std::collections::HashMap;

use crate::core::error::{McpError, McpResult};
use crate::protocol::LATEST_PROTOCOL_VERSION;
use crate::protocol::discovery::*;
use crate::protocol::types::{ServerCapabilities, ServerInfo};

/// Handler for RPC discovery requests
pub struct DiscoveryHandler {
    registry: MethodRegistry,
}

impl DiscoveryHandler {
    /// Create a new discovery handler with the standard MCP method registry
    pub fn new() -> Self {
        Self {
            registry: MethodRegistry::build_standard_registry(),
        }
    }

    /// Create a discovery handler with a custom method registry
    pub fn with_registry(registry: MethodRegistry) -> Self {
        Self { registry }
    }

    /// Handle an rpc.discover request
    pub async fn handle(
        &self,
        server_info: &ServerInfo,
        capabilities: &ServerCapabilities,
        params: Option<Value>,
    ) -> McpResult<DiscoverResult> {
        let request: DiscoverRequest = match params {
            Some(p) => serde_json::from_value(p)
                .map_err(|e| McpError::Validation(format!("Invalid discover params: {e}")))?,
            None => DiscoverRequest {
                filter: Some(DiscoveryFilter::All),
                include_schemas: true,
                include_capabilities: true,
            },
        };

        // Filter methods based on the request
        let filtered_methods = match &request.filter {
            Some(DiscoveryFilter::Client) => self
                .registry
                .filter_by_direction(MethodDirection::ClientToServer),
            Some(DiscoveryFilter::Server) => self
                .registry
                .filter_by_direction(MethodDirection::ServerToClient),
            Some(DiscoveryFilter::Notifications) => {
                self.registry.filter_by_type(MethodType::Notification)
            }
            Some(DiscoveryFilter::Category(category)) => self.registry.filter_by_category(category),
            Some(DiscoveryFilter::All) | None => self.registry.get_methods().iter().collect(),
        };

        // Group methods by category
        let mut methods_by_category: HashMap<String, Vec<MethodInfo>> = HashMap::new();

        for method in filtered_methods {
            let category = if let Some(tags) = &method.tags {
                tags.first()
                    .cloned()
                    .unwrap_or_else(|| "uncategorized".to_string())
            } else {
                "uncategorized".to_string()
            };

            let mut method_info = method.clone();

            // Clear schema fields if not requested
            if !request.include_schemas {
                method_info.params_schema = None;
                method_info.result_schema = None;
            }

            methods_by_category
                .entry(category)
                .or_default()
                .push(method_info);
        }

        // Build capabilities information if requested
        let discovered_capabilities = if request.include_capabilities {
            Some(DiscoveredCapabilities {
                server: Some(ServerCapabilityInfo {
                    tools: capabilities.tools.is_some(),
                    resources: capabilities.resources.is_some(),
                    prompts: capabilities.prompts.is_some(),
                    logging: capabilities.logging.is_some(),
                    completions: capabilities.completions.is_some(),
                    experimental: capabilities
                        .experimental
                        .as_ref()
                        .map(|exp| exp.keys().cloned().collect()),
                }),
                required_client: None, // Can be customized based on server requirements
                optional_client: Some(ClientCapabilityInfo {
                    sampling: true,
                    roots: true,
                    elicitation: true,
                    experimental: None,
                }),
            })
        } else {
            None
        };

        // Build metadata
        let metadata = Some(DiscoveryMetadata {
            server_name: Some(server_info.name.clone()),
            server_version: Some(server_info.version.clone()),
            documentation_url: None, // Can be customized
            support_contact: None,   // Can be customized
            rate_limits: None,       // Can be customized
        });

        Ok(DiscoverResult {
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
            methods: methods_by_category,
            capabilities: discovered_capabilities,
            metadata,
        })
    }
}

impl Default for DiscoveryHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::Implementation;

    #[tokio::test]
    async fn test_discovery_handler() {
        let handler = DiscoveryHandler::new();
        let server_info = Implementation::new("test-server", "1.0.0");
        let capabilities = ServerCapabilities::default();

        // Test with no params (defaults to all)
        let result = handler
            .handle(&server_info, &capabilities, None)
            .await
            .unwrap();

        assert_eq!(result.protocol_version, LATEST_PROTOCOL_VERSION);
        assert!(!result.methods.is_empty());
        assert!(result.capabilities.is_some());
        assert!(result.metadata.is_some());
    }

    #[tokio::test]
    async fn test_discovery_with_filter() {
        let handler = DiscoveryHandler::new();
        let server_info = Implementation::new("test-server", "1.0.0");
        let capabilities = ServerCapabilities::default();

        // Test with client filter
        let params = serde_json::json!({
            "filter": "client",
            "include_schemas": false,
            "include_capabilities": false
        });

        let result = handler
            .handle(&server_info, &capabilities, Some(params))
            .await
            .unwrap();

        // All methods should be client-to-server
        for methods in result.methods.values() {
            for method in methods {
                assert_eq!(method.direction, MethodDirection::ClientToServer);
                assert!(method.params_schema.is_none());
                assert!(method.result_schema.is_none());
            }
        }

        assert!(result.capabilities.is_none());
    }

    #[tokio::test]
    async fn test_discovery_with_category_filter() {
        let handler = DiscoveryHandler::new();
        let server_info = Implementation::new("test-server", "1.0.0");
        let capabilities = ServerCapabilities::default();

        // Test with category filter
        let params = serde_json::json!({
            "filter": {"category": "tools"}
        });

        let result = handler
            .handle(&server_info, &capabilities, Some(params))
            .await
            .unwrap();

        // Should only have tool-related methods
        for methods in result.methods.values() {
            for method in methods {
                assert!(method.name.starts_with("tools/"));
            }
        }
    }
}

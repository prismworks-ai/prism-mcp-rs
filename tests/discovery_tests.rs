// ! Tests for the RPC Discovery mechanism

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::protocol::discovery::*;
use prism_mcp_rs::server::discovery_handler::DiscoveryHandler;
use serde_json::json;

#[tokio::test]
async fn test_discovery_all_methods() {
    let handler = DiscoveryHandler::new();
    let server_info = Implementation::new("test-server", "1.0.0");
    let capabilities = ServerCapabilities::default();

    let request = json!({
        "filter": "all",
        "include_schemas": false,
        "include_capabilities": true
    });

    let result = handler
        .handle(&server_info, &capabilities, Some(request))
        .await
        .unwrap();

    assert_eq!(result.protocol_version, LATEST_PROTOCOL_VERSION);
    assert!(!result.methods.is_empty());
    assert!(result.capabilities.is_some());

    // Check that core methods are present
    let all_methods: Vec<_> = result.methods.values().flat_map(|v| v.iter()).collect();

    assert!(all_methods.iter().any(|m| m.name == "initialize"));
    assert!(all_methods.iter().any(|m| m.name == "ping"));
    assert!(all_methods.iter().any(|m| m.name == "rpc.discover"));
    assert!(all_methods.iter().any(|m| m.name == "tools/list"));
    assert!(all_methods.iter().any(|m| m.name == "resources/list"));
}

#[tokio::test]
async fn test_discovery_filter_client() {
    let handler = DiscoveryHandler::new();
    let server_info = ServerInfo::new("test-server".to_string(), "1.0.0".to_string());

    let capabilities = ServerCapabilities::default();

    let request = json!({
        "filter": "client",
        "include_schemas": false,
        "include_capabilities": false
    });

    let result = handler
        .handle(&server_info, &capabilities, Some(request))
        .await
        .unwrap();

    // All methods should be client-to-server
    for (_category, methods) in &result.methods {
        for method in methods {
            assert_eq!(method.direction, MethodDirection::ClientToServer);
        }
    }

    assert!(result.capabilities.is_none());
}

#[tokio::test]
async fn test_discovery_filter_server() {
    let handler = DiscoveryHandler::new();
    let server_info = ServerInfo::new("test-server".to_string(), "1.0.0".to_string());

    let capabilities = ServerCapabilities::default();

    let request = json!({
        "filter": "server",
        "include_schemas": false,
        "include_capabilities": false
    });

    let result = handler
        .handle(&server_info, &capabilities, Some(request))
        .await
        .unwrap();

    // All methods should be server-to-client
    for (_category, methods) in &result.methods {
        for method in methods {
            assert_eq!(method.direction, MethodDirection::ServerToClient);
        }
    }

    // Should include methods like sampling/createMessage, roots/list, elicitation/create
    let all_methods: Vec<_> = result.methods.values().flat_map(|v| v.iter()).collect();

    assert!(
        all_methods
            .iter()
            .any(|m| m.name == "sampling/createMessage")
    );
    assert!(all_methods.iter().any(|m| m.name == "roots/list"));
    assert!(all_methods.iter().any(|m| m.name == "elicitation/create"));
}

#[tokio::test]
async fn test_discovery_filter_notifications() {
    let handler = DiscoveryHandler::new();
    let server_info = ServerInfo::new("test-server".to_string(), "1.0.0".to_string());

    let capabilities = ServerCapabilities::default();

    let request = json!({
        "filter": "notifications",
        "include_schemas": false,
        "include_capabilities": false
    });

    let result = handler
        .handle(&server_info, &capabilities, Some(request))
        .await
        .unwrap();

    // All methods should be notifications
    for (_category, methods) in &result.methods {
        for method in methods {
            assert_eq!(method.method_type, MethodType::Notification);
            assert!(method.name.contains("notifications/"));
        }
    }
}

#[tokio::test]
async fn test_discovery_filter_category() {
    let handler = DiscoveryHandler::new();
    let server_info = ServerInfo::new("test-server".to_string(), "1.0.0".to_string());

    let capabilities = ServerCapabilities::default();

    // Test filtering by "tools" category
    let request = json!({
        "filter": {"category": "tools"},
        "include_schemas": false,
        "include_capabilities": false
    });

    let result = handler
        .handle(&server_info, &capabilities, Some(request))
        .await
        .unwrap();

    // All methods should be tool-related
    for (_category, methods) in &result.methods {
        for method in methods {
            assert!(
                method.name.starts_with("tools/")
                    || method.tags.as_ref().unwrap().contains(&"tools".to_string())
            );
        }
    }
}

#[tokio::test]
async fn test_discovery_capabilities() {
    let handler = DiscoveryHandler::new();
    let server_info = ServerInfo::new("test-server".to_string(), "1.0.0".to_string());

    // Create capabilities with various features enabled
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability::default()),
        resources: Some(ResourcesCapability::default()),
        prompts: Some(PromptsCapability::default()),
        logging: Some(LoggingCapability::default()),
        completions: Some(CompletionsCapability::default()),
        sampling: None,
        experimental: None,
    };

    let request = json!({
        "filter": "all",
        "include_schemas": false,
        "include_capabilities": true
    });

    let result = handler
        .handle(&server_info, &capabilities, Some(request))
        .await
        .unwrap();

    assert!(result.capabilities.is_some());
    let caps = result.capabilities.unwrap();

    assert!(caps.server.is_some());
    let server_caps = caps.server.unwrap();
    assert!(server_caps.tools);
    assert!(server_caps.resources);
    assert!(server_caps.prompts);
    assert!(server_caps.logging);
    assert!(server_caps.completions);
}

#[tokio::test]
async fn test_discovery_metadata() {
    let handler = DiscoveryHandler::new();
    let server_info = ServerInfo::new("test-server".to_string(), "2.5.0".to_string());
    let capabilities = ServerCapabilities::default();

    let request = json!({
        "filter": "all",
        "include_schemas": false,
        "include_capabilities": false
    });

    let result = handler
        .handle(&server_info, &capabilities, Some(request))
        .await
        .unwrap();

    assert!(result.metadata.is_some());
    let metadata = result.metadata.unwrap();

    assert_eq!(metadata.server_name, Some("test-server".to_string()));
    assert_eq!(metadata.server_version, Some("2.5.0".to_string()));
}

#[tokio::test]
async fn test_discovery_default_params() {
    let handler = DiscoveryHandler::new();
    let server_info = ServerInfo::new("test-server".to_string(), "1.0.0".to_string());

    let capabilities = ServerCapabilities::default();

    // Test with no params (should use defaults)
    let result = handler
        .handle(&server_info, &capabilities, None)
        .await
        .unwrap();

    // Default should include all methods and capabilities
    assert!(!result.methods.is_empty());
    assert!(result.capabilities.is_some());
}

#[tokio::test]
async fn test_discovery_method_properties() {
    let handler = DiscoveryHandler::new();
    let server_info = ServerInfo::new("test-server".to_string(), "1.0.0".to_string());

    let capabilities = ServerCapabilities::default();

    let result = handler
        .handle(&server_info, &capabilities, None)
        .await
        .unwrap();

    // Find specific methods and check their properties
    let all_methods: Vec<_> = result.methods.values().flat_map(|v| v.iter()).collect();

    // Check tools/call supports progress and cancellation
    let tools_call = all_methods.iter().find(|m| m.name == "tools/call").unwrap();
    assert!(tools_call.supports_progress);
    assert!(tools_call.supports_cancellation);

    // Check initialize doesn't support progress or cancellation
    let initialize = all_methods.iter().find(|m| m.name == "initialize").unwrap();
    assert!(!initialize.supports_progress);
    assert!(!initialize.supports_cancellation);

    // Check ping is bidirectional
    let ping = all_methods.iter().find(|m| m.name == "ping").unwrap();
    assert_eq!(ping.direction, MethodDirection::Bidirectional);
}

#[tokio::test]
async fn test_discovery_serialization() {
    // Test that discovery types serialize/deserialize correctly
    let request = DiscoverRequest {
        filter: Some(DiscoveryFilter::Category("tools".to_string())),
        include_schemas: true,
        include_capabilities: false,
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: DiscoverRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(request, deserialized);

    // Test DiscoverResult serialization
    let mut methods = std::collections::HashMap::new();
    methods.insert(
        "tools".to_string(),
        vec![MethodInfo {
            name: "tools/list".to_string(),
            description: Some("List tools".to_string()),
            method_type: MethodType::Request,
            direction: MethodDirection::ClientToServer,
            params_schema: None,
            result_schema: None,
            requires_auth: false,
            supports_progress: false,
            supports_cancellation: true,
            tags: Some(vec!["tools".to_string()]),
        }],
    );

    let result = DiscoverResult {
        protocol_version: "2025-06-18".to_string(),
        methods,
        capabilities: None,
        metadata: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: DiscoverResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.protocol_version, deserialized.protocol_version);
    assert_eq!(result.methods.len(), deserialized.methods.len());
}

#[tokio::test]
async fn test_discovery_with_server_integration() {
    // Test discovery through actual server
    let server = McpServer::new("discovery-test-server".to_string(), "1.0.0".to_string());

    // Add a tool to the server
    server
        .add_simple_tool("test_tool", "A test tool", |_args| {
            Ok(vec![ContentBlock::text("Test response")])
        })
        .await
        .unwrap();

    // The server should now support discovery
    // This would be tested through actual transport in integration tests
    assert!(server.has_tools().await);
}

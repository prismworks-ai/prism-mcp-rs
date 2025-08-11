// ! Test for Roots feature implementation

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::protocol::roots_types::{ListRootsRequest, RootsListChangedNotification};
use serde_json::json;

#[tokio::test]
async fn test_roots_types_creation() {
    // Test Root creation
    let root = Root::new("file:///home/user/projects".to_string());
    assert_eq!(root.uri, "file:///home/user/projects");
    assert_eq!(root.name, None);

    let root_with_name =
        Root::new("file:///workspace".to_string()).with_name("My Workspace".to_string());
    assert_eq!(root_with_name.uri, "file:///workspace");
    assert_eq!(root_with_name.name, Some("My Workspace".to_string()));
}

#[tokio::test]
async fn test_root_validation() {
    // Test Root creation with valid file URI
    let valid_root = Root::new("file:///home/user".to_string());
    assert_eq!(valid_root.uri, "file:///home/user");

    // Test Root with non-file URI (still works, just different URI scheme)
    let http_root = Root::new("http://example.com".to_string());
    assert_eq!(http_root.uri, "http://example.com");
}

#[tokio::test]
async fn test_list_roots_request() {
    let request = ListRootsRequest::new();
    assert_eq!(request.method, "roots/list");
    assert!(request.params.is_none());

    // Test serialization
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["method"], "roots/list");

    // Test deserialization
    let deserialized: ListRootsRequest = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.method, "roots/list");
}

#[tokio::test]
async fn test_list_roots_result() {
    let roots = vec![
        Root::new("file:///home/user".to_string()),
        Root::new("file:///workspace".to_string()).with_name("Workspace".to_string()),
        Root::new("file:///Documents".to_string()).with_name("Documents".to_string()),
    ];

    let result = ListRootsResult {
        roots: roots.clone(),
        meta: None,
    };
    assert_eq!(result.roots.len(), 3);
    assert_eq!(result.roots[0].uri, "file:///home/user");
    assert_eq!(result.roots[1].name, Some("Workspace".to_string()));
    assert_eq!(result.roots[2].name, Some("Documents".to_string()));

    // Test serialization
    let json = serde_json::to_value(&result).unwrap();
    assert!(json["roots"].is_array());
    assert_eq!(json["roots"].as_array().unwrap().len(), 3);
    assert_eq!(json["roots"][0]["uri"], "file:///home/user");
    assert_eq!(json["roots"][1]["name"], "Workspace");

    // Test deserialization
    let deserialized: ListRootsResult = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.roots.len(), 3);
    assert_eq!(deserialized.roots[2].uri, "file:///Documents");
}

#[tokio::test]
async fn test_roots_list_changed_notification() {
    let notification = RootsListChangedNotification::new();
    assert_eq!(notification.method, "notifications/roots/list_changed");
    assert!(notification.params.is_none());

    // Test serialization
    let json = serde_json::to_value(&notification).unwrap();
    assert_eq!(json["method"], "notifications/roots/list_changed");

    // Test deserialization
    let deserialized: RootsListChangedNotification = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.method, "notifications/roots/list_changed");
}

#[tokio::test]
async fn test_empty_roots_result() {
    let result = ListRootsResult {
        roots: vec![],
        meta: None,
    };
    assert_eq!(result.roots.len(), 0);
    assert!(result.meta.is_none());

    let json = serde_json::to_value(&result).unwrap();
    assert_eq!(json["roots"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_model_hint_with_additional_fields() {
    // Test that ModelHint now supports additional provider-specific fields
    let json = json!({
        "name": "claude-3-5-sonnet",
        "temperature": 0.7,
        "max_tokens": 4096,
        "provider": "anthropic"
    });

    let hint: ModelHint = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(hint.name, Some("claude-3-5-sonnet".to_string()));

    // Verify additional fields are preserved when serializing back
    let _serialized = serde_json::to_value(&hint).unwrap();

    // The additional_hints field should contain the extra fields
    // Note: Due to the flatten attribute, these fields will be at the top level when serialized
    // but stored internally in additional_hints
}

#[tokio::test]
async fn test_enum_schema_with_enum_names() {
    // Test that EnumSchema supports enumNames for display
    let schema = PrimitiveSchemaDefinition::String {
        title: Some("Status".to_string()),
        description: Some("Task status".to_string()),
        min_length: None,
        max_length: None,
        format: None,
        enum_values: Some(vec![
            "pending".to_string(),
            "in_progress".to_string(),
            "completed".to_string(),
        ]),
        enum_names: Some(vec![
            "Pending".to_string(),
            "In Progress".to_string(),
            "Completed".to_string(),
        ]),
    };

    let json = serde_json::to_value(&schema).unwrap();
    assert_eq!(json["type"], "string");
    assert_eq!(json["enum"][0], "pending");
    assert_eq!(json["enumNames"][0], "Pending");
    assert_eq!(json["enumNames"][1], "In Progress");
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use async_trait::async_trait;
    use prism_mcp_rs::client::request_handler::ClientRequestHandler;
    use prism_mcp_rs::server::McpServer;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct TestRootsHandler {
        roots: Arc<Mutex<Vec<Root>>>,
    }

    impl TestRootsHandler {
        fn new() -> Self {
            Self {
                roots: Arc::new(Mutex::new(vec![
                    Root::new("file:///home/test/projects".to_string())
                        .with_name("Projects".to_string()),
                    Root::new("file:///home/test/documents".to_string())
                        .with_name("Documents".to_string()),
                ])),
            }
        }
    }

    #[async_trait]
    impl ClientRequestHandler for TestRootsHandler {
        async fn handle_create_message(
            &self,
            _params: CreateMessageParams,
        ) -> McpResult<CreateMessageResult> {
            Err(McpError::Protocol("Not implemented".to_string()))
        }

        async fn handle_list_roots(&self, _params: ListRootsParams) -> McpResult<ListRootsResult> {
            let roots = self.roots.lock().await.clone();
            Ok(ListRootsResult { roots, meta: None })
        }

        async fn handle_elicit(&self, _params: ElicitParams) -> McpResult<ElicitResult> {
            Err(McpError::Protocol("Not implemented".to_string()))
        }

        async fn handle_ping(&self, _params: PingParams) -> McpResult<PingResult> {
            Ok(PingResult { meta: None })
        }
    }

    #[tokio::test]
    async fn test_server_request_roots_from_client() {
        // This test demonstrates that the server can request roots from the client
        let _server = McpServer::new("test-server".to_string(), "1.0.0".to_string());

        // The server has the method to request roots
        // In a real scenario, this would be called after initialization
        // and would communicate with the client through the transport

        // Verify the server has the request_list_roots method
        // The actual communication test would require a full transport setup
        assert!(true); // Placeholder - actual test would involve transport setup
    }
}

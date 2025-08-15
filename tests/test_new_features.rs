// ! Integration tests for newly implemented features

use prism_mcp_rs::Implementation; // Use the root-level Implementation
use prism_mcp_rs::protocol::*;
use serde_json::json;
use std::collections::HashMap;

// TODO: Fix these tests - RequestMetadata and ResponseMetadata types don't exist
// They need to be updated to use the actual types available in the metadata module

/*
#[test]
fn test_metadata_module() {
    // Test request metadata
    let req_meta = metadata::RequestMetadata::with_progress_token("test-123")
        .add_custom("source", "test")
        .unwrap();

    assert_eq!(req_meta.progress_token, Some(json!("test-123")));
    assert_eq!(req_meta.custom.get("source"), Some(&json!("test")));

    // Test response metadata
    let resp_meta = metadata::ResponseMetadata::with_processing_time(100)
        .set_timestamp("2025-01-12T15:00:00Z")
        .set_request_id("req-123");

    assert_eq!(resp_meta.processing_time, Some(100));
    assert_eq!(
        resp_meta.timestamp,
        Some("2025-01-12T15:00:00Z".to_string())
    );
    assert_eq!(resp_meta.request_id, Some("req-123".to_string()));
}
*/

#[test]
fn test_batch_operations() {
    use batch::*;

    // Test batch request creation
    let batch = BatchRequest::new()
        .add_call(
            json!(1),
            "method1".to_string(),
            Some(json!({"key": "value"})),
        )
        .unwrap()
        .add_notify("notification1".to_string(), Some(json!({"data": "test"})))
        .unwrap();

    assert_eq!(batch.len(), 2);
    assert!(!batch.is_empty());

    // Test batch response
    let batch_resp = BatchResponse::new()
        .add_success(json!(1), json!({"result": "success"}))
        .unwrap()
        .add_failure(
            json!(2),
            error_codes::METHOD_NOT_FOUND,
            "Method not found".to_string(),
            None,
        );

    assert_eq!(batch_resp.len(), 2);
    assert!(!batch_resp.all_successful());
    assert!(batch_resp.has_errors());
}

#[test]
fn test_schema_introspection() {
    use schema_introspection::*;

    // Test schema builder
    let builder = SchemaBuilder::new();
    let result = builder.build();

    assert_eq!(result.protocol.version, "2025-06-18");
    assert!(
        result
            .protocol
            .supported_versions
            .contains(&"2025-06-18".to_string())
    );

    // Test introspection provider
    let provider = IntrospectionProvider::new();
    let introspection = provider.build_complete_introspection();

    assert!(!introspection.transports.is_empty());
    assert!(!introspection.capabilities.capabilities.is_empty());
}

#[test]
fn test_discovery_enhancements() {
    use discovery::*;

    // Test method registry
    let registry = MethodRegistry::build_standard_registry();
    let methods = registry.get_methods();

    assert!(!methods.is_empty());
    assert!(methods.iter().any(|m| m.name == "initialize"));
    assert!(methods.iter().any(|m| m.name == "rpc.discover"));

    // Test filtering
    let tool_methods = registry.filter_by_category("tools");
    assert!(!tool_methods.is_empty());
    assert!(tool_methods.iter().all(|m| m.name.starts_with("tools/")));

    let client_to_server = registry.filter_by_direction(MethodDirection::ClientToServer);
    assert!(!client_to_server.is_empty());
}

#[test]
fn test_messages_have_metadata() {
    use messages::*;

    // Test that request types have metadata fields
    let init_params = InitializeParams {
        protocol_version: "2025-06-18".to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation::new("test", "1.0.0"),
        meta: Some(HashMap::from([("test".to_string(), json!("value"))])),
    };

    assert!(init_params.meta.is_some());

    // Test that response types have metadata fields
    let init_result = InitializeResult {
        protocol_version: "2025-06-18".to_string(),
        capabilities: ServerCapabilities::default(),
        server_info: Implementation::new("test-server", "1.0.0"),
        instructions: None,
        meta: Some(HashMap::from([("processing_time".to_string(), json!(100))])),
    };

    assert!(init_result.meta.is_some());
}
/*
#[test]
fn test_progress_token_in_metadata() {
    use metadata::*;

    // Test progress token in request metadata
    let meta = RequestMetadata::with_progress_token("progress-456");
    assert_eq!(meta.progress_token, Some(json!("progress-456")));

    // Test conversion to/from HashMap (for compatibility)
    let hashmap = meta.to_hashmap();
    assert!(hashmap.is_some());

    let map = hashmap.unwrap();
    assert_eq!(map.get("progressToken"), Some(&json!("progress-456")));

    // Test round-trip conversion
    let meta2 = RequestMetadata::from_hashmap(Some(map));
    assert_eq!(meta2.progress_token, Some(json!("progress-456")));
}
*/

#[cfg(feature = "streaming-http")]
#[test]
fn test_streaming_http_exists() {
    use prism_mcp_rs::transport::streaming_http::*;

    // Test that streaming config exists
    let config = StreamingConfig::default();
    assert!(config.enable_chunked_transfer);
    assert_eq!(config.chunk_threshold, 8192);

    // Test memory improved config
    let mem_config = StreamingConfig::memory_improved();
    assert_eq!(mem_config.chunk_threshold, 4096);

    // Test performance improved config
    let perf_config = StreamingConfig::performance_improved();
    assert_eq!(perf_config.chunk_threshold, 32768);

    // Test content analyzer
    let analyzer = ContentAnalyzer::new();
    assert_eq!(analyzer.streaming_threshold(), 8192);
}
/*
#[test]
fn test_all_modules_accessible() {
    // Verify all new modules are accessible
    use prism_mcp_rs::protocol::{
        batch::BatchRequest, discovery::MethodRegistry, metadata::RequestMetadata,
        schema_introspection::SchemaBuilder,
    };

    // If this compiles, the modules are properly exported
    let _batch = BatchRequest::new();
    let _meta = RequestMetadata::new();
    let _schema = SchemaBuilder::new();
    let _registry = MethodRegistry::new();
}
*/

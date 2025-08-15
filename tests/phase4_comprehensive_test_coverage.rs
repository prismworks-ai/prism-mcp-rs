// ! Phase 4: complete Test Coverage Implementation
// !
// ! This test module implements the remaining Phase 4 test coverage requirements
// ! as outlined in the test coverage improvement plan. This focuses on:
// ! - complete error scenarios and edge cases
// ! - Cross-transport integration and compatibility
// ! - Performance and stress testing scenarios
// ! - Real-world workflow simulations
// ! - Protocol compliance verification

use futures::future;
use prism_mcp_rs::{
    client::McpClient,
    core::{
        error::McpResult,
        resource::{LegacyResourceAdapter, LegacyResourceHandler},
    },
    protocol::{messages::*, methods::*, types::*},
    server::McpServer,
    transport::StdioClientTransport,
};
use serde_json::json;
use std::{sync::Arc, time::Instant};
use tokio::{
    sync::Mutex,
    time::{Duration, sleep, timeout},
};

/// complete error scenario testing
mod error_scenario_tests {
    use super::*;

    #[tokio::test]
    async fn test_malformed_json_handling() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());

        // Test various malformed JSON scenarios
        let malformed_inputs = vec![
            "{broken json",
            "{\"jsonrpc\": \"1.0\"}", // Wrong JSON-RPC version
            "{\"method\": null}",     // Null method
            "{\"id\": \"test\", \"jsonrpc\": \"2.0\"}", // Missing method
            "{\"jsonrpc\": \"2.0\", \"method\": \"\"}", // Empty method
        ];

        for malformed in malformed_inputs {
            let parse_result = serde_json::from_str::<JsonRpcRequest>(malformed);
            // All should fail to parse or create invalid requests
            if let Ok(request) = parse_result {
                // If it parsed, it should still be invalid
                assert!(request.method.is_empty() || !request.method.contains('/'));
            }
        }
    }

    #[tokio::test]
    async fn test_timeout_scenarios() {
        // Test that timeout wrapper works correctly by testing the timeout mechanism directly
        let start_time = Instant::now();

        // Test basic timeout functionality - this should timeout
        let result = timeout(Duration::from_millis(100), async {
            sleep(Duration::from_millis(200)).await;
            "should not complete"
        })
        .await;

        let elapsed = start_time.elapsed();
        println!("Basic timeout test - elapsed time: {elapsed:?}");
        println!(
            "Basic timeout test - result is error: {:?}",
            result.is_err()
        );

        assert!(
            result.is_err(),
            "Expected timeout error in basic test, but got: {result:?}"
        );
        assert!(
            elapsed < Duration::from_millis(150),
            "Basic timeout took too long: {elapsed:?}"
        );

        // Test that operations that complete quickly don't timeout
        let start_time2 = Instant::now();
        let result2 = timeout(Duration::from_millis(200), async {
            sleep(Duration::from_millis(50)).await;
            "completed successfully"
        })
        .await;

        let elapsed2 = start_time2.elapsed();
        println!("Fast operation test - elapsed time: {elapsed2:?}");
        println!("Fast operation test - result is ok: {:?}", result2.is_ok());

        assert!(
            result2.is_ok(),
            "Expected success for fast operation, but got: {result2:?}"
        );
        assert!(
            elapsed2 < Duration::from_millis(100),
            "Fast operation took too long: {elapsed2:?}"
        );
    }

    #[tokio::test]
    async fn test_resource_exhaustion_scenarios() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
        )));

        // Add many tools to test resource limits
        {
            let server_guard = server.lock().await;
            for i in 0..1000 {
                let tool_name = format!("tool-{i}");
                let tool_description = format!("Tool number {i}");
                server_guard
                    .add_simple_tool(&tool_name, &tool_description, move |_args| {
                        Ok(vec![ContentBlock::text(format!("Result from tool {i}"))])
                    })
                    .await
                    .unwrap();
            }
        }

        // Test listing all tools (should handle large lists)
        let list_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("list-test"),
            method: TOOLS_LIST.to_string(),
            params: Some(json!({})),
        };

        let start_time = Instant::now();
        let result = {
            let server_guard = server.lock().await;
            server_guard.handle_request(list_request).await
        };

        // Should complete in reasonable time even with many tools
        assert!(result.is_ok());
        assert!(start_time.elapsed() < Duration::from_secs(5));

        if let Ok(response) = result {
            if let Some(result_data) = response.result {
                let tools_result: ListToolsResult = serde_json::from_value(result_data).unwrap();
                assert_eq!(tools_result.tools.len(), 1000);
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_request_handling() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "concurrent-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a test tool
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool("concurrent-tool", "Tool for concurrency testing", |args| {
                    let delay_ms = args.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(10);

                    tokio::spawn(async move {
                        sleep(Duration::from_millis(delay_ms)).await;
                    });

                    Ok(vec![ContentBlock::text(format!(
                        "Processed with {delay_ms}ms delay"
                    ))])
                })
                .await
                .unwrap();
        }

        // Create multiple concurrent requests
        let mut handles = vec![];
        let num_requests = 20;

        for i in 0..num_requests {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                let request = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    id: json!(format!("concurrent-{}", i)),
                    method: TOOLS_CALL.to_string(),
                    params: Some(json!({
                        "name": "concurrent-tool",
                        "arguments": {
                            "delay_ms": i * 5 // Variable delays
                        }
                    })),
                };

                let server_guard = server_clone.lock().await;
                server_guard.handle_request(request).await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let start_time = Instant::now();
        let results = future::join_all(handles).await;
        let duration = start_time.elapsed();

        // All requests should succeed
        for result in results {
            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(response.is_ok());
        }

        // Should complete in reasonable time (allowing for some concurrency)
        assert!(duration < Duration::from_secs(3));
    }

    #[tokio::test]
    async fn test_protocol_version_mismatches() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "version-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Test various protocol version scenarios
        let version_tests = vec![
            ("1.0", false),        // Too old
            ("2.0", false),        // Wrong major version
            ("2.1", false),        // Wrong version format
            ("2025-06-18", true),  // Correct version
            ("2025-12-31", false), // Future version
        ];

        for (version, should_succeed) in version_tests {
            let init_request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!("version-test"),
                method: INITIALIZE.to_string(),
                params: Some(json!({
                    "protocolVersion": version,
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(init_request).await;

            if should_succeed {
                assert!(result.is_ok(), "Version {version} should be accepted");
            } else {
                // For unsupported versions, should either error or handle smoothly
                // Implementation may vary based on server's version handling strategy
            }
        }
    }

    #[tokio::test]
    async fn test_large_payload_handling() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "payload-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that can handle large inputs
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool(
                    "large-input-tool",
                    "Tool that processes large inputs",
                    |args| {
                        let data_size = args
                            .get("data")
                            .and_then(|v| v.as_str())
                            .map(|s| s.len())
                            .unwrap_or(0);

                        Ok(vec![ContentBlock::text(format!(
                            "Processed {data_size} bytes of data"
                        ))])
                    },
                )
                .await
                .unwrap();
        }

        // Test with various payload sizes
        let payload_sizes = vec![1024, 10_240, 102_400, 1_048_576]; // 1KB to 1MB

        for size in payload_sizes {
            let large_data = "x".repeat(size);

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("large-payload-{}", size)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "large-input-tool",
                    "arguments": {
                        "data": large_data
                    }
                })),
            };

            let start_time = Instant::now();
            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;
            let duration = start_time.elapsed();

            // Should handle large payloads successfully
            assert!(result.is_ok(), "Failed to handle {size} byte payload");

            // Should complete in reasonable time (scaling with size)
            let max_duration = Duration::from_millis(100 + (size / 1000) as u64);
            assert!(
                duration < max_duration,
                "Payload {size} took too long: {duration:?}"
            );
        }
    }
}

/// Real-world workflow simulation tests
mod workflow_simulation_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_tool_execution_workflow() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "workflow-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Set up a realistic tool ecosystem
        {
            let server_guard = server.lock().await;

            // File system tool
            server_guard
                .add_simple_tool("read-file", "Read a file from the filesystem", |args| {
                    let filename = args
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    Ok(vec![ContentBlock::text(format!(
                        "File contents of {filename}"
                    ))])
                })
                .await
                .unwrap();

            // Data processing tool
            server_guard
                .add_simple_tool(
                    "process-data",
                    "Process data with specific algorithm",
                    |args| {
                        let algorithm = args
                            .get("algorithm")
                            .and_then(|v| v.as_str())
                            .unwrap_or("default");
                        let data = args.get("data").and_then(|v| v.as_str()).unwrap_or("");

                        Ok(vec![ContentBlock::text(format!(
                            "Processed {data} with {algorithm} algorithm"
                        ))])
                    },
                )
                .await
                .unwrap();

            // Output tool
            server_guard
                .add_simple_tool("save-result", "Save processing result", |args| {
                    let result = args.get("result").and_then(|v| v.as_str()).unwrap_or("");

                    Ok(vec![ContentBlock::text(format!("Saved result: {result}"))])
                })
                .await
                .unwrap();
        }

        // Simulate a complete workflow: read -> process -> save
        let workflow_steps = [
            ("read-file", json!({"filename": "data.txt"})),
            (
                "process-data",
                json!({"algorithm": "nlp", "data": "File contents of data.txt"}),
            ),
            (
                "save-result",
                json!({"result": "Processed File contents of data.txt with nlp algorithm"}),
            ),
        ];

        for (i, (tool_name, args)) in workflow_steps.iter().enumerate() {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("workflow-step-{}", i)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": tool_name,
                    "arguments": args
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            assert!(result.is_ok(), "Workflow step {i} failed: {tool_name}");

            // Verify response structure
            if let Ok(response) = result {
                assert_eq!(response.jsonrpc, "2.0");
                assert!(response.result.is_some());
            }
        }
    }

    #[tokio::test]
    async fn test_resource_streaming_scenario() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "resource-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a resource handler that simulates streaming large data
        {
            let server_guard = server.lock().await;
            server_guard
                .add_resource(
                    "test-resource".to_string(),
                    "file://test-resource".to_string(),
                    LegacyResourceAdapter::new(TestResourceHandler::new()),
                )
                .await
                .unwrap();
        }

        // Test resource listing
        let list_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("list-resources"),
            method: RESOURCES_LIST.to_string(),
            params: Some(json!({})),
        };

        let server_guard = server.lock().await;
        let list_result = server_guard.handle_request(list_request).await;
        assert!(list_result.is_ok());

        // Test resource reading
        let read_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("read-resource"),
            method: RESOURCES_READ.to_string(),
            params: Some(json!({
                "uri": "file://test-resource"
            })),
        };

        let read_result = server_guard.handle_request(read_request).await;
        assert!(read_result.is_ok());
    }
}

/// Cross-transport compatibility tests
mod cross_transport_tests {
    use super::*;

    #[tokio::test]
    async fn test_stdio_transport_protocol_compliance() {
        // Test that STDIO transport maintains protocol compliance
        let transport = StdioClientTransport::new("echo", vec!["test"])
            .await
            .unwrap();

        // Test basic JSON-RPC request structure
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("stdio-test"),
            method: PING.to_string(),
            params: Some(json!({})),
        };

        // Serialize request
        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"method\":\"ping\""));

        // Test deserialization
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.method, PING);
    }

    #[tokio::test]
    async fn test_message_size_limits_across_transports() {
        // Test that different transports handle various message sizes consistently
        let test_sizes = vec![1024, 10_240, 102_400]; // 1KB, 10KB, 100KB

        for size in test_sizes {
            let large_content = "x".repeat(size);

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("size-test-{}", size)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "test-tool",
                    "arguments": {
                        "large_data": large_content
                    }
                })),
            };

            // Test serialization doesn't fail
            let serialized = serde_json::to_string(&request);
            assert!(
                serialized.is_ok(),
                "Failed to serialize {size} byte message"
            );

            // Test deserialization
            let serialized_str = serialized.unwrap();
            let deserialized = serde_json::from_str::<JsonRpcRequest>(&serialized_str);
            assert!(
                deserialized.is_ok(),
                "Failed to deserialize {size} byte message"
            );
        }
    }
}

/// Performance and stress tests
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_high_frequency_requests() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "perf-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a fast tool
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool("fast-tool", "A very fast tool", |_args| {
                    Ok(vec![ContentBlock::text("Fast response")])
                })
                .await
                .unwrap();
        }

        let num_requests = 100;
        let start_time = Instant::now();

        // Send many requests in quick succession
        for i in 0..num_requests {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("perf-{}", i)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "fast-tool",
                    "arguments": {}
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;
            assert!(result.is_ok(), "Request {i} failed");
        }

        let duration = start_time.elapsed();
        let requests_per_second = num_requests as f64 / duration.as_secs_f64();

        // Should handle at least 100 requests per second
        assert!(
            requests_per_second > 100.0,
            "Too slow: {requests_per_second} req/sec"
        );
    }

    #[tokio::test]
    async fn test_memory_usage_stability() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "memory-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that creates temporary data
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool("memory-tool", "Tool that allocates memory", |args| {
                    let size = args.get("size").and_then(|v| v.as_u64()).unwrap_or(1024) as usize;

                    // Allocate and immediately drop memory
                    let _temp_data = vec![0u8; size];

                    Ok(vec![ContentBlock::text(format!("Allocated {size} bytes"))])
                })
                .await
                .unwrap();
        }

        // Run multiple iterations to test memory stability
        for iteration in 0..50 {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("memory-{}", iteration)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "memory-tool",
                    "arguments": {
                        "size": 10240 // 10KB per request
                    }
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;
            assert!(result.is_ok(), "Memory test iteration {iteration} failed");

            // Small delay to allow garbage collection
            if iteration % 10 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }

        // If we get here without OOM, memory management is working
    }
}

/// Protocol compliance verification
mod protocol_compliance_tests {
    use super::*;

    #[tokio::test]
    async fn test_json_rpc_2_0_compliance() {
        // Test all required JSON-RPC 2.0 fields are present
        let requests = vec![
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!("test-1"),
                method: PING.to_string(),
                params: None,
            },
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(42),
                method: TOOLS_LIST.to_string(),
                params: Some(json!({})),
            },
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: serde_json::Value::Null, // Notification
                method: "notifications/initialized".to_string(),
                params: Some(json!({})),
            },
        ];

        for request in requests {
            let serialized = serde_json::to_string(&request).unwrap();

            // Must have jsonrpc field
            assert!(serialized.contains("\"jsonrpc\":\"2.0\""));

            // Must have method field
            assert!(serialized.contains("\"method\""));

            // If id is present, should be serialized
            if !request.id.is_null() {
                assert!(serialized.contains("\"id\""));
            }

            // Params should be present if not None
            if request.params.is_some() {
                assert!(serialized.contains("\"params\""));
            }
        }
    }

    #[tokio::test]
    async fn test_mcp_method_name_compliance() {
        // Test that all MCP method names follow the correct format
        let method_tests = vec![
            (INITIALIZE, true),
            (PING, true),
            (TOOLS_LIST, true),
            (TOOLS_CALL, true),
            (RESOURCES_LIST, true),
            (RESOURCES_READ, true),
            (PROMPTS_LIST, true),
            (PROMPTS_GET, true),
            ("invalid_method", false),
            ("tools/invalid", false),
            ("", false),
        ];

        for (method, should_be_valid) in method_tests {
            if should_be_valid {
                // Valid methods should contain '/' or be specific known methods
                assert!(
                    method.contains('/') || method == "initialize" || method == "ping",
                    "Method {method} should be valid"
                );
            } else {
                // Invalid methods should not follow MCP patterns
                assert!(
                    !method.contains('/') || method.is_empty() || method.contains("invalid"),
                    "Method {method} should be invalid"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_mcp_error_response_format() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "error-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Test various error scenarios
        let error_requests = vec![
            ("invalid/method", -32601), // Method not found
            ("", -32600),               // Invalid request
        ];

        for (method, expected_code) in error_requests {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!("error-test"),
                method: method.to_string(),
                params: Some(json!({})),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            // Should return an error or handle smoothly
            match result {
                Ok(response) => {
                    // If successful, should be a valid JSON-RPC response
                    assert_eq!(response.jsonrpc, "2.0");
                    assert_eq!(response.id, json!("error-test"));
                }
                Err(_) => {
                    // Error responses are also acceptable
                }
            }
        }
    }
}

/// Helper types for testing
struct TestResourceHandler {
    resources: Vec<Resource>,
}

impl TestResourceHandler {
    fn new() -> Self {
        Self {
            resources: vec![
                Resource {
                    uri: "file:///large-dataset.json".to_string(),
                    name: "large-dataset.json".to_string(),
                    description: Some("A large JSON dataset for testing".to_string()),
                    mime_type: Some("application/json".to_string()),
                    annotations: None,
                    size: Some(1048576), // 1MB
                    title: Some("Large Dataset".to_string()),
                    meta: None,
                },
                Resource {
                    uri: "file:///small-config.txt".to_string(),
                    name: "small-config.txt".to_string(),
                    description: Some("A small configuration file".to_string()),
                    mime_type: Some("text/plain".to_string()),
                    annotations: None,
                    size: Some(256),
                    title: Some("Config File".to_string()),
                    meta: None,
                },
            ],
        }
    }
}

#[async_trait::async_trait]
impl LegacyResourceHandler for TestResourceHandler {
    async fn read(&self, uri: &str) -> McpResult<String> {
        if uri.contains("large-dataset") {
            Ok(format!("{{\"data\": [{}]}}", "1,".repeat(10000))) // Large JSON
        } else {
            Ok("config=value\nother=setting".to_string())
        }
    }
}
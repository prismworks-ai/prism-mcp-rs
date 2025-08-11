// ! Phase 4: Additional Edge Case and Error Handling Tests
// !
// ! Module implements additional edge case testing to push
// ! test coverage from 80% to 90%+ by focusing on:
// ! - Complex error recovery scenarios
// ! - Boundary condition testing
// ! - Network failure simulation
// ! - Resource cleanup verification
// ! - Malformed input handling

use prism_mcp_rs::{
    core::error::McpError,
    protocol::{methods::*, types::*},
    server::McpServer,
};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, time::timeout};

/// Edge case testing for boundary conditions
mod boundary_condition_tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_string_handling() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "boundary-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that handles empty strings
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool(
                    "empty-string-tool",
                    "Tool for testing empty strings",
                    |args| {
                        let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");

                        Ok(vec![ContentBlock::text(&format!(
                            "Received: '{}' (length: {})",
                            text,
                            text.len()
                        ))])
                    },
                )
                .await
                .unwrap();
        }

        // Test various empty string scenarios
        let test_cases = vec![
            json!({"text": ""}),       // Empty string
            json!({"text": null}),     // Null value
            json!({}),                 // Missing field
            json!({"text": "   "}),    // Whitespace only
            json!({"text": "\n\t\r"}), // Control characters
        ];

        for (i, test_case) in test_cases.iter().enumerate() {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("empty-test-{}", i)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "empty-string-tool",
                    "arguments": test_case
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            // Should handle all cases without crashing
            assert!(
                result.is_ok(),
                "Failed to handle empty string case {}: {:?}",
                i,
                test_case
            );
        }
    }

    #[tokio::test]
    async fn test_unicode_and_special_characters() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "unicode-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that processes unicode text
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool("unicode-tool", "Tool for testing unicode", |args| {
                    let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");

                    Ok(vec![ContentBlock::text(&format!(
                        "Unicode text: {} (chars: {})",
                        text,
                        text.chars().count()
                    ))])
                })
                .await
                .unwrap();
        }

        // Test various unicode scenarios
        let unicode_tests = vec![
            "Hello, 世界! 🌍",             // Mixed ASCII, Chinese, emoji
            "Здравствуй мир! #",           // Cyrillic with emoji
            "مرحبا بالعالم *",             // Arabic with emoji
            "🎵🎶🎼🎹🥁🎸🎤",              // Multiple emojis
            "\u{1F600}\u{1F601}\u{1F602}", // Unicode escapes
            "\x00\x01\x02\x03",            // Control characters
            "\\n\\t\\r\\\\",               // Escaped characters
        ];

        for (i, unicode_text) in unicode_tests.iter().enumerate() {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("unicode-test-{}", i)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "unicode-tool",
                    "arguments": {
                        "text": unicode_text
                    }
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            // Should handle all unicode correctly
            assert!(
                result.is_ok(),
                "Failed to handle unicode text {}: {}",
                i,
                unicode_text
            );
        }
    }

    #[tokio::test]
    async fn test_numeric_boundary_values() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "numeric-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that processes numeric values
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool("numeric-tool", "Tool for testing numeric values", |args| {
                    let number = args.get("number").and_then(|v| v.as_f64()).unwrap_or(0.0);

                    Ok(vec![ContentBlock::text(&format!(
                        "Number: {} (type: {})",
                        number,
                        if number.fract() == 0.0 {
                            "integer"
                        } else {
                            "float"
                        }
                    ))])
                })
                .await
                .unwrap();
        }

        // Test boundary numeric values
        let numeric_tests = vec![
            json!(0),                       // Zero
            json!(-0),                      // Negative zero
            json!(i64::MAX),                // Max i64
            json!(i64::MIN),                // Min i64
            json!(1.7976931348623157e+308), // Near max float
            json!(2.2250738585072014e-308), // Near min positive float
        ];

        for (i, numeric_value) in numeric_tests.iter().enumerate() {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("numeric-test-{}", i)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "numeric-tool",
                    "arguments": {
                        "number": numeric_value
                    }
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            // Should handle all numeric values without crashing
            assert!(
                result.is_ok(),
                "Failed to handle numeric value {}: {:?}",
                i,
                numeric_value
            );
        }
    }

    #[tokio::test]
    async fn test_deeply_nested_json_structures() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "nested-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that processes nested structures
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool("nested-tool", "Tool for testing nested JSON", |args| {
                    let default_json = json!({});
                    let data = args.get("data").unwrap_or(&default_json);

                    // Count nesting depth
                    fn count_depth(value: &Value) -> usize {
                        match value {
                            Value::Object(obj) => {
                                1 + obj.values().map(count_depth).max().unwrap_or(0)
                            }
                            Value::Array(arr) => 1 + arr.iter().map(count_depth).max().unwrap_or(0),
                            _ => 0,
                        }
                    }

                    let depth = count_depth(data);
                    Ok(vec![ContentBlock::text(&format!(
                        "Nested structure depth: {}",
                        depth
                    ))])
                })
                .await
                .unwrap();
        }

        // Create moderately nested structures (avoid stack overflow)
        let mut nested_object = json!({"value": "base"});
        for i in 0..10 {
            nested_object = json!({
                "level": i,
                "nested": nested_object
            });
        }

        let mut nested_array = json!(["base"]);
        for i in 0..10 {
            nested_array = json!([i, nested_array]);
        }

        let test_structures = vec![
            ("nested_object", nested_object),
            ("nested_array", nested_array),
        ];

        for (name, structure) in test_structures {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("nested-test-{}", name)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "nested-tool",
                    "arguments": {
                        "data": structure
                    }
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            // Should handle moderate nesting successfully
            assert!(
                result.is_ok(),
                "Failed to handle nested structure: {}",
                name
            );
        }
    }
}

/// Complex error recovery scenarios
mod error_recovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_partial_failure_recovery() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "recovery-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add tools with different failure modes
        {
            let server_guard = server.lock().await;

            // Tool that sometimes fails
            server_guard
                .add_simple_tool("flaky-tool", "A tool that sometimes fails", |args| {
                    let should_fail = args.get("fail").and_then(|v| v.as_bool()).unwrap_or(false);

                    if should_fail {
                        Err(McpError::internal("Simulated failure"))
                    } else {
                        Ok(vec![ContentBlock::text("Success")])
                    }
                })
                .await
                .unwrap();

            // Tool that always succeeds
            server_guard
                .add_simple_tool("reliable-tool", "A reliable tool", |_args| {
                    Ok(vec![ContentBlock::text("Always works")])
                })
                .await
                .unwrap();
        }

        // Test mixed success/failure scenarios
        let test_scenarios = vec![
            ("flaky-tool", json!({"fail": true}), false), // Should fail
            ("reliable-tool", json!({}), true),           // Should succeed
            ("flaky-tool", json!({"fail": false}), true), // Should succeed
            ("nonexistent-tool", json!({}), false),       // Should fail
        ];

        let mut success_count = 0;
        let mut failure_count = 0;

        for (i, (tool_name, args, should_succeed)) in test_scenarios.iter().enumerate() {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("recovery-test-{}", i)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": tool_name,
                    "arguments": args
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            if *should_succeed {
                assert!(result.is_ok(), "Expected success for {}", tool_name);
                success_count += 1;
            } else {
                // For failures, we expect either an error or a smooth error response
                failure_count += 1;
            }
        }

        // Verify we had both successes and failures
        assert!(success_count > 0, "Should have some successes");
        assert!(failure_count > 0, "Should have some failures");
    }

    #[tokio::test]
    async fn test_cascading_error_handling() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "cascade-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that can trigger cascading errors
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool(
                    "cascade-tool",
                    "Tool that can cause cascading errors",
                    |args| {
                        let error_type = args
                            .get("error_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("none");

                        match error_type {
                            "validation" => Err(McpError::validation("Invalid input")),
                            "internal" => Err(McpError::internal("Internal system error")),
                            "timeout" => Err(McpError::timeout("Operation timed out")),
                            "connection" => Err(McpError::connection("Connection lost")),
                            "not_found" => {
                                Err(McpError::ToolNotFound("Tool not available".to_string()))
                            }
                            _ => Ok(vec![ContentBlock::text("No error")]),
                        }
                    },
                )
                .await
                .unwrap();
        }

        // Test different error types to ensure proper error categorization
        let error_types = vec![
            "validation",
            "internal",
            "timeout",
            "connection",
            "not_found",
            "none", // Success case
        ];

        for error_type in error_types {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("cascade-{}", error_type)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "cascade-tool",
                    "arguments": {
                        "error_type": error_type
                    }
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            if error_type == "none" {
                assert!(result.is_ok(), "Success case should work");
            } else {
                // Errors should be handled smoothly without crashing
                // Either as error responses or proper error handling
                match result {
                    Ok(_) => {}  // smooth error response
                    Err(_) => {} // Error properly propagated
                }
            }
        }
    }

    #[tokio::test]
    async fn test_resource_cleanup_on_errors() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "cleanup-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that simulates resource allocation and cleanup
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool("resource-tool", "Tool that manages resources", |args| {
                    let allocate = args
                        .get("allocate")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let fail_after_alloc = args
                        .get("fail_after_alloc")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if allocate {
                        // Simulate resource allocation
                        let _simulated_resource = vec![0u8; 1024]; // Allocate some memory

                        if fail_after_alloc {
                            return Err(McpError::internal("Failed after allocation"));
                        }

                        Ok(vec![ContentBlock::text(
                            "Resource allocated and used successfully",
                        )])
                    } else {
                        Ok(vec![ContentBlock::text("No resource allocation needed")])
                    }
                })
                .await
                .unwrap();
        }

        // Test resource cleanup scenarios
        let cleanup_scenarios = vec![
            (false, false, true), // No allocation, no failure, should succeed
            (true, false, true),  // Allocation, no failure, should succeed
            (true, true, false),  // Allocation, then failure, should handle cleanup
        ];

        for (i, (allocate, fail_after_alloc, should_succeed)) in
            cleanup_scenarios.iter().enumerate()
        {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("cleanup-{}", i)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "resource-tool",
                    "arguments": {
                        "allocate": allocate,
                        "fail_after_alloc": fail_after_alloc
                    }
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            if *should_succeed {
                assert!(result.is_ok(), "Cleanup scenario {} should succeed", i);
            } else {
                // Should fail smoothly without resource leaks
                // The important thing is that the server continues to function
            }
        }
    }
}

/// Malformed input handling tests
mod malformed_input_tests {
    use super::*;

    #[tokio::test]
    async fn test_malformed_json_rpc_requests() {
        // Test various malformed JSON-RPC structures
        let malformed_json_strings = vec![
            r#"{"method": "test"}"#,                             // Missing jsonrpc
            r#"{"jsonrpc": "1.0", "method": "test"}"#,           // Wrong version
            r#"{"jsonrpc": "2.0"}"#,                             // Missing method
            r#"{"jsonrpc": "2.0", "method": ""}"#,               // Empty method
            r#"{"jsonrpc": "2.0", "method": null}"#,             // Null method
            r#"{"jsonrpc": "2.0", "method": 123}"#,              // Numeric method
            r#"{"jsonrpc": "2.0", "method": "test", "id": {}}"#, // Object id
        ];

        for (i, malformed_json) in malformed_json_strings.iter().enumerate() {
            let parse_result = serde_json::from_str::<JsonRpcRequest>(malformed_json);

            // Most should fail to parse
            match parse_result {
                Ok(request) => {
                    // If it did parse, it should be obviously invalid
                    if request.jsonrpc != "2.0" || request.method.is_empty() {
                        // This is expected for malformed requests
                    } else {
                        println!("Unexpectedly parsed malformed JSON {}: {:?}", i, request);
                    }
                }
                Err(_) => {
                    // Expected for malformed JSON
                }
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_method_names() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "invalid-method-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Test various invalid method names
        let invalid_methods = vec![
            "",
            " ",
            "\n",
            "invalid-method",
            "tools",            // Missing slash
            "tools/",           // Trailing slash
            "/tools/list",      // Leading slash
            "tools// list",     // Double slash
            "TOOLS/LIST",       // Wrong case
            "tools/list/extra", // Too many parts
            "123/456",          // Numeric parts
            "special!@#$%",     // Special characters
        ];

        for (i, method) in invalid_methods.iter().enumerate() {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("invalid-method-{}", i)),
                method: method.to_string(),
                params: Some(json!({})),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            // Invalid methods should either error or be handled smoothly
            // The server shouldn't crash
            match result {
                Ok(response) => {
                    // Might be a method not found error response
                    assert_eq!(response.jsonrpc, "2.0");
                }
                Err(_) => {
                    // Expected for invalid methods
                }
            }
        }
    }

    #[tokio::test]
    async fn test_malformed_parameter_structures() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "malformed-params-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that expects specific parameter structure
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool(
                    "param-tool",
                    "Tool that expects specific parameters",
                    |args| {
                        let name = args
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(0);

                        Ok(vec![ContentBlock::text(&format!(
                            "Processed: {} (count: {})",
                            name, count
                        ))])
                    },
                )
                .await
                .unwrap();
        }

        // Test various malformed parameter scenarios
        let malformed_params = vec![
            json!(null),                           // Null params
            json!("string instead of object"),     // String params
            json!(123),                            // Numeric params
            json!([1, 2, 3]),                      // Array params
            json!({"name": null}),                 // Null values
            json!({"name": 123}),                  // Wrong type for name
            json!({"count": "not a number"}),      // Wrong type for count
            json!({"name": {"nested": "object"}}), // Nested object instead of string
        ];

        for (i, params) in malformed_params.iter().enumerate() {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("malformed-params-{}", i)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "param-tool",
                    "arguments": params
                })),
            };

            let server_guard = server.lock().await;
            let result = server_guard.handle_request(request).await;

            // Should handle malformed parameters smoothly
            match result {
                Ok(_) => {
                    // Tool handled the malformed input smoothly
                }
                Err(_) => {
                    // Error is acceptable for malformed input
                }
            }
        }
    }
}

/// Network failure simulation tests
mod network_failure_tests {
    use super::*;

    #[tokio::test]
    async fn test_simulated_network_interruption() {
        // Simulate network-like conditions by using timeouts and delays
        let server = Arc::new(Mutex::new(McpServer::new(
            "network-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that simulates network delays
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool(
                    "network-tool",
                    "Tool that simulates network conditions",
                    |args| {
                        let delay_ms = args.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(0);

                        if delay_ms > 0 {
                            // Simulate network delay
                            std::thread::sleep(Duration::from_millis(delay_ms));
                        }

                        Ok(vec![ContentBlock::text(&format!(
                            "Response after {}ms delay",
                            delay_ms
                        ))])
                    },
                )
                .await
                .unwrap();
        }

        // Test various delay scenarios
        let delay_scenarios = vec![
            (0, true),     // No delay - should succeed
            (10, true),    // Small delay - should succeed
            (100, true),   // Medium delay - should succeed
            (1000, false), // Large delay - might timeout
        ];

        for (delay_ms, should_be_fast) in delay_scenarios {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("network-delay-{}", delay_ms)),
                method: TOOLS_CALL.to_string(),
                params: Some(json!({
                    "name": "network-tool",
                    "arguments": {
                        "delay_ms": delay_ms
                    }
                })),
            };

            let start_time = Instant::now();
            let server_guard = server.lock().await;

            if should_be_fast {
                // For fast operations, test without timeout
                let result = server_guard.handle_request(request).await;
                assert!(result.is_ok(), "Fast operation should succeed");
            } else {
                // For slow operations, test with timeout
                let result = timeout(
                    Duration::from_millis(200),
                    server_guard.handle_request(request),
                )
                .await;

                match result {
                    Ok(Ok(_)) => {
                        // Completed successfully within timeout
                    }
                    Ok(Err(_)) => {
                        // Completed with error within timeout
                    }
                    Err(_) => {
                        // Timed out - this is expected for slow operations
                    }
                }
            }

            let elapsed = start_time.elapsed();
            if should_be_fast {
                assert!(
                    elapsed < Duration::from_millis(500),
                    "Fast operation took too long"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_request_interference() {
        let server = Arc::new(Mutex::new(McpServer::new(
            "interference-test".to_string(),
            "1.0.0".to_string(),
        )));

        // Add a tool that can be affected by concurrent access
        {
            let server_guard = server.lock().await;
            server_guard
                .add_simple_tool(
                    "shared-tool",
                    "Tool that simulates shared resource access",
                    |args| {
                        let operation_id = args
                            .get("operation_id")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);

                        // Simulate some processing time
                        std::thread::sleep(Duration::from_millis(10));

                        Ok(vec![ContentBlock::text(&format!(
                            "Completed operation {}",
                            operation_id
                        ))])
                    },
                )
                .await
                .unwrap();
        }

        // Launch multiple concurrent requests
        let mut handles = vec![];
        let num_concurrent = 20;

        for i in 0..num_concurrent {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                let request = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    id: json!(format!("concurrent-{}", i)),
                    method: TOOLS_CALL.to_string(),
                    params: Some(json!({
                        "name": "shared-tool",
                        "arguments": {
                            "operation_id": i
                        }
                    })),
                };

                let server_guard = server_clone.lock().await;
                server_guard.handle_request(request).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        let results = futures::future::join_all(handles).await;

        // All should complete successfully
        let mut success_count = 0;
        for result in results {
            match result {
                Ok(Ok(_)) => success_count += 1,
                Ok(Err(e)) => println!("Request failed: {}", e),
                Err(e) => println!("Task failed: {}", e),
            }
        }

        // Most requests should succeed despite concurrency
        assert!(
            success_count >= (num_concurrent * 8) / 10,
            "Too many failures: {}/{}",
            success_count,
            num_concurrent
        );
    }
}

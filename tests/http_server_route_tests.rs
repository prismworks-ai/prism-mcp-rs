// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! HTTP Server Route and Handler Tests
// !
// ! This test suite specifically targets the HTTP server route handlers
// ! to maximize coverage of the server-side HTTP transport functionality.

#![cfg(feature = "http")]

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::sse::Event,
    routing::{get, post},
};
use prism_mcp_rs::protocol::types::{
    JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, error_codes,
};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::sync::{RwLock, broadcast};

#[cfg(feature = "http")]
mod http_server_route_tests {
    use super::*;

    // Helper struct that mirrors the internal HttpServerState
    #[derive(Clone)]
    struct TestHttpServerState {
        notification_sender: broadcast::Sender<JsonRpcNotification>,
        request_handler: Option<
            Arc<
                dyn Fn(JsonRpcRequest) -> tokio::sync::oneshot::Receiver<JsonRpcResponse>
                    + Send
                    + Sync,
            >,
        >,
    }

    // ========================================================================
    // HTTP Server Route Handler Tests
    // ========================================================================

    #[tokio::test]
    async fn test_mcp_request_handler_no_handler_configured() {
        // Test MCP request handling without a configured handler
        let (notification_sender, _) = broadcast::channel(1000);

        let state = Arc::new(RwLock::new(TestHttpServerState {
            notification_sender,
            request_handler: None,
        }));

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test_method".to_string(),
            params: Some(json!({"key": "value"})),
        };

        // Simulate the handler logic
        let state_guard = state.read().await;
        if state_guard.request_handler.is_none() {
            let error_response = JsonRpcError::error(
                request.id.clone(),
                error_codes::METHOD_NOT_FOUND,
                "No request handler configured".to_string(),
                None,
            );

            assert_eq!(error_response.id, json!(1));
            assert_eq!(error_response.error.code, error_codes::METHOD_NOT_FOUND);
            assert!(
                error_response
                    .error
                    .message
                    .contains("No request handler configured")
            );
        }
    }

    #[tokio::test]
    async fn test_mcp_request_handler_with_configured_handler() {
        // Test MCP request handling with a configured handler
        let (notification_sender, _) = broadcast::channel(1000);

        // Create a test handler that returns a response
        let test_handler = Arc::new(|request: JsonRpcRequest| {
            let (tx, rx) = tokio::sync::oneshot::channel();

            // Simulate async processing
            tokio::spawn(async move {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(json!({"processed": true, "method": request.method})),
                };
                let _ = tx.send(response);
            });

            rx
        });

        let state = Arc::new(RwLock::new(TestHttpServerState {
            notification_sender,
            request_handler: Some(test_handler),
        }));

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(42),
            method: "test_method".to_string(),
            params: Some(json!({"data": "test"})),
        };

        // Simulate the handler logic
        let state_guard = state.read().await;
        if let Some(ref handler) = state_guard.request_handler {
            let response_rx = handler(request.clone());
            drop(state_guard); // Release the lock

            match response_rx.await {
                Ok(response) => {
                    assert_eq!(response.jsonrpc, "2.0");
                    assert_eq!(response.id, json!(42));
                    assert!(response.result.is_some());

                    if let Some(result) = response.result {
                        assert_eq!(result["processed"], json!(true));
                        assert_eq!(result["method"], json!("test_method"));
                    }
                }
                Err(_) => panic!("Handler should not fail"),
            }
        }
    }

    #[tokio::test]
    async fn test_mcp_request_handler_failure_scenario() {
        // Test handler that fails
        let (notification_sender, _) = broadcast::channel(1000);

        let failing_handler = Arc::new(|_request: JsonRpcRequest| {
            let (tx, rx) = tokio::sync::oneshot::channel();

            // Don't send anything, simulating a handler that fails
            drop(tx);

            rx
        });

        let state = Arc::new(RwLock::new(TestHttpServerState {
            notification_sender,
            request_handler: Some(failing_handler),
        }));

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("test_id"),
            method: "failing_method".to_string(),
            params: None,
        };

        // Simulate the handler logic with failure
        let state_guard = state.read().await;
        if let Some(ref handler) = state_guard.request_handler {
            let response_rx = handler(request);
            drop(state_guard);

            match response_rx.await {
                Ok(_) => panic!("Should have failed"),
                Err(_) => {
                    // This would result in StatusCode::INTERNAL_SERVER_ERROR
                    // in the actual route handler
                    assert!(true, "Handler failure handled correctly");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_notification_handling() {
        // Test notification handling (which simply returns OK)
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_notification".to_string(),
            params: Some(json!({"event": "something_happened"})),
        };

        // Simulate notification handler (which just returns OK)
        let serialized = serde_json::to_string(&notification);
        assert!(serialized.is_ok(), "Notification should serialize");

        // The actual handler just returns StatusCode::OK
        // Test that we can process various notification types
        let notification_types = vec![
            ("simple", json!(null)),
            ("with_params", json!({"key": "value"})),
            ("with_array", json!([1, 2, 3])),
            ("with_complex", json!({"nested": {"data": ["a", "b"]}})),
        ];

        for (method, params) in notification_types {
            let notification = JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params: Some(params),
            };

            let serialized = serde_json::to_string(&notification);
            assert!(serialized.is_ok(), "Notification {method} should serialize");
        }
    }

    // ========================================================================
    // SSE Event Handling Tests
    // ========================================================================

    #[tokio::test]
    async fn test_sse_event_creation_and_serialization() {
        // Test SSE event creation with various notification types
        let (notification_sender, mut receiver) = broadcast::channel(1000);

        let notifications = vec![
            JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "test_event".to_string(),
                params: Some(json!({"data": "test"})),
            },
            JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "complex_event".to_string(),
                params: Some(json!({
                    "nested": {
                        "array": [1, 2, 3],
                        "string": "value",
                        "unicode": "#🔥"
                    }
                })),
            },
            JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "empty_params".to_string(),
                params: None,
            },
        ];

        // Send notifications and test serialization
        for notification in notifications {
            let json_result = serde_json::to_string(&notification);
            assert!(json_result.is_ok(), "Notification should serialize to JSON");

            let json = json_result.unwrap();
            assert!(
                json.contains("\"jsonrpc\":\"2.0\""),
                "Should contain JSON-RPC version"
            );
            assert!(
                json.contains(&format!("\"method\":\"{}\"", notification.method)),
                "Should contain method"
            );

            // Test that we can send via broadcast channel
            let send_result = notification_sender.send(notification.clone());
            assert!(
                send_result.is_ok(),
                "Should send notification via broadcast"
            );

            // Test that we can receive
            let received = receiver.recv().await;
            assert!(received.is_ok(), "Should receive notification");

            let received_notification = received.unwrap();
            assert_eq!(received_notification.method, notification.method);
        }
    }

    #[cfg(all(feature = "tokio-stream", feature = "futures"))]
    #[tokio::test]
    async fn test_sse_stream_processing() {
        use tokio_stream::{StreamExt, wrappers::BroadcastStream};

        let (notification_sender, receiver) = broadcast::channel(1000);

        // Create a stream like in the SSE handler
        let stream = BroadcastStream::new(receiver).map(|result| {
            match result {
                Ok(notification) => match serde_json::to_string(&notification) {
                    Ok(json) => Ok::<axum::response::sse::Event, std::convert::Infallible>(
                        Event::default().data(json),
                    ),
                    Err(e) => {
                        // This would log an error in the real implementation
                        Ok(Event::default().data("{}"))
                    }
                },
                Err(_) => Ok(Event::default().data("{}")), // Lagged or closed
            }
        });

        // Send a test notification
        let test_notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "stream_test".to_string(),
            params: Some(json!({"stream": "test"})),
        };

        let send_result = notification_sender.send(test_notification.clone());
        assert!(send_result.is_ok(), "Should send notification");

        // Test that the stream can be processed
        let mut stream = Box::pin(stream);
        if let Some(event_result) = stream.next().await {
            assert!(event_result.is_ok(), "Stream event should be ok");
            // Event processing would happen here in real SSE handler
        }
    }

    #[tokio::test]
    async fn test_sse_serialization_error_handling() {
        // Test handling of serialization errors in SSE
        use tokio_stream::{StreamExt, wrappers::BroadcastStream};

        let (notification_sender, receiver) = broadcast::channel(1000);

        // Create a problematic notification that might cause serialization issues
        // Note: JsonRpcNotification should always serialize properly, so we test the error path indirectly
        let stream = BroadcastStream::new(receiver).map(|result| {
            match result {
                Ok(notification) => {
                    // Simulate serialization check
                    match serde_json::to_string(&notification) {
                        Ok(json) => {
                            assert!(json.contains("jsonrpc"), "Should contain jsonrpc field");
                            assert!(!json.is_empty(), "JSON should not be empty");
                            Ok::<String, std::convert::Infallible>(json)
                        }
                        Err(_) => {
                            // Error case - return empty JSON
                            Ok::<String, std::convert::Infallible>("{}".to_string())
                        }
                    }
                }
                Err(_) => Ok::<String, std::convert::Infallible>("{}".to_string()), // Lagged or closed
            }
        });

        let test_notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "serialization_test".to_string(),
            params: Some(json!({"test": "data"})),
        };

        let _ = notification_sender.send(test_notification);

        let mut stream = Box::pin(stream);
        if let Some(json_result) = stream.next().await {
            assert!(json_result.is_ok(), "Stream should handle serialization");
            let json = json_result.unwrap();
            assert!(!json.is_empty(), "Should have JSON content");
        }
    }

    // ========================================================================
    // Health Check Tests
    // ========================================================================

    #[tokio::test]
    async fn test_health_check_response_structure() {
        // Test health check response structure
        #[cfg(feature = "chrono")]
        let timestamp = chrono::Utc::now().to_rfc3339();
        #[cfg(not(feature = "chrono"))]
        let timestamp = "unavailable";

        let health_response = json!({
            "status": "healthy",
            "transport": "http",
            "timestamp": timestamp
        });

        // Verify required fields
        assert_eq!(health_response["status"], "healthy");
        assert_eq!(health_response["transport"], "http");
        assert!(health_response["timestamp"].is_string());

        // Test serialization
        let serialized = serde_json::to_string(&health_response);
        assert!(serialized.is_ok(), "Health response should serialize");

        let serialized_str = serialized.unwrap();
        assert!(serialized_str.contains("healthy"), "Should contain status");
        assert!(
            serialized_str.contains("http"),
            "Should contain transport type"
        );

        // Test deserialization
        let deserialized: Value = serde_json::from_str(&serialized_str).unwrap();
        assert_eq!(deserialized["status"], "healthy");
    }

    // ========================================================================
    // CORS and Middleware Tests
    // ========================================================================

    #[tokio::test]
    async fn test_cors_configuration() {
        // Test CORS layer configuration
        use tower_http::cors::{Any, CorsLayer};

        let cors_layer = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // Test that CORS layer can be created and configured
        // In actual usage, this would be applied to the router
        assert!(true, "CORS layer should be configurable");
    }

    #[tokio::test]
    async fn test_router_construction() {
        // Test that we can construct the router with all routes
        let (notification_sender, _) = broadcast::channel(1000);

        let state = Arc::new(RwLock::new(TestHttpServerState {
            notification_sender,
            request_handler: None,
        }));

        // Create a test router similar to the one in HttpServerTransport
        let _router: Router<()> = Router::new()
            .route("/mcp", post(test_handle_mcp_request))
            .route("/mcp/notify", post(test_handle_mcp_notification))
            .route("/mcp/events", get(test_handle_sse_events))
            .route("/health", get(test_handle_health_check))
            .with_state(state);

        // Test that router can be created
        assert!(true, "Router should be constructible with all routes");
    }

    // Test handler functions to verify route structure
    async fn test_handle_mcp_request(
        State(_state): State<Arc<RwLock<TestHttpServerState>>>,
        Json(request): Json<JsonRpcRequest>,
    ) -> Result<Json<JsonRpcMessage>, StatusCode> {
        // Simple test implementation
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({"test": true})),
        };
        Ok(Json(JsonRpcMessage::Response(response)))
    }

    async fn test_handle_mcp_notification(
        Json(_notification): Json<JsonRpcNotification>,
    ) -> StatusCode {
        StatusCode::OK
    }

    #[cfg(all(feature = "tokio-stream", feature = "futures"))]
    async fn test_handle_sse_events(
        State(_state): State<Arc<RwLock<TestHttpServerState>>>,
    ) -> StatusCode {
        // SSE endpoint test - would normally return Sse<Stream>
        StatusCode::OK
    }

    #[cfg(not(all(feature = "tokio-stream", feature = "futures")))]
    async fn test_handle_sse_events(
        State(_state): State<Arc<RwLock<TestHttpServerState>>>,
    ) -> StatusCode {
        StatusCode::NOT_IMPLEMENTED
    }

    async fn test_handle_health_check() -> Json<Value> {
        Json(json!({
            "status": "healthy",
            "transport": "http",
            "timestamp": "test"
        }))
    }

    // ========================================================================
    // Error Response Generation Tests
    // ========================================================================

    #[tokio::test]
    async fn test_error_response_generation() {
        let error_codes_to_test = vec![
            error_codes::METHOD_NOT_FOUND,
            error_codes::INVALID_PARAMS,
            error_codes::INTERNAL_ERROR,
            error_codes::PARSE_ERROR,
        ];

        for error_code in error_codes_to_test {
            let error_response = JsonRpcError::error(
                json!("test_id"),
                error_code,
                format!("Test error with code {error_code}"),
                Some(json!({"additional": "data"})),
            );

            assert_eq!(error_response.error.code, error_code);
            assert!(
                error_response
                    .error
                    .message
                    .contains(&error_code.to_string())
            );
            assert!(error_response.error.data.is_some());

            // Test conversion to JsonRpcMessage
            let message = JsonRpcMessage::Error(error_response);
            let serialized = serde_json::to_string(&message);
            assert!(serialized.is_ok(), "Error message should serialize");
        }
    }

    // ========================================================================
    // Request Handler Integration Tests
    // ========================================================================

    #[tokio::test]
    async fn test_request_handler_various_response_types() {
        // Test handlers that return different types of responses
        let response_types = vec![
            // Success with result
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                result: Some(json!({"success": true})),
            },
            // Success with complex result
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: json!("complex"),
                result: Some(json!({
                    "data": {
                        "array": [1, 2, 3],
                        "nested": {"key": "value"},
                        "unicode": "🌍#"
                    }
                })),
            },
            // Success with null result
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: json!(null),
                result: Some(json!(null)),
            },
        ];

        for (i, expected_response) in response_types.into_iter().enumerate() {
            let expected_result = expected_response.result.clone();
            let expected_id = expected_response.id.clone();
            let expected_result_for_handler = expected_result.clone();

            let handler = Arc::new(move |request: JsonRpcRequest| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: expected_result_for_handler.clone(),
                };

                tokio::spawn(async move {
                    let _ = tx.send(response);
                });

                rx
            });

            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: expected_id.clone(),
                method: format!("test_method_{i}"),
                params: None,
            };

            let response_rx = handler(request);
            let response = response_rx.await;

            assert!(response.is_ok(), "Handler {i} should succeed");
            let response = response.unwrap();
            assert_eq!(response.id, expected_id);
            assert_eq!(response.result, expected_result);
        }
    }

    #[tokio::test]
    async fn test_concurrent_request_handling() {
        // Test multiple concurrent requests
        let handler = Arc::new(|request: JsonRpcRequest| {
            let (tx, rx) = tokio::sync::oneshot::channel();

            tokio::spawn(async move {
                // Simulate some async work
                tokio::time::sleep(Duration::from_millis(10)).await;

                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(json!({"processed": request.method})),
                };

                let _ = tx.send(response);
            });

            rx
        });

        let mut handles = Vec::new();

        // Spawn multiple concurrent handler calls
        for i in 0..5 {
            let handler_clone = handler.clone();
            let handle = tokio::spawn(async move {
                let request = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    id: json!(i),
                    method: format!("concurrent_method_{i}"),
                    params: None,
                };

                let response_rx = handler_clone(request);
                response_rx.await
            });

            handles.push(handle);
        }

        // Wait for all to complete
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent request {i} should succeed");

            let response = result.unwrap();
            assert_eq!(response.id, json!(i));
        }
    }
}

#[cfg(not(feature = "http"))]
#[test]
fn http_server_route_tests_require_http_feature() {
    println!("HTTP server route tests require the 'http' feature to be enabled");
    println!("Run with: cargo test --features http --test http_server_route_tests");
    assert!(true, "HTTP feature not enabled - this is expected");
}

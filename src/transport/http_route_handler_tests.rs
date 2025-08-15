// Tests for HTTP route handlers to achieve maximum coverage
#[cfg(test)]
#[cfg(not(coverage))]
mod route_handler_tests {
    use super::super::http::*;
    use crate::protocol::types::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, JsonRpcError, JsonRpcMessage, error_codes};
    use crate::transport::traits::{ServerTransport, TransportConfig};
    use serde_json::{json, Value};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use axum::{Json, extract::State, http::StatusCode};
    use tokio::sync::broadcast;

    // Helper functions
    fn create_test_request(id: Value, method: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params: Some(json!({"test": "data"})),
        }
    }

    fn create_test_notification(method: &str) -> JsonRpcNotification {
        JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(json!({"test": "notification"})),
        }
    }

    #[tokio::test]
    async fn test_handle_mcp_request_with_handler() {
        let (notification_sender, _) = broadcast::channel(100);
        
        // Create a mock request handler
        let handler = Arc::new(|request: JsonRpcRequest| {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({"handled": true, "method": request.method})),
            };
            let _ = tx.send(response);
            rx
        });
        
        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender,
            request_handler: Some(handler),
        }));
        
        let request = create_test_request(Value::from(123), "test_method");
        let state_extract = State(state);
        let json_request = Json(request.clone());
        
        let result = handle_mcp_request(state_extract, json_request).await;
        
        assert!(result.is_ok());
        if let Ok(Json(JsonRpcMessage::Response(response))) = result {
            assert_eq!(response.id, Value::from(123));
            assert!(response.result.is_some());
            if let Some(result_value) = response.result {
                assert_eq!(result_value["handled"], true);
                assert_eq!(result_value["method"], "test_method");
            }
        } else {
            panic!("Expected successful response");
        }
    }

    #[tokio::test]
    async fn test_handle_mcp_request_without_handler() {
        let (notification_sender, _) = broadcast::channel(100);
        
        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender,
            request_handler: None,
        }));
        
        let request = create_test_request(Value::from(456), "test_method");
        let state_extract = State(state);
        let json_request = Json(request.clone());
        
        let result = handle_mcp_request(state_extract, json_request).await;
        
        assert!(result.is_ok());
        if let Ok(Json(JsonRpcMessage::Error(error))) = result {
            assert_eq!(error.id, Value::from(456));
            assert_eq!(error.error.code, error_codes::METHOD_NOT_FOUND);
            assert!(error.error.message.contains("No request handler configured"));
        } else {
            panic!("Expected error response when no handler is configured");
        }
    }

    #[tokio::test]
    async fn test_handle_mcp_request_handler_failure() {
        let (notification_sender, _) = broadcast::channel(100);
        
        // Create a handler that drops the sender (simulates failure)
        let handler = Arc::new(|_request: JsonRpcRequest| {
            let (_tx, rx) = tokio::sync::oneshot::channel::<JsonRpcResponse>();
            // Drop tx immediately to simulate handler failure
            rx
        });
        
        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender,
            request_handler: Some(handler),
        }));
        
        let request = create_test_request(Value::from(789), "failing_method");
        let state_extract = State(state);
        let json_request = Json(request.clone());
        
        let result = handle_mcp_request(state_extract, json_request).await;
        
        // Should return internal server error when handler fails
        assert!(result.is_err());
        if let Err(status) = result {
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    #[tokio::test]
    async fn test_handle_mcp_notification() {
        let notification = create_test_notification("test_notification");
        let json_notification = Json(notification);
        
        let result = handle_mcp_notification(json_notification).await;
        
        // Notifications should always return OK
        assert_eq!(result, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_health_check() {
        let result = handle_health_check().await;
        
        let Json(health_data) = result;
        assert_eq!(health_data["status"], "healthy");
        assert_eq!(health_data["transport"], "http");
        assert!(health_data["timestamp"].is_string());
    }

    #[cfg(all(feature = "tokio-stream", feature = "futures"))]
    #[tokio::test]
    async fn test_handle_sse_events() {
        let (notification_sender, _) = broadcast::channel(100);
        
        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender: notification_sender.clone(),
            request_handler: None,
        }));
        
        let state_extract = State(state);
        
        // This should return an SSE stream
        let _sse_stream = handle_sse_events(state_extract).await;
        
        // We can't easily test the actual streaming without setting up a full server,
        // but we can verify the function doesn't panic and returns the expected type
    }

    #[cfg(not(all(feature = "tokio-stream", feature = "futures")))]
    #[tokio::test]
    async fn test_handle_sse_events_not_implemented() {
        let (notification_sender, _) = broadcast::channel(100);
        
        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender,
            request_handler: None,
        }));
        
        let state_extract = State(state);
        
        let result = handle_sse_events(state_extract).await;
        
        // Should return NOT_IMPLEMENTED when features are not available
        assert_eq!(result, StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn test_server_request_handler_conversion() {
        let mut server = HttpServerTransport::new("127.0.0.1:0");
        
        // Test the sync trait method (this is mainly for API compatibility)
        let handler: crate::transport::traits::ServerRequestHandler = 
            |request: JsonRpcRequest| {
                Box::pin(async move {
                    Ok(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(json!({"converted": true})),
                    })
                })
            };
        
        // This should not panic even though it's a no-op in the current implementation
        server.set_request_handler(handler);
    }

    #[tokio::test]
    async fn test_server_state_management() {
        let (notification_sender, _) = broadcast::channel(100);
        
        let state = HttpServerState {
            notification_sender: notification_sender.clone(),
            request_handler: None,
        };
        
        // Test that state can be cloned (required for Axum)
        let _cloned_state = state.clone();
        
        // Test notification sender functionality
        let test_notification = create_test_notification("state_test");
        let result = notification_sender.send(test_notification);
        
        // Should succeed even with no receivers
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_request_handler_setup() {
        let mut server = HttpServerTransport::new("127.0.0.1:0");
        
        let handler = |request: JsonRpcRequest| {
            let (tx, rx) = tokio::sync::oneshot::channel();
            
            // Simulate async processing
            tokio::spawn(async move {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(json!({
                        "async_handled": true,
                        "original_method": request.method
                    })),
                };
                let _ = tx.send(response);
            });
            
            rx
        };
        
        server.set_request_handler(handler).await;
        
        // The handler should be set successfully
        // (We can't easily verify this without accessing private fields)
    }

    #[tokio::test]
    async fn test_different_request_id_types() {
        let (notification_sender, _) = broadcast::channel(100);
        
        let handler = Arc::new(|request: JsonRpcRequest| {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(), // Echo back the same ID
                result: Some(json!({"id_type": format!("{:?}", request.id)})),
            };
            let _ = tx.send(response);
            rx
        });
        
        let state = Arc::new(RwLock::new(HttpServerState {
            notification_sender,
            request_handler: Some(handler),
        }));
        
        // Test with different ID types
        let test_cases = vec![
            Value::from(123),                                    // Number
            Value::String("string-id".to_string()),            // String
            Value::Array(vec![Value::from(1), Value::from(2)]), // Array
            Value::Null,                                        // Null
        ];
        
        for test_id in test_cases {
            let request = create_test_request(test_id.clone(), "id_test");
            let state_extract = State(state.clone());
            let json_request = Json(request);
            
            let result = handle_mcp_request(state_extract, json_request).await;
            
            assert!(result.is_ok());
            if let Ok(Json(JsonRpcMessage::Response(response))) = result {
                assert_eq!(response.id, test_id);
            } else {
                panic!("Expected successful response for ID type: {:?}", test_id);
            }
        }
    }

    #[tokio::test]
    async fn test_notification_with_different_params() {
        // Test notifications with various parameter types
        let test_cases = vec![
            ("simple", json!({"key": "value"})),
            ("array", json!([1, 2, 3, "test"])),
            ("nested", json!({"level1": {"level2": {"data": true}}})),
            ("null_params", Value::Null),
        ];
        
        for (method_suffix, params) in test_cases {
            let notification = JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: format!("test_{}", method_suffix),
                params: Some(params),
            };
            
            let json_notification = Json(notification);
            let result = handle_mcp_notification(json_notification).await;
            
            assert_eq!(result, StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_server_info_formatting() {
        let server = HttpServerTransport::new("0.0.0.0:8080");
        let info = server.server_info();
        
        assert!(info.contains("HTTP server transport"));
        assert!(info.contains("0.0.0.0:8080"));
        
        // Test with different bind addresses
        let server2 = HttpServerTransport::new("127.0.0.1:3000");
        let info2 = server2.server_info();
        
        assert!(info2.contains("127.0.0.1:3000"));
    }
}
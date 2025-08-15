// Additional edge case tests for HTTP transport to achieve maximum coverage
#[cfg(test)]
#[cfg(not(coverage))]
mod edge_case_tests {
    use super::super::http::*;
    use crate::protocol::types::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification};
    use crate::transport::traits::{Transport, ServerTransport, TransportConfig};
    use serde_json::{json, Value};
    use std::time::Duration;
    use std::collections::HashMap;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};
    use tokio::sync::mpsc;

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
    async fn test_send_request_with_existing_id() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 42,
                "result": {"success": true}
            })))
            .mount(&mock_server)
            .await;
            
        let mut transport = HttpClientTransport::new(mock_server.uri(), None).await.unwrap();
        
        // Request with existing ID should keep the ID
        let request = create_test_request(Value::from(42), "test_method");
        let result = transport.send_request(request).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, Value::from(42));
    }

    #[tokio::test]
    async fn test_send_request_with_timeout_config() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {"success": true}
                }))
                .set_delay(Duration::from_millis(500)))
            .mount(&mock_server)
            .await;
            
        let mut config = TransportConfig::default();
        config.read_timeout_ms = Some(1000); // 1 second timeout
        
        let mut transport = HttpClientTransport::with_config(
            mock_server.uri(),
            None,
            config,
        ).await.unwrap();
        
        let request = create_test_request(Value::from(1), "test_method");
        let result = transport.send_request(request).await;
        
        // Should succeed within timeout
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_notification_with_write_timeout() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/mcp/notify"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(200)))
            .mount(&mock_server)
            .await;
            
        let mut config = TransportConfig::default();
        config.write_timeout_ms = Some(500); // 500ms timeout
        
        let mut transport = HttpClientTransport::with_config(
            mock_server.uri(),
            None,
            config,
        ).await.unwrap();
        
        let notification = create_test_notification("test_notification");
        let result = transport.send_notification(notification).await;
        
        // Should succeed within timeout
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_headers_applied_to_requests() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/mcp"))
            .and(header("x-custom-header", "test-value"))
            .and(header("authorization", "Bearer token123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"success": true}
            })))
            .mount(&mock_server)
            .await;
            
        let mut config = TransportConfig::default();
        config.headers.insert("X-Custom-Header".to_string(), "test-value".to_string());
        config.headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        
        let mut transport = HttpClientTransport::with_config(
            mock_server.uri(),
            None,
            config,
        ).await.unwrap();
        
        let request = create_test_request(Value::from(1), "test_method");
        let result = transport.send_request(request).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_headers_applied_to_notifications() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/mcp/notify"))
            .and(header("x-notification-header", "notify-value"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
            
        let mut config = TransportConfig::default();
        config.headers.insert("X-Notification-Header".to_string(), "notify-value".to_string());
        
        let mut transport = HttpClientTransport::with_config(
            mock_server.uri(),
            None,
            config,
        ).await.unwrap();
        
        let notification = create_test_notification("test_notification");
        let result = transport.send_notification(notification).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_receive_notification_when_disconnected() {
        let mut transport = HttpClientTransport::new("http://localhost:3000", None).await.unwrap();
        
        // Simulate disconnected notification channel
        let (_, rx) = mpsc::unbounded_channel::<JsonRpcNotification>();
        drop(rx); // Close the receiver
        
        // Replace with a closed channel
        transport.notification_receiver = None;
        
        let result = transport.receive_notification().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_http_client_builder_config() {
        // Test different timeout configurations
        let mut config = TransportConfig::default();
        config.read_timeout_ms = Some(30_000);
        config.connect_timeout_ms = Some(15_000);
        
        let transport = HttpClientTransport::with_config(
            "http://localhost:3000",
            None,
            config,
        ).await;
        
        assert!(transport.is_ok());
        let transport = transport.unwrap();
        assert_eq!(transport.config.read_timeout_ms, Some(30_000));
        assert_eq!(transport.config.connect_timeout_ms, Some(15_000));
    }

    #[tokio::test]
    async fn test_http_client_default_config() {
        let transport = HttpClientTransport::new("http://localhost:3000", None).await.unwrap();
        
        // Should use default timeout values
        assert_eq!(transport.config.read_timeout_ms, None);
        assert_eq!(transport.config.connect_timeout_ms, None);
    }

    #[tokio::test]
    async fn test_connection_state_after_close() {
        let mut transport = HttpClientTransport::new("http://localhost:3000", None).await.unwrap();
        
        assert!(transport.is_connected());
        
        transport.close().await.unwrap();
        
        assert!(!transport.is_connected());
        
        // Connection info should reflect disconnected state
        let info = transport.connection_info();
        assert!(info.contains("Disconnected"));
    }

    #[tokio::test]
    async fn test_server_transport_notification_broadcast() {
        let mut server = HttpServerTransport::new("127.0.0.1:0");
        
        let notification = create_test_notification("broadcast_test");
        let result = server.send_notification(notification).await;
        
        // Should succeed even with no listeners
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_server_transport_with_custom_config() {
        let mut config = TransportConfig::default();
        config.compression = true;
        config.headers.insert("Server".to_string(), "MCP-Test/1.0".to_string());
        
        let server = HttpServerTransport::with_config("127.0.0.1:9999", config);
        
        assert_eq!(server.bind_addr, "127.0.0.1:9999");
        assert!(server.config.compression);
        assert_eq!(server.config.headers.get("Server"), Some(&"MCP-Test/1.0".to_string()));
    }

    #[tokio::test]
    async fn test_server_stop_when_not_running() {
        let mut server = HttpServerTransport::new("127.0.0.1:0");
        
        assert!(!server.is_running());
        
        // Should be able to stop even when not running
        let result = server.stop().await;
        assert!(result.is_ok());
        assert!(!server.is_running());
    }

    #[tokio::test]
    async fn test_invalid_header_values_in_config() {
        let mut config = TransportConfig::default();
        // Add headers that might fail to parse (should be handled smoothly)
        config.headers.insert("Valid-Header".to_string(), "valid-value".to_string());
        config.headers.insert("Invalid\nHeader".to_string(), "invalid\nvalue".to_string());
        
        let transport = HttpClientTransport::with_config(
            "http://localhost:3000",
            None,
            config,
        ).await;
        
        // Should succeed even with invalid headers (they'll be skipped)
        assert!(transport.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_request_tracking() {
        let transport = HttpClientTransport::new("http://localhost:3000", None).await.unwrap();
        
        // Track multiple requests simultaneously
        let mut request_ids = vec![];
        for i in 1..=10 {
            let id = Value::from(i);
            transport.track_request(&id).await;
            request_ids.push(id);
        }
        
        assert_eq!(transport.active_request_count().await, 10);
        
        // Untrack them all
        for id in request_ids {
            transport.untrack_request(&id).await;
        }
        
        assert_eq!(transport.active_request_count().await, 0);
    }

    #[tokio::test]
    async fn test_request_id_counter_thread_safety() {
        let transport = std::sync::Arc::new(
            HttpClientTransport::new("http://localhost:3000", None).await.unwrap()
        );
        
        let mut handles = vec![];
        
        // Spawn multiple tasks generating request IDs concurrently
        for _ in 0..10 {
            let transport_clone = transport.clone();
            let handle = tokio::spawn(async move {
                let mut ids = vec![];
                for _ in 0..10 {
                    ids.push(transport_clone.next_request_id().await);
                }
                ids
            });
            handles.push(handle);
        }
        
        let mut all_ids = vec![];
        for handle in handles {
            let ids = handle.await.unwrap();
            all_ids.extend(ids);
        }
        
        // All IDs should be unique
        all_ids.sort();
        let mut unique_ids = all_ids.clone();
        unique_ids.dedup();
        
        assert_eq!(all_ids.len(), unique_ids.len());
        assert_eq!(all_ids.len(), 100); // 10 tasks * 10 IDs each
    }

    #[tokio::test]
    async fn test_sse_url_none_case() {
        // Explicitly test None case for SSE URL
        let transport = HttpClientTransport::new(
            "http://localhost:3000",
            None::<String>,
        ).await.unwrap();
        
        assert!(transport.sse_url.is_none());
        
        let info = transport.connection_info();
        assert!(info.contains("sse: None"));
    }

    #[tokio::test] 
    async fn test_sse_url_some_case() {
        let transport = HttpClientTransport::new(
            "http://localhost:3000", 
            Some("http://localhost:3000/events".to_string()),
        ).await.unwrap();
        
        assert!(transport.sse_url.is_some());
        assert_eq!(transport.sse_url.as_ref().unwrap(), "http://localhost:3000/events");
    }

    #[tokio::test]
    async fn test_transport_config_defaults() {
        let config = TransportConfig::default();
        
        assert_eq!(config.read_timeout_ms, None);
        assert_eq!(config.write_timeout_ms, None);
        assert_eq!(config.connect_timeout_ms, None);
        assert!(config.headers.is_empty());
    }

    #[tokio::test]
    async fn test_request_with_different_value_types() {
        let transport = HttpClientTransport::new("http://localhost:3000", None).await.unwrap();
        
        // Test tracking requests with different Value types
        let string_id = Value::String("string-id".to_string());
        let number_id = Value::Number(serde_json::Number::from(123));
        let array_id = Value::Array(vec![Value::from(1), Value::from(2)]);
        
        transport.track_request(&string_id).await;
        transport.track_request(&number_id).await;
        transport.track_request(&array_id).await;
        
        assert_eq!(transport.active_request_count().await, 3);
        
        transport.untrack_request(&string_id).await;
        transport.untrack_request(&number_id).await;
        transport.untrack_request(&array_id).await;
        
        assert_eq!(transport.active_request_count().await, 0);
    }
}
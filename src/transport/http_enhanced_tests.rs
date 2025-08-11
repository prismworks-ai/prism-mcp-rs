// improved tests for HTTP transport to achieve >90% coverage
#[cfg(test)]
#[cfg(not(coverage))]
mod improved_tests {
    use super::super::http::*;
    use crate::protocol::types::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification};
    use crate::transport::traits::{Transport, ServerTransport, TransportConfig};
    use serde_json::{json, Value};
    use std::time::Duration;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

    // Helper function to create a test request
    fn create_test_request(id: Value, method: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params: Some(json!({"test": "data"})),
        }
    }

    // Helper function to create a test response
    fn create_test_response(id: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({"success": true})),
        }
    }

    // Helper function to create a test notification
    fn create_test_notification(method: &str) -> JsonRpcNotification {
        JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(json!({"test": "notification"})),
        }
    }

    #[tokio::test]
    async fn test_request_id_generation() {
        let transport = HttpClientTransport::new("http://localhost:3000", None).await.unwrap();
        
        let id1 = transport.next_request_id().await;
        let id2 = transport.next_request_id().await;
        let id3 = transport.next_request_id().await;
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[tokio::test]
    async fn test_request_tracking() {
        let transport = HttpClientTransport::new("http://localhost:3000", None).await.unwrap();
        
        let request_id = Value::from(123);
        
        // Initially no active requests
        assert_eq!(transport.active_request_count().await, 0);
        
        // Track a request
        transport.track_request(&request_id).await;
        assert_eq!(transport.active_request_count().await, 1);
        
        // Track another request
        let request_id2 = Value::from(456);
        transport.track_request(&request_id2).await;
        assert_eq!(transport.active_request_count().await, 2);
        
        // Untrack first request
        transport.untrack_request(&request_id).await;
        assert_eq!(transport.active_request_count().await, 1);
        
        // Untrack second request
        transport.untrack_request(&request_id2).await;
        assert_eq!(transport.active_request_count().await, 0);
    }

    #[tokio::test]
    async fn test_send_request_with_mock_server() {
        let mock_server = MockServer::start().await;
        
        // Set up mock response
        let response = create_test_response(Value::from(1));
        Mock::given(method("POST"))
            .and(path("/mcp"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&mock_server)
            .await;
            
        let mut transport = HttpClientTransport::new(mock_server.uri(), None).await.unwrap();
        
        let request = create_test_request(Value::from(1), "test_method");
        let result = transport.send_request(request).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, Value::from(1));
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn test_send_request_auto_id_generation() {
        let mock_server = MockServer::start().await;
        
        // Set up mock to accept any request
        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"success": true}
            })))
            .mount(&mock_server)
            .await;
            
        let mut transport = HttpClientTransport::new(mock_server.uri(), None).await.unwrap();
        
        // Request with null ID should get auto-generated ID
        let request = create_test_request(Value::Null, "test_method");
        let result = transport.send_request(request).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, Value::from(1));
    }

    #[tokio::test]
    async fn test_send_request_http_error() {
        let mock_server = MockServer::start().await;
        
        // Set up mock to return HTTP error
        Mock::given(method("POST"))
            .and(path("/mcp"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;
            
        let mut transport = HttpClientTransport::new(mock_server.uri(), None).await.unwrap();
        
        let request = create_test_request(Value::from(1), "test_method");
        let result = transport.send_request(request).await;
        
        assert!(result.is_err());
        if let Err(crate::core::error::McpError::Http(msg)) = result {
            assert!(msg.contains("HTTP error: 500"));
        } else {
            panic!("Expected HTTP error");
        }
    }

    #[tokio::test]
    async fn test_send_request_connection_error() {
        // Use invalid URL to trigger connection error
        let mut transport = HttpClientTransport::new("http://127.0.0.1:1", None).await.unwrap();
        
        let request = create_test_request(Value::from(1), "test_method");
        let result = transport.send_request(request).await;
        
        assert!(result.is_err());
        match result {
            Err(crate::core::error::McpError::Connection(_)) | 
            Err(crate::core::error::McpError::Http(_)) => {}, // Expected
            _ => panic!("Expected connection error"),
        }
    }

    #[tokio::test]
    async fn test_send_notification() {
        let mock_server = MockServer::start().await;
        
        // Set up mock for notification endpoint
        Mock::given(method("POST"))
            .and(path("/mcp/notify"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
            
        let mut transport = HttpClientTransport::new(mock_server.uri(), None).await.unwrap();
        
        let notification = create_test_notification("test_notification");
        let result = transport.send_notification(notification).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_receive_notification_empty() {
        let mut transport = HttpClientTransport::new("http://localhost:3000", None).await.unwrap();
        
        let result = transport.receive_notification().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_close_transport() {
        let mut transport = HttpClientTransport::new("http://localhost:3000", None).await.unwrap();
        
        assert!(transport.is_connected());
        
        let result = transport.close().await;
        assert!(result.is_ok());
        assert!(!transport.is_connected());
        assert!(!transport.has_notification_receiver());
    }

    #[tokio::test]
    async fn test_connection_info() {
        let transport = HttpClientTransport::new(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
        ).await.unwrap();
        
        let info = transport.connection_info();
        assert!(info.contains("HTTP transport"));
        assert!(info.contains("http://localhost:3000"));
        assert!(info.contains("http://localhost:3000/events"));
        assert!(info.contains("Connected"));
    }

    #[tokio::test]
    async fn test_http_server_lifecycle() {
        let mut transport = HttpServerTransport::new("127.0.0.1:0");
        
        assert!(!transport.is_running());
        
        // Start server
        let result = transport.start().await;
        assert!(result.is_ok());
        assert!(transport.is_running());
        
        // Stop server
        let result = transport.stop().await;
        assert!(result.is_ok());
        assert!(!transport.is_running());
    }

    #[tokio::test]
    async fn test_http_server_send_notification() {
        let mut transport = HttpServerTransport::new("127.0.0.1:0");
        
        let notification = create_test_notification("server_notification");
        let result = transport.send_notification(notification).await;
        
        // Should succeed even without SSE clients
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_http_server_info() {
        let transport = HttpServerTransport::new("127.0.0.1:8080");
        
        let info = transport.server_info();
        assert!(info.contains("HTTP server transport"));
        assert!(info.contains("127.0.0.1:8080"));
    }
}
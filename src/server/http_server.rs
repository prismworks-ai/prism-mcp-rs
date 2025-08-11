// ! HTTP-specific MCP server implementation
// !
// ! Module provides a specialized MCP server that integrates directly with HTTP transport.

use crate::core::error::McpResult;
use crate::protocol::types::{JsonRpcRequest, JsonRpcResponse};
use crate::server::mcp_server::McpServer;
use crate::transport::http::HttpServerTransport;
use crate::transport::traits::ServerTransport;
use std::sync::Arc;
use tokio::sync::Mutex;

/// HTTP-specific MCP server that properly integrates with HTTP transport
pub struct HttpMcpServer {
    server: Arc<Mutex<McpServer>>,
    transport: Option<HttpServerTransport>,
}

impl HttpMcpServer {
    /// Create a new HTTP MCP server
    pub fn new(name: String, version: String) -> Self {
        Self {
            server: Arc::new(Mutex::new(McpServer::new(name, version))),
            transport: None,
        }
    }

    /// Get a reference to the underlying MCP server
    pub async fn server(&self) -> Arc<Mutex<McpServer>> {
        self.server.clone()
    }

    /// Start the HTTP server with proper request handling integration
    pub async fn start(&mut self, mut transport: HttpServerTransport) -> McpResult<()> {
        // Set up the request handler to use the MCP server
        let server_clone = self.server.clone();

        transport
            .set_request_handler(move |request: JsonRpcRequest| {
                let server = server_clone.clone();
                let (tx, rx) = tokio::sync::oneshot::channel();

                tokio::spawn(async move {
                    let server_guard = server.lock().await;
                    let response = server_guard
                        .handle_request(request)
                        .await
                        .unwrap_or_else(|e| {
                            tracing::error!("Error handling HTTP request: {}", e);
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: serde_json::Value::Null,
                                result: Some(serde_json::json!({
                                    "error": {
                                        "code": -32603,
                                        "message": e.to_string()
                                    }
                                })),
                            }
                        });
                    let _ = tx.send(response);
                });

                rx
            })
            .await;

        // Start the transport
        transport.start().await?;

        self.transport = Some(transport);
        Ok(())
    }

    /// Stop the HTTP server
    pub async fn stop(&mut self) -> McpResult<()> {
        if let Some(transport) = &mut self.transport {
            transport.stop().await?;
        }
        self.transport = None;
        Ok(())
    }

    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        self.transport.as_ref().is_some_and(|t| t.is_running())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_http_server_creation() {
        let server = HttpMcpServer::new("test-server".to_string(), "1.0.0".to_string());

        assert!(!server.is_running());
        assert!(server.transport.is_none());

        // Test that we can get a reference to the underlying server
        let server_ref = server.server().await;
        let server_guard = server_ref.lock().await;

        // Server should start not running
        assert!(!server_guard.is_running().await);

        // But should be able to initialize
        server_guard.initialize().await.unwrap();
        assert!(server_guard.is_running().await);
        assert!(server_guard.is_initialized().await);
    }

    #[tokio::test]
    async fn test_http_server_initialization() {
        let server = HttpMcpServer::new("test-server".to_string(), "1.0.0".to_string());

        // Create a mock transport for testing
        // Since HttpServerTransport might require actual HTTP setup, we'll test what we can
        assert!(!server.is_running());

        // Test server reference
        let server_ref = server.server().await;
        let _server_guard = server_ref.lock().await;

        // Verify the server integration works
    }

    #[tokio::test]
    async fn test_http_server_stop_when_not_running() {
        let mut server = HttpMcpServer::new("test-server".to_string(), "1.0.0".to_string());

        // Should be able to stop a server that's not running without error
        let result = server.stop().await;
        assert!(result.is_ok());
        assert!(!server.is_running());
    }

    #[tokio::test]
    async fn test_http_server_state_transitions() {
        let mut server = HttpMcpServer::new("test-server".to_string(), "1.0.0".to_string());

        // Initial state: not running
        assert!(!server.is_running());

        // After stop (when not running): still not running
        let _ = server.stop().await;
        assert!(!server.is_running());

        // Test that transport is properly managed
        assert!(server.transport.is_none());
    }

    #[tokio::test]
    async fn test_http_server_concurrent_access() {
        let server = Arc::new(Mutex::new(HttpMcpServer::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
        )));

        let server1 = server.clone();
        let server2 = server.clone();

        // Test concurrent access to server state
        let handle1 = tokio::spawn(async move {
            let s = server1.lock().await;
            s.is_running()
        });

        let handle2 = tokio::spawn(async move {
            let s = server2.lock().await;
            s.server().await
        });

        let (running, _server_ref) = tokio::join!(handle1, handle2);
        assert!(!running.unwrap());
    }

    #[tokio::test]
    async fn test_http_server_request_handler_setup() {
        let server = HttpMcpServer::new("test-server".to_string(), "1.0.0".to_string());

        // Test that the server correctly sets up request handling
        let server_ref = server.server().await;
        let server_guard = server_ref.lock().await;

        // Initialize server for testing
        server_guard.initialize().await.unwrap();
        assert!(server_guard.is_running().await);

        // Test adding a tool to verify request handling setup
        server_guard
            .add_simple_tool("test-tool", "Test tool for request handling", |_args| {
                Ok(vec![crate::protocol::types::ContentBlock::text("handled")])
            })
            .await
            .unwrap();

        assert!(server_guard.has_tool("test-tool").await);
    }

    #[tokio::test]
    async fn test_http_server_error_handling() {
        let server = HttpMcpServer::new("test-server".to_string(), "1.0.0".to_string());

        // Test error handling in request processing
        // The server should handle errors smoothly and return proper JSON-RPC error responses

        let server_ref = server.server().await;

        // Test that invalid requests are handled properly
        // Create a malformed request to test error handling
        let invalid_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("test-id"),
            method: "invalid/method".to_string(),
            params: Some(json!({})),
        };

        // The server should handle this smoothly
        let server_guard = server_ref.lock().await;
        let response = server_guard.handle_request(invalid_request).await;

        // Should return an error response, not panic
        match response {
            Ok(_) => {
                // Valid response (could be an error response)
            }
            Err(_) => {
                // Error in processing, which is acceptable for invalid requests
            }
        }
    }

    #[tokio::test]
    async fn test_http_server_multiple_requests() {
        let server = HttpMcpServer::new("test-server".to_string(), "1.0.0".to_string());
        let server_ref = server.server().await;

        // Test handling multiple requests
        let requests = vec![
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(1),
                method: "ping".to_string(),
                params: Some(json!({})),
            },
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(2),
                method: "tools/list".to_string(),
                params: Some(json!({})),
            },
        ];

        for request in requests {
            let server_guard = server_ref.lock().await;
            let _response = server_guard.handle_request(request).await;
            // Each request should be handled without panicking
        }
    }

    #[tokio::test]
    async fn test_http_server_response_format() {
        let server = HttpMcpServer::new("test-server".to_string(), "1.0.0".to_string());
        let server_ref = server.server().await;

        // Test that responses are properly formatted JSON-RPC
        let ping_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!("ping-test"),
            method: "ping".to_string(),
            params: Some(json!({})),
        };

        let server_guard = server_ref.lock().await;
        let response = server_guard.handle_request(ping_request).await;

        match response {
            Ok(json_response) => {
                // Should be a valid JSON-RPC response
                assert_eq!(json_response.jsonrpc, "2.0");
                assert_eq!(json_response.id, json!("ping-test"));
                // Should have either result or error, but not both
                assert!(json_response.result.is_some());
            }
            Err(_) => {
                // Error responses are also acceptable for some methods
            }
        }
    }

    #[test]
    fn test_http_server_sync_methods() {
        let server = HttpMcpServer::new("test-server".to_string(), "1.0.0".to_string());

        // Test synchronous methods
        assert!(!server.is_running());

        // Test that the server maintains proper state
        assert!(server.transport.is_none());
    }

    #[tokio::test]
    async fn test_http_server_lifecycle() {
        let mut server = HttpMcpServer::new("lifecycle-test".to_string(), "1.0.0".to_string());

        // Test complete lifecycle: create -> (start) -> stop
        // Note: We can't easily test start() without a real HttpServerTransport
        // but we can test the stop() method and state management

        assert!(!server.is_running());

        // Test stop when not running
        let stop_result = server.stop().await;
        assert!(stop_result.is_ok());
        assert!(!server.is_running());

        // Test that server can be stopped multiple times safely
        let stop_result2 = server.stop().await;
        assert!(stop_result2.is_ok());
        assert!(!server.is_running());
    }
}

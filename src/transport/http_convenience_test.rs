// ! complete tests for HTTP transport convenience methods
// !
// ! This test suite validates all production-grade convenience methods
// ! added to HttpClientTransport

#![cfg(test)]
#![cfg(not(coverage))]
#![allow(unexpected_cfgs)]

#[cfg(test)]
mod tests {
    use super::super::http::HttpClientTransport;
    use super::super::http_convenience::*;
    use super::super::traits::Transport;
    use crate::core::error::McpError;
    use crate::protocol::types::JsonRpcRequest;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::time::Duration;

    // ============================================================================
    // Helper Functions
    // ============================================================================

    async fn create_test_transport() -> HttpClientTransport {
        HttpClientTransport::new("http://localhost:3000", None)
            .await
            .expect("Failed to create test transport")
    }

    async fn create_test_transport_with_sse() -> HttpClientTransport {
        HttpClientTransport::new(
            "http://localhost:3000",
            Some("http://localhost:3000/events"),
        )
        .await
        .expect("Failed to create test transport with SSE")
    }

    // ============================================================================
    // Builder Pattern Tests
    // ============================================================================

    #[tokio::test]
    async fn test_builder_basic_configuration() {
        let transport = HttpClientTransport::builder()
            .base_url("http://localhost:3000")
            .build()
            .await
            .expect("Builder should create transport");

        assert_eq!(transport.get_base_url(), "http://localhost:3000");
        assert!(transport.get_sse_url().is_none());
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn test_builder_full_configuration() {
        let transport = HttpClientTransport::builder()
            .base_url("http://localhost:3000")
            .sse_url("http://localhost:3000/events")
            .timeout(15_000)
            .connect_timeout(5_000)
            .header("Authorization", "Bearer test-token")
            .header("X-Client-Version", "1.0.0")
            .compression(true)
            .max_message_size(1024 * 1024)
            .build()
            .await
            .expect("Builder should create fully configured transport");

        assert_eq!(transport.get_base_url(), "http://localhost:3000");
        assert_eq!(
            transport.get_sse_url(),
            Some("http://localhost:3000/events")
        );

        let config = transport.get_config();
        assert_eq!(config.read_timeout_ms, Some(15_000));
        assert_eq!(config.write_timeout_ms, Some(15_000));
        assert_eq!(config.connect_timeout_ms, Some(5_000));
        assert_eq!(config.max_message_size, Some(1024 * 1024));
        assert!(config.compression);
        assert!(config.headers.contains_key("Authorization"));
        assert!(config.headers.contains_key("X-Client-Version"));
    }

    #[tokio::test]
    async fn test_builder_missing_base_url() {
        let result = HttpClientTransport::builder().timeout(30_000).build().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::Protocol(msg) => {
                assert!(msg.contains("Base URL is required"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[tokio::test]
    async fn test_builder_fluent_interface() {
        // Test that builder methods can be chained in any order
        let transport = HttpClientTransport::builder()
            .compression(false)
            .header("User-Agent", "test-client")
            .timeout(20_000)
            .base_url("http://example.com")
            .sse_url("http://example.com/sse")
            .max_message_size(512 * 1024)
            .connect_timeout(10_000)
            .build()
            .await
            .expect("Fluent builder should work");

        assert_eq!(transport.get_base_url(), "http://example.com");
        assert_eq!(transport.get_sse_url(), Some("http://example.com/sse"));
    }

    // ============================================================================
    // URL and Endpoint Management Tests
    // ============================================================================

    #[tokio::test]
    async fn test_get_endpoints() {
        let transport = create_test_transport().await;
        let endpoints = transport.get_endpoints();

        assert_eq!(endpoints.mcp, "http://localhost:3000/mcp");
        assert_eq!(endpoints.notify, "http://localhost:3000/mcp/notify");
        assert_eq!(endpoints.health, "http://localhost:3000/health");
        assert!(endpoints.events.is_none());
    }

    #[tokio::test]
    async fn test_get_endpoints_with_sse() {
        let transport = create_test_transport_with_sse().await;
        let endpoints = transport.get_endpoints();

        assert_eq!(endpoints.mcp, "http://localhost:3000/mcp");
        assert_eq!(endpoints.notify, "http://localhost:3000/mcp/notify");
        assert_eq!(endpoints.health, "http://localhost:3000/health");
        assert_eq!(
            endpoints.events,
            Some("http://localhost:3000/events".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_base_url() {
        let transport = create_test_transport().await;
        assert_eq!(transport.get_base_url(), "http://localhost:3000");
    }

    #[tokio::test]
    async fn test_get_sse_url() {
        let transport_no_sse = create_test_transport().await;
        assert!(transport_no_sse.get_sse_url().is_none());

        let transport_with_sse = create_test_transport_with_sse().await;
        assert_eq!(
            transport_with_sse.get_sse_url(),
            Some("http://localhost:3000/events")
        );
    }

    // ============================================================================
    // Configuration Management Tests
    // ============================================================================

    #[tokio::test]
    async fn test_get_config() {
        let transport = create_test_transport().await;
        let config = transport.get_config();

        // Verify default configuration values
        assert_eq!(config.connect_timeout_ms, Some(30_000));
        assert_eq!(config.read_timeout_ms, Some(60_000));
        assert_eq!(config.write_timeout_ms, Some(30_000));
        assert_eq!(config.max_message_size, Some(16 * 1024 * 1024));
        assert!(!config.compression);
    }

    #[tokio::test]
    async fn test_update_headers() {
        let mut transport = create_test_transport().await;

        let mut new_headers = HashMap::new();
        new_headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
        new_headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        transport.update_headers(new_headers);

        // Verify headers were added (we can't directly check HeaderMap in this test)
        // This would require additional methods to inspect headers
        // For now, we just ensure the method doesn't panic
    }

    #[tokio::test]
    async fn test_set_timeout() {
        let mut transport = create_test_transport().await;

        transport.set_timeout(45_000);

        let config = transport.get_config();
        assert_eq!(config.read_timeout_ms, Some(45_000));
        assert_eq!(config.write_timeout_ms, Some(45_000));
    }

    // ============================================================================
    // Health and Status Tests
    // ============================================================================

    #[tokio::test]
    async fn test_is_healthy() {
        let transport = create_test_transport().await;
        // Should return true since we're connected
        assert!(transport.is_healthy());
    }

    #[tokio::test]
    async fn test_connection_stats() {
        let transport = create_test_transport().await;
        let stats = transport.get_connection_stats().await;

        // Verify default stats structure
        assert_eq!(stats.requests_sent, 0);
        assert_eq!(stats.responses_received, 0);
        assert_eq!(stats.request_failures, 0);
        assert_eq!(stats.notifications_sent, 0);
        assert_eq!(stats.notifications_received, 0);
        assert_eq!(stats.reconnect_attempts, 0);
    }

    // ============================================================================
    // Request Building Tests
    // ============================================================================

    #[tokio::test]
    async fn test_call_method_simple_structure() {
        let mut transport = create_test_transport().await;

        // This will fail because there's no server, but we can test the structure
        let result = transport.call_method_simple("test_method").await;

        // Should fail with connection error (no actual server running)
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::Http(_) | McpError::Connection(_) => {
                // Expected - no server running
            }
            other => panic!("Unexpected error type: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_call_method_with_params_structure() {
        let mut transport = create_test_transport().await;

        #[derive(serde::Serialize)]
        struct TestParams {
            value: i32,
            name: String,
        }

        #[derive(serde::Deserialize, Debug)]
        struct TestResponse {
            #[allow(dead_code)]
            result: String,
        }

        let params = TestParams {
            value: 42,
            name: "test".to_string(),
        };

        // This will fail because there's no server, but we can test the structure
        let result: Result<TestResponse, _> = transport.call_method("test_method", params).await;

        // Should fail with connection error (no actual server running)
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::Http(_) | McpError::Connection(_) => {
                // Expected - no server running
            }
            other => panic!("Unexpected error type: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_batch_requests_structure() {
        let mut transport = create_test_transport().await;

        let requests = vec![
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "test1".to_string(),
                params: None,
                id: Value::from(1),
            },
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "test2".to_string(),
                params: Some(Value::from("test")),
                id: Value::from(2),
            },
        ];

        // This will fail because there's no server, but we can test the structure
        let result = transport.batch_requests(requests).await;

        // Should fail with connection error (no actual server running)
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::Http(_) | McpError::Connection(_) => {
                // Expected - no server running
            }
            other => panic!("Unexpected error type: {other:?}"),
        }
    }

    // ============================================================================
    // Retry Configuration Tests
    // ============================================================================

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();

        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(10));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.retry_on_timeout);
        assert!(config.retry_on_connection);
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();

        assert_eq!(policy.default.max_attempts, 3);
        assert!(policy.method_specific.is_empty());
    }

    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 1.5,
            retry_on_timeout: false,
            retry_on_connection: true,
        };

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 1.5);
        assert!(!config.retry_on_timeout);
        assert!(config.retry_on_connection);
    }

    // ============================================================================
    // Connection Management Tests
    // ============================================================================

    #[tokio::test]
    async fn test_test_connection_no_server() {
        let transport = create_test_transport().await;

        // Should return false since there's no server running
        let is_connected = transport
            .test_connection()
            .await
            .expect("test_connection should not fail");

        assert!(!is_connected);
    }

    #[tokio::test]
    async fn test_reconnect_structure() {
        let mut transport = create_test_transport().await;

        // Test that reconnect doesn't panic and preserves configuration
        let original_base_url = transport.get_base_url().to_string();
        let original_config = transport.get_config().clone();

        let result = transport.reconnect().await;

        // Should succeed (creating new transport with same config)
        assert!(result.is_ok());

        // Verify configuration is preserved
        assert_eq!(transport.get_base_url(), original_base_url);
        assert_eq!(
            transport.get_config().connect_timeout_ms,
            original_config.connect_timeout_ms
        );
        assert_eq!(
            transport.get_config().read_timeout_ms,
            original_config.read_timeout_ms
        );
    }

    // ============================================================================
    // Metrics and Observability Tests
    // ============================================================================

    #[tokio::test]
    async fn test_export_metrics() {
        let transport = create_test_transport().await;

        let metrics = transport
            .export_metrics()
            .await
            .expect("export_metrics should succeed");

        // Verify metrics structure
        assert_eq!(metrics.connection_stats.requests_sent, 0);
        assert_eq!(metrics.performance.avg_latency, Duration::from_millis(0));
        assert_eq!(metrics.errors.total_errors, 0);
    }

    #[tokio::test]
    async fn test_placeholder_methods() {
        let mut transport = create_test_transport().await;

        // Test placeholder methods that don't cause panics
        transport.enable_request_logging(true);
        transport.enable_request_logging(false);

        let last_error = transport.get_last_error();
        assert!(last_error.is_none());

        transport.set_retry_policy(RetryPolicy::default());
    }

    // ============================================================================
    // Error Handling Tests
    // ============================================================================

    #[tokio::test]
    async fn test_call_with_retry_no_server() {
        let mut transport = create_test_transport().await;

        #[derive(serde::Serialize, Clone)]
        struct TestParams {
            value: i32,
        }

        #[derive(serde::Deserialize, Debug)]
        struct TestResponse {
            #[allow(dead_code)]
            result: String,
        }

        let params = TestParams { value: 42 };
        let retry_config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            retry_on_timeout: true,
            retry_on_connection: true,
        };

        let start = std::time::Instant::now();
        let result: Result<TestResponse, _> = transport
            .call_with_retry("test_method", params, retry_config)
            .await;
        let elapsed = start.elapsed();

        // Should fail after retries
        assert!(result.is_err());

        // Should have taken some time due to retries (at least initial_delay)
        assert!(elapsed >= Duration::from_millis(10));

        match result.unwrap_err() {
            McpError::Http(_) | McpError::Connection(_) => {
                // Expected - connection failed and retries exhausted
            }
            other => panic!("Unexpected error type: {other:?}"),
        }
    }

    // ============================================================================
    // Type Safety Tests
    // ============================================================================

    #[test]
    fn test_type_definitions() {
        // Test that all our types can be constructed and have expected properties

        let _server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            protocol_version: "2025-06-18".to_string(),
            capabilities: HashMap::new(),
            metadata: HashMap::new(),
        };

        let _stats = ConnectionStats {
            requests_sent: 100,
            responses_received: 95,
            request_failures: 5,
            notifications_sent: 10,
            notifications_received: 8,
            uptime: Duration::from_secs(3600),
            connected_at: Some(std::time::Instant::now()),
            last_success_at: Some(std::time::Instant::now()),
            last_error_at: None,
            avg_response_time: Duration::from_millis(150),
            reconnect_attempts: 2,
        };

        let _endpoints = HttpEndpoints {
            mcp: "http://localhost:3000/mcp".to_string(),
            notify: "http://localhost:3000/mcp/notify".to_string(),
            events: Some("http://localhost:3000/events".to_string()),
            health: "http://localhost:3000/health".to_string(),
        };

        let _metrics = TransportMetrics {
            connection_stats: ConnectionStats::default(),
            performance: PerformanceMetrics::default(),
            errors: ErrorMetrics::default(),
        };
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[tokio::test]
    async fn test_full_workflow_without_server() {
        // Test a complete workflow using convenience methods
        let mut transport = HttpClientTransport::builder()
            .base_url("http://localhost:3000")
            .sse_url("http://localhost:3000/events")
            .timeout(5_000)
            .header("Authorization", "Bearer test-token")
            .build()
            .await
            .expect("Failed to build transport");

        // Check health
        assert!(transport.is_healthy());

        // Get stats
        let stats = transport.get_connection_stats().await;
        assert_eq!(stats.requests_sent, 0);

        // Test connection (should fail - no server)
        let connected = transport
            .test_connection()
            .await
            .expect("test_connection failed");
        assert!(!connected);

        // Get endpoints
        let endpoints = transport.get_endpoints();
        assert_eq!(endpoints.mcp, "http://localhost:3000/mcp");

        // Update configuration
        transport.set_timeout(10_000);
        assert_eq!(transport.get_config().read_timeout_ms, Some(10_000));

        // Export metrics
        let metrics = transport
            .export_metrics()
            .await
            .expect("export_metrics failed");
        assert_eq!(metrics.connection_stats.requests_sent, 0);
    }

    #[tokio::test]
    async fn test_builder_validation() {
        // Test various builder validation scenarios

        // Empty builder should fail
        let result = HttpClientTransport::builder().build().await;
        assert!(result.is_err());

        // Builder with just base URL should succeed
        let result = HttpClientTransport::builder()
            .base_url("http://localhost:3000")
            .build()
            .await;
        assert!(result.is_ok());

        // Builder with all options should succeed
        let result = HttpClientTransport::builder()
            .base_url("http://localhost:3000")
            .sse_url("http://localhost:3000/events")
            .timeout(30_000)
            .connect_timeout(10_000)
            .max_message_size(1024 * 1024)
            .compression(true)
            .header("User-Agent", "test-client")
            .header("Accept", "application/json")
            .build()
            .await;
        assert!(result.is_ok());
    }
}

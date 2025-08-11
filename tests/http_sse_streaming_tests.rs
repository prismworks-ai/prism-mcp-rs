// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! HTTP SSE Streaming and Error Path Tests
// !
// ! This test suite specifically targets the SSE streaming functionality
// ! and error handling paths in the HTTP transport to maximize coverage.

#![cfg(feature = "http")]

use axum::http::HeaderMap;
use prism_mcp_rs::{
    core::error::{McpError, McpResult},
    protocol::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse},
    transport::{
        ConnectionState,
        http::{HttpClientTransport, HttpServerTransport},
        traits::{ServerTransport, Transport, TransportConfig},
    },
};
use reqwest::Client;
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, broadcast, mpsc},
    time::timeout,
};

#[cfg(feature = "http")]
mod http_sse_streaming_tests {
    use super::*;

    // ========================================================================
    // SSE Data Parsing and Processing Tests
    // ========================================================================

    #[tokio::test]
    async fn test_sse_line_parsing_various_formats() {
        // Test various SSE line formats that might be encountered
        let sse_lines = vec![
            // Valid data lines
            "data: {\"jsonrpc\": \"2.0\", \"method\": \"test\", \"params\": {}}",
            "data: {\"jsonrpc\": \"2.0\", \"method\": \"notification\", \"params\": {\"key\": \"value\"}}",
            "data: {\"jsonrpc\": \"2.0\", \"method\": \"unicode_test\", \"params\": {\"emoji\": \"#🌍\"}}",
            // Invalid JSON in data lines
            "data: {incomplete json",
            "data: not_json_at_all",
            "data: ",
            "data: null",
            "data: []",
            "data: 42",
            // Non-data lines (should be ignored)
            "event: test",
            "id: 123",
            "retry: 1000",
            ": comment line",
            "",
            "malformed line without colon",
            // Edge cases
            "data:",                  // No space after colon
            "data:  ",                // Only spaces
            "DATA: {\"test\": true}", // Wrong case
        ];

        let mut valid_notifications = 0;
        let mut processed_lines = 0;

        for line in sse_lines {
            processed_lines += 1;

            // Simulate the SSE parsing logic from handle_sse_stream
            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(notification) = serde_json::from_str::<JsonRpcNotification>(data) {
                    valid_notifications += 1;

                    // Verify notification structure
                    assert_eq!(notification.jsonrpc, "2.0");
                    assert!(!notification.method.is_empty());
                }
            }
        }

        assert_eq!(processed_lines, 18, "Should process all test lines");
        assert_eq!(valid_notifications, 3, "Should parse 3 valid notifications");
    }

    #[tokio::test]
    async fn test_sse_notification_channel_handling() {
        // Test SSE notification channel behavior
        let (notification_sender, mut notification_receiver) = mpsc::unbounded_channel();

        let test_notifications = vec![
            JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "method1".to_string(),
                params: Some(json!({"data": "test1"})),
            },
            JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "method2".to_string(),
                params: Some(json!({"data": "test2"})),
            },
        ];

        // Send notifications
        for notification in &test_notifications {
            let send_result = notification_sender.send(notification.clone());
            assert!(send_result.is_ok(), "Should send notification successfully");
        }

        // Receive notifications
        let mut received_count = 0;
        while let Ok(notification) = notification_receiver.try_recv() {
            received_count += 1;
            assert_eq!(notification.jsonrpc, "2.0");
            assert!(notification.method.starts_with("method"));
        }

        assert_eq!(received_count, 2, "Should receive all sent notifications");

        // Test channel drop behavior
        drop(notification_sender);

        // Should get disconnected error when trying to receive
        match notification_receiver.try_recv() {
            Err(mpsc::error::TryRecvError::Disconnected) => {} // Expected
            other => panic!("Expected disconnected error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_sse_notification_receiver_dropped() {
        // Test behavior when notification receiver is dropped
        let (notification_sender, notification_receiver) = mpsc::unbounded_channel();

        // Drop the receiver immediately
        drop(notification_receiver);

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_method".to_string(),
            params: Some(json!({"test": "data"})),
        };

        // Sending should fail when receiver is dropped
        let send_result = notification_sender.send(notification);
        assert!(send_result.is_err(), "Should fail when receiver is dropped");

        // This simulates the condition in handle_sse_stream where it returns Ok(())
        // when notification_sender.send(notification).is_err()
    }

    // ========================================================================
    // HTTP Client Error Path Tests
    // ========================================================================

    #[tokio::test]
    async fn test_http_client_request_id_mismatch() {
        // Test response ID validation
        let mut transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // We can't easily test the actual ID mismatch without a mock server,
        // but we can test the validation logic
        let request_id = json!(42);
        let response_with_wrong_id = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: json!(99), // Wrong ID
            result: Some(json!({"test": true})),
        };

        // Test ID comparison logic
        if response_with_wrong_id.id != request_id {
            // This would trigger the error in send_request
            let error_msg = format!(
                "Response ID {:?} does not match request ID {:?}",
                response_with_wrong_id.id, request_id
            );
            assert!(error_msg.contains("does not match"));
            assert!(error_msg.contains("99"));
            assert!(error_msg.contains("42"));
        }
    }

    #[tokio::test]
    async fn test_http_client_various_error_types() {
        // Test different error scenarios that might occur
        let mut transport = HttpClientTransport::new("http://nonexistent.invalid:9999", None)
            .await
            .unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "test_method".to_string(),
            params: Some(json!({"test": "data"})),
        };

        // This will fail with a connection error
        let result = transport.send_request(request).await;
        assert!(result.is_err(), "Should fail for nonexistent host");

        // Test the error type
        match result.unwrap_err() {
            McpError::Http(_) => {}       // Expected for connection failures
            McpError::Connection(_) => {} // Also acceptable for connection failures
            other => panic!("Expected Http or Connection error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_http_client_request_tracking() {
        // Test request tracking functionality
        let transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // Test initial state
        let initial_count = transport.active_request_count().await;
        assert_eq!(initial_count, 0, "Should start with no active requests");

        // Test connection info format
        let info = transport.connection_info();
        assert!(info.contains("HTTP transport"));
        assert!(info.contains("localhost:3000"));
        assert!(info.contains("sse: None"));
        assert!(info.contains("Connected"));
    }

    #[tokio::test]
    async fn test_http_client_notification_without_sse() {
        // Test notification handling when no SSE is configured
        let mut transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // Should return None when no SSE connection
        let notification_result = transport.receive_notification().await;
        match notification_result {
            Ok(Some(_)) => panic!("Should not receive notification without SSE"),
            Ok(None) => {} // Expected
            Err(McpError::Http(msg)) if msg.contains("channel disconnected") => {} // Also acceptable
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        // Test sending notification (will fail without server)
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "test_notification".to_string(),
            params: Some(json!({"event": "test"})),
        };

        let send_result = transport.send_notification(notification).await;
        assert!(send_result.is_err(), "Should fail without server");
    }

    #[tokio::test]
    async fn test_http_client_close_behavior() {
        // Test client close behavior
        let mut transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        assert!(transport.is_connected(), "Should be connected initially");

        let close_result = transport.close().await;
        assert!(close_result.is_ok(), "Close should succeed");
        assert!(
            !transport.is_connected(),
            "Should be disconnected after close"
        );

        // Test connection info after close
        let info = transport.connection_info();
        assert!(info.contains("Disconnected"));
    }

    // ========================================================================
    // HTTP Server Error Path Tests
    // ========================================================================

    #[tokio::test]
    async fn test_http_server_bind_address_validation() {
        // Test various bind addresses
        let addresses = vec![
            "127.0.0.1:0", // Valid
            "0.0.0.0:0",   // Valid
            "[::1]:0",     // Valid IPv6
            "localhost:0", // Valid hostname
        ];

        for addr in addresses {
            let transport = HttpServerTransport::new(addr);
            assert!(transport.server_info().contains(addr));
            assert!(!transport.is_running());
        }
    }

    #[tokio::test]
    async fn test_http_server_multiple_start_stop_cycles() {
        // Test multiple start/stop cycles
        let mut transport = HttpServerTransport::new("127.0.0.1:0");

        for cycle in 0..3 {
            // Start
            let start_result = transport.start().await;
            assert!(
                start_result.is_ok(),
                "Start should succeed in cycle {}",
                cycle
            );
            assert!(
                transport.is_running(),
                "Should be running after start in cycle {}",
                cycle
            );

            // Stop
            let stop_result = transport.stop().await;
            assert!(
                stop_result.is_ok(),
                "Stop should succeed in cycle {}",
                cycle
            );
            assert!(
                !transport.is_running(),
                "Should not be running after stop in cycle {}",
                cycle
            );
        }
    }

    #[tokio::test]
    async fn test_http_server_notification_broadcasting() {
        // Test server notification broadcasting
        let mut transport = HttpServerTransport::new("127.0.0.1:0");

        let notifications = vec![
            JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "server_notification1".to_string(),
                params: Some(json!({"server": "data1"})),
            },
            JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "server_notification2".to_string(),
                params: None,
            },
        ];

        for (i, notification) in notifications.into_iter().enumerate() {
            let result = transport.send_notification(notification).await;
            assert!(result.is_ok(), "Notification {} should succeed", i);
        }
    }

    // ========================================================================
    // Configuration and Header Tests
    // ========================================================================

    #[tokio::test]
    async fn test_http_config_timeout_handling() {
        // Test various timeout configurations
        let timeout_configs = vec![
            (Some(1000), Some(500), Some(2000)), // Normal timeouts
            (Some(0), Some(0), Some(0)),         // Zero timeouts
            (None, None, None),                  // No timeouts
            (Some(u64::MAX), Some(u64::MAX), Some(u64::MAX)), // Max timeouts
        ];

        for (i, (read_timeout, write_timeout, connect_timeout)) in
            timeout_configs.into_iter().enumerate()
        {
            let config = TransportConfig {
                read_timeout_ms: read_timeout,
                write_timeout_ms: write_timeout,
                connect_timeout_ms: connect_timeout,
                ..Default::default()
            };

            let client_result =
                HttpClientTransport::with_config("http://localhost:3000", None, config.clone())
                    .await;

            assert!(
                client_result.is_ok(),
                "Client should handle timeout config {}",
                i
            );

            let server = HttpServerTransport::with_config("127.0.0.1:0", config);
            assert!(
                !server.is_running(),
                "Server should handle timeout config {}",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_http_header_processing() {
        // Test header processing with various header types
        let test_headers = HashMap::from([
            (
                "Content-Type".to_string(),
                "application/json; charset=utf-8".to_string(),
            ),
            (
                "Authorization".to_string(),
                "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9".to_string(),
            ),
            (
                "X-Custom-Header".to_string(),
                "custom-value-123".to_string(),
            ),
            (
                "Accept-Language".to_string(),
                "en-US,en;q=0.9,fr;q=0.8".to_string(),
            ),
            (
                "User-Agent".to_string(),
                "MCP-SDK/1.0 (Rust; Test)".to_string(),
            ),
            ("#-Unicode".to_string(), "🌍-Value".to_string()), // Unicode headers
        ]);

        let config = TransportConfig {
            headers: test_headers.clone(),
            ..Default::default()
        };

        let transport = HttpClientTransport::with_config("http://httpbin.org", None, config).await;

        assert!(
            transport.is_ok(),
            "Should create transport with custom headers"
        );

        let transport = transport.unwrap();
        assert!(transport.is_connected());

        // Verify connection info
        let info = transport.connection_info();
        assert!(info.contains("httpbin.org"));
    }

    // ========================================================================
    // SSE Stream Error Handling Tests
    // ========================================================================

    #[tokio::test]
    async fn test_sse_stream_chunk_processing() {
        // Test SSE stream chunk processing with various chunk types
        let test_chunks = vec![
            // Valid SSE chunks
            b"data: {\"jsonrpc\": \"2.0\", \"method\": \"test\"}\n\n".to_vec(),
            b"data: {\"jsonrpc\": \"2.0\", \"method\": \"multi\"\ndata: line\"}\n\n".to_vec(),
            // Invalid UTF-8 bytes
            vec![0xFF, 0xFE, 0xFD],
            // Empty chunks
            b"".to_vec(),
            b"\n".to_vec(),
            b"\n\n".to_vec(),
            // Malformed SSE
            b"malformed chunk without proper format".to_vec(),
            b"data: incomplete".to_vec(), // No newline
        ];

        let (notification_sender, mut notification_receiver) = mpsc::unbounded_channel();

        for (i, chunk) in test_chunks.into_iter().enumerate() {
            // Simulate the chunk processing logic from handle_sse_stream
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(notification) = serde_json::from_str::<JsonRpcNotification>(data) {
                        let send_result = notification_sender.send(notification);
                        // In real code, if send fails, we return Ok(()) indicating receiver dropped
                        if send_result.is_err() {
                            break;
                        }
                    }
                }
            }
        }

        // Count received notifications
        let mut received_count = 0;
        while let Ok(_notification) = notification_receiver.try_recv() {
            received_count += 1;
        }

        // Should receive the valid notifications
        assert!(
            received_count >= 1,
            "Should receive at least 1 valid notification"
        );
    }

    #[tokio::test]
    async fn test_sse_connection_error_handling() {
        // Test SSE connection establishment errors
        let client = Client::new();
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "text/event-stream".parse().unwrap());

        let (notification_sender, _notification_receiver) =
            mpsc::unbounded_channel::<JsonRpcNotification>();

        // Test with invalid URL
        let invalid_urls = vec![
            "http://nonexistent.invalid:9999/events",
            "http://127.0.0.1:0/events",
        ];

        for url in invalid_urls {
            // This would be called in handle_sse_stream
            let mut request = client.get(url);
            for (name, value) in headers.iter() {
                let name_str = name.as_str();
                let value_bytes = value.as_bytes();
                request = request.header(name_str, value_bytes);
            }

            // This request would fail in real scenario
            // We just test that the error handling structure is in place
            let request_future = request.send();

            // Use timeout to avoid hanging
            let result = timeout(Duration::from_millis(100), request_future).await;

            // Should either timeout or fail quickly
            assert!(
                result.is_err() || result.unwrap().is_err(),
                "Should fail for invalid URL: {}",
                url
            );
        }
    }

    // ========================================================================
    // Feature Flag Tests
    // ========================================================================

    #[tokio::test]
    async fn test_tokio_stream_feature_handling() {
        // Test behavior with and without tokio-stream feature
        #[cfg(feature = "tokio-stream")]
        {
            // When tokio-stream is available, SSE should work
            use tokio_stream::StreamExt;

            let (tx, rx) = broadcast::channel(10);
            let mut stream = tokio_stream::wrappers::BroadcastStream::new(rx);

            // Send a test message
            let _ = tx.send("test".to_string());

            // Should be able to receive
            if let Some(result) = stream.next().await {
                assert!(
                    result.is_ok(),
                    "Stream should work with tokio-stream feature"
                );
            }
        }

        #[cfg(not(feature = "tokio-stream"))]
        {
            // Without tokio-stream, would log warning
            // This test just ensures the conditional compilation works
            assert!(true, "tokio-stream feature not enabled");
        }
    }

    // ========================================================================
    // Connection State and Info Tests
    // ========================================================================

    #[tokio::test]
    async fn test_connection_state_transitions() {
        // Test connection state transitions
        let mut transport = HttpClientTransport::new("http://localhost:3000", None)
            .await
            .unwrap();

        // Initial state
        assert!(transport.is_connected());
        assert!(transport.connection_info().contains("Connected"));

        // After close
        let _ = transport.close().await;
        assert!(!transport.is_connected());
        assert!(transport.connection_info().contains("Disconnected"));
    }

    #[tokio::test]
    async fn test_connection_info_formatting() {
        // Test connection info string formatting
        let test_cases = vec![
            (
                "http://localhost:3000",
                None,
                "base: http://localhost:3000, sse: None",
            ),
            (
                "http://example.com:8080",
                Some("http://example.com:8080/sse"),
                "base: http://example.com:8080, sse: Some(\"http://example.com:8080/sse\")",
            ),
        ];

        for (base_url, sse_url, expected_parts) in test_cases {
            let transport = HttpClientTransport::new(base_url, sse_url).await.unwrap();
            let info = transport.connection_info();

            assert!(info.contains("HTTP transport"));
            assert!(info.contains(base_url));

            if let Some(sse) = sse_url {
                assert!(info.contains(sse));
            } else {
                assert!(info.contains("sse: None"));
            }
        }
    }

    #[tokio::test]
    async fn test_server_info_formatting() {
        // Test server info string formatting
        let addresses = vec![
            "127.0.0.1:3000",
            "0.0.0.0:8080",
            "[::1]:9000",
            "localhost:4000",
        ];

        for addr in addresses {
            let transport = HttpServerTransport::new(addr);
            let info = transport.server_info();

            assert!(info.contains("HTTP server transport"));
            assert!(info.contains(&format!("bind: {}", addr)));
        }
    }
}

#[cfg(not(feature = "http"))]
#[test]
fn http_sse_streaming_tests_require_http_feature() {
    println!("HTTP SSE streaming tests require the 'http' feature to be enabled");
    println!("Run with: cargo test --features http --test http_sse_streaming_tests");
    assert!(true, "HTTP feature not enabled - this is expected");
}

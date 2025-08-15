// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

// ! complete tests for transport layer components
// !
// ! Phase 1: Fixed shallow tests with functional testing
// ! This replaces assert!(true) with actual functionality testing

#![cfg(feature = "stdio")]

use prism_mcp_rs::{
    core::error::McpError,
    protocol::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse},
    transport::traits::{ReconnectConfig, ServerTransport, TransportStats},
    transport::{ConnectionState, StdioServerTransport, TransportConfig},
};
use serde_json::json;

#[cfg(test)]
mod stdio_transport_tests {
    use super::*;

    #[test]
    fn test_stdio_transport_creation() {
        // Test actual object creation and verify initial state
        let transport = StdioServerTransport::new();

        // Verify transport was created with expected initial state
        assert!(
            !transport.is_running(),
            "New transport should not be running"
        );
        assert_eq!(
            transport.server_info(),
            "STDIO server transport (running: false)"
        );

        // Verify the transport has default configuration
        let default_config = TransportConfig::default();
        // We can't directly access the config, but we can verify it was constructed properly
        // by testing with a custom config
        let custom_transport = StdioServerTransport::with_config(default_config);
        assert!(!custom_transport.is_running());
    }

    #[test]
    fn test_stdio_transport_with_config() {
        // Test transport creation with custom configuration
        let config = TransportConfig {
            read_timeout_ms: Some(30_000),
            write_timeout_ms: Some(15_000),
            max_message_size: Some(1024 * 1024), // 1MB
            compression: true,
            ..Default::default()
        };

        let transport = StdioServerTransport::with_config(config.clone());

        // Verify transport was created successfully
        assert!(!transport.is_running());
        assert_eq!(
            transport.server_info(),
            "STDIO server transport (running: false)"
        );

        // Test that the config values are what we expect
        assert_eq!(config.read_timeout_ms, Some(30_000));
        assert_eq!(config.write_timeout_ms, Some(15_000));
        assert_eq!(config.max_message_size, Some(1024 * 1024));
        assert!(config.compression);
    }

    #[test]
    fn test_connection_state_enum() {
        // Test all ConnectionState variants and their properties
        let states = vec![
            ConnectionState::Disconnected,
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::Reconnecting,
            ConnectionState::Closing,
            ConnectionState::Error("test error".to_string()),
        ];

        // Test equality and pattern matching
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);

        // Test error state with different messages
        let error1 = ConnectionState::Error("network failure".to_string());
        let error2 = ConnectionState::Error("network failure".to_string());
        let error3 = ConnectionState::Error("timeout".to_string());

        assert_eq!(
            error1, error2,
            "Error states with same message should be equal"
        );
        assert_ne!(
            error1, error3,
            "Error states with different messages should not be equal"
        );

        // Test Debug formatting (ensure states can be debugged)
        for state in states {
            let debug_str = format!("{state:?}");
            assert!(!debug_str.is_empty(), "Debug output should not be empty");

            // Verify specific state formatting
            match state {
                ConnectionState::Error(ref msg) => {
                    assert!(
                        debug_str.contains(msg),
                        "Debug output should contain error message"
                    );
                }
                _ => {
                    // Other states should have their names in debug output
                    let state_name = match state {
                        ConnectionState::Disconnected => "Disconnected",
                        ConnectionState::Connecting => "Connecting",
                        ConnectionState::Connected => "Connected",
                        ConnectionState::Reconnecting => "Reconnecting",
                        ConnectionState::Closing => "Closing",
                        ConnectionState::Error(_) => unreachable!(),
                    };
                    assert!(debug_str.contains(state_name));
                }
            }
        }
    }

    #[test]
    fn test_transport_error_handling() {
        // Test McpError variants that are transport-related
        let transport_error = McpError::Transport("Connection failed".to_string());
        let timeout_error = McpError::Timeout("Request timed out".to_string());

        // Test error message extraction
        assert_eq!(
            transport_error.to_string(),
            "Transport error: Connection failed"
        );
        assert_eq!(
            timeout_error.to_string(),
            "Timeout error: Request timed out"
        );

        // Test error type matching
        match transport_error {
            McpError::Transport(ref msg) => {
                assert_eq!(msg, "Connection failed");
            }
            _ => panic!("Expected Transport error"),
        }

        match timeout_error {
            McpError::Timeout(ref msg) => {
                assert_eq!(msg, "Request timed out");
            }
            _ => panic!("Expected Timeout error"),
        }

        // Test error conversion to Result
        let result_with_transport_error: Result<(), McpError> = Err(transport_error);
        assert!(result_with_transport_error.is_err());

        let result_with_timeout_error: Result<(), McpError> = Err(timeout_error);
        assert!(result_with_timeout_error.is_err());
    }

    #[test]
    fn test_transport_config() {
        // Test TransportConfig creation and field access
        let config = TransportConfig {
            read_timeout_ms: Some(30_000),
            write_timeout_ms: Some(15_000),
            connect_timeout_ms: Some(10_000),
            max_message_size: Some(2 * 1024 * 1024), // 2MB
            keep_alive_ms: Some(60_000),
            compression: true,
            headers: std::collections::HashMap::from([
                ("Authorization".to_string(), "Bearer token123".to_string()),
                ("User-Agent".to_string(), "MCP-SDK/1.0".to_string()),
            ]),
        };

        // Verify all fields are set correctly
        assert_eq!(config.read_timeout_ms, Some(30_000));
        assert_eq!(config.write_timeout_ms, Some(15_000));
        assert_eq!(config.connect_timeout_ms, Some(10_000));
        assert_eq!(config.max_message_size, Some(2 * 1024 * 1024));
        assert_eq!(config.keep_alive_ms, Some(60_000));
        assert!(config.compression);
        assert_eq!(config.headers.len(), 2);
        assert_eq!(
            config.headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            config.headers.get("User-Agent"),
            Some(&"MCP-SDK/1.0".to_string())
        );

        // Test cloning
        let cloned_config = config.clone();
        assert_eq!(config.read_timeout_ms, cloned_config.read_timeout_ms);
        assert_eq!(config.headers.len(), cloned_config.headers.len());
    }

    #[test]
    fn test_transport_config_defaults() {
        // Test default configuration values
        let config = TransportConfig::default();

        // Verify all default values are sensible
        assert_eq!(
            config.connect_timeout_ms,
            Some(30_000),
            "Default connect timeout should be 30 seconds"
        );
        assert_eq!(
            config.read_timeout_ms,
            Some(60_000),
            "Default read timeout should be 60 seconds"
        );
        assert_eq!(
            config.write_timeout_ms,
            Some(30_000),
            "Default write timeout should be 30 seconds"
        );
        assert_eq!(
            config.max_message_size,
            Some(16 * 1024 * 1024),
            "Default max message size should be 16MB"
        );
        assert_eq!(
            config.keep_alive_ms,
            Some(30_000),
            "Default keep-alive should be 30 seconds"
        );
        assert!(
            !config.compression,
            "Compression should be disabled by default"
        );
        assert!(
            config.headers.is_empty(),
            "Headers should be empty by default"
        );

        // Test that we can modify the default config
        let mut modified_config = config;
        modified_config.compression = true;
        modified_config.read_timeout_ms = Some(45_000);

        assert!(modified_config.compression);
        assert_eq!(modified_config.read_timeout_ms, Some(45_000));
    }

    #[tokio::test]
    async fn test_stdio_server_transport_lifecycle() {
        // Test server transport lifecycle without actual I/O
        let mut transport = StdioServerTransport::new();

        // Initial state
        assert!(!transport.is_running());
        assert_eq!(
            transport.server_info(),
            "STDIO server transport (running: false)"
        );

        // Test stop when not running (should be safe)
        let stop_result = transport.stop().await;
        assert!(
            stop_result.is_ok(),
            "Stopping non-running transport should succeed"
        );
        assert!(!transport.is_running());
    }

    #[tokio::test]
    async fn test_stdio_server_handle_request() {
        // Test request handling with various scenarios
        let mut transport = StdioServerTransport::new();

        // Test with unknown method (default behavior)
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "unknown_method".to_string(),
            params: None,
        };

        let result = transport.handle_request(request).await;
        assert!(result.is_err(), "Unknown method should return error");

        match result.unwrap_err() {
            McpError::Protocol(msg) => {
                assert!(
                    msg.contains("unknown_method"),
                    "Error should mention the unknown method"
                );
                assert!(
                    msg.contains("not found"),
                    "Error should indicate method not found"
                );
            }
            _ => panic!("Expected Protocol error for unknown method"),
        }

        // Test with different method names
        let methods_to_test = vec!["nonexistent", "test_method", "capabilities"];

        for method in methods_to_test {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: json!(format!("test_{}", method)),
                method: method.to_string(),
                params: Some(json!({})),
            };

            let result = transport.handle_request(request).await;
            assert!(result.is_err(), "Method '{method}' should return error");
        }
    }

    #[test]
    fn test_reconnect_config() {
        // Test ReconnectConfig functionality
        let config = ReconnectConfig::default();

        // Verify default values
        assert!(config.enabled, "Reconnection should be enabled by default");
        assert_eq!(config.max_attempts, Some(5));
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30_000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.jitter_factor, 0.1);

        // Test custom configuration
        let custom_config = ReconnectConfig {
            enabled: false,
            max_attempts: Some(3),
            initial_delay_ms: 500,
            max_delay_ms: 15_000,
            backoff_multiplier: 1.5,
            jitter_factor: 0.2,
        };

        assert!(!custom_config.enabled);
        assert_eq!(custom_config.max_attempts, Some(3));
        assert_eq!(custom_config.initial_delay_ms, 500);
        assert_eq!(custom_config.max_delay_ms, 15_000);
        assert_eq!(custom_config.backoff_multiplier, 1.5);
        assert_eq!(custom_config.jitter_factor, 0.2);

        // Test unlimited attempts
        let unlimited_config = ReconnectConfig {
            max_attempts: None,
            ..Default::default()
        };
        assert_eq!(unlimited_config.max_attempts, None);
    }

    #[test]
    fn test_transport_stats() {
        // Test TransportStats functionality
        let mut stats = TransportStats::default();

        // Verify default values
        assert_eq!(stats.requests_sent, 0);
        assert_eq!(stats.responses_received, 0);
        assert_eq!(stats.notifications_sent, 0);
        assert_eq!(stats.notifications_received, 0);
        assert_eq!(stats.connection_errors, 0);
        assert_eq!(stats.protocol_errors, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.uptime_ms, 0);

        // Test manual modification (simulating real usage)
        stats.requests_sent = 10;
        stats.responses_received = 8;
        stats.notifications_sent = 5;
        stats.notifications_received = 3;
        stats.connection_errors = 1;
        stats.protocol_errors = 2;
        stats.bytes_sent = 1024;
        stats.bytes_received = 2048;
        stats.uptime_ms = 30_000;

        assert_eq!(stats.requests_sent, 10);
        assert_eq!(stats.responses_received, 8);
        assert_eq!(stats.notifications_sent, 5);
        assert_eq!(stats.notifications_received, 3);
        assert_eq!(stats.connection_errors, 1);
        assert_eq!(stats.protocol_errors, 2);
        assert_eq!(stats.bytes_sent, 1024);
        assert_eq!(stats.bytes_received, 2048);
        assert_eq!(stats.uptime_ms, 30_000);

        // Test cloning
        let cloned_stats = stats.clone();
        assert_eq!(stats.requests_sent, cloned_stats.requests_sent);
        assert_eq!(stats.bytes_sent, cloned_stats.bytes_sent);
    }

    #[test]
    fn test_json_rpc_message_construction() {
        // Test construction of JSON-RPC messages for transport testing

        // Test request construction
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(42),
            method: "test_method".to_string(),
            params: Some(json!({"key": "value"})),
        };

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, json!(42));
        assert_eq!(request.method, "test_method");
        assert!(request.params.is_some());

        // Test notification construction
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "notification_method".to_string(),
            params: Some(json!({"event": "something_happened"})),
        };

        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method, "notification_method");
        assert!(notification.params.is_some());

        // Test response construction
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: json!(42),
            result: Some(json!({"status": "success"})),
        };

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, json!(42));
        assert!(response.result.is_some());

        // Test serialization/deserialization
        let request_json = serde_json::to_string(&request).expect("Should serialize request");
        let parsed_request: JsonRpcRequest =
            serde_json::from_str(&request_json).expect("Should deserialize request");
        assert_eq!(parsed_request.method, "test_method");

        let notification_json =
            serde_json::to_string(&notification).expect("Should serialize notification");
        let parsed_notification: JsonRpcNotification =
            serde_json::from_str(&notification_json).expect("Should deserialize notification");
        assert_eq!(parsed_notification.method, "notification_method");
    }

    #[test]
    fn test_transport_config_edge_cases() {
        // Test edge cases and boundary conditions for TransportConfig

        // Test with all None values
        let minimal_config = TransportConfig {
            connect_timeout_ms: None,
            read_timeout_ms: None,
            write_timeout_ms: None,
            max_message_size: None,
            keep_alive_ms: None,
            compression: false,
            headers: std::collections::HashMap::new(),
        };

        assert!(minimal_config.connect_timeout_ms.is_none());
        assert!(minimal_config.read_timeout_ms.is_none());
        assert!(minimal_config.write_timeout_ms.is_none());
        assert!(minimal_config.max_message_size.is_none());
        assert!(minimal_config.keep_alive_ms.is_none());

        // Test with very large values
        let large_config = TransportConfig {
            connect_timeout_ms: Some(u64::MAX),
            read_timeout_ms: Some(u64::MAX),
            write_timeout_ms: Some(u64::MAX),
            max_message_size: Some(usize::MAX),
            keep_alive_ms: Some(u64::MAX),
            compression: true,
            headers: std::collections::HashMap::new(),
        };

        assert_eq!(large_config.connect_timeout_ms, Some(u64::MAX));
        assert_eq!(large_config.max_message_size, Some(usize::MAX));

        // Test with zero values
        let zero_config = TransportConfig {
            connect_timeout_ms: Some(0),
            read_timeout_ms: Some(0),
            write_timeout_ms: Some(0),
            max_message_size: Some(0),
            keep_alive_ms: Some(0),
            compression: false,
            headers: std::collections::HashMap::new(),
        };

        assert_eq!(zero_config.connect_timeout_ms, Some(0));
        assert_eq!(zero_config.max_message_size, Some(0));
    }

    #[test]
    fn test_connection_state_transitions() {
        // Test logical state transitions
        let states = vec![
            ConnectionState::Disconnected,
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::Reconnecting,
            ConnectionState::Closing,
            ConnectionState::Error("Connection lost".to_string()),
        ];

        // Test that we can transition between states logically
        let mut current_state = ConnectionState::Disconnected;
        assert_eq!(current_state, ConnectionState::Disconnected);

        // Disconnected -> Connecting
        current_state = ConnectionState::Connecting;
        assert_eq!(current_state, ConnectionState::Connecting);

        // Connecting -> Connected
        current_state = ConnectionState::Connected;
        assert_eq!(current_state, ConnectionState::Connected);

        // Connected -> Error
        current_state = ConnectionState::Error("Network timeout".to_string());
        assert!(matches!(current_state, ConnectionState::Error(_)));

        // Error -> Reconnecting
        current_state = ConnectionState::Reconnecting;
        assert_eq!(current_state, ConnectionState::Reconnecting);

        // Reconnecting -> Connected
        current_state = ConnectionState::Connected;
        assert_eq!(current_state, ConnectionState::Connected);

        // Connected -> Closing
        current_state = ConnectionState::Closing;
        assert_eq!(current_state, ConnectionState::Closing);

        // Closing -> Disconnected
        current_state = ConnectionState::Disconnected;
        assert_eq!(current_state, ConnectionState::Disconnected);

        // Test that all states can be pattern matched
        for state in states {
            match state {
                ConnectionState::Disconnected => assert_eq!(state, ConnectionState::Disconnected),
                ConnectionState::Connecting => assert_eq!(state, ConnectionState::Connecting),
                ConnectionState::Connected => assert_eq!(state, ConnectionState::Connected),
                ConnectionState::Reconnecting => assert_eq!(state, ConnectionState::Reconnecting),
                ConnectionState::Closing => assert_eq!(state, ConnectionState::Closing),
                ConnectionState::Error(ref msg) => {
                    assert!(!msg.is_empty(), "Error message should not be empty");
                    assert!(matches!(state, ConnectionState::Error(_)));
                }
            }
        }
    }
}

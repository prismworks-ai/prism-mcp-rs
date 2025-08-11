// ! Client builder for MCP clients
// !
// ! Provides a builder pattern for creating and configuring MCP clients.

use crate::client::McpClient;
use crate::core::error::McpResult;
use crate::protocol::types::ClientCapabilities;
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: Option<u32>,
    /// Initial delay between retry attempts (ms)
    pub initial_delay_ms: u64,
    /// Maximum delay between retry attempts (ms)
    pub max_delay_ms: u64,
    /// Backoff multiplier for retry delays
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: Some(3),
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Configuration for connections
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Whether to enable keep-alive
    pub keep_alive: bool,
    /// Whether to enable compression
    pub compression: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30000,
            keep_alive: true,
            compression: false,
        }
    }
}

/// Builder for creating MCP clients with configuration
pub struct McpClientBuilder {
    name: Option<String>,
    version: Option<String>,
    capabilities: Option<ClientCapabilities>,
    timeout: Option<Duration>,
    retry_config: Option<RetryConfig>,
    connection_config: Option<ConnectionConfig>,
}

impl McpClientBuilder {
    /// Create a new client builder
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            capabilities: None,
            timeout: None,
            retry_config: None,
            connection_config: None,
        }
    }

    /// Set client name
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set client version
    pub fn with_version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set client capabilities
    pub fn with_capabilities(mut self, capabilities: ClientCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set retry configuration
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = Some(retry_config);
        self
    }

    /// Set connection configuration
    pub fn with_connection_config(mut self, connection_config: ConnectionConfig) -> Self {
        self.connection_config = Some(connection_config);
        self
    }

    /// Build the client
    pub fn build(self) -> McpResult<McpClient> {
        let mut client = McpClient::new(
            self.name.unwrap_or_else(|| "mcp-client".to_string()),
            self.version.unwrap_or_else(|| "1.0.0".to_string()),
        );

        client.set_capabilities(self.capabilities.unwrap_or_default());

        Ok(client)
    }
}

impl Default for McpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Legacy alias for compatibility - single definition only
pub type ClientBuilder = McpClientBuilder;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::SamplingCapability;

    #[test]
    fn test_client_builder_default() {
        let builder = McpClientBuilder::new();
        assert!(builder.name.is_none());
        assert!(builder.version.is_none());
        assert!(builder.capabilities.is_none());
        assert!(builder.retry_config.is_none());
        assert!(builder.connection_config.is_none());
    }

    #[test]
    fn test_client_builder_configuration() {
        let builder = McpClientBuilder::new()
            .with_name("test-client".to_string())
            .with_version("2.0.0".to_string())
            .with_capabilities(ClientCapabilities {
                sampling: Some(SamplingCapability::default()),
                experimental: Some(std::collections::HashMap::new()),
                ..Default::default()
            });

        assert_eq!(builder.name, Some("test-client".to_string()));
        assert_eq!(builder.version, Some("2.0.0".to_string()));
        assert!(builder.capabilities.is_some());
    }

    #[test]
    fn test_client_builder_with_retry_config() {
        let retry_config = RetryConfig {
            max_attempts: Some(5),
            initial_delay_ms: 500,
            max_delay_ms: 60000,
            backoff_multiplier: 1.5,
        };

        let builder = McpClientBuilder::new().with_retry_config(retry_config.clone());

        assert!(builder.retry_config.is_some());
        let config = builder.retry_config.unwrap();
        assert_eq!(config.max_attempts, Some(5));
        assert_eq!(config.initial_delay_ms, 500);
        assert_eq!(config.max_delay_ms, 60000);
        assert!((config.backoff_multiplier - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_client_builder_with_connection_config() {
        let connection_config = ConnectionConfig {
            timeout_ms: 45000,
            keep_alive: false,
            compression: true,
        };

        let builder = McpClientBuilder::new().with_connection_config(connection_config.clone());

        assert!(builder.connection_config.is_some());
        let config = builder.connection_config.unwrap();
        assert_eq!(config.timeout_ms, 45000);
        assert!(!config.keep_alive);
        assert!(config.compression);
    }

    #[test]
    fn test_client_builder_build_with_defaults() {
        let _client = McpClientBuilder::new().build().unwrap();
        // The client should be built successfully with default values
        // We can't easily inspect internal state, but build() should succeed
    }

    #[test]
    fn test_client_builder_build_with_full_config() {
        let retry_config = RetryConfig {
            max_attempts: Some(3),
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        };

        let connection_config = ConnectionConfig {
            timeout_ms: 30000,
            keep_alive: true,
            compression: false,
        };

        let _client = McpClientBuilder::new()
            .with_name("test-client".to_string())
            .with_version("1.0.0".to_string())
            .with_capabilities(ClientCapabilities::default())
            .with_retry_config(retry_config)
            .with_connection_config(connection_config)
            .build()
            .unwrap();

        // Build should succeed with full configuration
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, Some(3));
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_connection_config_default() {
        let config = ConnectionConfig::default();
        assert_eq!(config.timeout_ms, 30000);
        assert!(config.keep_alive);
        assert!(!config.compression);
    }

    #[test]
    fn test_client_builder_default_trait() {
        let builder1 = McpClientBuilder::default();
        let builder2 = McpClientBuilder::new();

        // Both should have the same default state
        assert_eq!(builder1.name, builder2.name);
        assert_eq!(builder1.version, builder2.version);
    }

    #[test]
    fn test_client_builder_method_chaining() {
        // Test that all methods return Self for chaining
        let _builder = McpClientBuilder::new()
            .with_name("test".to_string())
            .with_version("1.0".to_string())
            .with_capabilities(ClientCapabilities::default())
            .with_retry_config(RetryConfig::default())
            .with_connection_config(ConnectionConfig::default());

        // If this compiles, method chaining works correctly
    }

    #[test]
    fn test_legacy_client_builder_alias() {
        // Test that the legacy alias works
        let _builder: ClientBuilder = McpClientBuilder::new();
        // If this compiles, the type alias is working
    }

    #[test]
    fn test_retry_config_clone_debug() {
        let config = RetryConfig::default();
        let cloned = config.clone();
        assert_eq!(config.max_attempts, cloned.max_attempts);

        // Test that Debug is implemented
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("RetryConfig"));
    }

    #[test]
    fn test_connection_config_clone_debug() {
        let config = ConnectionConfig::default();
        let cloned = config.clone();
        assert_eq!(config.timeout_ms, cloned.timeout_ms);

        // Test that Debug is implemented
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("ConnectionConfig"));
    }
}

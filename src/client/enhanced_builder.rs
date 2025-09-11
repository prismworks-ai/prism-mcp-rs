//! Enhanced client builder with fluent interface
//! Consolidates all client creation into single builder pattern

use crate::client::McpClient;
use crate::core::enhanced_errors::{McpError, McpResult};
use crate::protocol::types::{ClientCapabilities, ClientInfo};
// use std::collections::HashMap; // Not needed
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: Option<u32>,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
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
    pub timeout_ms: u64,
    pub keep_alive: bool,
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

/// Enhanced client builder with fluent interface
#[derive(Debug)]
pub struct McpClientBuilder {
    name: Option<String>,
    version: Option<String>,
    capabilities: Option<ClientCapabilities>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    validate_requests: Option<bool>,
    validate_responses: Option<bool>,
}

impl McpClientBuilder {
    /// Create a new client builder
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            capabilities: None,
            timeout: None,
            max_retries: None,
            validate_requests: None,
            validate_responses: None,
        }
    }

    /// Set client name (flexible input types)
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set client version (flexible input types)
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set client capabilities
    pub fn capabilities(mut self, capabilities: ClientCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Set request timeout with type safety
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set maximum retry attempts
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Enable or disable request validation
    pub fn validate_requests(mut self, validate: bool) -> Self {
        self.validate_requests = Some(validate);
        self
    }

    /// Enable or disable response validation
    pub fn validate_responses(mut self, validate: bool) -> Self {
        self.validate_responses = Some(validate);
        self
    }

    /// Build the client with validation
    pub fn build(self) -> McpResult<McpClient> {
        let name = self
            .name
            .ok_or_else(|| McpError::validation("Client name is required"))?;
        let version = self.version.unwrap_or_else(|| "1.0.0".to_string());

        let client_info = ClientInfo::new(name, version);
        let capabilities = self.capabilities.unwrap_or_default();

        let config = crate::client::mcp_client::ClientConfig {
            request_timeout_ms: self.timeout.map(|d| d.as_millis() as u64).unwrap_or(30000),
            max_retries: self.max_retries.unwrap_or(3),
            retry_delay_ms: 1000,
            validate_requests: self.validate_requests.unwrap_or(true),
            validate_responses: self.validate_responses.unwrap_or(true),
        };

        Ok(McpClient::from_parts(client_info, capabilities, config))
    }
}

impl Default for McpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenient constructors for common cases
impl McpClientBuilder {
    /// Quick builder for development/testing
    pub fn development(name: impl Into<String>) -> Self {
        Self::new()
            .name(name)
            .version("dev")
            .timeout(Duration::from_secs(10))
            .max_retries(1)
    }

    /// Production-ready builder with safe defaults
    pub fn production(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self::new()
            .name(name)
            .version(version)
            .timeout(Duration::from_secs(30))
            .max_retries(3)
            .validate_requests(true)
            .validate_responses(true)
    }
}

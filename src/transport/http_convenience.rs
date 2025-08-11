// ! Production-grade convenience methods for HttpClientTransport
// !
// ! Module extends the basic HttpClientTransport with high-level convenience
// ! methods expected in a production SDK.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::core::error::{McpError, McpResult};
use crate::protocol::types::{JsonRpcRequest, JsonRpcResponse};
use crate::transport::http::HttpClientTransport;
use crate::transport::traits::{Transport, TransportConfig};

// ============================================================================
// Additional Types for Convenience Methods
// ============================================================================

/// Server information returned by get_server_info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Supported protocol version
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: HashMap<String, Value>,
    /// Additional server metadata
    pub metadata: HashMap<String, Value>,
}

/// Connection statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Number of requests sent
    pub requests_sent: u64,
    /// Number of successful responses
    pub responses_received: u64,
    /// Number of failed requests
    pub request_failures: u64,
    /// Number of notifications sent
    pub notifications_sent: u64,
    /// Number of notifications received
    pub notifications_received: u64,
    /// Total connection uptime
    pub uptime: Duration,
    /// Connection start time
    pub connected_at: Option<Instant>,
    /// Last successful request time
    pub last_success_at: Option<Instant>,
    /// Last error time
    pub last_error_at: Option<Instant>,
    /// Average response time
    pub avg_response_time: Duration,
    /// Number of reconnection attempts
    pub reconnect_attempts: u64,
}

/// HTTP endpoint URLs
#[derive(Debug, Clone)]
pub struct HttpEndpoints {
    /// Main MCP endpoint
    pub mcp: String,
    /// Notification endpoint
    pub notify: String,
    /// SSE events endpoint
    pub events: Option<String>,
    /// Health check endpoint
    pub health: String,
}

/// Retry configuration for resilient requests
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Whether to retry on timeout errors
    pub retry_on_timeout: bool,
    /// Whether to retry on connection errors
    pub retry_on_connection: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            retry_on_timeout: true,
            retry_on_connection: true,
        }
    }
}

/// Retry policy for automatic retries
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Default retry configuration
    pub default: RetryConfig,
    /// Method-specific retry configurations
    pub method_specific: HashMap<String, RetryConfig>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            default: RetryConfig::default(),
            method_specific: HashMap::new(),
        }
    }
}

/// Transport metrics for observability
#[derive(Debug, Clone, Default)]
pub struct TransportMetrics {
    /// Connection statistics
    pub connection_stats: ConnectionStats,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Error metrics
    pub errors: ErrorMetrics,
}

/// Performance metrics
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Average request latency
    pub avg_latency: Duration,
    /// 95th percentile latency
    pub p95_latency: Duration,
    /// 99th percentile latency
    pub p99_latency: Duration,
    /// Requests per second
    pub requests_per_second: f64,
    /// Throughput in bytes per second
    pub throughput_bps: f64,
}

/// Error metrics
#[derive(Debug, Clone, Default)]
pub struct ErrorMetrics {
    /// Total error count
    pub total_errors: u64,
    /// Timeout errors
    pub timeout_errors: u64,
    /// Connection errors
    pub connection_errors: u64,
    /// Protocol errors
    pub protocol_errors: u64,
    /// HTTP errors by status code
    pub http_errors: HashMap<u16, u64>,
}

// ============================================================================
// improved HttpClientTransport with Convenience Methods
// ============================================================================

/// Extended functionality for HttpClientTransport
pub struct HttpClientTransportExtensions {
    /// Connection statistics
    stats: Arc<Mutex<ConnectionStats>>,
    /// Request ID counter
    request_counter: Arc<AtomicU64>,
    /// Retry policy
    retry_policy: Arc<Mutex<RetryPolicy>>,
    /// Request logging enabled
    request_logging: Arc<Mutex<bool>>,
    /// Last error
    last_error: Arc<Mutex<Option<McpError>>>,
    /// Response time tracker
    response_times: Arc<Mutex<Vec<Duration>>>,
}

impl Default for HttpClientTransportExtensions {
    fn default() -> Self {
        Self {
            stats: Arc::new(Mutex::new(ConnectionStats::default())),
            request_counter: Arc::new(AtomicU64::new(0)),
            retry_policy: Arc::new(Mutex::new(RetryPolicy::default())),
            request_logging: Arc::new(Mutex::new(false)),
            last_error: Arc::new(Mutex::new(None)),
            response_times: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Production-grade convenience methods for HttpClientTransport
impl HttpClientTransport {
    // ============================================================================
    // 1. Connection Health & Status
    // ============================================================================

    /// Send a health check request and measure response time
    pub async fn ping(&mut self) -> McpResult<Duration> {
        let start = Instant::now();

        let health_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "ping".to_string(),
            params: Some(Value::Object(serde_json::Map::new())),
            id: Value::from(self.next_request_id().await),
        };

        match self.send_request(health_request).await {
            Ok(_) => {
                let duration = start.elapsed();
                Ok(duration)
            }
            Err(_e) => {
                // Try the health endpoint as fallback
                let url = format!("{}/health", self.base_url);
                let _response = self
                    .client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| McpError::Http(format!("Health check failed: {e}")))?
                    .error_for_status()
                    .map_err(|e| McpError::Http(format!("Health check failed: {e}")))?;

                let duration = start.elapsed();
                Ok(duration)
            }
        }
    }

    /// Retrieve server capabilities and version information
    pub async fn get_server_info(&mut self) -> McpResult<ServerInfo> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {
                    "name": "prism-mcp-rs",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            id: Value::from(self.next_request_id().await),
        };

        let response = self.send_request(request).await?;

        if let Some(result) = response.result {
            let server_info = ServerInfo {
                name: result
                    .get("serverInfo")
                    .and_then(|info| info.get("name"))
                    .and_then(|name| name.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                version: result
                    .get("serverInfo")
                    .and_then(|info| info.get("version"))
                    .and_then(|version| version.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                protocol_version: result
                    .get("protocolVersion")
                    .and_then(|version| version.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                capabilities: result
                    .get("capabilities")
                    .and_then(|caps| caps.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
                metadata: result
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            };
            Ok(server_info)
        } else {
            Err(McpError::Protocol(
                "Invalid server info response".to_string(),
            ))
        }
    }

    /// Get connection statistics for monitoring
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        // This would require extending HttpClientTransport with statistics tracking
        // For now, return basic stats
        ConnectionStats {
            requests_sent: 0, // Would be tracked in actual implementation
            responses_received: 0,
            request_failures: 0,
            notifications_sent: 0,
            notifications_received: 0,
            uptime: Duration::from_secs(0),
            connected_at: Some(Instant::now()),
            last_success_at: None,
            last_error_at: None,
            avg_response_time: Duration::from_millis(0),
            reconnect_attempts: 0,
        }
    }

    /// Quick health check based on recent activity
    pub fn is_healthy(&self) -> bool {
        self.is_connected()
    }

    // ============================================================================
    // 2. Type-Safe Request Builders & Shortcuts
    // ============================================================================

    /// Type-safe method calling with automatic serialization
    pub async fn call_method<T: Serialize, R: for<'de> Deserialize<'de>>(
        &mut self,
        method: &str,
        params: T,
    ) -> McpResult<R> {
        let request =
            JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params: Some(serde_json::to_value(params).map_err(|e| {
                    McpError::Protocol(format!("Failed to serialize parameters: {e}"))
                })?),
                id: Value::from(self.next_request_id().await),
            };

        let response = self.send_request(request).await?;

        if let Some(result) = response.result {
            serde_json::from_value(result)
                .map_err(|e| McpError::Protocol(format!("Failed to deserialize response: {e}")))
        } else {
            Err(McpError::Protocol("Missing result in response".to_string()))
        }
    }

    /// Simple method call without parameters
    pub async fn call_method_simple(&mut self, method: &str) -> McpResult<Value> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: None,
            id: Value::from(self.next_request_id().await),
        };

        let response = self.send_request(request).await?;
        response
            .result
            .ok_or_else(|| McpError::Protocol("Missing result in response".to_string()))
    }

    /// Send multiple requests efficiently
    pub async fn batch_requests(
        &mut self,
        requests: Vec<JsonRpcRequest>,
    ) -> McpResult<Vec<JsonRpcResponse>> {
        // For HTTP transport, we send requests sequentially
        // A more complete implementation could use HTTP/2 multiplexing
        let mut responses = Vec::with_capacity(requests.len());

        for request in requests {
            let response = self.send_request(request).await?;
            responses.push(response);
        }

        Ok(responses)
    }

    // ============================================================================
    // 3. Connection Management
    // ============================================================================

    /// Reconnect with same configuration
    pub async fn reconnect(&mut self) -> McpResult<()> {
        // Close current connection
        self.close().await?;

        // Create new transport with same configuration
        let new_transport =
            Self::with_config(&self.base_url, self.sse_url.as_ref(), self.config.clone()).await?;

        // Replace current transport state
        *self = new_transport;

        Ok(())
    }

    /// Test if connection is working without modifying state
    pub async fn test_connection(&self) -> McpResult<bool> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    // ============================================================================
    // 4. Configuration Updates
    // ============================================================================

    /// Update headers without recreating transport
    pub fn update_headers(&mut self, new_headers: HashMap<String, String>) {
        for (key, value) in new_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                key.parse::<axum::http::HeaderName>(),
                value.parse::<axum::http::HeaderValue>(),
            ) {
                self.headers.insert(header_name, header_value);
            }
        }
    }

    /// Update timeout configuration
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.config.read_timeout_ms = Some(timeout_ms);
        self.config.write_timeout_ms = Some(timeout_ms);
    }

    /// Access current configuration
    pub fn get_config(&self) -> &TransportConfig {
        &self.config
    }

    // ============================================================================
    // 5. URL and Endpoint Management
    // ============================================================================

    /// Get current base URL
    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    /// Get current SSE URL
    pub fn get_sse_url(&self) -> Option<&str> {
        self.sse_url.as_deref()
    }

    /// Get all endpoint URLs
    pub fn get_endpoints(&self) -> HttpEndpoints {
        HttpEndpoints {
            mcp: format!("{}/mcp", self.base_url),
            notify: format!("{}/mcp/notify", self.base_url),
            events: self.sse_url.clone(),
            health: format!("{}/health", self.base_url),
        }
    }

    // ============================================================================
    // 6. Retry and Resilience
    // ============================================================================

    /// Method call with automatic retry logic
    pub async fn call_with_retry<T: Serialize + Clone, R: for<'de> Deserialize<'de>>(
        &mut self,
        method: &str,
        params: T,
        retry_config: RetryConfig,
    ) -> McpResult<R> {
        let mut last_error = None;
        let mut delay = retry_config.initial_delay;

        for attempt in 0..=retry_config.max_attempts {
            match self.call_method(method, params.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e.clone());

                    // Don't retry on the last attempt
                    if attempt == retry_config.max_attempts {
                        break;
                    }

                    // Check if we should retry this error
                    let should_retry = match &e {
                        McpError::Timeout(_) => retry_config.retry_on_timeout,
                        McpError::Connection(_) => retry_config.retry_on_connection,
                        McpError::Http(_) => retry_config.retry_on_connection,
                        _ => false,
                    };

                    if !should_retry {
                        break;
                    }

                    // Wait before retry
                    tokio::time::sleep(delay).await;

                    // Exponential backoff
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * retry_config.backoff_multiplier) as u64,
                        ),
                        retry_config.max_delay,
                    );
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| McpError::Protocol("Retry failed without error".to_string())))
    }

    /// Set retry policy for automatic retries (would require state extension)
    pub fn set_retry_policy(&mut self, _policy: RetryPolicy) {
        // Implementation would require extending HttpClientTransport with retry state
        // For now, this is a placeholder
    }

    // ============================================================================
    // 7. Debugging and Observability
    // ============================================================================

    /// Enable/disable request/response logging (placeholder)
    pub fn enable_request_logging(&mut self, _enabled: bool) {
        // Implementation would require extending HttpClientTransport with logging state
        // For now, this is a placeholder
    }

    /// Get the last error that occurred (placeholder)
    pub fn get_last_error(&self) -> Option<&McpError> {
        // Implementation would require extending HttpClientTransport with error tracking
        // For now, this is a placeholder
        None
    }

    /// Export detailed metrics for monitoring (placeholder)
    pub async fn export_metrics(&self) -> McpResult<TransportMetrics> {
        Ok(TransportMetrics {
            connection_stats: self.get_connection_stats().await,
            performance: PerformanceMetrics::default(),
            errors: ErrorMetrics::default(),
        })
    }
}

// ============================================================================
// 6. Builder Pattern Support
// ============================================================================

/// Fluent builder for HttpClientTransport configuration
pub struct HttpClientTransportBuilder {
    base_url: Option<String>,
    sse_url: Option<String>,
    config: TransportConfig,
}

impl HttpClientTransport {
    /// Create a new builder for HttpClientTransport
    pub fn builder() -> HttpClientTransportBuilder {
        HttpClientTransportBuilder {
            base_url: None,
            sse_url: None,
            config: TransportConfig::default(),
        }
    }
}

impl HttpClientTransportBuilder {
    /// Set the base URL
    pub fn base_url<S: Into<String>>(mut self, url: S) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the SSE URL for notifications
    pub fn sse_url<S: Into<String>>(mut self, url: S) -> Self {
        self.sse_url = Some(url.into());
        self
    }

    /// Set request timeout
    pub fn timeout(mut self, ms: u64) -> Self {
        self.config.read_timeout_ms = Some(ms);
        self.config.write_timeout_ms = Some(ms);
        self
    }

    /// Add a custom header
    pub fn header<S: Into<String>>(mut self, key: S, value: S) -> Self {
        self.config.headers.insert(key.into(), value.into());
        self
    }

    /// Enable or disable compression
    pub fn compression(mut self, enabled: bool) -> Self {
        self.config.compression = enabled;
        self
    }

    /// Set connection timeout
    pub fn connect_timeout(mut self, ms: u64) -> Self {
        self.config.connect_timeout_ms = Some(ms);
        self
    }

    /// Set maximum message size
    pub fn max_message_size(mut self, size: usize) -> Self {
        self.config.max_message_size = Some(size);
        self
    }

    /// Build the HttpClientTransport
    pub async fn build(self) -> McpResult<HttpClientTransport> {
        let base_url = self
            .base_url
            .ok_or_else(|| McpError::protocol("Base URL is required"))?;

        HttpClientTransport::with_config(base_url, self.sse_url.clone(), self.config).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder_pattern() {
        let result = HttpClientTransport::builder()
            .base_url("http://localhost:3000")
            .sse_url("http://localhost:3000/events")
            .timeout(30_000)
            .header("Authorization", "Bearer token")
            .compression(true)
            .build()
            .await;

        assert!(result.is_ok());
        let transport = result.unwrap();
        assert_eq!(transport.get_base_url(), "http://localhost:3000");
        assert_eq!(
            transport.get_sse_url(),
            Some("http://localhost:3000/events")
        );
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.retry_on_timeout);
        assert!(config.retry_on_connection);
    }

    #[test]
    fn test_http_endpoints() {
        let base_url = "http://localhost:3000";
        let endpoints = HttpEndpoints {
            mcp: format!("{}/mcp", base_url),
            notify: format!("{}/mcp/notify", base_url),
            events: Some("http://localhost:3000/events".to_string()),
            health: format!("{}/health", base_url),
        };

        assert_eq!(endpoints.mcp, "http://localhost:3000/mcp");
        assert_eq!(endpoints.notify, "http://localhost:3000/mcp/notify");
        assert_eq!(endpoints.health, "http://localhost:3000/health");
    }

    #[test]
    fn test_connection_stats_default() {
        let stats = ConnectionStats::default();
        assert_eq!(stats.requests_sent, 0);
        assert_eq!(stats.responses_received, 0);
        assert_eq!(stats.uptime, Duration::from_secs(0));
    }
}

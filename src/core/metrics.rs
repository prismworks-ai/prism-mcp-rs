// ! Metrics collection for the MCP SDK
// !
// ! Module provides structured metrics collection for error tracking,
// ! performance monitoring, and operational insights.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::core::error::McpError;

/// Metrics collector for MCP operations
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    /// Error counters by category and recoverability
    error_counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    /// Request counters by method
    request_counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    /// Connection attempt counters
    connection_counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    /// Retry attempt counters
    retry_counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            error_counters: Arc::new(RwLock::new(HashMap::new())),
            request_counters: Arc::new(RwLock::new(HashMap::new())),
            connection_counters: Arc::new(RwLock::new(HashMap::new())),
            retry_counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record an error occurrence
    pub async fn record_error(&self, error: &McpError, context: &str) {
        let category = error.category();
        let recoverable = error.is_recoverable();

        // Create metric key
        let key = format!(
            "mcp_errors_total:category={category}:recoverable={recoverable}:context={context}"
        );

        self.increment_counter(&self.error_counters, &key).await;

        // Also record by category only
        let category_key = format!("mcp_errors_by_category:{category}");
        self.increment_counter(&self.error_counters, &category_key)
            .await;

        // Log the metric for external systems
        info!(
            target: "mcp_metrics",
            error_category = category,
            error_recoverable = recoverable,
            error_context = context,
            "Error recorded in metrics"
        );
    }

    /// Record a request
    pub async fn record_request(&self, method: &str, transport: &str) {
        let key = format!("mcp_requests_total:method={method}:transport={transport}");
        self.increment_counter(&self.request_counters, &key).await;

        info!(
            target: "mcp_metrics",
            request_method = method,
            transport_type = transport,
            "Request recorded in metrics"
        );
    }

    /// Record a connection attempt
    pub async fn record_connection_attempt(&self, transport: &str, success: bool) {
        let key = format!("mcp_connections_total:transport={transport}:success={success}");
        self.increment_counter(&self.connection_counters, &key)
            .await;

        info!(
            target: "mcp_metrics",
            transport_type = transport,
            connection_success = success,
            "Connection attempt recorded in metrics"
        );
    }

    /// Record a retry attempt
    pub async fn record_retry_attempt(
        &self,
        operation: &str,
        attempt: u32,
        error_category: &str,
        will_retry: bool,
    ) {
        let key = format!(
            "mcp_retries_total:operation={operation}:attempt={attempt}:error_category={error_category}:will_retry={will_retry}"
        );
        self.increment_counter(&self.retry_counters, &key).await;

        info!(
            target: "mcp_metrics",
            retry_operation = operation,
            retry_attempt = attempt,
            error_category = error_category,
            will_retry_again = will_retry,
            "Retry attempt recorded in metrics"
        );
    }

    /// Get current error metrics
    pub async fn get_error_metrics(&self) -> HashMap<String, u64> {
        let counters = self.error_counters.read().await;
        counters
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    /// Get current request metrics
    pub async fn get_request_metrics(&self) -> HashMap<String, u64> {
        let counters = self.request_counters.read().await;
        counters
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    /// Get current connection metrics
    pub async fn get_connection_metrics(&self) -> HashMap<String, u64> {
        let counters = self.connection_counters.read().await;
        counters
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    /// Get current retry metrics
    pub async fn get_retry_metrics(&self) -> HashMap<String, u64> {
        let counters = self.retry_counters.read().await;
        counters
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> MetricsSummary {
        MetricsSummary {
            errors: self.get_error_metrics().await,
            requests: self.get_request_metrics().await,
            connections: self.get_connection_metrics().await,
            retries: self.get_retry_metrics().await,
        }
    }

    /// Reset all metrics (useful for testing)
    pub async fn reset(&self) {
        self.error_counters.write().await.clear();
        self.request_counters.write().await.clear();
        self.connection_counters.write().await.clear();
        self.retry_counters.write().await.clear();

        warn!(target: "mcp_metrics", "Metrics collector reset");
    }

    /// Internal helper to increment a counter
    async fn increment_counter(
        &self,
        counters: &Arc<RwLock<HashMap<String, AtomicU64>>>,
        key: &str,
    ) {
        let mut counters_guard = counters.write().await;
        let counter = counters_guard
            .entry(key.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }
}

/// Summary of all metrics
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub errors: HashMap<String, u64>,
    pub requests: HashMap<String, u64>,
    pub connections: HashMap<String, u64>,
    pub retries: HashMap<String, u64>,
}

/// Global metrics collector instance
static GLOBAL_METRICS: once_cell::sync::Lazy<MetricsCollector> =
    once_cell::sync::Lazy::new(MetricsCollector::new);

/// Get the global metrics collector
pub fn global_metrics() -> &'static MetricsCollector {
    &GLOBAL_METRICS
}

/// Helper macro for recording errors with metrics
#[macro_export]
macro_rules! record_error_metric {
    ($error:expr, $context:expr) => {
        let metrics = $crate::core::metrics::global_metrics();
        metrics.record_error($error, $context).await;
    };
}

/// Helper macro for recording requests with metrics
#[macro_export]
macro_rules! record_request_metric {
    ($method:expr, $transport:expr) => {
        let metrics = $crate::core::metrics::global_metrics();
        metrics.record_request($method, $transport).await;
    };
}

/// Helper macro for recording connection attempts with metrics
#[macro_export]
macro_rules! record_connection_metric {
    ($transport:expr, $success:expr) => {
        let metrics = $crate::core::metrics::global_metrics();
        metrics
            .record_connection_attempt($transport, $success)
            .await;
    };
}

/// Helper macro for recording retry attempts with metrics
#[macro_export]
macro_rules! record_retry_metric {
    ($operation:expr, $attempt:expr, $error_category:expr, $will_retry:expr) => {
        let metrics = $crate::core::metrics::global_metrics();
        metrics
            .record_retry_attempt($operation, $attempt, $error_category, $will_retry)
            .await;
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::McpError;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let metrics = MetricsCollector::new();
        let summary = metrics.get_all_metrics().await;

        assert!(summary.errors.is_empty());
        assert!(summary.requests.is_empty());
        assert!(summary.connections.is_empty());
        assert!(summary.retries.is_empty());
    }

    #[tokio::test]
    async fn test_error_recording() {
        let metrics = MetricsCollector::new();
        let error = McpError::connection("test error");

        metrics.record_error(&error, "test_context").await;

        let error_metrics = metrics.get_error_metrics().await;
        assert!(!error_metrics.is_empty());

        // Should have both specific and category metrics
        let has_specific = error_metrics.keys().any(|k| k.contains("test_context"));
        let has_category = error_metrics.keys().any(|k| k.contains("connection"));

        assert!(has_specific);
        assert!(has_category);
    }

    #[tokio::test]
    async fn test_request_recording() {
        let metrics = MetricsCollector::new();

        metrics.record_request("tools/list", "http").await;

        let request_metrics = metrics.get_request_metrics().await;
        assert!(!request_metrics.is_empty());

        let key_exists = request_metrics
            .keys()
            .any(|k| k.contains("tools/list") && k.contains("http"));
        assert!(key_exists);
    }

    #[tokio::test]
    async fn test_connection_recording() {
        let metrics = MetricsCollector::new();

        metrics.record_connection_attempt("websocket", true).await;
        metrics.record_connection_attempt("websocket", false).await;

        let connection_metrics = metrics.get_connection_metrics().await;
        assert!(!connection_metrics.is_empty());

        let success_key_exists = connection_metrics
            .keys()
            .any(|k| k.contains("websocket") && k.contains("success=true"));
        let failure_key_exists = connection_metrics
            .keys()
            .any(|k| k.contains("websocket") && k.contains("success=false"));

        assert!(success_key_exists);
        assert!(failure_key_exists);
    }

    #[tokio::test]
    async fn test_retry_recording() {
        let metrics = MetricsCollector::new();

        metrics
            .record_retry_attempt("send_request", 1, "connection", true)
            .await;

        let retry_metrics = metrics.get_retry_metrics().await;
        assert!(!retry_metrics.is_empty());

        let key_exists = retry_metrics.keys().any(|k| {
            k.contains("send_request") && k.contains("attempt=1") && k.contains("connection")
        });
        assert!(key_exists);
    }

    #[tokio::test]
    async fn test_metrics_reset() {
        let metrics = MetricsCollector::new();
        let error = McpError::timeout("test");

        metrics.record_error(&error, "test").await;
        assert!(!metrics.get_error_metrics().await.is_empty());

        metrics.reset().await;
        assert!(metrics.get_error_metrics().await.is_empty());
    }

    #[tokio::test]
    async fn test_global_metrics() {
        let metrics = global_metrics();
        let error = McpError::validation("test global metrics");

        metrics.record_error(&error, "global_test").await;

        let error_metrics = metrics.get_error_metrics().await;
        let has_global_test = error_metrics.keys().any(|k| k.contains("global_test"));
        assert!(has_global_test);
    }
}

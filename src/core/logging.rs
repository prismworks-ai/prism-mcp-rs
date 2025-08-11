// ! Structured logging for the MCP SDK
// !
// ! Module provides structured error logging with categorization,
// ! context preservation, and integration with the metrics system.

use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{Level, error, info, span, warn};

use crate::core::error::McpError;
use crate::core::metrics::global_metrics;

/// Log level for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorLogLevel {
    /// Critical errors that require immediate attention
    Critical,
    /// Errors that affect functionality but system can continue
    Error,
    /// Warnings about potential issues
    Warning,
    /// Informational error context
    Info,
}

impl From<&McpError> for ErrorLogLevel {
    fn from(error: &McpError) -> Self {
        match error {
            // Critical system errors
            McpError::Internal(_) => ErrorLogLevel::Critical,

            // Errors that break functionality
            McpError::Transport(_)
            | McpError::Protocol(_)
            | McpError::Serialization(_)
            | McpError::Authentication(_) => ErrorLogLevel::Error,

            // Recoverable errors
            McpError::Connection(_) | McpError::Timeout(_) | McpError::Io(_) => {
                ErrorLogLevel::Warning
            }

            // Client errors (user/input issues)
            McpError::Validation(_)
            | McpError::ToolNotFound(_)
            | McpError::ResourceNotFound(_)
            | McpError::PromptNotFound(_)
            | McpError::MethodNotFound(_)
            | McpError::InvalidParams(_)
            | McpError::InvalidUri(_)
            | McpError::Url(_) => ErrorLogLevel::Info,

            // Transport-specific errors
            #[cfg(feature = "http")]
            McpError::Http(_) => ErrorLogLevel::Warning,

            #[cfg(feature = "websocket")]
            McpError::WebSocket(_) => ErrorLogLevel::Warning,

            #[cfg(feature = "validation")]
            McpError::SchemaValidation(_) => ErrorLogLevel::Info,

            // Cancellation is informational
            McpError::Auth(_) => ErrorLogLevel::Warning,
            McpError::Cancelled(_) => ErrorLogLevel::Info,
        }
    }
}

/// Extended error context for logging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Operation being performed when error occurred
    pub operation: String,
    /// Transport type (stdio, http, websocket)
    pub transport: Option<String>,
    /// Request method if applicable
    pub method: Option<String>,
    /// Client/server identifier
    pub component: Option<String>,
    /// Session or connection ID
    pub session_id: Option<String>,
    /// Additional context data
    pub extra: HashMap<String, Value>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            operation: "unknown".to_string(),
            transport: None,
            method: None,
            component: None,
            session_id: None,
            extra: HashMap::new(),
        }
    }
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            ..Default::default()
        }
    }

    /// Set transport type
    pub fn with_transport(mut self, transport: impl Into<String>) -> Self {
        self.transport = Some(transport.into());
        self
    }

    /// Set method name
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Set component identifier
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add extra context data
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// improved error logging with metrics integration
pub struct ErrorLogger;

impl ErrorLogger {
    /// Log an error with full context and metrics
    pub async fn log_error(error: &McpError, context: ErrorContext) {
        let category = error.category();
        let recoverable = error.is_recoverable();
        let log_level = ErrorLogLevel::from(error);

        // Record metrics
        let metrics = global_metrics();
        metrics.record_error(error, &context.operation).await;

        // Create structured log entry
        let log_data = json!({
            "error_category": category,
            "error_recoverable": recoverable,
            "error_message": error.to_string(),
            "operation": context.operation,
            "transport": context.transport,
            "method": context.method,
            "component": context.component,
            "session_id": context.session_id,
            "extra_context": context.extra,
        });

        // Log at appropriate level
        match log_level {
            ErrorLogLevel::Critical => {
                error!(
                    target: "mcp_errors",
                    error_category = category,
                    error_recoverable = recoverable,
                    operation = context.operation.as_str(),
                    "CRITICAL MCP Error: {} - {}",
                    error,
                    serde_json::to_string(&log_data).unwrap_or_default()
                );
            }
            ErrorLogLevel::Error => {
                error!(
                    target: "mcp_errors",
                    error_category = category,
                    error_recoverable = recoverable,
                    operation = context.operation.as_str(),
                    "MCP Error: {} - {}",
                    error,
                    serde_json::to_string(&log_data).unwrap_or_default()
                );
            }
            ErrorLogLevel::Warning => {
                warn!(
                    target: "mcp_errors",
                    error_category = category,
                    error_recoverable = recoverable,
                    operation = context.operation.as_str(),
                    "MCP Warning: {} - {}",
                    error,
                    serde_json::to_string(&log_data).unwrap_or_default()
                );
            }
            ErrorLogLevel::Info => {
                info!(
                    target: "mcp_errors",
                    error_category = category,
                    error_recoverable = recoverable,
                    operation = context.operation.as_str(),
                    "MCP Info: {} - {}",
                    error,
                    serde_json::to_string(&log_data).unwrap_or_default()
                );
            }
        }
    }

    /// Log a retry attempt with context
    pub async fn log_retry_attempt(
        error: &McpError,
        attempt: u32,
        max_attempts: u32,
        will_retry: bool,
        context: ErrorContext,
    ) {
        let category = error.category();
        let recoverable = error.is_recoverable();

        // Record retry metrics
        let metrics = global_metrics();
        metrics
            .record_retry_attempt(&context.operation, attempt, category, will_retry)
            .await;

        let log_data = json!({
            "error_category": category,
            "error_recoverable": recoverable,
            "error_message": error.to_string(),
            "retry_attempt": attempt,
            "max_attempts": max_attempts,
            "will_retry_again": will_retry,
            "operation": context.operation,
            "transport": context.transport,
            "method": context.method,
            "component": context.component,
            "session_id": context.session_id,
            "extra_context": context.extra,
        });

        if will_retry {
            warn!(
                target: "mcp_retries",
                error_category = category,
                retry_attempt = attempt,
                max_attempts = max_attempts,
                operation = context.operation.as_str(),
                "MCP Retry Attempt {}/{}: {} - {}",
                attempt,
                max_attempts,
                error,
                serde_json::to_string(&log_data).unwrap_or_default()
            );
        } else {
            error!(
                target: "mcp_retries",
                error_category = category,
                retry_attempt = attempt,
                max_attempts = max_attempts,
                operation = context.operation.as_str(),
                "MCP Retry Failed (Final): {} - {}",
                error,
                serde_json::to_string(&log_data).unwrap_or_default()
            );
        }
    }

    /// Log successful recovery after retries
    pub async fn log_retry_success(operation: &str, total_attempts: u32, context: ErrorContext) {
        let metrics = global_metrics();
        metrics
            .record_retry_attempt(operation, total_attempts, "success", false)
            .await;

        let log_data = json!({
            "operation": operation,
            "total_attempts": total_attempts,
            "transport": context.transport,
            "method": context.method,
            "component": context.component,
            "session_id": context.session_id,
            "extra_context": context.extra,
        });

        info!(
            target: "mcp_retries",
            operation = operation,
            total_attempts = total_attempts,
            "MCP Retry Success: Operation '{}' succeeded after {} attempts - {}",
            operation,
            total_attempts,
            serde_json::to_string(&log_data).unwrap_or_default()
        );
    }

    /// Create a logging span for an operation
    pub fn create_operation_span(operation: &str, context: &ErrorContext) -> tracing::Span {
        span!(
            Level::INFO,
            "mcp_operation",
            operation = operation,
            transport = context.transport.as_deref(),
            method = context.method.as_deref(),
            component = context.component.as_deref(),
            session_id = context.session_id.as_deref(),
        )
    }
}

impl McpError {
    /// Log this error with structured context
    pub async fn log_with_context(&self, context: ErrorContext) {
        ErrorLogger::log_error(self, context).await;
    }

    /// Log this error with basic context
    pub async fn log_error(&self, operation: &str) {
        let context = ErrorContext::new(operation);
        ErrorLogger::log_error(self, context).await;
    }
}

/// Helper macro for logging errors with automatic context
#[macro_export]
macro_rules! log_mcp_error {
    ($error:expr, $operation:expr) => {
        $error.log_error($operation).await;
    };
    ($error:expr, $context:expr) => {
        $error.log_with_context($context).await;
    };
}

/// Helper macro for logging retry attempts
#[macro_export]
macro_rules! log_mcp_retry {
    ($error:expr, $attempt:expr, $max:expr, $will_retry:expr, $context:expr) => {
        $crate::core::logging::ErrorLogger::log_retry_attempt(
            $error,
            $attempt,
            $max,
            $will_retry,
            $context,
        )
        .await;
    };
}

/// Helper macro for logging successful retries
#[macro_export]
macro_rules! log_mcp_retry_success {
    ($operation:expr, $attempts:expr, $context:expr) => {
        $crate::core::logging::ErrorLogger::log_retry_success($operation, $attempts, $context)
            .await;
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::McpError;

    #[test]
    fn test_error_log_levels() {
        assert_eq!(
            ErrorLogLevel::from(&McpError::internal("test")),
            ErrorLogLevel::Critical
        );
        assert_eq!(
            ErrorLogLevel::from(&McpError::protocol("test")),
            ErrorLogLevel::Error
        );
        assert_eq!(
            ErrorLogLevel::from(&McpError::connection("test")),
            ErrorLogLevel::Warning
        );
        assert_eq!(
            ErrorLogLevel::from(&McpError::validation("test")),
            ErrorLogLevel::Info
        );
    }

    #[test]
    fn test_error_context_builder() {
        let context = ErrorContext::new("test_operation")
            .with_transport("http")
            .with_method("tools/list")
            .with_component("client")
            .with_session_id("sess_123")
            .with_extra("user_id", json!("user_456"));

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.transport, Some("http".to_string()));
        assert_eq!(context.method, Some("tools/list".to_string()));
        assert_eq!(context.component, Some("client".to_string()));
        assert_eq!(context.session_id, Some("sess_123".to_string()));
        assert_eq!(context.extra.get("user_id"), Some(&json!("user_456")));
    }

    #[tokio::test]
    async fn test_error_logging() {
        let error = McpError::connection("Test connection error");
        let context = ErrorContext::new("connect")
            .with_transport("websocket")
            .with_component("client");

        // This test mainly ensures the logging doesn't panic
        ErrorLogger::log_error(&error, context).await;
    }

    #[tokio::test]
    async fn test_retry_logging() {
        let error = McpError::timeout("Request timeout");
        let context = ErrorContext::new("send_request")
            .with_transport("http")
            .with_method("tools/call");

        // Test retry attempt logging
        ErrorLogger::log_retry_attempt(&error, 1, 3, true, context.clone()).await;

        // Test final retry failure
        ErrorLogger::log_retry_attempt(&error, 3, 3, false, context.clone()).await;

        // Test retry success
        ErrorLogger::log_retry_success("send_request", 2, context).await;
    }

    #[tokio::test]
    async fn test_error_extension_methods() {
        let error = McpError::validation("Invalid input");

        // Test basic error logging
        error.log_error("validate_input").await;

        // Test error logging with context
        let context = ErrorContext::new("validate_request")
            .with_method("tools/call")
            .with_extra("input_size", json!(1024));
        error.log_with_context(context).await;
    }

    #[test]
    fn test_operation_span_creation() {
        // Initialize a tracing subscriber for testing
        let _subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        let context = ErrorContext::new("test_op")
            .with_transport("stdio")
            .with_component("server");

        let span = ErrorLogger::create_operation_span("test_operation", &context);
        // In test environments, spans may be disabled based on configuration
        // We just verify the span was created without errors
        let _span_guard = span.enter();
        // If this executes without panic, the span creation is working
    }
}

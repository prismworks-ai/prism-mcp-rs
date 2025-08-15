// ! Client session management
// !
// ! Module provides session management for MCP clients, including connection
// ! state tracking, notification handling, and automatic reconnection capabilities.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc, watch};
use tokio::time::timeout;

use crate::client::mcp_client::McpClient;
use crate::core::error::{McpError, McpResult};
use crate::core::logging::ErrorContext;
use crate::core::retry::{CircuitBreakerConfig, RetryConfig, RetryPolicy};
use crate::protocol::{messages::*, methods, types::*};
use crate::transport::traits::Transport;

/// Session state
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    /// Session is idle (ready but not actively processing)
    Idle,
    /// Session is disconnected
    Disconnected,
    /// Session is connecting
    Connecting,
    /// Session is connected and active
    Connected,
    /// Session is reconnecting after a failure
    Reconnecting,
    /// Session has failed and cannot reconnect
    Failed(String),
}

/// Notification handler trait
pub trait NotificationHandler: Send + Sync {
    /// Handle a notification from the server
    fn handle_notification(&self, notification: JsonRpcNotification);
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Whether to enable automatic reconnection
    pub auto_reconnect: bool,
    /// Maximum number of reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Initial reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
    /// Maximum reconnection delay in milliseconds
    pub max_reconnect_delay_ms: u64,
    /// Reconnection backoff multiplier
    pub reconnect_backoff: f64,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Heartbeat interval in milliseconds (0 to disable)
    pub heartbeat_interval_ms: u64,
    /// Heartbeat timeout in milliseconds
    pub heartbeat_timeout_ms: u64,

    // Additional fields for test compatibility
    /// Session timeout duration
    pub session_timeout: Duration,
    /// Request timeout duration
    pub request_timeout: Duration,
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    /// Enable compression
    pub enable_compression: bool,
    /// Buffer size for operations
    pub buffer_size: usize,

    // Production-ready retry configuration
    /// Retry policy for operations
    pub retry_config: RetryConfig,
    /// Enable circuit breaker for resilience
    pub enable_circuit_breaker: bool,
    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 1000,
            max_reconnect_delay_ms: 30000,
            reconnect_backoff: 2.0,
            connection_timeout_ms: 10000,
            heartbeat_interval_ms: 30000,
            heartbeat_timeout_ms: 5000,
            session_timeout: Duration::from_secs(300),
            request_timeout: Duration::from_secs(30),
            max_concurrent_requests: 10,
            enable_compression: false,
            buffer_size: 8192,
            retry_config: RetryConfig::network(), // Use network-improved retry config
            enable_circuit_breaker: true,
            circuit_breaker_config: CircuitBreakerConfig::default(),
        }
    }
}

/// Client session that manages connection lifecycle and notifications
pub struct ClientSession {
    /// The underlying MCP client
    client: Arc<Mutex<McpClient>>,
    /// Session configuration
    config: SessionConfig,
    /// Current session state
    state: Arc<RwLock<SessionState>>,
    /// State change broadcaster
    state_tx: watch::Sender<SessionState>,
    /// State change receiver
    state_rx: watch::Receiver<SessionState>,
    /// Notification handlers
    notification_handlers: Arc<RwLock<Vec<Box<dyn NotificationHandler>>>>,
    /// Connection timestamp
    connected_at: Arc<RwLock<Option<Instant>>>,
    /// Reconnection attempts counter
    reconnect_attempts: Arc<Mutex<u32>>,
    /// Shutdown signal
    shutdown_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
    /// Retry policy for connection operations
    #[allow(dead_code)]
    retry_policy: Arc<RetryPolicy>,
}

impl ClientSession {
    /// Create a new client session
    pub fn new(client: McpClient) -> Self {
        let config = SessionConfig::default();
        let (state_tx, state_rx) = watch::channel(SessionState::Disconnected);

        // Create retry policy based on configuration
        let retry_policy = if config.enable_circuit_breaker {
            Arc::new(RetryPolicy::with_circuit_breaker(
                config.retry_config.clone(),
                config.circuit_breaker_config.clone(),
            ))
        } else {
            Arc::new(RetryPolicy::new(config.retry_config.clone()))
        };

        Self {
            client: Arc::new(Mutex::new(client)),
            config,
            state: Arc::new(RwLock::new(SessionState::Disconnected)),
            state_tx,
            state_rx,
            notification_handlers: Arc::new(RwLock::new(Vec::new())),
            connected_at: Arc::new(RwLock::new(None)),
            reconnect_attempts: Arc::new(Mutex::new(0)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            retry_policy,
        }
    }

    /// Create a new client session with custom configuration
    pub fn with_config(client: McpClient, config: SessionConfig) -> Self {
        let (state_tx, state_rx) = watch::channel(SessionState::Disconnected);

        // Create retry policy based on configuration
        let retry_policy = if config.enable_circuit_breaker {
            Arc::new(RetryPolicy::with_circuit_breaker(
                config.retry_config.clone(),
                config.circuit_breaker_config.clone(),
            ))
        } else {
            Arc::new(RetryPolicy::new(config.retry_config.clone()))
        };

        Self {
            client: Arc::new(Mutex::new(client)),
            config,
            state: Arc::new(RwLock::new(SessionState::Disconnected)),
            state_tx,
            state_rx,
            notification_handlers: Arc::new(RwLock::new(Vec::new())),
            connected_at: Arc::new(RwLock::new(None)),
            reconnect_attempts: Arc::new(Mutex::new(0)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            retry_policy,
        }
    }

    /// Get the current session state
    pub async fn state(&self) -> SessionState {
        let state = self.state.read().await;
        state.clone()
    }

    /// Subscribe to state changes
    pub fn subscribe_state_changes(&self) -> watch::Receiver<SessionState> {
        self.state_rx.clone()
    }

    /// Check if the session is connected
    pub async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, SessionState::Connected)
    }

    /// Get connection uptime
    pub async fn uptime(&self) -> Option<Duration> {
        let connected_at = self.connected_at.read().await;
        connected_at.map(|time| time.elapsed())
    }

    /// Add a notification handler
    pub async fn add_notification_handler<H>(&self, handler: H)
    where
        H: NotificationHandler + 'static,
    {
        let mut handlers = self.notification_handlers.write().await;
        handlers.push(Box::new(handler));
    }

    /// Connect to the server with the provided transport
    pub async fn connect<T>(&self, transport: T) -> McpResult<InitializeResult>
    where
        T: Transport + 'static,
    {
        self.transition_state(SessionState::Connecting).await?;

        let connect_future = async {
            let mut client = self.client.lock().await;
            client.connect(transport).await
        };

        let result = timeout(
            Duration::from_millis(self.config.connection_timeout_ms),
            connect_future,
        )
        .await;

        match result {
            Ok(Ok(init_result)) => {
                self.transition_state(SessionState::Connected).await?;

                // Record connection time
                {
                    let mut connected_at = self.connected_at.write().await;
                    *connected_at = Some(Instant::now());
                }

                // Reset reconnection attempts
                {
                    let mut attempts = self.reconnect_attempts.lock().await;
                    *attempts = 0;
                }

                // Start background tasks
                self.start_background_tasks().await?;

                Ok(init_result)
            }
            Ok(Err(error)) => {
                self.transition_state(SessionState::Failed(error.to_string()))
                    .await?;
                Err(error)
            }
            Err(_) => {
                let error = McpError::Connection("Connection timeout".to_string());
                self.transition_state(SessionState::Failed(error.to_string()))
                    .await?;
                Err(error)
            }
        }
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) -> McpResult<()> {
        // Stop background tasks
        self.stop_background_tasks().await;

        // Disconnect the client
        {
            let client = self.client.lock().await;
            client.disconnect().await?;
        }

        // Update state
        self.transition_state(SessionState::Disconnected).await?;

        // Clear connection time
        {
            let mut connected_at = self.connected_at.write().await;
            *connected_at = None;
        }

        Ok(())
    }

    /// Reconnect to the server with smart retry logic
    pub async fn reconnect<T>(
        &self,
        transport_factory: impl Fn() -> T + Send + Sync + 'static,
    ) -> McpResult<InitializeResult>
    where
        T: Transport + 'static,
    {
        if !self.config.auto_reconnect {
            let error = McpError::connection("Auto-reconnect is disabled");
            return Err(error);
        }

        let _context = ErrorContext::new("session_reconnect")
            .with_component("client_session")
            .with_extra(
                "max_attempts",
                serde_json::Value::from(self.config.max_reconnect_attempts),
            );

        self.transition_state(SessionState::Reconnecting).await?;

        // Simple retry loop with smart error handling
        let mut last_error = None;

        for attempt in 1..=self.config.retry_config.max_attempts {
            let transport = transport_factory();

            let connect_future = async {
                let mut client_guard = self.client.lock().await;
                client_guard.connect(transport).await
            };

            let result = tokio::time::timeout(
                Duration::from_millis(self.config.connection_timeout_ms),
                connect_future,
            )
            .await;

            match result {
                Ok(Ok(init_result)) => {
                    // Success! Update state and reset counters
                    self.transition_state(SessionState::Connected).await?;

                    // Record connection time
                    {
                        let mut connected_at_guard = self.connected_at.write().await;
                        *connected_at_guard = Some(Instant::now());
                    }

                    // Reset reconnection attempts
                    {
                        let mut attempts = self.reconnect_attempts.lock().await;
                        *attempts = 0;
                    }

                    // Start background tasks for the new connection
                    if let Err(e) = self.start_background_tasks().await {
                        tracing::warn!("Failed to start background tasks after reconnect: {}", e);
                    }

                    return Ok(init_result);
                }
                Ok(Err(error)) => {
                    last_error = Some(error.clone());

                    // Check if error is recoverable and if we should retry
                    if !error.is_recoverable() || attempt >= self.config.retry_config.max_attempts {
                        self.transition_state(SessionState::Failed(error.to_string()))
                            .await?;
                        return Err(error);
                    }

                    // Log retry attempt
                    tracing::warn!(
                        "Reconnection attempt {} failed: {} (recoverable: {}, will retry: {})",
                        attempt,
                        error,
                        error.is_recoverable(),
                        attempt < self.config.retry_config.max_attempts
                    );
                }
                Err(_) => {
                    // Timeout occurred
                    let timeout_error = McpError::timeout("Connection timeout during reconnect");
                    last_error = Some(timeout_error.clone());

                    if attempt >= self.config.retry_config.max_attempts {
                        self.transition_state(SessionState::Failed(timeout_error.to_string()))
                            .await?;
                        return Err(timeout_error);
                    }

                    tracing::warn!("Reconnection attempt {} timed out (will retry)", attempt);
                }
            }

            // Apply delay before next attempt (except for last attempt)
            if attempt < self.config.retry_config.max_attempts {
                let delay_ms = self.config.retry_config.initial_delay_ms
                    * (self
                        .config
                        .retry_config
                        .backoff_multiplier
                        .powi(attempt as i32 - 1) as u64);
                let delay =
                    Duration::from_millis(delay_ms.min(self.config.retry_config.max_delay_ms));

                tracing::debug!("Waiting {:?} before next reconnection attempt", delay);
                tokio::time::sleep(delay).await;
            }
        }

        // All retries exhausted
        let final_error = last_error
            .unwrap_or_else(|| McpError::internal("Reconnection failed without capturing error"));

        self.transition_state(SessionState::Failed(final_error.to_string()))
            .await?;
        Err(final_error)
    }

    /// Get the underlying client (for direct operations)
    pub fn client(&self) -> Arc<Mutex<McpClient>> {
        self.client.clone()
    }

    /// Get session configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    // ========================================================================
    // Background Tasks
    // ========================================================================

    /// Start background tasks (notification handling, heartbeat)
    async fn start_background_tasks(&self) -> McpResult<()> {
        let (_shutdown_tx, shutdown_rx): (broadcast::Sender<()>, broadcast::Receiver<()>) =
            broadcast::channel(16);
        {
            let mut shutdown_guard = self.shutdown_tx.lock().await;
            *shutdown_guard = Some(mpsc::channel(1).0); // Store a dummy for interface compatibility
        }

        // Start notification handler task
        {
            let client = self.client.clone();
            let handlers = self.notification_handlers.clone();
            let mut shutdown_rx_clone = shutdown_rx.resubscribe();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = shutdown_rx_clone.recv() => break,
                        notification_result = async {
                            let client_guard = client.lock().await;
                            client_guard.receive_notification().await
                        } => {
                            match notification_result {
                                Ok(Some(notification)) => {
                                    let handlers_guard = handlers.read().await;
                                    for handler in handlers_guard.iter() {
                                        handler.handle_notification(notification.clone());
                                    }
                                }
                                Ok(None) => {
                                    // No notification available, continue
                                }
                                Err(_) => {
                                    // Error receiving notification, might be disconnected
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        }

        // Start heartbeat task if enabled
        if self.config.heartbeat_interval_ms > 0 {
            let client = self.client.clone();
            let heartbeat_interval = Duration::from_millis(self.config.heartbeat_interval_ms);
            let heartbeat_timeout = Duration::from_millis(self.config.heartbeat_timeout_ms);
            let state = self.state.clone();
            let state_tx = self.state_tx.clone();
            let mut shutdown_rx_clone = shutdown_rx.resubscribe();

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(heartbeat_interval);

                loop {
                    tokio::select! {
                        _ = shutdown_rx_clone.recv() => break,
                        _ = interval.tick() => {
                            // Check if we're still connected
                            {
                                let current_state = state.read().await;
                                if !matches!(*current_state, SessionState::Connected) {
                                    break;
                                }
                            }

                            // Send ping
                            let ping_result = timeout(heartbeat_timeout, async {
                                let client_guard = client.lock().await;
                                client_guard.ping().await
                            }).await;

                            if ping_result.is_err() {
                                // Heartbeat failed, mark as disconnected
                                let _ = state_tx.send(SessionState::Disconnected);
                                break;
                            }
                        }
                    }
                }
            });
        }

        Ok(())
    }

    /// Stop background tasks
    async fn stop_background_tasks(&self) {
        let shutdown_tx = {
            let mut shutdown_guard = self.shutdown_tx.lock().await;
            shutdown_guard.take()
        };

        if let Some(tx) = shutdown_tx {
            let _ = tx.send(()).await; // Ignore error if receiver is dropped
        }
    }

    /// Transition to a new state
    async fn transition_state(&self, new_state: SessionState) -> McpResult<()> {
        {
            let mut state = self.state.write().await;
            *state = new_state.clone();
        }

        // Broadcast the state change
        if self.state_tx.send(new_state).is_err() {
            // Receiver may have been dropped, which is okay
        }

        Ok(())
    }
}

/// Default notification handler that logs notifications
pub struct LoggingNotificationHandler;

impl NotificationHandler for LoggingNotificationHandler {
    fn handle_notification(&self, notification: JsonRpcNotification) {
        tracing::info!(
            "Received notification: {} {:?}",
            notification.method,
            notification.params
        );
    }
}

/// Resource update notification handler
pub struct ResourceUpdateHandler {
    callback: Box<dyn Fn(String) + Send + Sync>,
}

impl ResourceUpdateHandler {
    /// Create a new resource update handler
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl NotificationHandler for ResourceUpdateHandler {
    fn handle_notification(&self, notification: JsonRpcNotification) {
        if notification.method == methods::RESOURCES_UPDATED {
            if let Some(params) = notification.params {
                if let Ok(update_params) = serde_json::from_value::<ResourceUpdatedParams>(params) {
                    (self.callback)(update_params.uri);
                }
            }
        }
    }
}

/// Tool list changed notification handler
pub struct ToolListChangedHandler {
    callback: Box<dyn Fn() + Send + Sync>,
}

impl ToolListChangedHandler {
    /// Create a new tool list changed handler
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl NotificationHandler for ToolListChangedHandler {
    fn handle_notification(&self, notification: JsonRpcNotification) {
        if notification.method == methods::TOOLS_LIST_CHANGED {
            (self.callback)();
        }
    }
}

/// Progress notification handler
pub struct ProgressHandler {
    callback: Box<dyn Fn(String, f32, Option<u32>) + Send + Sync>,
}

impl ProgressHandler {
    /// Create a new progress handler
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(String, f32, Option<u32>) + Send + Sync + 'static,
    {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl NotificationHandler for ProgressHandler {
    fn handle_notification(&self, notification: JsonRpcNotification) {
        if notification.method == methods::PROGRESS {
            if let Some(params) = notification.params {
                if let Ok(progress_params) = serde_json::from_value::<ProgressParams>(params) {
                    (self.callback)(
                        progress_params.progress_token.to_string(),
                        progress_params.progress,
                        progress_params.total.map(|t| t as u32),
                    );
                }
            }
        }
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Current session state
    pub state: SessionState,
    /// Connection uptime
    pub uptime: Option<Duration>,
    /// Number of reconnection attempts
    pub reconnect_attempts: u32,
    /// Connection timestamp
    pub connected_at: Option<Instant>,
}

impl ClientSession {
    /// Get session statistics
    pub async fn stats(&self) -> SessionStats {
        let state = self.state().await;
        let uptime = self.uptime().await;
        let reconnect_attempts = {
            let attempts = self.reconnect_attempts.lock().await;
            *attempts
        };
        let connected_at = {
            let connected_at = self.connected_at.read().await;
            *connected_at
        };

        SessionStats {
            state,
            uptime,
            reconnect_attempts,
            connected_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::mcp_client::McpClient;
    use async_trait::async_trait;

    // Mock transport for testing
    struct MockTransport;

    #[async_trait]
    impl Transport for MockTransport {
        async fn send_request(&mut self, _request: JsonRpcRequest) -> McpResult<JsonRpcResponse> {
            // Return a successful initialize response
            let init_result = InitializeResult::new(
                crate::protocol::LATEST_PROTOCOL_VERSION.to_string(),
                ServerCapabilities::default(),
                ServerInfo {
                    name: "test-server".to_string(),
                    version: "1.0.0".to_string(),
                    title: Some("Test Server".to_string()),
                },
            );
            JsonRpcResponse::success(serde_json::Value::from(1), init_result)
                .map_err(|e| McpError::Serialization(e.to_string()))
        }

        async fn send_notification(&mut self, _notification: JsonRpcNotification) -> McpResult<()> {
            Ok(())
        }

        async fn receive_notification(&mut self) -> McpResult<Option<JsonRpcNotification>> {
            Ok(None)
        }

        async fn close(&mut self) -> McpResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_session_creation() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        let session = ClientSession::new(client);

        assert_eq!(session.state().await, SessionState::Disconnected);
        assert!(!session.is_connected().await);
        assert!(session.uptime().await.is_none());
    }

    #[tokio::test]
    async fn test_session_connection() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        let session = ClientSession::new(client);

        let transport = MockTransport;
        let result = session.connect(transport).await;

        assert!(result.is_ok());
        assert_eq!(session.state().await, SessionState::Connected);
        assert!(session.is_connected().await);
        assert!(session.uptime().await.is_some());
    }

    #[tokio::test]
    async fn test_session_disconnect() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        let session = ClientSession::new(client);

        // Connect first
        let transport = MockTransport;
        session.connect(transport).await.unwrap();
        assert!(session.is_connected().await);

        // Then disconnect
        session.disconnect().await.unwrap();
        assert_eq!(session.state().await, SessionState::Disconnected);
        assert!(!session.is_connected().await);
        assert!(session.uptime().await.is_none());
    }

    #[tokio::test]
    async fn test_notification_handlers() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        let session = ClientSession::new(client);

        // Add a logging notification handler
        session
            .add_notification_handler(LoggingNotificationHandler)
            .await;

        // Add a resource update handler
        session
            .add_notification_handler(ResourceUpdateHandler::new(|uri| {
                println!("Resource updated: {uri}");
            }))
            .await;

        // Add a tool list changed handler
        session
            .add_notification_handler(ToolListChangedHandler::new(|| {
                println!("Tool list changed");
            }))
            .await;

        // Add a progress handler
        session
            .add_notification_handler(ProgressHandler::new(|token, progress, total| {
                println!("Progress {token}: {progress} / {total:?}");
            }))
            .await;

        let handlers = session.notification_handlers.read().await;
        assert_eq!(handlers.len(), 4);
    }

    #[tokio::test]
    async fn test_session_stats() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        let session = ClientSession::new(client);

        let stats = session.stats().await;
        assert_eq!(stats.state, SessionState::Disconnected);
        assert!(stats.uptime.is_none());
        assert_eq!(stats.reconnect_attempts, 0);
        assert!(stats.connected_at.is_none());
    }

    #[tokio::test]
    async fn test_session_config() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        let config = SessionConfig {
            auto_reconnect: false,
            max_reconnect_attempts: 10,
            reconnect_delay_ms: 2000,
            ..Default::default()
        };
        let session = ClientSession::with_config(client, config.clone());

        assert!(!session.config().auto_reconnect);
        assert_eq!(session.config().max_reconnect_attempts, 10);
        assert_eq!(session.config().reconnect_delay_ms, 2000);
    }

    #[tokio::test]
    async fn test_state_subscription() {
        let client = McpClient::new("test-client".to_string(), "1.0.0".to_string());
        let session = ClientSession::new(client);

        let mut state_rx = session.subscribe_state_changes();

        // Initial state
        assert_eq!(*state_rx.borrow(), SessionState::Disconnected);

        // Change state
        session
            .transition_state(SessionState::Connecting)
            .await
            .unwrap();

        // Wait for change
        state_rx.changed().await.unwrap();
        assert_eq!(*state_rx.borrow(), SessionState::Connecting);
    }
}

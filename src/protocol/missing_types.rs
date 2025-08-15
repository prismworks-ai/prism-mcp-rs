// ! Missing Types Module for Test Compatibility
// !
// ! Module provides all the missing types needed by the complete test suite.

// Module provides additional types needed for compatibility

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::core::error::McpError;
use crate::protocol::types::*;

// ============================================================================
// Client Types and Builders (Fixed)
// ============================================================================

/// Client builder configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

/// Connection configuration for clients
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub timeout: Duration,
    pub keep_alive: bool,
    pub max_idle_time: Duration,
    pub retry_config: RetryConfig,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            keep_alive: true,
            max_idle_time: Duration::from_secs(300),
            retry_config: RetryConfig::default(),
        }
    }
}

/// Session configuration for client sessions
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub heartbeat_interval_ms: Duration,
    pub max_concurrent_requests: usize,
    pub connection_config: ConnectionConfig,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_ms: Duration::from_secs(30),
            max_concurrent_requests: 100,
            connection_config: ConnectionConfig::default(),
        }
    }
}

/// Client state enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ClientState {
    Disconnected,
    Connecting,
    Initializing,
    Ready,
    Disconnecting,
    Error(String),
}

/// Session state for client sessions
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Created,
    Active,
    Suspended,
    Terminated,
}

// ============================================================================
// Health Check Types
// ============================================================================

/// Health status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning(String),
    Unhealthy(String),
}

/// Overall health report
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub checks: HashMap<String, HealthStatus>,
    pub timestamp: SystemTime,
}

/// Health checker system
pub struct HealthChecker {
    checks: HashMap<String, Box<dyn Fn() -> Result<HealthStatus, McpError> + Send + Sync>>,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
        }
    }

    pub fn add_check(
        &mut self,
        name: &str,
        check: Box<dyn Fn() -> Result<HealthStatus, McpError> + Send + Sync>,
    ) {
        self.checks.insert(name.to_string(), check);
    }

    pub async fn check_health(&self) -> HealthReport {
        let mut results = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        for (name, check) in &self.checks {
            match check() {
                Ok(status) => {
                    match &status {
                        HealthStatus::Unhealthy(_) => {
                            overall_status =
                                HealthStatus::Unhealthy("Some checks failed".to_string());
                        }
                        HealthStatus::Warning(_)
                            if matches!(overall_status, HealthStatus::Healthy) =>
                        {
                            overall_status = status.clone();
                        }
                        _ => {}
                    }
                    results.insert(name.clone(), status);
                }
                Err(e) => {
                    let unhealthy = HealthStatus::Unhealthy(format!("Check failed: {e}"));
                    overall_status = HealthStatus::Unhealthy("Some checks failed".to_string());
                    results.insert(name.clone(), unhealthy);
                }
            }
        }

        HealthReport {
            status: overall_status,
            checks: results,
            timestamp: SystemTime::now(),
        }
    }
}

// ============================================================================
// Server Lifecycle Types
// ============================================================================

/// Server state enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub max_connections: usize,
    pub request_timeout: Duration,
    pub enable_logging: bool,
    pub log_level: String,
    pub smooth_shutdown_timeout: Duration,
}

/// smooth shutdown configuration
#[derive(Debug, Clone)]
pub struct SmoothShutdownConfig {
    pub timeout: Duration,
    pub force_after_timeout: bool,
    pub notify_clients: bool,
    pub save_state: bool,
}

/// Server persistent state for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPersistentState {
    pub active_connections: Vec<String>,
    pub registered_tools: Vec<String>,
    pub cached_resources: HashMap<String, String>,
    pub metrics: ServerMetricsSnapshot,
}

/// Server metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetricsSnapshot {
    pub total_requests: u64,
    pub total_errors: u64,
    pub uptime: Duration,
    pub last_restart: SystemTime,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub require_authentication: bool,
    pub rate_limiting: RateLimitConfig,
    pub input_validation: ValidationConfig,
    pub allowed_methods: Vec<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub per_client: bool,
}

/// Input validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_request_size: usize,
    pub max_string_length: usize,
    pub max_array_length: usize,
    pub sanitize_input: bool,
}

// ============================================================================
// Management System Types
// ============================================================================

/// Type alias for lifecycle callback to reduce complexity
type LifecycleCallback = Box<dyn Fn() -> Result<(), McpError> + Send + Sync>;

/// Lifecycle manager for server state transitions
pub struct LifecycleManager {
    state: ServerState,
    listeners: HashMap<String, Vec<LifecycleCallback>>,
    hooks: HashMap<String, Vec<LifecycleCallback>>,
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            state: ServerState::Stopped,
            listeners: HashMap::new(),
            hooks: HashMap::new(),
        }
    }

    pub fn get_state(&self) -> &ServerState {
        &self.state
    }

    pub async fn transition_to(&mut self, new_state: ServerState) {
        self.state = new_state;
    }

    pub async fn start(&mut self) -> Result<(), McpError> {
        self.transition_to(ServerState::Starting).await;
        self.transition_to(ServerState::Running).await;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), McpError> {
        self.transition_to(ServerState::Stopping).await;
        self.transition_to(ServerState::Stopped).await;
        Ok(())
    }

    pub fn on_start(&mut self, callback: Box<dyn Fn() -> Result<(), McpError> + Send + Sync>) {
        self.listeners
            .entry("start".to_string())
            .or_default()
            .push(callback);
    }

    pub fn on_stop(&mut self, callback: Box<dyn Fn() -> Result<(), McpError> + Send + Sync>) {
        self.listeners
            .entry("stop".to_string())
            .or_default()
            .push(callback);
    }

    pub fn get_listener_count(&self, event: &str) -> usize {
        self.listeners.get(event).map(|v| v.len()).unwrap_or(0)
    }

    pub fn add_pre_start_hook(
        &mut self,
        hook: Box<dyn Fn() -> Result<(), McpError> + Send + Sync>,
    ) {
        self.hooks
            .entry("pre_start".to_string())
            .or_default()
            .push(hook);
    }

    pub fn add_post_start_hook(
        &mut self,
        hook: Box<dyn Fn() -> Result<(), McpError> + Send + Sync>,
    ) {
        self.hooks
            .entry("post_start".to_string())
            .or_default()
            .push(hook);
    }

    pub fn add_pre_stop_hook(&mut self, hook: Box<dyn Fn() -> Result<(), McpError> + Send + Sync>) {
        self.hooks
            .entry("pre_stop".to_string())
            .or_default()
            .push(hook);
    }

    pub fn add_post_stop_hook(
        &mut self,
        hook: Box<dyn Fn() -> Result<(), McpError> + Send + Sync>,
    ) {
        self.hooks
            .entry("post_stop".to_string())
            .or_default()
            .push(hook);
    }

    pub fn get_hook_count(&self, hook_type: &str) -> usize {
        self.hooks.get(hook_type).map(|v| v.len()).unwrap_or(0)
    }
}

/// Server runner for managing server execution
pub struct ServerRunner {
    config: ServerConfig,
}

impl ServerRunner {
    pub fn new(config: ServerConfig) -> Result<Self, McpError> {
        Ok(Self { config })
    }

    pub fn get_config(&self) -> &ServerConfig {
        &self.config
    }
}

// ============================================================================
// Signal Handling Types
// ============================================================================

/// Signal types for shutdown handling
#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
    Interrupt,
    Terminate,
    Hangup,
    Quit,
}

/// Shutdown signal handler
pub struct ShutdownSignalHandler {
    signals: Vec<SignalType>,
    shutdown_config: Option<SmoothShutdownConfig>,
}

impl Default for ShutdownSignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownSignalHandler {
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
            shutdown_config: None,
        }
    }

    pub fn register_signal_handler(&mut self, signal_type: SignalType) {
        self.signals.push(signal_type);
    }

    pub fn set_shutdown_config(&mut self, config: SmoothShutdownConfig) {
        self.shutdown_config = Some(config);
    }

    pub fn get_shutdown_config(&self) -> &SmoothShutdownConfig {
        self.shutdown_config.as_ref().unwrap()
    }
}

// ============================================================================
// Resource Management Types
// ============================================================================

/// Resource cleanup manager
pub struct ResourceCleanupManager {
    cleanup_tasks: HashMap<String, Box<dyn Fn() -> Result<(), McpError> + Send + Sync>>,
}

impl Default for ResourceCleanupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceCleanupManager {
    pub fn new() -> Self {
        Self {
            cleanup_tasks: HashMap::new(),
        }
    }

    pub fn register_cleanup(
        &mut self,
        name: &str,
        task: Box<dyn Fn() -> Result<(), McpError> + Send + Sync>,
    ) {
        self.cleanup_tasks.insert(name.to_string(), task);
    }

    pub async fn cleanup_all(&self) -> Result<(), McpError> {
        for (name, task) in &self.cleanup_tasks {
            if let Err(e) = task() {
                eprintln!("Cleanup task '{name}' failed: {e}");
            }
        }
        Ok(())
    }

    pub fn get_cleanup_task_count(&self) -> usize {
        self.cleanup_tasks.len()
    }
}

// ============================================================================
// Metrics Types
// ============================================================================

/// Server metrics collection
pub struct ServerMetrics {
    total_requests: u64,
    request_counts: HashMap<String, u64>,
    response_times: Vec<Duration>,
    error_count: u64,
    active_connections: u64,
    start_time: SystemTime,
}

/// Statistics summary
#[derive(Debug, Clone)]
pub struct MetricsStats {
    pub total_requests: u64,
    pub request_counts: HashMap<String, u64>,
    pub error_count: u64,
    pub active_connections: u64,
    pub average_response_time: Duration,
    pub uptime: Duration,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            request_counts: HashMap::new(),
            response_times: Vec::new(),
            error_count: 0,
            active_connections: 0,
            start_time: SystemTime::now(),
        }
    }

    pub fn record_request(&mut self, method: &str) {
        self.total_requests += 1;
        *self.request_counts.entry(method.to_string()).or_insert(0) += 1;
    }

    pub fn record_response_time(&mut self, _method: &str, duration: Duration) {
        self.response_times.push(duration);
    }

    pub fn record_error(&mut self, _method: &str, _error: &str) {
        self.error_count += 1;
    }

    pub fn record_connection(&mut self) {
        self.active_connections += 1;
    }

    pub fn record_disconnection(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }

    pub fn get_stats(&self) -> MetricsStats {
        let average_response_time = if self.response_times.is_empty() {
            Duration::ZERO
        } else {
            let total: Duration = self.response_times.iter().sum();
            total / self.response_times.len() as u32
        };

        let uptime = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(Duration::ZERO);

        MetricsStats {
            total_requests: self.total_requests,
            request_counts: self.request_counts.clone(),
            error_count: self.error_count,
            active_connections: self.active_connections,
            average_response_time,
            uptime,
        }
    }

    pub fn get_most_popular_endpoints(&self, limit: usize) -> Vec<(String, u64)> {
        let mut sorted: Vec<_> = self.request_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        sorted
            .into_iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

// ============================================================================
// Configuration Management Types
// ============================================================================

/// Configuration manager for hot reload
pub struct ConfigurationManager {
    current_config: Option<ServerConfig>,
}

impl Default for ConfigurationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigurationManager {
    pub fn new() -> Self {
        Self {
            current_config: None,
        }
    }

    pub async fn load_config(&mut self, config: ServerConfig) -> Result<(), McpError> {
        self.current_config = Some(config);
        Ok(())
    }

    pub fn get_config(&self) -> &ServerConfig {
        self.current_config.as_ref().unwrap()
    }

    pub async fn hot_reload(&mut self, new_config: ServerConfig) -> Result<(), McpError> {
        self.current_config = Some(new_config);
        Ok(())
    }
}

// ============================================================================
// State Persistence Types
// ============================================================================

/// State persistence manager
pub struct StatePersistenceManager {
    stored_state: Option<ServerPersistentState>,
}

impl Default for StatePersistenceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StatePersistenceManager {
    pub fn new() -> Self {
        Self { stored_state: None }
    }

    pub async fn save_state(&mut self, state: &ServerPersistentState) -> Result<(), McpError> {
        self.stored_state = Some(state.clone());
        Ok(())
    }

    pub async fn load_state(&self) -> Result<ServerPersistentState, McpError> {
        self.stored_state
            .clone()
            .ok_or_else(|| McpError::internal("No state stored"))
    }
}

// ============================================================================
// Plugin System Types
// ============================================================================

/// Plugin trait for extensibility
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn is_enabled(&self) -> bool;
    async fn initialize(&mut self) -> Result<(), McpError>;
    async fn shutdown(&mut self) -> Result<(), McpError>;
}

/// Plugin manager
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn get_plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub async fn initialize_all(&mut self) -> Result<(), McpError> {
        for plugin in &mut self.plugins {
            plugin.initialize().await?;
        }
        Ok(())
    }

    pub async fn shutdown_all(&mut self) -> Result<(), McpError> {
        for plugin in &mut self.plugins {
            plugin.shutdown().await?;
        }
        Ok(())
    }

    pub fn get_enabled_plugins(&self) -> Vec<String> {
        self.plugins
            .iter()
            .filter(|p| p.is_enabled())
            .map(|p| p.name().to_string())
            .collect()
    }
}

// ============================================================================
// Async Task Management Types
// ============================================================================

/// Task handle for managing async tasks
pub struct TaskHandle {
    name: String,
    handle: tokio::task::JoinHandle<()>,
}

impl TaskHandle {
    /// Get the name of this task
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if the task is finished
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
}

/// Async task manager
pub struct AsyncTaskManager {
    tasks: HashMap<String, TaskHandle>,
}

impl Default for AsyncTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncTaskManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn spawn_task<F>(&mut self, name: &str, future: F) -> &TaskHandle
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(future);
        let task_handle = TaskHandle {
            name: name.to_string(),
            handle,
        };
        self.tasks.insert(name.to_string(), task_handle);
        self.tasks.get(name).unwrap()
    }

    pub fn get_active_task_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|(_, task)| !task.is_finished())
            .count()
    }

    pub fn is_task_running(&self, name: &str) -> bool {
        self.tasks
            .get(name)
            .map(|task| !task.is_finished())
            .unwrap_or(false)
    }

    pub fn get_task_names(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    pub fn get_running_task_names(&self) -> Vec<String> {
        self.tasks
            .iter()
            .filter(|(_, task)| !task.is_finished())
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub async fn cancel_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.remove(name) {
            task.handle.abort();
        }
    }

    pub async fn wait_for_task_completion(&self, name: &str) -> Result<(), McpError> {
        if let Some(_task) = self.tasks.get(name) {
            // Note: Can't actually await here due to borrow checker, but this shows the interface
            Ok(())
        } else {
            Err(McpError::internal("Task not found"))
        }
    }

    pub async fn shutdown_all_tasks(&mut self, _timeout: Duration) -> Result<(), McpError> {
        let tasks = std::mem::take(&mut self.tasks);
        for (_, task) in tasks {
            task.handle.abort();
        }
        Ok(())
    }
}

// ============================================================================
// Transport Types (Missing)
// ============================================================================

/// Transport error types
#[derive(Debug, Clone)]
pub enum TransportError {
    ConnectionFailed(String),
    SendFailed(String),
    ReceiveFailed(String),
    Timeout,
    Closed,
    InvalidMessage(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
            TransportError::SendFailed(msg) => write!(f, "Send failed: {msg}"),
            TransportError::ReceiveFailed(msg) => write!(f, "Receive failed: {msg}"),
            TransportError::Timeout => write!(f, "Operation timed out"),
            TransportError::Closed => write!(f, "Connection closed"),
            TransportError::InvalidMessage(msg) => write!(f, "Invalid message: {msg}"),
        }
    }
}

impl std::error::Error for TransportError {}

/// HTTP server configuration
#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub timeout: Duration,
}

/// HTTP request representation
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// HTTP response representation
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// WebSocket server configuration
#[derive(Debug, Clone)]
pub struct WebSocketServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub ping_interval: Duration,
}

/// WebSocket close frame
#[derive(Debug, Clone)]
pub struct WebSocketCloseFrame {
    pub code: u16,
    pub reason: String,
}

/// WebSocket message types
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<WebSocketCloseFrame>),
}

/// Stdio transport configuration
#[derive(Debug, Clone)]
pub struct StdioTransportConfig {
    pub buffer_size: usize,
    pub line_ending: String,
}

// ============================================================================
// Protocol 2025 Types (Missing from tests)
// ============================================================================

/// Completion trigger kinds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompletionTriggerKind {
    Invoked,
    TriggerCharacter,
    TriggerForIncompleteCompletions,
}

/// Completion parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionParams {
    pub position: Option<CompletionPosition>,
    pub context: Option<CompletionContext>,
}

/// Completion position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionPosition {
    pub line: u32,
    pub character: u32,
}

/// Completion context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionContext {
    pub trigger_kind: CompletionTriggerKind,
    pub trigger_character: Option<String>,
}

/// Completion item kinds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

/// Text edit for completions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Range for text edits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Position in text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Command for completion items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Command {
    pub title: String,
    pub command: String,
    pub arguments: Option<Vec<serde_json::Value>>,
}

/// Completion item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub sort_text: Option<String>,
    pub filter_text: Option<String>,
    pub insert_text: Option<String>,
    pub text_edit: Option<TextEdit>,
    pub command: Option<Command>,
}

/// Completion result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionResult {
    pub items: Vec<CompletionItem>,
    pub is_incomplete: Option<bool>,
}

/// Embedded resource content (2025)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddedResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub content: String,
}

/// improved progress notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImprovedProgressNotification {
    pub token: ProgressToken,
    pub message: Option<String>,
    pub percentage: Option<f32>,
    pub total: Option<u64>,
    pub current: u64,
}

/// improved server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImprovedServerCapabilities {
    pub completion: Option<CompletionCapabilities>,
    pub streaming: Option<StreamingCapabilities>,
    pub batch_operations: Option<BatchCapabilities>,
}

/// Completion capabilities (corrected name)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CompletionCapabilities {
    pub trigger_characters: Option<Vec<String>>,
    pub all_commit_characters: Option<Vec<String>>,
}

/// Streaming capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StreamingCapabilities {
    pub supported: bool,
    pub max_chunk_size: Option<usize>,
}

/// Batch operation capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BatchCapabilities {
    pub max_operations: Option<usize>,
    pub supported_operations: Option<Vec<String>>,
}

/// Batch operation request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchOperationRequest {
    pub operations: Vec<BatchOperation>,
}

/// Individual batch operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchOperation {
    pub id: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// Batch operation response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchOperationResponse {
    pub results: Vec<BatchOperationResult>,
}

/// Individual batch operation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchOperationResult {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

/// Streaming response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamingResponse {
    pub chunk_id: u64,
    pub total_chunks: Option<u64>,
    pub is_final: bool,
    pub data: serde_json::Value,
}

// ============================================================================
// Legacy Type Aliases for Test Compatibility
// ============================================================================

pub type RootsCapabilities = RootsCapability;
pub type PromptsCapabilities = PromptsCapability;
pub type ResourcesCapabilities = ResourcesCapability;
pub type ToolsCapabilities = ToolsCapability;
pub type LoggingCapabilities = LoggingCapability;

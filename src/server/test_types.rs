// ! Additional server lifecycle types for complete testing

#![cfg(not(coverage))]
#![allow(unexpected_cfgs)]

use crate::core::error::{McpError, McpResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

// Missing lifecycle manager types
#[derive(Debug, Clone, PartialEq)]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Stopping,
}

pub struct LifecycleManager {
    state: ServerState,
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
        }
    }

    pub fn get_state(&self) -> ServerState {
        self.state.clone()
    }

    pub async fn transition_to(&mut self, state: ServerState) {
        self.state = state;
    }

    pub async fn start(&mut self) -> McpResult<()> {
        self.transition_to(ServerState::Starting).await;
        self.transition_to(ServerState::Running).await;
        Ok(())
    }

    pub async fn stop(&mut self) -> McpResult<()> {
        self.transition_to(ServerState::Stopping).await;
        self.transition_to(ServerState::Stopped).await;
        Ok(())
    }

    pub fn get_listener_count(&self, _event: &str) -> usize {
        0
    }
    pub fn get_hook_count(&self, _hook: &str) -> usize {
        0
    }

    pub fn on_start(&mut self, _callback: Box<dyn Fn() -> McpResult<()> + Send + Sync>) {}
    pub fn on_stop(&mut self, _callback: Box<dyn Fn() -> McpResult<()> + Send + Sync>) {}
    pub fn add_pre_start_hook(&mut self, _callback: Box<dyn Fn() -> McpResult<()> + Send + Sync>) {}
    pub fn add_post_start_hook(&mut self, _callback: Box<dyn Fn() -> McpResult<()> + Send + Sync>) {
    }
    pub fn add_pre_stop_hook(&mut self, _callback: Box<dyn Fn() -> McpResult<()> + Send + Sync>) {}
    pub fn add_post_stop_hook(&mut self, _callback: Box<dyn Fn() -> McpResult<()> + Send + Sync>) {}
}

// Missing configuration types
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

#[derive(Debug, Clone)]
pub struct SmoothShutdownConfig {
    pub timeout: Duration,
    pub force_after_timeout: bool,
    pub notify_clients: bool,
    pub save_state: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub require_authentication: bool,
    pub rate_limiting: RateLimitConfig,
    pub input_validation: ValidationConfig,
    pub allowed_methods: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub per_client: bool,
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_request_size: usize,
    pub max_string_length: usize,
    pub max_array_length: usize,
    pub sanitize_input: bool,
}

// Missing health and management types
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
    Warning(String),
}

pub struct ServerRunner {
    config: ServerConfig,
}

impl ServerRunner {
    pub fn new(config: ServerConfig) -> McpResult<Self> {
        Ok(Self { config })
    }

    pub fn get_config(&self) -> &ServerConfig {
        &self.config
    }
}

pub struct ShutdownSignalHandler;

impl Default for ShutdownSignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownSignalHandler {
    pub fn new() -> Self {
        Self
    }
    pub fn register_signal_handler(&mut self, _signal: SignalType) {}
    pub fn set_shutdown_config(&mut self, _config: SmoothShutdownConfig) {}
    pub fn get_shutdown_config(&self) -> SmoothShutdownConfig {
        SmoothShutdownConfig {
            timeout: Duration::from_secs(5),
            force_after_timeout: true,
            notify_clients: true,
            save_state: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SignalType {
    Interrupt,
    Terminate,
}

// Additional placeholder types for tests
pub struct HealthChecker;

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self
    }
    pub fn add_check(&mut self, _name: &str, _check: Box<dyn Fn() -> McpResult<HealthStatus>>) {}
    pub async fn check_health(&self) -> OverallHealth {
        OverallHealth {
            status: HealthStatus::Healthy,
            checks: HashMap::new(),
        }
    }
}

pub struct OverallHealth {
    pub status: HealthStatus,
    pub checks: HashMap<String, HealthStatus>,
}

pub struct ResourceCleanupManager;

impl Default for ResourceCleanupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceCleanupManager {
    pub fn new() -> Self {
        Self
    }
    pub fn register_cleanup(&mut self, _name: &str, _cleanup: Box<dyn Fn() -> McpResult<()>>) {}
    pub async fn cleanup_all(&self) -> McpResult<()> {
        Ok(())
    }
    pub fn get_cleanup_task_count(&self) -> usize {
        0
    }
}

pub struct ServerMetrics;

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self
    }
    pub fn record_request(&mut self, _method: &str) {}
    pub fn record_response_time(&mut self, _method: &str, _duration: Duration) {}
    pub fn record_error(&mut self, _method: &str, _error: &str) {}
    pub fn record_connection(&mut self) {}
    pub fn record_disconnection(&mut self) {}
    pub fn get_stats(&self) -> ServerStats {
        ServerStats {
            total_requests: 0,
            request_counts: HashMap::new(),
            error_count: 0,
            active_connections: 0,
            average_response_time: Duration::ZERO,
        }
    }
    pub fn get_most_popular_endpoints(&self, _limit: usize) -> Vec<(String, usize)> {
        vec![]
    }
}

pub struct ServerStats {
    pub total_requests: usize,
    pub request_counts: HashMap<String, usize>,
    pub error_count: usize,
    pub active_connections: usize,
    pub average_response_time: Duration,
}

pub struct ConfigurationManager;

impl Default for ConfigurationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigurationManager {
    pub fn new() -> Self {
        Self
    }
    pub async fn load_config(&mut self, _config: ServerConfig) -> McpResult<()> {
        Ok(())
    }
    pub fn get_config(&self) -> ServerConfig {
        ServerConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            max_connections: 50,
            request_timeout: Duration::from_secs(30),
            enable_logging: true,
            log_level: "info".to_string(),
            smooth_shutdown_timeout: Duration::from_secs(10),
        }
    }
    pub async fn hot_reload(&mut self, _config: ServerConfig) -> McpResult<()> {
        Ok(())
    }
}

pub struct StatePersistenceManager;

impl Default for StatePersistenceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StatePersistenceManager {
    pub fn new() -> Self {
        Self
    }
    pub async fn save_state(&self, _state: &ServerPersistentState) -> McpResult<()> {
        Ok(())
    }
    pub async fn load_state(&self) -> McpResult<ServerPersistentState> {
        Ok(ServerPersistentState {
            active_connections: vec![],
            registered_tools: vec![],
            cached_resources: HashMap::new(),
            metrics: ServerMetricsSnapshot {
                total_requests: 0,
                total_errors: 0,
                uptime: Duration::ZERO,
                last_restart: std::time::SystemTime::now(),
            },
        })
    }
}

pub struct ServerPersistentState {
    pub active_connections: Vec<String>,
    pub registered_tools: Vec<String>,
    pub cached_resources: HashMap<String, String>,
    pub metrics: ServerMetricsSnapshot,
}

pub struct ServerMetricsSnapshot {
    pub total_requests: usize,
    pub total_errors: usize,
    pub uptime: Duration,
    pub last_restart: std::time::SystemTime,
}

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

    pub async fn initialize_all(&mut self) -> McpResult<()> {
        for plugin in &mut self.plugins {
            plugin.initialize().await?;
        }
        Ok(())
    }

    pub async fn shutdown_all(&mut self) -> McpResult<()> {
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

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn is_enabled(&self) -> bool;
    async fn initialize(&mut self) -> Result<(), McpError>;
    async fn shutdown(&mut self) -> Result<(), McpError>;
}

pub struct AsyncTaskManager;

impl Default for AsyncTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncTaskManager {
    pub fn new() -> Self {
        Self
    }
    pub fn spawn_task<F>(&mut self, _name: &str, _task: F) -> tokio::task::JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(async {})
    }
    pub fn get_active_task_count(&self) -> usize {
        0
    }
    pub fn is_task_running(&self, _name: &str) -> bool {
        false
    }
    pub async fn cancel_task(&mut self, _name: &str) {}
    pub async fn wait_for_task_completion(&self, _name: &str) {}
    pub async fn shutdown_all_tasks(&self, _timeout: Duration) -> McpResult<()> {
        Ok(())
    }
}

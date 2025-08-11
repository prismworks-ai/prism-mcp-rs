// ! Plugin system types and traits

use crate::core::error::McpResult;
use crate::core::prompt::Prompt;
use crate::core::resource::Resource;
use crate::core::tool::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Main plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Get the plugin version
    fn version(&self) -> &str;

    /// Get the plugin description
    fn description(&self) -> &str;

    /// Initialize the plugin
    async fn initialize(&mut self) -> McpResult<()>;

    /// Shutdown the plugin
    async fn shutdown(&mut self) -> McpResult<()>;

    /// Configure the plugin with custom settings
    async fn configure(&mut self, config: Value) -> McpResult<()>;

    /// Get the plugin capabilities
    fn capabilities(&self) -> PluginCapabilities;

    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
}

/// Plugin capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginCapabilities {
    /// Whether the plugin provides tools
    pub provides_tools: bool,

    /// Whether the plugin provides resources
    pub provides_resources: bool,

    /// Whether the plugin provides prompts
    pub provides_prompts: bool,

    /// Whether the plugin supports hot reload
    pub supports_hot_reload: bool,
}

/// Plugin metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin author
    pub author: Option<String>,

    /// Plugin license
    pub license: Option<String>,

    /// Plugin homepage
    pub homepage: Option<String>,

    /// Plugin repository
    pub repository: Option<String>,

    /// Plugin keywords
    pub keywords: Vec<String>,

    /// Plugin categories
    pub categories: Vec<String>,
}

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Plugin file path
    pub path: PathBuf,

    /// Whether the plugin is enabled
    pub enabled: bool,

    /// Plugin capabilities
    pub capabilities: PluginCapabilities,

    /// Plugin metadata
    pub metadata: PluginMetadata,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Whether the plugin is enabled
    pub enabled: bool,

    /// Whether to auto-reload on changes
    pub auto_reload: bool,

    /// Path to plugin configuration file
    pub config_path: Option<PathBuf>,

    /// Plugin-specific settings
    pub settings: Value,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_reload: false,
            config_path: None,
            settings: Value::Null,
        }
    }
}

/// Loaded plugin container
pub struct LoadedPlugin {
    /// The plugin instance
    pub plugin: Box<dyn Plugin>,

    /// Plugin configuration
    pub config: PluginConfig,

    /// Registered tools from this plugin
    pub tools: HashMap<String, Arc<Tool>>,

    /// Registered resources from this plugin
    pub resources: HashMap<String, Arc<Resource>>,

    /// Registered prompts from this plugin
    pub prompts: HashMap<String, Arc<Prompt>>,
}

/// Plugin error types
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Failed to load plugin: {0}")]
    LoadFailed(String),

    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Incompatible plugin version: {0}")]
    IncompatibleVersion(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        PluginError::Io(err.to_string())
    }
}

// ! Plugin configuration management
// !
// ! Module handles plugin configuration files and manifests.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin name/identifier
    pub name: String,

    /// Whether the plugin is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Path to the plugin file (optional, will search in default locations)
    pub path: Option<String>,

    /// Plugin-specific configuration
    pub config: Option<Value>,

    /// Environment variables to set for the plugin
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Auto-reload on file change
    #[serde(default)]
    pub auto_reload: bool,

    /// Load priority (lower numbers load first)
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_enabled() -> bool {
    true
}

fn default_priority() -> i32 {
    100
}

/// Plugin manifest (plugin.yaml in plugin directory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub plugin: PluginInfo,

    /// Required dependencies
    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    /// Tool definition
    pub tool: ToolDefinition,

    /// Build information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildInfo>,

    /// Installation instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<InstallInfo>,
}

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin ID
    pub id: String,

    /// Plugin name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Plugin author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Plugin description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Plugin homepage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Plugin repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// Plugin license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Keywords for discovery
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Required MCP SDK version
    pub mcp_version: String,
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency plugin ID
    pub plugin: String,

    /// Required version range
    pub version: String,

    /// Is optional
    #[serde(default)]
    pub optional: bool,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Input schema (JSON Schema)
    pub input_schema: Value,

    /// Output schema (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,

    /// Tool examples
    #[serde(default)]
    pub examples: Vec<ToolExample>,
}

/// Tool usage example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    /// Example name
    pub name: String,

    /// Example description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Input arguments
    pub input: Value,

    /// Expected output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
}

/// Build information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    /// Build command
    pub command: String,

    /// Build directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,

    /// Output file
    pub output: String,
}

/// Installation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallInfo {
    /// Pre-install script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_install: Option<String>,

    /// Post-install script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install: Option<String>,

    /// Required system dependencies
    #[serde(default)]
    pub system_deps: Vec<String>,
}

/// Plugin configuration set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigSet {
    /// Plugin configurations
    pub plugins: Vec<PluginConfig>,

    /// Global plugin settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<PluginSettings>,
}

/// Global plugin settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    /// Plugin directory paths
    #[serde(default)]
    pub plugin_dirs: Vec<PathBuf>,

    /// Auto-load plugins from directories
    #[serde(default = "default_auto_load")]
    pub auto_load: bool,

    /// Enable hot reload
    #[serde(default = "default_hot_reload")]
    pub hot_reload: bool,

    /// Plugin isolation level
    #[serde(default)]
    pub isolation: IsolationLevel,

    /// Maximum plugin load time (seconds)
    #[serde(default = "default_load_timeout")]
    pub load_timeout: u64,
}

fn default_auto_load() -> bool {
    true
}

fn default_hot_reload() -> bool {
    false
}

fn default_load_timeout() -> u64 {
    30
}

/// Plugin isolation level
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IsolationLevel {
    /// No isolation (default)
    #[default]
    None,

    /// Thread isolation
    Thread,

    /// Process isolation
    Process,

    /// Container isolation
    Container,
}

impl PluginConfig {
    /// Create a simple plugin configuration
    pub fn simple(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            path: None,
            config: None,
            env: HashMap::new(),
            auto_reload: false,
            priority: 100,
        }
    }

    /// Set the plugin path
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set plugin configuration
    pub fn with_config(mut self, config: Value) -> Self {
        self.config = Some(config);
        self
    }

    /// Enable auto-reload
    pub fn with_auto_reload(mut self) -> Self {
        self.auto_reload = true;
        self
    }
}

impl PluginManifest {
    /// Load manifest from YAML file
    pub async fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| e.to_string())?;

        serde_yaml::from_str(&content).map_err(|e| e.to_string())
    }

    /// Save manifest to YAML file
    pub async fn to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        let content = serde_yaml::to_string(self).map_err(|e| e.to_string())?;

        tokio::fs::write(path, content)
            .await
            .map_err(|e| e.to_string())
    }
}

impl PluginConfigSet {
    /// Load configuration set from YAML file
    pub async fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| e.to_string())?;

        serde_yaml::from_str(&content).map_err(|e| e.to_string())
    }

    /// Save configuration set to YAML file
    pub async fn to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        let content = serde_yaml::to_string(self).map_err(|e| e.to_string())?;

        tokio::fs::write(path, content)
            .await
            .map_err(|e| e.to_string())
    }

    /// Sort plugins by priority
    pub fn sort_by_priority(&mut self) {
        self.plugins.sort_by_key(|p| p.priority);
    }
}

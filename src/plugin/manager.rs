// ! High-level plugin manager
// !
// ! Module provides the main interface for managing plugins in an MCP server.

use crate::core::error::{McpError, McpResult};
use crate::plugin::{
    PluginConfig, PluginError, PluginEvent, PluginLoader, PluginMetadata, PluginResult,
    ToolRegistry,
};
use crate::protocol::types::{Tool, ToolResult};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Type alias for event handlers to reduce complexity
type EventHandlers = Vec<Box<dyn Fn(PluginEvent) + Send + Sync>>;

/// Plugin manager for MCP servers
pub struct PluginManager {
    /// Plugin loader
    loader: Arc<RwLock<PluginLoader>>,

    /// Tool registry
    registry: Arc<RwLock<ToolRegistry>>,

    /// Plugin configurations
    configs: Arc<RwLock<HashMap<String, PluginConfig>>>,

    /// Event handlers
    event_handlers: Arc<RwLock<EventHandlers>>,

    /// Enabled plugins
    enabled: Arc<RwLock<HashMap<String, bool>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            loader: Arc::new(RwLock::new(PluginLoader::new())),
            registry: Arc::new(RwLock::new(ToolRegistry::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: Arc::new(RwLock::new(Vec::new())),
            enabled: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a plugin from configuration
    pub async fn load_plugin(&self, config: PluginConfig) -> PluginResult<()> {
        info!("Loading plugin: {}", config.name);

        // Store configuration
        self.configs
            .write()
            .await
            .insert(config.name.clone(), config.clone());

        // Find or use explicit path
        let path = if let Some(ref explicit_path) = config.path {
            Path::new(explicit_path).to_path_buf()
        } else {
            self.loader
                .read()
                .await
                .find_plugin(&config.name)
                .ok_or_else(|| PluginError::NotFound(config.name.clone()))?
        };

        // Load the plugin
        let plugin_arc = {
            let mut loader = self.loader.write().await;
            loader.load_plugin(&path)?
        };

        // Initialize the plugin in async context
        {
            let mut plugin_write = plugin_arc.write().await;
            plugin_write
                .initialize()
                .await
                .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;
        }

        // Configure the plugin if needed
        if let Some(ref plugin_config) = config.config {
            let plugin_box = {
                let loader = self.loader.write().await;
                loader
                    .get_plugin(&config.name)
                    .ok_or_else(|| PluginError::NotFound(config.name.clone()))?
                    .clone()
            };

            let mut plugin_lock = plugin_box.write().await;
            plugin_lock
                .configure(plugin_config.clone())
                .await
                .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;
        }

        // Get plugin reference for registration
        let _plugin = {
            let loader = self.loader.read().await;
            loader
                .get_plugin(&config.name)
                .ok_or_else(|| PluginError::NotFound(config.name.clone()))?
                .clone()
        };

        // Register the plugin's tool
        let metadata = {
            let plugin_lock = plugin_arc.read().await;
            plugin_lock.metadata()
        };
        let tool_def = {
            let plugin_lock = plugin_arc.read().await;
            plugin_lock.tool_definition()
        };

        self.registry
            .write()
            .await
            .register_plugin_tool(metadata.id.clone(), plugin_arc)
            .await?;

        // Mark as enabled
        self.enabled
            .write()
            .await
            .insert(metadata.id.clone(), config.enabled);

        // Emit event
        self.emit_event(PluginEvent::Loaded {
            plugin_id: metadata.id.clone(),
        })
        .await;

        self.emit_event(PluginEvent::ToolRegistered {
            plugin_id: metadata.id,
            tool_name: tool_def.name,
        })
        .await;

        Ok(())
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        info!("Unloading plugin: {}", plugin_id);

        // Unregister from registry first
        self.registry
            .write()
            .await
            .unregister_plugin(plugin_id)
            .await?;

        // Unload from loader
        self.loader.write().await.unload_plugin(plugin_id)?;

        // Remove from enabled list
        self.enabled.write().await.remove(plugin_id);

        // Emit event
        self.emit_event(PluginEvent::Unloaded {
            plugin_id: plugin_id.to_string(),
        })
        .await;

        Ok(())
    }

    /// Reload a plugin
    pub async fn reload_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        info!("Reloading plugin: {}", plugin_id);

        // Get current configuration
        let config = self
            .configs
            .read()
            .await
            .values()
            .find(|c| c.name == plugin_id)
            .cloned()
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        // Unload and reload
        self.unload_plugin(plugin_id).await?;
        self.load_plugin(config).await?;

        // Emit event
        self.emit_event(PluginEvent::Reloaded {
            plugin_id: plugin_id.to_string(),
        })
        .await;

        Ok(())
    }

    /// Enable a plugin
    pub async fn enable_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        self.enabled
            .write()
            .await
            .insert(plugin_id.to_string(), true);
        Ok(())
    }

    /// Disable a plugin
    pub async fn disable_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        self.enabled
            .write()
            .await
            .insert(plugin_id.to_string(), false);
        Ok(())
    }

    /// Check if a plugin is enabled
    pub async fn is_enabled(&self, plugin_id: &str) -> bool {
        self.enabled
            .read()
            .await
            .get(plugin_id)
            .copied()
            .unwrap_or(false)
    }

    /// Execute a tool from a plugin
    pub async fn execute_tool(&self, tool_name: &str, arguments: Value) -> McpResult<ToolResult> {
        // Find the plugin that provides this tool
        let registry = self.registry.read().await;
        let plugin_id = registry
            .find_plugin_for_tool(tool_name)
            .ok_or_else(|| McpError::ToolNotFound(tool_name.to_string()))?;

        // Check if enabled
        if !self.is_enabled(&plugin_id).await {
            return Err(McpError::Protocol(format!(
                "Plugin {plugin_id} is disabled"
            )));
        }

        // Execute the tool
        registry.execute_tool(tool_name, arguments).await
    }

    /// List all available tools
    pub async fn list_tools(&self) -> Vec<Tool> {
        let registry = self.registry.read().await;
        let enabled = self.enabled.read().await;

        registry
            .list_tools()
            .into_iter()
            .filter(|tool| {
                // Only include tools from enabled plugins
                registry
                    .find_plugin_for_tool(&tool.name)
                    .and_then(|id| enabled.get(&id))
                    .copied()
                    .unwrap_or(false)
            })
            .collect()
    }

    /// List all loaded plugins
    pub async fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.loader.read().await.list_plugins()
    }

    /// Add an event handler
    pub async fn on_event<F>(&self, handler: F)
    where
        F: Fn(PluginEvent) + Send + Sync + 'static,
    {
        self.event_handlers.write().await.push(Box::new(handler));
    }

    /// Emit an event to all handlers
    async fn emit_event(&self, event: PluginEvent) {
        let handlers = self.event_handlers.read().await;
        for handler in handlers.iter() {
            handler(event.clone());
        }
    }

    /// Load all plugins from a configuration directory
    pub async fn load_from_directory(&self, dir: &Path) -> McpResult<()> {
        let config_path = dir.join("plugins.yaml");
        if !config_path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&config_path)
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;

        let configs: Vec<PluginConfig> = serde_yaml::from_str(&content)
            .map_err(|e| McpError::Protocol(format!("Invalid plugin config: {e}")))?;

        for config in configs {
            if config.enabled {
                if let Err(e) = self.load_plugin(config).await {
                    error!("Failed to load plugin: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Add a plugin search path
    pub async fn add_search_path(&self, path: impl Into<std::path::PathBuf>) {
        self.loader.write().await.add_search_path(path);
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

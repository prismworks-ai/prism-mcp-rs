// ! Tool registry for managing plugin tools
// !
// ! Module maintains a registry of all tools provided by plugins.

use crate::core::error::{McpError, McpResult};
use crate::plugin::{PluginError, PluginResult, ToolPlugin, ToolResult};
use crate::protocol::types::Tool;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Registry for plugin-provided tools
pub struct ToolRegistry {
    /// Map of tool names to plugin IDs
    tool_to_plugin: HashMap<String, String>,

    /// Map of plugin IDs to plugin instances
    plugins: HashMap<String, Arc<RwLock<Box<dyn ToolPlugin>>>>,

    /// Tool definitions cache
    tools: HashMap<String, Tool>,

    /// Change notification handler
    change_handler: Option<Box<dyn Fn() + Send + Sync>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tool_to_plugin: HashMap::new(),
            plugins: HashMap::new(),
            tools: HashMap::new(),
            change_handler: None,
        }
    }

    /// Register a plugin and its tool
    pub async fn register_plugin_tool(
        &mut self,
        plugin_id: String,
        plugin: Arc<RwLock<Box<dyn ToolPlugin>>>,
    ) -> PluginResult<()> {
        let tool = {
            let plugin_lock = plugin.read().await;
            plugin_lock.tool_definition()
        };
        info!(
            "Registering tool '{}' from plugin '{}'",
            tool.name, plugin_id
        );

        // Check for conflicts
        if self.tool_to_plugin.contains_key(&tool.name) {
            return Err(PluginError::AlreadyLoaded(format!(
                "Tool '{}' already registered",
                tool.name
            )));
        }

        // Store mappings
        self.tool_to_plugin
            .insert(tool.name.clone(), plugin_id.clone());
        self.plugins.insert(plugin_id, plugin);
        self.tools.insert(tool.name.clone(), tool);

        // Notify change
        self.notify_change();

        Ok(())
    }

    /// Unregister a plugin and its tools
    pub async fn unregister_plugin(&mut self, plugin_id: &str) -> PluginResult<()> {
        info!("Unregistering plugin: {}", plugin_id);

        // Find and remove all tools from this plugin
        let tools_to_remove: Vec<String> = self
            .tool_to_plugin
            .iter()
            .filter(|(_, pid)| *pid == plugin_id)
            .map(|(name, _)| name.clone())
            .collect();

        for tool_name in tools_to_remove {
            self.tool_to_plugin.remove(&tool_name);
            self.tools.remove(&tool_name);
            debug!("Removed tool: {}", tool_name);
        }

        // Remove plugin
        self.plugins.remove(plugin_id);

        // Notify change
        self.notify_change();

        Ok(())
    }

    /// Execute a tool
    pub async fn execute_tool(&self, tool_name: &str, arguments: Value) -> McpResult<ToolResult> {
        // Find the plugin
        let plugin_id = self
            .tool_to_plugin
            .get(tool_name)
            .ok_or_else(|| McpError::ToolNotFound(tool_name.to_string()))?;

        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| McpError::Protocol(format!("Plugin {} not found", plugin_id)))?;

        // Execute the tool
        let plugin_lock = plugin.read().await;
        plugin_lock.execute(arguments).await
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.values().cloned().collect()
    }

    /// Find which plugin provides a tool
    pub fn find_plugin_for_tool(&self, tool_name: &str) -> Option<String> {
        self.tool_to_plugin.get(tool_name).cloned()
    }

    /// Get a specific tool definition
    pub fn get_tool(&self, tool_name: &str) -> Option<&Tool> {
        self.tools.get(tool_name)
    }

    /// Set a change notification handler
    pub fn on_change<F>(&mut self, handler: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.change_handler = Some(Box::new(handler));
    }

    /// Notify about registry changes
    fn notify_change(&self) {
        if let Some(ref handler) = self.change_handler {
            handler();
        }
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            total_plugins: self.plugins.len(),
            total_tools: self.tools.len(),
            tools_per_plugin: self.calculate_tools_per_plugin(),
        }
    }

    fn calculate_tools_per_plugin(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for plugin_id in self.tool_to_plugin.values() {
            *counts.entry(plugin_id.clone()).or_insert(0) += 1;
        }
        counts
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of plugins
    pub total_plugins: usize,

    /// Total number of tools
    pub total_tools: usize,

    /// Number of tools per plugin
    pub tools_per_plugin: HashMap<String, usize>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

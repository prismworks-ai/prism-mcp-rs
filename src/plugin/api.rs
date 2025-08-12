// ! Plugin API definitions
// !
// ! Module defines the core traits and types that plugins must implement
// ! to be compatible with the MCP plugin system.

use crate::core::error::McpResult;
use crate::protocol::types::{CallToolResult as ToolResult, Tool};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;

/// Core trait that all tool plugins must implement
#[async_trait]
pub trait ToolPlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Get tool definition
    fn tool_definition(&self) -> Tool;

    /// Execute the tool with given arguments
    async fn execute(&self, arguments: Value) -> McpResult<ToolResult>;

    /// Initialize the plugin (called once after loading)
    async fn initialize(&mut self) -> McpResult<()> {
        Ok(())
    }

    /// Shutdown the plugin (called before unloading)
    async fn shutdown(&mut self) -> McpResult<()> {
        Ok(())
    }

    /// Check if the plugin is healthy
    async fn health_check(&self) -> McpResult<()> {
        Ok(())
    }

    /// Handle configuration updates
    async fn configure(&mut self, _config: Value) -> McpResult<()> {
        Ok(())
    }

    /// Get plugin as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub id: String,

    /// Plugin name
    pub name: String,

    /// Plugin version (semver)
    pub version: String,

    /// Plugin author
    pub author: Option<String>,

    /// Plugin description
    pub description: Option<String>,

    /// Plugin homepage
    pub homepage: Option<String>,

    /// Plugin license
    pub license: Option<String>,

    /// Required MCP SDK version
    pub mcp_version: String,

    /// Plugin capabilities
    pub capabilities: PluginCapabilities,

    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,
}

/// Plugin capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginCapabilities {
    /// Supports hot reload
    pub hot_reload: bool,

    /// Supports configuration updates
    pub configurable: bool,

    /// Supports health checks
    pub health_check: bool,

    /// Is thread-safe
    pub thread_safe: bool,

    /// Supports multiple instances
    pub multi_instance: bool,

    /// Custom capabilities
    pub custom: Value,
}

/// Plugin dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency plugin ID
    pub plugin_id: String,

    /// Required version range
    pub version: String,

    /// Is optional
    pub optional: bool,
}

/// Plugin factory function type - FIXED signature
pub type PluginFactory = unsafe extern "C" fn() -> *mut Box<dyn ToolPlugin>;

/// Helper trait for plugin builders
pub trait PluginBuilder: Send + Sync {
    /// Build the plugin instance
    fn build(self) -> Box<dyn ToolPlugin>;

    /// Build with configuration
    fn build_with_config(self, _config: Value) -> McpResult<Box<dyn ToolPlugin>>
    where
        Self: Sized,
    {
        Ok(self.build())
    }
}

/// Standard plugin builder implementation
pub struct StandardPluginBuilder<T: ToolPlugin + Default + 'static> {
    config: Option<Value>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ToolPlugin + Default + 'static> Default for StandardPluginBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ToolPlugin + Default + 'static> StandardPluginBuilder<T> {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: Value) -> Self {
        self.config = Some(config);
        self
    }
}

impl<T: ToolPlugin + Default + 'static> PluginBuilder for StandardPluginBuilder<T> {
    fn build(self) -> Box<dyn ToolPlugin> {
        Box::new(T::default())
    }

    fn build_with_config(self, config: Value) -> McpResult<Box<dyn ToolPlugin>> {
        let mut plugin = T::default();
        // Use async runtime to initialize with config
        let runtime = tokio::runtime::Handle::current();
        runtime.block_on(plugin.configure(config))?;
        Ok(Box::new(plugin))
    }
}

/// FIXED Macro to simplify plugin exports with correct ABI
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        /// Plugin creation function with correct C ABI
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn _mcp_plugin_create() -> *mut Box<dyn $crate::plugin::ToolPlugin> {
            let plugin = Box::new(<$plugin_type>::default());
            let boxed: Box<dyn $crate::plugin::ToolPlugin> = plugin;
            Box::into_raw(Box::new(boxed))
        }

        /// Plugin version function
        #[unsafe(no_mangle)]
        pub extern "C" fn _mcp_plugin_version() -> *const u8 {
            concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr()
        }

        /// Plugin metadata function for introspection
        #[unsafe(no_mangle)]
        pub extern "C" fn _mcp_plugin_metadata() -> *const u8 {
            let metadata = <$plugin_type>::default().metadata();
            let json = serde_json::to_string(&metadata).unwrap_or_default();
            let c_str = std::ffi::CString::new(json).unwrap_or_default();
            c_str.into_raw() as *const u8
        }
    };
}

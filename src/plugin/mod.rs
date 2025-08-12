//! Plugin System for Dynamic Tool Loading
//!
//! This module provides a complete plugin system for dynamically loading and managing
//! MCP tools at runtime. It supports:
//!
//! - Dynamic library loading (.so, .dll, .dylib)
//! - Hot reloading without server restart
//! - Configuration-based plugin management
//! - Automatic tool discovery and registration
//! - Plugin isolation and lifecycle management

pub mod api;
pub mod config;
pub mod loader;
pub mod manager;
pub mod registry;
pub mod types;
pub mod watcher;

#[cfg(test)]
mod comprehensive_test;
#[cfg(test)]
mod loader_test;
#[cfg(test)]
mod manager_test;

pub use api::{
    PluginBuilder, PluginCapabilities, PluginDependency, PluginFactory, PluginMetadata,
    StandardPluginBuilder, ToolPlugin,
};
// Re-export ToolResult from protocol types
pub use crate::protocol::types::CallToolResult as ToolResult;
pub use config::{PluginConfig, PluginManifest};
pub use loader::PluginLoader;
pub use manager::PluginManager;
pub use registry::ToolRegistry;
pub use types::*;
pub use watcher::PluginWatcher;

// Re-export the macro
pub use crate::export_plugin;

use crate::core::error::McpError;

/// Plugin system errors
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),

    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Invalid plugin: {0}")]
    InvalidPlugin(String),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    #[error("Plugin communication error: {0}")]
    CommunicationError(String),
}

impl From<PluginError> for McpError {
    fn from(err: PluginError) -> Self {
        McpError::Protocol(err.to_string())
    }
}

/// Plugin lifecycle events
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// Plugin was loaded
    Loaded { plugin_id: String },

    /// Plugin was unloaded
    Unloaded { plugin_id: String },

    /// Plugin was reloaded
    Reloaded { plugin_id: String },

    /// Plugin encountered an error
    Error { plugin_id: String, error: String },

    /// Plugin tool was registered
    ToolRegistered {
        plugin_id: String,
        tool_name: String,
    },

    /// Plugin tool was unregistered
    ToolUnregistered {
        plugin_id: String,
        tool_name: String,
    },
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

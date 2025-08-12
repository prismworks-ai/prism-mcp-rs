//! Dynamic plugin loader
//!
//! Module handles the low-level loading of plugin libraries from disk.

use crate::plugin::{PluginError, PluginMetadata, PluginResult, ToolPlugin};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Plugin loader for dynamic libraries
pub struct PluginLoader {
    /// Loaded libraries
    libraries: HashMap<String, LoadedPlugin>,

    /// Plugin search paths
    search_paths: Vec<PathBuf>,
}

/// Loaded plugin information
struct LoadedPlugin {
    /// The dynamic library
    #[allow(dead_code)]
    library: Library,

    /// Plugin metadata
    metadata: PluginMetadata,

    /// Path to the plugin file
    path: PathBuf,

    /// Plugin instance wrapped in Arc<RwLock>
    instance: Arc<RwLock<Box<dyn ToolPlugin>>>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
            search_paths: vec![
                PathBuf::from("./plugins"),
                PathBuf::from("/usr/local/lib/prism-mcp-plugins"),
                PathBuf::from("~/.mcp/plugins"),
            ],
        }
    }

    /// Add a search path
    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// Load a plugin from a file
    pub fn load_plugin(&mut self, path: &Path) -> PluginResult<Arc<RwLock<Box<dyn ToolPlugin>>>> {
        info!("Loading plugin from: {:?}", path);

        // Check if already loaded
        let path_str = path.to_string_lossy().to_string();
        if self.libraries.contains_key(&path_str) {
            return Err(PluginError::AlreadyLoaded(path_str));
        }

        // Load the library
        let library = unsafe {
            Library::new(path).map_err(|e| {
                error!("Failed to load library: {}", e);
                PluginError::LoadFailed(e.to_string())
            })?
        };

        // Get the plugin creation function with correct signature
        let create_fn: Symbol<unsafe extern "C" fn() -> *mut Box<dyn ToolPlugin>> = unsafe {
            library.get(b"_mcp_plugin_create\0").map_err(|e| {
                error!("Plugin missing _mcp_plugin_create function: {}", e);
                PluginError::InvalidPlugin("Missing _mcp_plugin_create export".to_string())
            })?
        };

        // Create plugin instance safely
        let instance = unsafe {
            let raw_box = create_fn();
            if raw_box.is_null() {
                return Err(PluginError::InvalidPlugin(
                    "Plugin creation returned null".to_string(),
                ));
            }
            *Box::from_raw(raw_box)
        };

        // Skip initialization here - it will be done by the manager in an async context

        // Get metadata
        let metadata = instance.metadata();
        info!("Loaded plugin: {} v{}", metadata.name, metadata.version);

        // Store the loaded plugin
        let instance_arc = Arc::new(RwLock::new(instance));
        let loaded = LoadedPlugin {
            library,
            metadata: metadata.clone(),
            path: path.to_path_buf(),
            instance: instance_arc.clone(),
        };

        self.libraries.insert(path_str, loaded);
        Ok(instance_arc)
    }

    /// Unload a plugin
    pub fn unload_plugin(&mut self, plugin_id: &str) -> PluginResult<()> {
        info!("Unloading plugin: {}", plugin_id);

        // Find and remove the plugin
        let path_to_remove = self
            .libraries
            .iter()
            .find(|(_, p)| p.metadata.id == plugin_id)
            .map(|(path, _)| path.clone());

        if let Some(path) = path_to_remove {
            // Remove from libraries map
            if let Some(loaded) = self.libraries.remove(&path) {
                // Note: The plugin will be properly shut down by the manager
                // before this is called
                info!("Plugin {} unloaded", loaded.metadata.id);
                Ok(())
            } else {
                Err(PluginError::NotFound(plugin_id.to_string()))
            }
        } else {
            Err(PluginError::NotFound(plugin_id.to_string()))
        }
    }

    /// Reload a plugin - returns the existing instance if reload is not needed
    pub fn reload_plugin(
        &mut self,
        plugin_id: &str,
    ) -> PluginResult<Arc<RwLock<Box<dyn ToolPlugin>>>> {
        info!("Reloading plugin: {}", plugin_id);

        // Find the plugin path and get existing instance
        let (_plugin_path, existing_instance) = self
            .libraries
            .iter()
            .find(|(_, p)| p.metadata.id == plugin_id)
            .map(|(_, p)| (p.path.clone(), p.instance.clone()))
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        // For now, return the existing instance to avoid runtime issues
        // In production, you'd implement proper hot-reloading here
        info!(
            "Plugin reload requested for {}, returning existing instance",
            plugin_id
        );
        Ok(existing_instance)
    }

    /// Find a plugin file by name
    pub fn find_plugin(&self, name: &str) -> Option<PathBuf> {
        let extensions = if cfg!(windows) {
            vec!["dll"]
        } else if cfg!(target_os = "macos") {
            vec!["dylib"]
        } else {
            vec!["so"]
        };

        for search_path in &self.search_paths {
            for ext in &extensions {
                let path = search_path.join(format!("{name}.{ext}"));
                if path.exists() {
                    return Some(path);
                }

                // Also try with lib prefix on Unix
                if !cfg!(windows) {
                    let path = search_path.join(format!("lib{name}.{ext}"));
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    /// Get a plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<Arc<RwLock<Box<dyn ToolPlugin>>>> {
        self.libraries
            .values()
            .find(|p| p.metadata.id == plugin_id)
            .map(|p| p.instance.clone())
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.libraries
            .values()
            .map(|p| p.metadata.clone())
            .collect()
    }

    /// Get plugin by path
    pub fn get_plugin_by_path(&self, path: &str) -> Option<Arc<RwLock<Box<dyn ToolPlugin>>>> {
        self.libraries.get(path).map(|p| p.instance.clone())
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Drop is handled automatically by libloading
// The Library type will unload the dynamic library when dropped

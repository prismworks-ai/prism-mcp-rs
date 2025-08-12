// ! File system watcher for plugin hot reload
// !
// ! Module watches plugin files for changes and triggers automatic reloads.

use crate::plugin::{PluginError, PluginManager, PluginResult};
use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Plugin file watcher for hot reload
pub struct PluginWatcher {
    /// File system watcher
    watcher: Option<notify::RecommendedWatcher>,

    /// Plugin manager reference
    manager: Arc<PluginManager>,

    /// Watched paths and their plugin IDs
    watched_paths: Arc<RwLock<HashMap<PathBuf, String>>>,

    /// Reload debounce (milliseconds)
    debounce_ms: u64,

    /// Last reload times for debouncing
    last_reload: Arc<RwLock<HashMap<String, std::time::Instant>>>,
}

impl PluginWatcher {
    /// Create a new plugin watcher
    pub fn new(manager: Arc<PluginManager>) -> Self {
        Self {
            watcher: None,
            manager,
            watched_paths: Arc::new(RwLock::new(HashMap::new())),
            debounce_ms: 500,
            last_reload: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start watching for plugin changes
    pub async fn start(&mut self) -> PluginResult<()> {
        let watched_paths = self.watched_paths.clone();
        let manager = self.manager.clone();
        let last_reload = self.last_reload.clone();
        let debounce_ms = self.debounce_ms;

        // Create the watcher
        let (tx, rx) = std::sync::mpsc::channel();

        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if let Err(e) = tx.send(event) {
                    error!("Failed to send watch event: {}", e);
                }
            }
        })
        .map_err(|e| PluginError::LoadFailed(format!("Failed to create watcher: {e}")))?;

        // Spawn event handler task
        let _handle = tokio::spawn(async move {
            while let Ok(event) = rx.recv() {
                Self::handle_event(event, &watched_paths, &manager, &last_reload, debounce_ms)
                    .await;
            }
        });

        self.watcher = Some(watcher);

        info!("Plugin watcher started");
        Ok(())
    }

    /// Stop watching
    pub fn stop(&mut self) {
        self.watcher = None;
        info!("Plugin watcher stopped");
    }

    /// Watch a plugin file
    pub async fn watch_plugin(&mut self, path: &Path, plugin_id: String) -> PluginResult<()> {
        if let Some(ref mut watcher) = self.watcher {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| PluginError::LoadFailed(format!("Failed to watch path: {e}")))?;

            self.watched_paths
                .write()
                .await
                .insert(path.to_path_buf(), plugin_id.clone());

            info!("Watching plugin file: {:?} ({})", path, plugin_id);
            Ok(())
        } else {
            Err(PluginError::LoadFailed("Watcher not started".to_string()))
        }
    }

    /// Unwatch a plugin file
    pub async fn unwatch_plugin(&mut self, path: &Path) -> PluginResult<()> {
        if let Some(ref mut watcher) = self.watcher {
            watcher
                .unwatch(path)
                .map_err(|e| PluginError::LoadFailed(format!("Failed to unwatch path: {e}")))?;

            self.watched_paths.write().await.remove(path);

            info!("Stopped watching plugin file: {:?}", path);
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Watch a directory for new plugins
    pub async fn watch_directory(&mut self, path: &Path) -> PluginResult<()> {
        if let Some(ref mut watcher) = self.watcher {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| PluginError::LoadFailed(format!("Failed to watch directory: {e}")))?;

            info!("Watching plugin directory: {:?}", path);
            Ok(())
        } else {
            Err(PluginError::LoadFailed("Watcher not started".to_string()))
        }
    }

    /// Handle file system events
    async fn handle_event(
        event: Event,
        watched_paths: &Arc<RwLock<HashMap<PathBuf, String>>>,
        manager: &Arc<PluginManager>,
        last_reload: &Arc<RwLock<HashMap<String, std::time::Instant>>>,
        debounce_ms: u64,
    ) {
        match event.kind {
            EventKind::Modify(ModifyKind::Data(_)) | EventKind::Modify(ModifyKind::Any) => {
                // File was modified
                for path in event.paths {
                    if let Some(plugin_id) = watched_paths.read().await.get(&path) {
                        // Check debounce
                        let should_reload = {
                            let mut last = last_reload.write().await;
                            let now = std::time::Instant::now();

                            if let Some(last_time) = last.get(plugin_id) {
                                if now.duration_since(*last_time).as_millis() < debounce_ms as u128
                                {
                                    false
                                } else {
                                    last.insert(plugin_id.clone(), now);
                                    true
                                }
                            } else {
                                last.insert(plugin_id.clone(), now);
                                true
                            }
                        };

                        if should_reload {
                            info!("Plugin file changed, reloading: {}", plugin_id);
                            if let Err(e) = manager.reload_plugin(plugin_id).await {
                                error!("Failed to reload plugin {}: {}", plugin_id, e);
                            }
                        } else {
                            debug!("Skipping reload due to debounce: {}", plugin_id);
                        }
                    }
                }
            }

            EventKind::Create(CreateKind::File) => {
                // New file created in watched directory
                for path in event.paths {
                    if Self::is_plugin_file(&path) {
                        info!("New plugin file detected: {:?}", path);
                        // Could auto-load if configured
                    }
                }
            }

            EventKind::Remove(RemoveKind::File) => {
                // File was removed
                for path in event.paths {
                    if let Some(plugin_id) = watched_paths.read().await.get(&path) {
                        warn!("Plugin file removed: {} ({:?})", plugin_id, path);
                        // Could auto-unload if configured
                    }
                }
            }

            _ => {
                // Ignore other events
            }
        }
    }

    /// Check if a path is a plugin file
    fn is_plugin_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            matches!(ext_str.as_ref(), "so" | "dll" | "dylib")
        } else {
            false
        }
    }

    /// Set debounce time in milliseconds
    pub fn set_debounce(&mut self, ms: u64) {
        self.debounce_ms = ms;
    }

    /// Get watched paths
    pub async fn get_watched_paths(&self) -> Vec<(PathBuf, String)> {
        self.watched_paths
            .read()
            .await
            .iter()
            .map(|(p, id)| (p.clone(), id.clone()))
            .collect()
    }
}

impl Drop for PluginWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

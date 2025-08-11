// ! Comprehensive tests for plugin modules to achieve 90% coverage

#[cfg(test)]
mod tests {
    use crate::core::error::McpError;
    use crate::plugin::api::PluginDependency;
    use crate::plugin::watcher::PluginWatcher;
    use crate::plugin::*;
    use serde_json::{Value, json};
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    // ==================== PluginError Tests ====================

    #[test]
    fn test_plugin_error_all_variants() {
        // Test all error variants and their messages
        let err = PluginError::LoadFailed("failed to load".to_string());
        assert_eq!(err.to_string(), "Plugin load failed: failed to load");

        let err = PluginError::NotFound("plugin123".to_string());
        assert_eq!(err.to_string(), "Plugin not found: plugin123");

        let err = PluginError::AlreadyLoaded("plugin456".to_string());
        assert_eq!(err.to_string(), "Plugin already loaded: plugin456");

        let err = PluginError::InitializationFailed("init failed".to_string());
        assert_eq!(err.to_string(), "Plugin initialization failed: init failed");

        let err = PluginError::InvalidPlugin("invalid".to_string());
        assert_eq!(err.to_string(), "Invalid plugin: invalid");

        let err = PluginError::VersionMismatch {
            expected: "1.0.0".to_string(),
            actual: "2.0.0".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Version mismatch: expected 1.0.0, got 2.0.0"
        );

        let err = PluginError::MissingDependency("dep1".to_string());
        assert_eq!(err.to_string(), "Missing dependency: dep1");

        let err = PluginError::CommunicationError("comm error".to_string());
        assert_eq!(err.to_string(), "Plugin communication error: comm error");
    }

    #[test]
    fn test_plugin_error_to_mcp_error_conversion() {
        let plugin_err = PluginError::LoadFailed("test".to_string());
        let mcp_err: McpError = plugin_err.into();

        match mcp_err {
            McpError::Protocol(msg) => assert!(msg.contains("Plugin load failed")),
            _ => panic!("Expected Protocol error"),
        }

        // Test another variant
        let plugin_err = PluginError::NotFound("missing".to_string());
        let mcp_err: McpError = plugin_err.into();
        match mcp_err {
            McpError::Protocol(msg) => assert!(msg.contains("Plugin not found")),
            _ => panic!("Expected Protocol error"),
        }
    }

    // ==================== PluginEvent Tests ====================

    #[test]
    fn test_plugin_event_clone() {
        let event1 = PluginEvent::Loaded {
            plugin_id: "test".to_string(),
        };
        let event2 = event1.clone();

        match event2 {
            PluginEvent::Loaded { plugin_id } => assert_eq!(plugin_id, "test"),
            _ => panic!("Clone failed"),
        }

        // Test Error event cloning
        let event = PluginEvent::Error {
            plugin_id: "test".to_string(),
            error: "error msg".to_string(),
        };
        let cloned = event.clone();
        match cloned {
            PluginEvent::Error { plugin_id, error } => {
                assert_eq!(plugin_id, "test");
                assert_eq!(error, "error msg");
            }
            _ => panic!("Clone failed"),
        }
    }

    // ==================== Plugin Types Tests ====================

    #[test]
    fn test_plugin_config_default() {
        let config = types::PluginConfig::default();
        assert!(config.enabled);
        assert!(!config.auto_reload);
        assert!(config.config_path.is_none());
        assert_eq!(config.settings, Value::Null);
    }

    #[test]
    fn test_plugin_error_from_io_error() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let plugin_err: types::PluginError = io_err.into();

        match plugin_err {
            types::PluginError::Io(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected Io error variant"),
        }

        // Test another IO error kind
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let plugin_err: types::PluginError = io_err.into();
        match plugin_err {
            types::PluginError::Io(msg) => assert!(msg.contains("access denied")),
            _ => panic!("Expected Io error variant"),
        }
    }

    // ==================== Plugin Watcher Tests ====================

    #[tokio::test]
    async fn test_plugin_watcher_creation() {
        let manager = Arc::new(PluginManager::new());
        let watcher = PluginWatcher::new(manager);

        // Test getting watched paths when empty
        let paths = watcher.get_watched_paths().await;
        assert_eq!(paths.len(), 0);
    }

    #[tokio::test]
    async fn test_plugin_watcher_debounce() {
        let manager = Arc::new(PluginManager::new());
        let mut watcher = PluginWatcher::new(manager);

        // Test setting different debounce values
        watcher.set_debounce(100);
        watcher.set_debounce(500);
        watcher.set_debounce(1000);
        // Method should complete without error
    }

    #[tokio::test]
    async fn test_plugin_watcher_watch_methods() {
        let manager = Arc::new(PluginManager::new());
        let mut watcher = PluginWatcher::new(manager);

        // Test that watch methods fail when watcher not started
        let result = watcher
            .watch_plugin(Path::new("/test/plugin.so"), "test".to_string())
            .await;
        assert!(result.is_err());

        let result = watcher.watch_directory(Path::new("/test/plugins")).await;
        assert!(result.is_err());

        // Test unwatch when not started (should succeed)
        let result = watcher.unwatch_plugin(Path::new("/test/plugin.so")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_plugin_watcher_stop() {
        let manager = Arc::new(PluginManager::new());
        let mut watcher = PluginWatcher::new(manager);

        // Test stopping when not started
        watcher.stop();
        // Should complete without error

        // Test drop behavior
        drop(watcher);
        // Should clean up properly
    }

    // ==================== ToolRegistry Tests ====================

    #[tokio::test]
    async fn test_tool_registry_operations() {
        let registry = ToolRegistry::new();

        // Test initial empty state
        assert_eq!(registry.list_tools().len(), 0);
        assert!(registry.get_tool("nonexistent").is_none());
        assert!(registry.find_plugin_for_tool("nonexistent").is_none());

        // Test stats on empty registry
        let stats = registry.stats();
        assert_eq!(stats.total_plugins, 0);
        assert_eq!(stats.total_tools, 0);
        assert!(stats.tools_per_plugin.is_empty());
    }

    #[tokio::test]
    async fn test_registry_change_notification_setup() {
        let mut registry = ToolRegistry::new();
        let changed = Arc::new(AtomicBool::new(false));
        let changed_clone = changed.clone();

        // Set change handler
        registry.on_change(move || {
            changed_clone.store(true, Ordering::SeqCst);
        });

        // Handler is set but shouldn't be triggered yet
        assert!(!changed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_registry_default_trait() {
        let registry1 = ToolRegistry::new();
        let registry2 = ToolRegistry::default();

        // Both should create empty registries
        assert_eq!(registry1.list_tools().len(), 0);
        assert_eq!(registry2.list_tools().len(), 0);
    }

    // ==================== Config Types Tests ====================

    #[test]
    fn test_isolation_level_default() {
        use crate::plugin::config::IsolationLevel;
        let level = IsolationLevel::default();
        assert!(matches!(level, IsolationLevel::None));
    }

    #[test]
    fn test_plugin_config_simple_builder() {
        use crate::plugin::config::PluginConfig;

        let config = PluginConfig::simple("test_plugin");
        assert_eq!(config.name, "test_plugin");
        assert!(config.enabled);
        assert_eq!(config.priority, 100);
        assert!(!config.auto_reload);

        // Test builder methods
        let config = PluginConfig::simple("test")
            .with_path("/path/to/plugin")
            .with_auto_reload();
        assert_eq!(config.path, Some("/path/to/plugin".to_string()));
        assert!(config.auto_reload);

        let config_value = json!({"key": "value"});
        let config = PluginConfig::simple("test").with_config(config_value.clone());
        assert_eq!(config.config, Some(config_value));
    }

    #[tokio::test]
    async fn test_plugin_manifest_from_file_nonexistent() {
        use crate::plugin::config::PluginManifest;

        let result = PluginManifest::from_file("/nonexistent/manifest.yaml").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plugin_config_set_from_file_nonexistent() {
        use crate::plugin::config::PluginConfigSet;

        let result = PluginConfigSet::from_file("/nonexistent/config.yaml").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_config_set_sort_by_priority() {
        use crate::plugin::config::{PluginConfig, PluginConfigSet};

        let mut config_set = PluginConfigSet {
            plugins: vec![
                PluginConfig {
                    name: "plugin1".to_string(),
                    enabled: true,
                    path: None,
                    config: None,
                    env: std::collections::HashMap::new(),
                    auto_reload: false,
                    priority: 200,
                },
                PluginConfig {
                    name: "plugin2".to_string(),
                    enabled: true,
                    path: None,
                    config: None,
                    env: std::collections::HashMap::new(),
                    auto_reload: false,
                    priority: 50,
                },
                PluginConfig {
                    name: "plugin3".to_string(),
                    enabled: true,
                    path: None,
                    config: None,
                    env: std::collections::HashMap::new(),
                    auto_reload: false,
                    priority: 100,
                },
            ],
            settings: None,
        };

        config_set.sort_by_priority();

        // Should be sorted by priority (ascending)
        assert_eq!(config_set.plugins[0].name, "plugin2"); // priority 50
        assert_eq!(config_set.plugins[1].name, "plugin3"); // priority 100
        assert_eq!(config_set.plugins[2].name, "plugin1"); // priority 200
    }

    // ==================== API Types Tests ====================

    #[test]
    fn test_plugin_dependency_creation() {
        let dep = PluginDependency {
            plugin_id: "dep1".to_string(),
            version: "^1.0.0".to_string(),
            optional: false,
        };

        assert_eq!(dep.plugin_id, "dep1");
        assert_eq!(dep.version, "^1.0.0");
        assert!(!dep.optional);

        let optional_dep = PluginDependency {
            plugin_id: "dep2".to_string(),
            version: "~2.0.0".to_string(),
            optional: true,
        };

        assert!(optional_dep.optional);
    }

    #[test]
    fn test_plugin_capabilities_default() {
        use crate::plugin::api::PluginCapabilities;

        let caps = PluginCapabilities::default();
        assert!(!caps.hot_reload);
        assert!(!caps.configurable);
        assert!(!caps.health_check);
        assert!(!caps.thread_safe);
        assert!(!caps.multi_instance);
        assert_eq!(caps.custom, Value::Null);
    }

    // ==================== PluginLoader Send/Sync Tests ====================

    #[test]
    fn test_plugin_loader_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PluginLoader>();
    }

    // ==================== PluginManager Tests ====================

    #[tokio::test]
    async fn test_plugin_manager_load_with_invalid_config() {
        let manager = PluginManager::new();

        // Test loading with a config that has no path and non-existent plugin
        let config = PluginConfig {
            name: "nonexistent".to_string(),
            enabled: true,
            path: None,
            config: None,
            env: std::collections::HashMap::new(),
            auto_reload: false,
            priority: 100,
        };

        let result = manager.load_plugin(config).await;
        assert!(result.is_err());
    }
}

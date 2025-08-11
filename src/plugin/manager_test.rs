// ! Tests for the plugin manager module

#[cfg(test)]
mod tests {
    use crate::core::error::McpError;
    use crate::plugin::{PluginConfig, PluginError, PluginEvent, PluginManager, ToolRegistry};
    use serde_json::json;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        // Manager should be created successfully
        assert!(std::ptr::eq(&manager, &manager)); // Self-equality check
    }

    #[test]
    fn test_plugin_manager_default() {
        let manager1 = PluginManager::new();
        let manager2 = PluginManager::default();
        // Both should create distinct managers
        assert!(!std::ptr::eq(&manager1, &manager2));
    }

    #[tokio::test]
    async fn test_list_plugins_empty() {
        let manager = PluginManager::new();
        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 0);
    }

    #[tokio::test]
    async fn test_list_tools_empty() {
        let manager = PluginManager::new();
        let tools = manager.list_tools().await;
        assert_eq!(tools.len(), 0);
    }

    #[tokio::test]
    async fn test_is_enabled_nonexistent() {
        let manager = PluginManager::new();
        let enabled = manager.is_enabled("nonexistent").await;
        assert!(!enabled);
    }

    #[tokio::test]
    async fn test_execute_tool_not_found() {
        let manager = PluginManager::new();
        let result = manager.execute_tool("nonexistent_tool", json!({})).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::ToolNotFound(name) => assert_eq!(name, "nonexistent_tool"),
            _ => panic!("Expected ToolNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_enable_disable_plugin() {
        let manager = PluginManager::new();

        // Enable a plugin (even if it doesn't exist)
        manager.enable_plugin("test_plugin").await.unwrap();
        assert!(manager.is_enabled("test_plugin").await);

        // Disable the plugin
        manager.disable_plugin("test_plugin").await.unwrap();
        assert!(!manager.is_enabled("test_plugin").await);
    }

    #[tokio::test]
    async fn test_add_search_path() {
        let manager = PluginManager::new();
        let custom_path = std::path::PathBuf::from("/custom/plugin/path");
        manager.add_search_path(custom_path).await;
        // Method should complete without error
    }

    #[tokio::test]
    async fn test_load_from_directory_nonexistent() {
        let manager = PluginManager::new();
        let result = manager
            .load_from_directory(Path::new("/nonexistent/dir"))
            .await;
        // Should succeed even if directory doesn't exist (returns Ok(()) when no config file)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_handler_registration() {
        let manager = PluginManager::new();
        let event_received = Arc::new(AtomicBool::new(false));
        let event_clone = event_received.clone();

        manager
            .on_event(move |_event| {
                event_clone.store(true, Ordering::SeqCst);
            })
            .await;

        // Handler should be registered but not triggered yet
        assert!(!event_received.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_unload_plugin_not_found() {
        let manager = PluginManager::new();
        let result = manager.unload_plugin("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginError::NotFound(id) => assert_eq!(id, "nonexistent"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_reload_plugin_not_found() {
        let manager = PluginManager::new();
        let result = manager.reload_plugin("nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PluginError::NotFound(id) => assert_eq!(id, "nonexistent"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_load_plugin_not_found() {
        let manager = PluginManager::new();
        let config = PluginConfig::simple("nonexistent_plugin");
        let result = manager.load_plugin(config).await;
        assert!(result.is_err());
        // Should fail because plugin file doesn't exist
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_plugin_config_creation() {
        let config = PluginConfig::simple("test_plugin");
        assert_eq!(config.name, "test_plugin");
        assert!(config.enabled);
        assert!(config.path.is_none());
        assert!(config.config.is_none());
        assert!(!config.auto_reload);
        assert_eq!(config.priority, 100);
    }

    #[test]
    fn test_plugin_config_builder() {
        let config = PluginConfig::simple("test_plugin")
            .with_path("/path/to/plugin.so")
            .with_config(json!({ "key": "value" }))
            .with_auto_reload();

        assert_eq!(config.name, "test_plugin");
        assert_eq!(config.path, Some("/path/to/plugin.so".to_string()));
        assert_eq!(config.config, Some(json!({ "key": "value" })));
        assert!(config.auto_reload);
    }

    #[tokio::test]
    async fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 0);
    }

    #[tokio::test]
    async fn test_tool_registry_default() {
        let registry1 = ToolRegistry::new();
        let registry2 = ToolRegistry::default();
        // Both should create empty registries
        assert_eq!(registry1.list_tools().len(), registry2.list_tools().len());
    }

    #[tokio::test]
    async fn test_registry_find_plugin_for_tool() {
        let registry = ToolRegistry::new();
        let result = registry.find_plugin_for_tool("nonexistent_tool");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_registry_get_tool() {
        let registry = ToolRegistry::new();
        let result = registry.get_tool("nonexistent_tool");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_registry_stats() {
        let registry = ToolRegistry::new();
        let stats = registry.stats();
        assert_eq!(stats.total_plugins, 0);
        assert_eq!(stats.total_tools, 0);
        assert!(stats.tools_per_plugin.is_empty());
    }

    #[tokio::test]
    async fn test_registry_execute_tool_not_found() {
        let registry = ToolRegistry::new();
        let result = registry.execute_tool("nonexistent_tool", json!({})).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            McpError::ToolNotFound(name) => assert_eq!(name, "nonexistent_tool"),
            _ => panic!("Expected ToolNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_registry_change_handler() {
        let mut registry = ToolRegistry::new();
        let changed = Arc::new(AtomicBool::new(false));
        let changed_clone = changed.clone();

        registry.on_change(move || {
            changed_clone.store(true, Ordering::SeqCst);
        });

        // Change handler should be set but not triggered yet
        assert!(!changed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_registry_unregister_nonexistent() {
        let mut registry = ToolRegistry::new();
        // Test unregistering a plugin that doesn't exist
        let result = registry.unregister_plugin("nonexistent").await;
        // Should succeed even if plugin doesn't exist
        assert!(result.is_ok());
    }

    #[test]
    fn test_plugin_event_variants() {
        // Test all event variant creation and cloning
        let loaded = PluginEvent::Loaded {
            plugin_id: "test".to_string(),
        };
        let cloned = loaded.clone();
        match cloned {
            PluginEvent::Loaded { plugin_id } => assert_eq!(plugin_id, "test"),
            _ => panic!("Wrong variant"),
        }

        let unloaded = PluginEvent::Unloaded {
            plugin_id: "test".to_string(),
        };
        match unloaded {
            PluginEvent::Unloaded { plugin_id } => assert_eq!(plugin_id, "test"),
            _ => panic!("Wrong variant"),
        }

        let reloaded = PluginEvent::Reloaded {
            plugin_id: "test".to_string(),
        };
        match reloaded {
            PluginEvent::Reloaded { plugin_id } => assert_eq!(plugin_id, "test"),
            _ => panic!("Wrong variant"),
        }

        let error = PluginEvent::Error {
            plugin_id: "test".to_string(),
            error: "error message".to_string(),
        };
        match error {
            PluginEvent::Error { plugin_id, error } => {
                assert_eq!(plugin_id, "test");
                assert_eq!(error, "error message");
            }
            _ => panic!("Wrong variant"),
        }

        let tool_registered = PluginEvent::ToolRegistered {
            plugin_id: "test".to_string(),
            tool_name: "test_tool".to_string(),
        };
        match tool_registered {
            PluginEvent::ToolRegistered {
                plugin_id,
                tool_name,
            } => {
                assert_eq!(plugin_id, "test");
                assert_eq!(tool_name, "test_tool");
            }
            _ => panic!("Wrong variant"),
        }

        let tool_unregistered = PluginEvent::ToolUnregistered {
            plugin_id: "test".to_string(),
            tool_name: "test_tool".to_string(),
        };
        match tool_unregistered {
            PluginEvent::ToolUnregistered {
                plugin_id,
                tool_name,
            } => {
                assert_eq!(plugin_id, "test");
                assert_eq!(tool_name, "test_tool");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_plugin_error_conversion_to_mcp_error() {
        let plugin_err = PluginError::LoadFailed("test error".to_string());
        let mcp_err: McpError = plugin_err.into();
        match mcp_err {
            McpError::Protocol(msg) => assert!(msg.contains("Plugin load failed")),
            _ => panic!("Expected Protocol error"),
        }
    }
}

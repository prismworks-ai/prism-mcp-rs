// ! Tests for the plugin loader module

#[cfg(test)]
mod tests {
    use crate::plugin::{PluginError, PluginLoader};

    #[test]
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::new();
        // Test that a new loader starts empty
        let plugins = loader.list_plugins();
        assert_eq!(plugins.len(), 0);
    }

    #[test]
    fn test_plugin_loader_default() {
        let loader = PluginLoader::default();
        // Test default trait implementation
        assert_eq!(loader.list_plugins().len(), 0);
    }

    #[test]
    fn test_add_search_path() {
        let mut loader = PluginLoader::new();
        // Test adding custom search paths
        loader.add_search_path("/custom/path");
        loader.add_search_path("./local/plugins");
        // Method should complete without panic
    }

    #[test]
    fn test_find_plugin_nonexistent() {
        let loader = PluginLoader::new();
        // Test finding a plugin that doesn't exist
        let result = loader.find_plugin("nonexistent_plugin");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_plugin_not_found() {
        let loader = PluginLoader::new();
        // Test getting a plugin that hasn't been loaded
        let result = loader.get_plugin("non_existent");
        assert!(result.is_none());
    }

    #[test]
    fn test_list_loaded_empty() {
        let loader = PluginLoader::new();
        // Test listing plugins when none are loaded
        let plugins = loader.list_plugins();
        assert_eq!(plugins.len(), 0);
    }

    #[test]
    fn test_unload_plugin_not_found() {
        let mut loader = PluginLoader::new();
        // Test unloading a plugin that doesn't exist
        let result = loader.unload_plugin("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_reload_plugin_not_found() {
        let mut loader = PluginLoader::new();
        // Test reloading a plugin that doesn't exist
        let result = loader.reload_plugin("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_load_plugin_invalid_path() {
        let mut loader = PluginLoader::new();
        // Test loading from an invalid path
        let result = loader.load_plugin(std::path::Path::new("/nonexistent/plugin.so"));
        assert!(matches!(result, Err(PluginError::LoadFailed(_))));
    }

    #[test]
    fn test_plugin_error_variants() {
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
    fn test_plugin_error_to_mcp_error() {
        use crate::core::error::McpError;

        let plugin_err = PluginError::LoadFailed("test".to_string());
        let mcp_err: McpError = plugin_err.into();

        match mcp_err {
            McpError::Protocol(msg) => assert!(msg.contains("Plugin load failed")),
            _ => panic!("Expected Protocol error"),
        }
    }
}

//! Integration tests for the plugin system
//!
//! These tests verify the plugin system types and basic functionality.
//! Note: Actual dynamic library loading requires compiled plugin binaries.

#![cfg(feature = "plugin")]

use prism_mcp_rs::plugin::{
    config::{PluginConfig, PluginConfigSet},
    types::{PluginCapabilities, PluginInfo, PluginMetadata},
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_plugin_config_creation() {
    // Test creating a basic plugin config
    let config = PluginConfig {
        name: "test_plugin".to_string(),
        enabled: true,
        path: Some("/path/to/plugin.so".to_string()),
        config: Some(json!({
            "max_value": 100.0,
            "debug": true
        })),
        priority: 100,
        env: HashMap::new(),
        auto_reload: false,
    };

    assert_eq!(config.name, "test_plugin");
    assert!(config.enabled);
    assert_eq!(config.priority, 100);
    assert!(!config.auto_reload);
}

#[test]
fn test_plugin_capabilities() {
    // Test plugin capabilities struct
    let capabilities = PluginCapabilities {
        provides_tools: true,
        provides_resources: false,
        provides_prompts: false,
        supports_hot_reload: true,
    };

    assert!(capabilities.provides_tools);
    assert!(!capabilities.provides_resources);
    assert!(!capabilities.provides_prompts);
    assert!(capabilities.supports_hot_reload);
}

#[test]
fn test_plugin_metadata() {
    // Test plugin metadata struct
    let metadata = PluginMetadata {
        author: Some("Test Author".to_string()),
        license: Some("MIT".to_string()),
        homepage: Some("https://example.com".to_string()),
        repository: Some("https://github.com/example/plugin".to_string()),
        keywords: vec!["test".to_string(), "plugin".to_string()],
        categories: vec!["utility".to_string()],
    };

    assert_eq!(metadata.author, Some("Test Author".to_string()));
    assert_eq!(metadata.license, Some("MIT".to_string()));
    assert_eq!(metadata.keywords.len(), 2);
    assert_eq!(metadata.categories.len(), 1);
}

#[test]
fn test_plugin_info() {
    // Test plugin info struct
    let info = PluginInfo {
        name: "test_plugin".to_string(),
        version: "1.0.0".to_string(),
        path: "/path/to/plugin.so".into(),
        enabled: true,
        capabilities: PluginCapabilities::default(),
        metadata: PluginMetadata::default(),
    };

    assert_eq!(info.name, "test_plugin");
    assert_eq!(info.version, "1.0.0");
    assert!(info.enabled);
}

#[test]
fn test_plugin_config_set() {
    // Test plugin config set
    let mut config_set = PluginConfigSet {
        plugins: vec![
            PluginConfig {
                name: "plugin1".to_string(),
                enabled: true,
                path: None,
                config: None,
                priority: 100,
                env: HashMap::new(),
                auto_reload: false,
            },
            PluginConfig {
                name: "plugin2".to_string(),
                enabled: false,
                path: None,
                config: None,
                priority: 50,
                env: HashMap::new(),
                auto_reload: true,
            },
        ],
        settings: None,
    };

    assert_eq!(config_set.plugins.len(), 2);

    // Test sorting by priority (sorts in ascending order - lower priority value first)
    config_set.sort_by_priority();
    assert_eq!(config_set.plugins[0].name, "plugin2"); // priority 50 comes first
    assert_eq!(config_set.plugins[1].name, "plugin1"); // priority 100 comes second
}

#[test]
fn test_plugin_config_with_environment() {
    // Test plugin config with environment variables
    let mut env = HashMap::new();
    env.insert("PLUGIN_MODE".to_string(), "production".to_string());
    env.insert("API_KEY".to_string(), "secret123".to_string());

    let config = PluginConfig {
        name: "env_plugin".to_string(),
        enabled: true,
        path: Some("/path/to/plugin.so".to_string()),
        config: None,
        priority: 100,
        env,
        auto_reload: false,
    };

    assert_eq!(config.env.len(), 2);
    assert_eq!(
        config.env.get("PLUGIN_MODE"),
        Some(&"production".to_string())
    );
    assert_eq!(config.env.get("API_KEY"), Some(&"secret123".to_string()));
}

// Note: The following tests would require actual plugin loading functionality
// which depends on having compiled plugin binaries available.
// These are commented out to avoid failures in CI/CD environments.

/*
#[tokio::test]
async fn test_plugin_load_and_execute() {
    // This test would require a compiled plugin binary
    // It's disabled by default to avoid CI failures

    // Example of what the test would look like:
    // let manager = PluginManager::new();
    // let config = PluginConfig { ... };
    // manager.load_plugin(config).await.unwrap();
    // let result = manager.execute_tool("tool_name", args).await.unwrap();
    // assert!(result.is_ok());
}

#[tokio::test]
async fn test_plugin_reload() {
    // This test would require a compiled plugin binary
    // It's disabled by default to avoid CI failures
}

#[tokio::test]
async fn test_plugin_error_handling() {
    // This test would require a compiled plugin binary
    // It's disabled by default to avoid CI failures
}

#[tokio::test]
async fn test_plugin_configuration() {
    // This test would require a compiled plugin binary
    // It's disabled by default to avoid CI failures
}

#[tokio::test]
async fn test_multiple_plugins() {
    // This test would require compiled plugin binaries
    // It's disabled by default to avoid CI failures
}
*/

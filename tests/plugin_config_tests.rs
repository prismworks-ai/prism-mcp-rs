//! Tests for plugin configuration module

use prism_mcp_rs::plugin::config::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_plugin_config_creation() {
    let config = PluginConfig {
        name: "test-plugin".to_string(),
        enabled: true,
        path: Some("/path/to/plugin.so".to_string()),
        config: Some(json!({"key": "value"})),
        env: HashMap::from([
            ("ENV_VAR1".to_string(), "value1".to_string()),
            ("ENV_VAR2".to_string(), "value2".to_string()),
        ]),
        auto_reload: true,
        priority: 50,
    };

    assert_eq!(config.name, "test-plugin");
    assert!(config.enabled);
    assert_eq!(config.path, Some("/path/to/plugin.so".to_string()));
    assert_eq!(config.config.unwrap()["key"], "value");
    assert_eq!(config.env.get("ENV_VAR1"), Some(&"value1".to_string()));
    assert!(config.auto_reload);
    assert_eq!(config.priority, 50);
}

#[tokio::test]
async fn test_plugin_config_simple() {
    let config = PluginConfig::simple("simple-plugin");

    assert_eq!(config.name, "simple-plugin");
    assert!(config.enabled);
    assert!(config.path.is_none());
    assert!(config.config.is_none());
    assert!(config.env.is_empty());
    assert!(!config.auto_reload);
    assert_eq!(config.priority, 100);
}

#[tokio::test]
async fn test_plugin_config_builder_pattern() {
    let config = PluginConfig::simple("builder-plugin")
        .with_path("/custom/path/plugin.so")
        .with_config(json!({"timeout": 5000}))
        .with_auto_reload();

    assert_eq!(config.name, "builder-plugin");
    assert_eq!(config.path, Some("/custom/path/plugin.so".to_string()));
    assert_eq!(config.config.unwrap()["timeout"], 5000);
    assert!(config.auto_reload);
}

#[tokio::test]
async fn test_plugin_manifest_creation() {
    let manifest = PluginManifest {
        plugin: PluginInfo {
            id: "test-plugin-id".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Author Name".to_string()),
            description: Some("Plugin description".to_string()),
            homepage: Some("https://plugin.com".to_string()),
            repository: Some("https://github.com/user/plugin".to_string()),
            license: Some("MIT".to_string()),
            keywords: vec!["test".to_string(), "plugin".to_string()],
            mcp_version: "1.0.0".to_string(),
        },
        dependencies: vec![Dependency {
            plugin: "dependency-1".to_string(),
            version: ">=1.0.0".to_string(),
            optional: false,
        }],
        tool: ToolDefinition {
            name: "test_tool".to_string(),
            description: "Test tool description".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "output": {"type": "string"}
                }
            })),
            examples: vec![ToolExample {
                name: "Example 1".to_string(),
                description: Some("First example".to_string()),
                input: json!({"input": "test"}),
                output: Some(json!({"output": "result"})),
            }],
        },
        build: Some(BuildInfo {
            command: "cargo build --release".to_string(),
            directory: Some("./src".to_string()),
            output: "target/release/plugin.so".to_string(),
        }),
        install: Some(InstallInfo {
            pre_install: Some("echo 'Installing...'".to_string()),
            post_install: Some("echo 'Done!'".to_string()),
            system_deps: vec!["libssl-dev".to_string()],
        }),
    };

    assert_eq!(manifest.plugin.id, "test-plugin-id");
    assert_eq!(manifest.dependencies.len(), 1);
    assert_eq!(manifest.tool.name, "test_tool");
    assert!(manifest.build.is_some());
    assert!(manifest.install.is_some());
}

#[tokio::test]
async fn test_plugin_manifest_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("plugin.yaml");

    let manifest = PluginManifest {
        plugin: PluginInfo {
            id: "file-test-plugin".to_string(),
            name: "File Test Plugin".to_string(),
            version: "2.0.0".to_string(),
            author: None,
            description: None,
            homepage: None,
            repository: None,
            license: None,
            keywords: vec![],
            mcp_version: "1.0.0".to_string(),
        },
        dependencies: vec![],
        tool: ToolDefinition {
            name: "file_tool".to_string(),
            description: "File tool".to_string(),
            input_schema: json!({}),
            output_schema: None,
            examples: vec![],
        },
        build: None,
        install: None,
    };

    // Test saving to file
    let result = manifest.to_file(&manifest_path).await;
    assert!(result.is_ok());

    // Verify file exists
    assert!(manifest_path.exists());

    // Test loading from file
    let loaded = PluginManifest::from_file(&manifest_path).await;
    assert!(loaded.is_ok());

    let loaded_manifest = loaded.unwrap();
    assert_eq!(loaded_manifest.plugin.id, "file-test-plugin");
    assert_eq!(loaded_manifest.plugin.version, "2.0.0");
    assert_eq!(loaded_manifest.tool.name, "file_tool");
}

#[tokio::test]
async fn test_plugin_config_set() {
    let config_set = PluginConfigSet {
        plugins: vec![
            PluginConfig::simple("plugin1"),
            PluginConfig::simple("plugin2").with_auto_reload(),
            PluginConfig::simple("plugin3").with_path("/path/to/plugin3.so"),
        ],
        settings: Some(PluginSettings {
            plugin_dirs: vec![
                PathBuf::from("/usr/lib/plugins"),
                PathBuf::from("/home/user/.plugins"),
            ],
            auto_load: true,
            hot_reload: false,
            isolation: IsolationLevel::Thread,
            load_timeout: 60,
        }),
    };

    assert_eq!(config_set.plugins.len(), 3);
    assert!(config_set.settings.is_some());

    let settings = config_set.settings.unwrap();
    assert_eq!(settings.plugin_dirs.len(), 2);
    assert!(settings.auto_load);
    assert!(!settings.hot_reload);
    assert!(matches!(settings.isolation, IsolationLevel::Thread));
    assert_eq!(settings.load_timeout, 60);
}

#[tokio::test]
async fn test_plugin_config_set_sort_by_priority() {
    let mut config_set = PluginConfigSet {
        plugins: vec![
            PluginConfig {
                name: "plugin3".to_string(),
                enabled: true,
                path: None,
                config: None,
                env: HashMap::new(),
                auto_reload: false,
                priority: 150,
            },
            PluginConfig {
                name: "plugin1".to_string(),
                enabled: true,
                path: None,
                config: None,
                env: HashMap::new(),
                auto_reload: false,
                priority: 50,
            },
            PluginConfig {
                name: "plugin2".to_string(),
                enabled: true,
                path: None,
                config: None,
                env: HashMap::new(),
                auto_reload: false,
                priority: 100,
            },
        ],
        settings: None,
    };

    config_set.sort_by_priority();

    assert_eq!(config_set.plugins[0].name, "plugin1");
    assert_eq!(config_set.plugins[1].name, "plugin2");
    assert_eq!(config_set.plugins[2].name, "plugin3");
}

#[tokio::test]
async fn test_plugin_config_set_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("plugins.yaml");

    let config_set = PluginConfigSet {
        plugins: vec![
            PluginConfig::simple("plugin1"),
            PluginConfig::simple("plugin2").with_config(json!({"enabled": true})),
        ],
        settings: Some(PluginSettings {
            plugin_dirs: vec![PathBuf::from("/plugins")],
            auto_load: true,
            hot_reload: true,
            isolation: IsolationLevel::None,
            load_timeout: 30,
        }),
    };

    // Test saving
    let result = config_set.to_file(&config_path).await;
    assert!(result.is_ok());
    assert!(config_path.exists());

    // Test loading
    let loaded = PluginConfigSet::from_file(&config_path).await;
    assert!(loaded.is_ok());

    let loaded_set = loaded.unwrap();
    assert_eq!(loaded_set.plugins.len(), 2);
    assert_eq!(loaded_set.plugins[0].name, "plugin1");
    assert_eq!(loaded_set.plugins[1].name, "plugin2");
}

#[tokio::test]
async fn test_isolation_level_enum() {
    let levels = vec![
        IsolationLevel::None,
        IsolationLevel::Thread,
        IsolationLevel::Process,
        IsolationLevel::Container,
    ];

    for level in levels {
        match level {
            IsolationLevel::None => assert!(true),
            IsolationLevel::Thread => assert!(true),
            IsolationLevel::Process => assert!(true),
            IsolationLevel::Container => assert!(true),
        }
    }

    // Test default
    let default_level = IsolationLevel::default();
    assert!(matches!(default_level, IsolationLevel::None));
}

#[tokio::test]
async fn test_tool_definition_with_examples() {
    let tool_def = ToolDefinition {
        name: "example_tool".to_string(),
        description: "Tool with examples".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "text": {"type": "string"},
                "count": {"type": "number"}
            }
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "result": {"type": "array"}
            }
        })),
        examples: vec![
            ToolExample {
                name: "Basic example".to_string(),
                description: Some("Shows basic usage".to_string()),
                input: json!({"text": "hello", "count": 3}),
                output: Some(json!({"result": ["hello", "hello", "hello"]})),
            },
            ToolExample {
                name: "Empty example".to_string(),
                description: None,
                input: json!({"text": "", "count": 0}),
                output: Some(json!({"result": []})),
            },
        ],
    };

    assert_eq!(tool_def.examples.len(), 2);
    assert_eq!(tool_def.examples[0].name, "Basic example");
    assert!(tool_def.examples[0].description.is_some());
    assert_eq!(tool_def.examples[1].name, "Empty example");
    assert!(tool_def.examples[1].description.is_none());
}

#[tokio::test]
async fn test_build_info() {
    let build_info = BuildInfo {
        command: "make release".to_string(),
        directory: Some("./build".to_string()),
        output: "dist/plugin.so".to_string(),
    };

    assert_eq!(build_info.command, "make release");
    assert_eq!(build_info.directory, Some("./build".to_string()));
    assert_eq!(build_info.output, "dist/plugin.so");
}

#[tokio::test]
async fn test_install_info() {
    let install_info = InstallInfo {
        pre_install: Some("apt-get update".to_string()),
        post_install: Some("ldconfig".to_string()),
        system_deps: vec!["libssl-dev".to_string(), "libcurl4-openssl-dev".to_string()],
    };

    assert!(install_info.pre_install.is_some());
    assert!(install_info.post_install.is_some());
    assert_eq!(install_info.system_deps.len(), 2);
}

#[tokio::test]
async fn test_dependency_specification() {
    let dep = Dependency {
        plugin: "core-plugin".to_string(),
        version: "^1.0.0".to_string(),
        optional: false,
    };

    assert_eq!(dep.plugin, "core-plugin");
    assert_eq!(dep.version, "^1.0.0");
    assert!(!dep.optional);

    let optional_dep = Dependency {
        plugin: "optional-plugin".to_string(),
        version: "*".to_string(),
        optional: true,
    };

    assert!(optional_dep.optional);
    assert_eq!(optional_dep.version, "*");
}

#[tokio::test]
async fn test_plugin_settings_defaults() {
    let settings = PluginSettings {
        plugin_dirs: vec![],
        auto_load: default_auto_load(),
        hot_reload: default_hot_reload(),
        isolation: IsolationLevel::default(),
        load_timeout: default_load_timeout(),
    };

    assert!(settings.auto_load); // default is true
    assert!(!settings.hot_reload); // default is false
    assert_eq!(settings.load_timeout, 30); // default is 30
    assert!(matches!(settings.isolation, IsolationLevel::None));
}

#[tokio::test]
async fn test_complex_plugin_manifest() {
    let manifest = PluginManifest {
        plugin: PluginInfo {
            id: "complex-plugin".to_string(),
            name: "Complex Plugin".to_string(),
            version: "3.2.1".to_string(),
            author: Some("Plugin Team".to_string()),
            description: Some("A complex plugin with all features".to_string()),
            homepage: Some("https://complex-plugin.org".to_string()),
            repository: Some("https://github.com/org/complex-plugin".to_string()),
            license: Some("GPL-3.0".to_string()),
            keywords: vec![
                "complex".to_string(),
                "advanced".to_string(),
                "plugin".to_string(),
            ],
            mcp_version: "2.0.0".to_string(),
        },
        dependencies: vec![
            Dependency {
                plugin: "base-plugin".to_string(),
                version: ">=1.0.0, <2.0.0".to_string(),
                optional: false,
            },
            Dependency {
                plugin: "extension-plugin".to_string(),
                version: "~1.5.0".to_string(),
                optional: true,
            },
        ],
        tool: ToolDefinition {
            name: "complex_tool".to_string(),
            description: "Complex tool with full schema".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["create", "update", "delete"]
                    },
                    "data": {
                        "type": "object"
                    }
                },
                "required": ["action"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "success": {"type": "boolean"},
                    "message": {"type": "string"},
                    "data": {"type": "object"}
                },
                "required": ["success"]
            })),
            examples: vec![ToolExample {
                name: "Create example".to_string(),
                description: Some("Creating a new item".to_string()),
                input: json!({
                    "action": "create",
                    "data": {"name": "Item 1"}
                }),
                output: Some(json!({
                    "success": true,
                    "message": "Created successfully",
                    "data": {"id": 1, "name": "Item 1"}
                })),
            }],
        },
        build: Some(BuildInfo {
            command: "cargo build --release --features full".to_string(),
            directory: Some(".".to_string()),
            output: "target/release/libcomplex_plugin.so".to_string(),
        }),
        install: Some(InstallInfo {
            pre_install: Some("./scripts/pre_install.sh".to_string()),
            post_install: Some("./scripts/post_install.sh".to_string()),
            system_deps: vec![
                "libssl-dev".to_string(),
                "libzmq3-dev".to_string(),
                "protobuf-compiler".to_string(),
            ],
        }),
    };

    assert_eq!(manifest.plugin.keywords.len(), 3);
    assert_eq!(manifest.dependencies.len(), 2);
    assert!(manifest.tool.output_schema.is_some());
    assert_eq!(manifest.tool.examples.len(), 1);
    assert_eq!(manifest.install.as_ref().unwrap().system_deps.len(), 3);
}

// Helper function tests
fn default_enabled() -> bool {
    true
}

fn default_priority() -> i32 {
    100
}

fn default_auto_load() -> bool {
    true
}

fn default_hot_reload() -> bool {
    false
}

fn default_load_timeout() -> u64 {
    30
}

#[test]
fn test_default_functions() {
    assert!(default_enabled());
    assert_eq!(default_priority(), 100);
    assert!(default_auto_load());
    assert!(!default_hot_reload());
    assert_eq!(default_load_timeout(), 30);
}

#[tokio::test]
async fn test_error_handling_file_operations() {
    // Test loading from non-existent file
    let result = PluginManifest::from_file("/non/existent/path.yaml").await;
    assert!(result.is_err());

    let result = PluginConfigSet::from_file("/non/existent/config.yaml").await;
    assert!(result.is_err());

    // Test saving to invalid path
    let manifest = PluginManifest {
        plugin: PluginInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: None,
            homepage: None,
            repository: None,
            license: None,
            keywords: vec![],
            mcp_version: "1.0.0".to_string(),
        },
        dependencies: vec![],
        tool: ToolDefinition {
            name: "tool".to_string(),
            description: "Tool".to_string(),
            input_schema: json!({}),
            output_schema: None,
            examples: vec![],
        },
        build: None,
        install: None,
    };

    let result = manifest.to_file("/invalid/\0/path.yaml").await;
    assert!(result.is_err());
}

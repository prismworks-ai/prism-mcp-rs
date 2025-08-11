//! Tests for plugin API module

use async_trait::async_trait;
use prism_mcp_rs::core::error::McpResult;
use prism_mcp_rs::plugin::api::*;
use prism_mcp_rs::protocol::types::{CallToolResult as ToolResult, Tool as ProtocolTool};
use serde_json::{Value, json};
use std::any::Any;
use std::collections::HashMap;

// Mock implementation of ToolPlugin for testing
struct TestToolPlugin {
    id: String,
    name: String,
    initialized: bool,
    configured: bool,
}

impl TestToolPlugin {
    fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            initialized: false,
            configured: false,
        }
    }
}

#[async_trait]
impl ToolPlugin for TestToolPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: self.id.clone(),
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            description: Some("Test plugin description".to_string()),
            homepage: Some("https://example.com".to_string()),
            license: Some("MIT".to_string()),
            mcp_version: "1.0.0".to_string(),
            capabilities: PluginCapabilities {
                hot_reload: true,
                configurable: true,
                health_check: true,
                thread_safe: true,
                multi_instance: false,
                custom: json!({"test": true}),
            },
            dependencies: vec![
                PluginDependency {
                    plugin_id: "dep1".to_string(),
                    version: ">=1.0.0".to_string(),
                    optional: false,
                },
                PluginDependency {
                    plugin_id: "dep2".to_string(),
                    version: "^2.0.0".to_string(),
                    optional: true,
                },
            ],
        }
    }

    fn tool_definition(&self) -> ProtocolTool {
        ProtocolTool {
            name: "test_tool".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: prism_mcp_rs::protocol::types::ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(
                    [("input".to_string(), json!({"type": "string"}))]
                        .into_iter()
                        .collect(),
                ),
                required: Some(vec!["input".to_string()]),
                additional_properties: Default::default(),
            },
            output_schema: None,
            annotations: None,
            title: None,
            meta: None,
        }
    }

    async fn execute(&self, arguments: Value) -> McpResult<ToolResult> {
        Ok(ToolResult {
            content: vec![prism_mcp_rs::protocol::types::ContentBlock::Text {
                text: format!("Executed with: {}", arguments),
                annotations: None,
                meta: None,
            }],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }

    async fn initialize(&mut self) -> McpResult<()> {
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> McpResult<()> {
        self.initialized = false;
        Ok(())
    }

    async fn health_check(&self) -> McpResult<()> {
        if self.initialized {
            Ok(())
        } else {
            Err(prism_mcp_rs::core::error::McpError::Protocol(
                "Plugin not initialized".to_string(),
            ))
        }
    }

    async fn configure(&mut self, _config: Value) -> McpResult<()> {
        self.configured = true;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Another plugin implementation for testing different scenarios
struct FailingPlugin {
    fail_on_init: bool,
    fail_on_execute: bool,
}

#[async_trait]
impl ToolPlugin for FailingPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "failing-plugin".to_string(),
            name: "Failing Plugin".to_string(),
            version: "0.1.0".to_string(),
            author: None,
            description: None,
            homepage: None,
            license: None,
            mcp_version: "1.0.0".to_string(),
            capabilities: PluginCapabilities::default(),
            dependencies: vec![],
        }
    }

    fn tool_definition(&self) -> ProtocolTool {
        ProtocolTool {
            name: "failing_tool".to_string(),
            description: None,
            input_schema: prism_mcp_rs::protocol::types::ToolInputSchema {
                schema_type: "object".to_string(),
                properties: None,
                required: None,
                additional_properties: Default::default(),
            },
            output_schema: None,
            annotations: None,
            title: None,
            meta: None,
        }
    }

    async fn execute(&self, _arguments: Value) -> McpResult<ToolResult> {
        if self.fail_on_execute {
            Err(prism_mcp_rs::core::error::McpError::Protocol(
                "Execution failed".to_string(),
            ))
        } else {
            Ok(ToolResult {
                content: vec![],
                is_error: Some(true),
                structured_content: None,
                meta: None,
            })
        }
    }

    async fn initialize(&mut self) -> McpResult<()> {
        if self.fail_on_init {
            Err(prism_mcp_rs::core::error::McpError::Protocol(
                "Initialization failed".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[tokio::test]
async fn test_plugin_metadata_creation() {
    let metadata = PluginMetadata {
        id: "test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.2.3".to_string(),
        author: Some("John Doe".to_string()),
        description: Some("A test plugin".to_string()),
        homepage: Some("https://test.com".to_string()),
        license: Some("Apache-2.0".to_string()),
        mcp_version: "2.0.0".to_string(),
        capabilities: PluginCapabilities::default(),
        dependencies: vec![],
    };

    assert_eq!(metadata.id, "test-plugin");
    assert_eq!(metadata.name, "Test Plugin");
    assert_eq!(metadata.version, "1.2.3");
    assert_eq!(metadata.author, Some("John Doe".to_string()));
    assert_eq!(metadata.mcp_version, "2.0.0");
}

#[tokio::test]
async fn test_plugin_capabilities() {
    let capabilities = PluginCapabilities {
        hot_reload: true,
        configurable: false,
        health_check: true,
        thread_safe: false,
        multi_instance: true,
        custom: json!({"feature": "enabled"}),
    };

    assert!(capabilities.hot_reload);
    assert!(!capabilities.configurable);
    assert!(capabilities.health_check);
    assert!(!capabilities.thread_safe);
    assert!(capabilities.multi_instance);
    assert_eq!(capabilities.custom["feature"], "enabled");
}

#[tokio::test]
async fn test_plugin_capabilities_default() {
    let capabilities = PluginCapabilities::default();

    assert!(!capabilities.hot_reload);
    assert!(!capabilities.configurable);
    assert!(!capabilities.health_check);
    assert!(!capabilities.thread_safe);
    assert!(!capabilities.multi_instance);
    assert_eq!(capabilities.custom, Value::Null);
}

#[tokio::test]
async fn test_plugin_dependency() {
    let dep = PluginDependency {
        plugin_id: "dependency-plugin".to_string(),
        version: ">=2.0.0, <3.0.0".to_string(),
        optional: true,
    };

    assert_eq!(dep.plugin_id, "dependency-plugin");
    assert_eq!(dep.version, ">=2.0.0, <3.0.0");
    assert!(dep.optional);
}

#[tokio::test]
async fn test_tool_plugin_lifecycle() {
    let mut plugin = TestToolPlugin::new("test-id", "Test Plugin");

    // Test metadata
    let metadata = plugin.metadata();
    assert_eq!(metadata.id, "test-id");
    assert_eq!(metadata.name, "Test Plugin");
    assert_eq!(metadata.version, "1.0.0");

    // Test tool definition
    let tool_def = plugin.tool_definition();
    assert_eq!(tool_def.name, "test_tool");
    assert_eq!(tool_def.description, Some("A test tool".to_string()));

    // Test initialization
    assert!(plugin.initialize().await.is_ok());
    assert!(plugin.initialized);

    // Test health check after initialization
    assert!(plugin.health_check().await.is_ok());

    // Test configuration
    let config = json!({"timeout": 5000});
    assert!(plugin.configure(config).await.is_ok());
    assert!(plugin.configured);

    // Test execution
    let args = json!({"input": "test data"});
    let result = plugin.execute(args.clone()).await.unwrap();
    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Test shutdown
    assert!(plugin.shutdown().await.is_ok());
    assert!(!plugin.initialized);

    // Health check should fail after shutdown
    assert!(plugin.health_check().await.is_err());
}

#[tokio::test]
async fn test_failing_plugin() {
    let mut plugin = FailingPlugin {
        fail_on_init: true,
        fail_on_execute: false,
    };

    // Test failed initialization
    assert!(plugin.initialize().await.is_err());

    // Test with successful init but failed execution
    plugin.fail_on_init = false;
    plugin.fail_on_execute = true;
    assert!(plugin.initialize().await.is_ok());

    let result = plugin.execute(json!({})).await;
    assert!(result.is_err());

    // Test with error result but successful execution
    plugin.fail_on_execute = false;
    let result = plugin.execute(json!({})).await.unwrap();
    assert_eq!(result.is_error, Some(true));
}

#[tokio::test]
async fn test_plugin_as_any() {
    let plugin = TestToolPlugin::new("test", "Test");
    let any_ref = plugin.as_any();

    // Should be able to downcast back to TestToolPlugin
    assert!(any_ref.downcast_ref::<TestToolPlugin>().is_some());
}

#[tokio::test]
async fn test_plugin_metadata_with_dependencies() {
    let metadata = PluginMetadata {
        id: "complex-plugin".to_string(),
        name: "Complex Plugin".to_string(),
        version: "2.5.0".to_string(),
        author: Some("Plugin Author".to_string()),
        description: Some("A complex plugin with dependencies".to_string()),
        homepage: Some("https://plugin.example.com".to_string()),
        license: Some("BSD-3-Clause".to_string()),
        mcp_version: "1.5.0".to_string(),
        capabilities: PluginCapabilities {
            hot_reload: true,
            configurable: true,
            health_check: true,
            thread_safe: true,
            multi_instance: true,
            custom: json!({
                "supports_async": true,
                "max_concurrent": 10
            }),
        },
        dependencies: vec![
            PluginDependency {
                plugin_id: "core-plugin".to_string(),
                version: "1.0.0".to_string(),
                optional: false,
            },
            PluginDependency {
                plugin_id: "optional-plugin".to_string(),
                version: "*".to_string(),
                optional: true,
            },
        ],
    };

    assert_eq!(metadata.dependencies.len(), 2);
    assert!(!metadata.dependencies[0].optional);
    assert!(metadata.dependencies[1].optional);
    assert!(metadata.capabilities.hot_reload);
    assert_eq!(metadata.capabilities.custom["max_concurrent"], 10);
}

#[tokio::test]
async fn test_plugin_builder_trait() {
    // Test that PluginBuilder trait can be used
    struct TestBuilder {
        name: String,
    }

    impl PluginBuilder for TestBuilder {
        fn build(self) -> Box<dyn ToolPlugin> {
            Box::new(TestToolPlugin::new("built", &self.name))
        }
    }

    let builder = TestBuilder {
        name: "Built Plugin".to_string(),
    };

    let plugin = builder.build();
    let metadata = plugin.metadata();
    assert_eq!(metadata.id, "built");
    assert_eq!(metadata.name, "Built Plugin");
}

// Test the export_plugin macro by simulating its expansion
#[test]
fn test_export_plugin_macro_pattern() {
    // This tests the pattern that the macro would generate
    #[derive(Default)]
    struct MacroTestPlugin;

    #[async_trait]
    impl ToolPlugin for MacroTestPlugin {
        fn metadata(&self) -> PluginMetadata {
            PluginMetadata {
                id: "macro-test".to_string(),
                name: "Macro Test".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                author: None,
                description: None,
                homepage: None,
                license: None,
                mcp_version: "1.0.0".to_string(),
                capabilities: PluginCapabilities::default(),
                dependencies: vec![],
            }
        }

        fn tool_definition(&self) -> ProtocolTool {
            ProtocolTool {
                name: "macro_tool".to_string(),
                description: None,
                input_schema: prism_mcp_rs::protocol::types::ToolInputSchema {
                    schema_type: "object".to_string(),
                    properties: None,
                    required: None,
                    additional_properties: Default::default(),
                },
                output_schema: None,
                annotations: None,
                title: None,
                meta: None,
            }
        }

        async fn execute(&self, _arguments: Value) -> McpResult<ToolResult> {
            Ok(ToolResult {
                content: vec![],
                is_error: Some(false),
                structured_content: None,
                meta: None,
            })
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // Simulate what the macro would generate
    fn _mcp_plugin_create() -> *mut dyn ToolPlugin {
        let plugin = Box::new(MacroTestPlugin::default());
        Box::into_raw(plugin)
    }

    fn _mcp_plugin_version() -> *const u8 {
        concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr()
    }

    // Test that these functions exist and can be called
    unsafe {
        let plugin_ptr = _mcp_plugin_create();
        assert!(!plugin_ptr.is_null());

        let version_ptr = _mcp_plugin_version();
        assert!(!version_ptr.is_null());

        // Clean up
        let _ = Box::from_raw(plugin_ptr);
    }
}

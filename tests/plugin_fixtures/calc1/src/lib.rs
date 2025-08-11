//! Simple test plugin for integration tests

use async_trait::async_trait;
use prism_mcp_rs::{
    core::error::{McpError, McpResult},
    export_plugin,
    plugin::{PluginCapabilities, PluginMetadata, ToolPlugin, ToolResult},
    protocol::types::{ContentBlock, Tool, ToolInputSchema},
};
use serde_json::{json, Value};
use std::any::Any;
use std::collections::HashMap;

/// Simple calculator plugin for testing
pub struct Calc1Plugin {
    enabled: bool,
}

impl Default for Calc1Plugin {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[async_trait]
impl ToolPlugin for Calc1Plugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "calc1".to_string(),
            name: "Calculator 1".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Test".to_string()),
            description: Some("Test calculator plugin 1".to_string()),
            homepage: None,
            license: Some("MIT".to_string()),
            mcp_version: "0.5.0".to_string(),
            capabilities: PluginCapabilities {
                hot_reload: true,
                configurable: true,
                health_check: true,
                thread_safe: true,
                multi_instance: false,
                custom: json!({}),
            },
            dependencies: vec![],
        }
    }

    fn tool_definition(&self) -> Tool {
        let mut properties = HashMap::new();
        properties.insert("operation".to_string(), json!({
            "type": "string",
            "enum": ["add", "subtract"],
            "description": "Mathematical operation"
        }));
        properties.insert("a".to_string(), json!({
            "type": "number",
            "description": "First operand"
        }));
        properties.insert("b".to_string(), json!({
            "type": "number",
            "description": "Second operand"
        }));

        Tool {
            name: "calc1".to_string(),
            title: Some("Calculator 1".to_string()),
            description: Some("Simple calculator for testing".to_string()),
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(properties),
                required: Some(vec!["operation".to_string(), "a".to_string(), "b".to_string()]),
                additional_properties: HashMap::new(),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        }
    }

    async fn execute(&self, arguments: Value) -> McpResult<ToolResult> {
        if !self.enabled {
            return Err(McpError::InvalidParams("Plugin is disabled".to_string()));
        }

        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams("Missing operation".to_string()))?;

        let a = arguments
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::InvalidParams("Missing operand a".to_string()))?;

        let b = arguments
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::InvalidParams("Missing operand b".to_string()))?;

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            _ => return Err(McpError::InvalidParams(format!("Unknown operation: {}", operation))),
        };

        Ok(ToolResult {
            content: vec![ContentBlock::Text {
                text: format!("{} {} {} = {}", a, operation, b, result),
                annotations: None,
                meta: None,
            }],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }

    async fn initialize(&mut self) -> McpResult<()> {
        self.enabled = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> McpResult<()> {
        self.enabled = false;
        Ok(())
    }

    async fn health_check(&self) -> McpResult<()> {
        if self.enabled {
            Ok(())
        } else {
            Err(McpError::Protocol("Plugin is disabled".to_string()))
        }
    }

    async fn configure(&mut self, config: Value) -> McpResult<()> {
        if let Some(enabled) = config.get("enabled").and_then(|v| v.as_bool()) {
            self.enabled = enabled;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Export the plugin
export_plugin!(Calc1Plugin);
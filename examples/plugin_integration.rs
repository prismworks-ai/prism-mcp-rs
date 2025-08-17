//! Plugin System Integration Example
//!
//! This example shows how to create, register, and use plugins with the MCP server.

use prism_mcp_rs::plugin::*;
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a math operations plugin
    let math_plugin = create_math_plugin();

    // Create a data processing plugin
    let data_plugin = create_data_plugin();

    // Initialize plugin manager
    let mut plugin_manager = PluginManager::new();

    // Register plugins
    plugin_manager.register_plugin(math_plugin)?;
    plugin_manager.register_plugin(data_plugin)?;

    // Create server with plugin manager
    let server = ServerBuilder::new()
        .name("plugin-server")
        .version("1.0.0")
        .with_plugin_manager(plugin_manager)
        .build()?;

    // Start server
    println!("🚀 Starting server with plugins...");
    server.run_with_stdio().await?;

    Ok(())
}

fn create_math_plugin() -> Box<dyn ToolPlugin> {
    PluginBuilder::new("math-ops")
        .version("1.0.0")
        .description("Mathematical operations plugin")
        .add_tool(
            ToolBuilder::new("add")
                .description("Add two numbers")
                .input_schema(json!({
                    "type": "object",
                    "properties": {
                        "a": { "type": "number" },
                        "b": { "type": "number" }
                    },
                    "required": ["a", "b"]
                }))
                .handler(|input| async move {
                    let a = input["a"].as_f64().unwrap_or(0.0);
                    let b = input["b"].as_f64().unwrap_or(0.0);
                    Ok(json!({ "result": a + b }))
                })
                .build(),
        )
        .add_tool(
            ToolBuilder::new("multiply")
                .description("Multiply two numbers")
                .input_schema(json!({
                    "type": "object",
                    "properties": {
                        "a": { "type": "number" },
                        "b": { "type": "number" }
                    },
                    "required": ["a", "b"]
                }))
                .handler(|input| async move {
                    let a = input["a"].as_f64().unwrap_or(0.0);
                    let b = input["b"].as_f64().unwrap_or(0.0);
                    Ok(json!({ "result": a * b }))
                })
                .build(),
        )
        .build()
}

fn create_data_plugin() -> Box<dyn ToolPlugin> {
    PluginBuilder::new("data-processing")
        .version("1.0.0")
        .description("Data processing and transformation plugin")
        .add_tool(
            ToolBuilder::new("json_filter")
                .description("Filter JSON array based on criteria")
                .input_schema(json!({
                    "type": "object",
                    "properties": {
                        "data": { "type": "array" },
                        "field": { "type": "string" },
                        "value": {}
                    },
                    "required": ["data", "field", "value"]
                }))
                .handler(|input| async move {
                    let data = input["data"].as_array().unwrap_or(&vec![]);
                    let field = input["field"].as_str().unwrap_or("");
                    let value = &input["value"];

                    let filtered: Vec<_> = data
                        .iter()
                        .filter(|item| item[field] == *value)
                        .cloned()
                        .collect();

                    Ok(json!({ "filtered": filtered }))
                })
                .build(),
        )
        .add_resource(
            ResourceBuilder::new("sample-data")
                .uri("data://sample.json")
                .description("Sample data for processing")
                .handler(|_| async move {
                    Ok(ResourceContents::Text {
                        text: json!([
                            { "id": 1, "name": "Alice", "age": 30 },
                            { "id": 2, "name": "Bob", "age": 25 },
                            { "id": 3, "name": "Charlie", "age": 35 }
                        ])
                        .to_string(),
                        mime_type: Some("application/json".to_string()),
                    })
                })
                .build(),
        )
        .build()
}

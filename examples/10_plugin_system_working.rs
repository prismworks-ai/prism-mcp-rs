//! Example 10: Plugin System (Working Version)
//! Demonstrates plugin architecture

use async_trait::async_trait;
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

/// Simple plugin interface
#[async_trait]
trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, input: serde_json::Value) -> McpResult<serde_json::Value>;
}

/// Math plugin
struct MathPlugin;

#[async_trait]
impl Plugin for MathPlugin {
    fn name(&self) -> &str {
        "math"
    }

    async fn execute(&self, _input: serde_json::Value) -> McpResult<serde_json::Value> {
        Ok(serde_json::json!({
            "plugin": "math",
            "result": "calculated"
        }))
    }
}

/// Plugin manager
struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    fn register(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    async fn execute(&self, name: &str, input: serde_json::Value) -> McpResult<serde_json::Value> {
        self.plugins
            .get(name)
            .ok_or_else(|| McpError::Validation(format!("Plugin '{}' not found", name)))?
            .execute(input)
            .await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = PluginManager::new();

    // Register plugins
    manager.register(Box::new(MathPlugin));

    // Execute plugin
    let result = manager.execute("math", serde_json::json!({"x": 5})).await?;
    println!("Plugin result: {}", result);

    Ok(())
}

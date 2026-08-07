//! Example: application-owned in-process extension registry.
//!
//! This does not demonstrate the SDK's optional native dynamic-plugin loader.
//! See `docs/guides/plugins.md` for that loader and its trust boundary.

use prism_mcp_rs::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Simple plugin trait
trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    #[allow(dead_code)]
    fn version(&self) -> &str;
    fn execute(&self, args: HashMap<String, Value>) -> McpResult<Value>;
}

// Example plugin
struct MathPlugin;

impl Plugin for MathPlugin {
    fn name(&self) -> &str {
        "math"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }

    fn execute(&self, args: HashMap<String, Value>) -> McpResult<Value> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("sum");

        let numbers = args
            .get("numbers")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect::<Vec<_>>())
            .unwrap_or_default();

        let result = match operation {
            "sum" => numbers.iter().sum::<f64>(),
            "product" => numbers.iter().product::<f64>(),
            "mean" => numbers.iter().sum::<f64>() / numbers.len() as f64,
            _ => 0.0,
        };

        Ok(json!(result))
    }
}

// Plugin manager
struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
}

impl PluginManager {
    fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn register(&self, plugin: Box<dyn Plugin>) {
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin.name().to_string(), plugin);
    }

    async fn execute(&self, name: &str, args: HashMap<String, Value>) -> McpResult<Value> {
        let plugins = self.plugins.read().await;
        plugins
            .get(name)
            .ok_or_else(|| McpError::ToolNotFound(name.to_string()))?
            .execute(args)
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let _server = McpServer::new("plugin-example".to_string(), "1.0.0".to_string());

    let plugin_manager = PluginManager::new();
    plugin_manager.register(Box::new(MathPlugin)).await;

    // Test plugin execution
    let result = plugin_manager
        .execute(
            "math",
            HashMap::from([
                ("operation".to_string(), json!("sum")),
                ("numbers".to_string(), json!([1, 2, 3, 4, 5])),
            ]),
        )
        .await?;

    println!("Plugin result: {}", result);
    println!("Plugin system example server created");
    Ok(())
}

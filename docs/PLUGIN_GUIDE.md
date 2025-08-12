# Plugin Development Guide

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Component Types](#component-types)
4. [Creating Plugins](#creating-plugins)
5. [Distribution](#distribution)
6. [Licensing](#licensing)
7. [Best Practices](#best-practices)
8. [API Reference](#api-reference)

## Overview

The Prism MCP SDK provides a plugin architecture that enables developers to create dynamically loadable extensions for MCP servers. Plugins are distributed as compiled dynamic libraries that can be loaded, unloaded, and reloaded at runtime without server restarts.

### Key Characteristics

- **Runtime Loading**: Plugins are loaded as dynamic libraries (.so, .dll, .dylib)
- **Component Isolation**: Each plugin operates in its own namespace
- **Version Management**: Independent versioning and dependency management
- **Distribution Flexibility**: Multiple distribution channels supported

### Related Documentation

- [Plugin Component Types Reference](PLUGIN_TYPES.md) - Detailed specifications for each component type
- [API Documentation](https://docs.rs/prism-mcp-rs) - Complete API reference (available after publication)
- [Examples](../examples/README.md) - Working plugin examples

## Architecture

### Plugin Hierarchy

A plugin serves as a container that can provide four types of MCP components:

```
Plugin (Dynamic Library)
├── Tools (Executable Functions)
├── Resources (Data Providers)
├── Prompts (Message Templates)
└── Completions (Autocomplete Providers)
```

### Component Definitions

#### Tools
Executable functions that perform operations and return results. Tools accept arguments and produce outputs.

**Use Cases**: Data processing, API calls, calculations, file operations

#### Resources
Data providers that expose readable content through URI-based access patterns. Resources support parameterized queries.

**Use Cases**: Database access, file system browsing, configuration management, API data exposure

#### Prompts
Template generators that produce structured message sequences for LLM interactions. Prompts accept arguments to customize output.

**Use Cases**: Code review templates, query builders, conversation starters, analysis frameworks

#### Completions
Autocomplete providers that suggest valid values for tool arguments, resource URIs, and prompt parameters.

**Use Cases**: File path completion, database schema hints, command suggestions, parameter validation

## Component Types

### Plugin Trait

The base trait that all plugins must implement:

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    async fn initialize(&mut self) -> McpResult<()>;
    async fn shutdown(&mut self) -> McpResult<()>;
    async fn configure(&mut self, config: Value) -> McpResult<()>;
    fn capabilities(&self) -> PluginCapabilities;
    fn metadata(&self) -> PluginMetadata;
}
```

### Plugin Capabilities

Declare which component types the plugin provides:

```rust
pub struct PluginCapabilities {
    pub provides_tools: bool,
    pub provides_resources: bool,
    pub provides_prompts: bool,
    pub supports_hot_reload: bool,
}
```

### Handler Traits

#### ToolHandler

```rust
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult>;
}
```

#### ResourceHandler

```rust
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    async fn read(
        &self,
        uri: &str,
        params: &HashMap<String, String>
    ) -> McpResult<Vec<ResourceContents>>;
}
```

#### PromptHandler

```rust
#[async_trait]
pub trait PromptHandler: Send + Sync {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult>;
}
```

#### CompletionHandler

```rust
#[async_trait]
pub trait CompletionHandler: Send + Sync {
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        context: Option<&CompletionContext>
    ) -> McpResult<Vec<String>>;
}
```

## Creating Plugins

### Project Structure

```toml
# Cargo.toml
[package]
name = "example-plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
prism-mcp-rs = "0.1.0"
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[lib]
crate-type = ["cdylib"]
```

### Complete Plugin Example

This example demonstrates a plugin that provides all four component types:

```rust
use prism_mcp_rs::{plugin::*, core::*, protocol::types::*, McpResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Main plugin structure
pub struct DataPlugin {
    name: String,
    version: String,
    data_store: Arc<Mutex<HashMap<String, Value>>>,
}

impl DataPlugin {
    pub fn new() -> Self {
        Self {
            name: "data-plugin".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// Plugin trait implementation
#[async_trait]
impl Plugin for DataPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        "Data management plugin with storage, retrieval, and analysis capabilities"
    }

    async fn initialize(&mut self) -> McpResult<()> {
        // Initialize plugin resources
        Ok(())
    }

    async fn shutdown(&mut self) -> McpResult<()> {
        // Clean up resources
        Ok(())
    }

    async fn configure(&mut self, config: Value) -> McpResult<()> {
        // Apply configuration
        Ok(())
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            provides_tools: true,
            provides_resources: true,
            provides_prompts: true,
            supports_hot_reload: true,
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            author: Some("Example Author".to_string()),
            license: Some("MIT".to_string()),
            homepage: Some("https://example.com".to_string()),
            repository: Some("https://github.com/example/plugin".to_string()),
            keywords: vec!["data".to_string(), "storage".to_string()],
            categories: vec!["database".to_string()],
        }
    }
}

// Tool Handler: Store data
pub struct StoreDataTool {
    data_store: Arc<Mutex<HashMap<String, Value>>>,
}

#[async_trait]
impl ToolHandler for StoreDataTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let key = arguments.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing 'key' parameter".to_string(),
            })?;
        
        let value = arguments.get("value")
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing 'value' parameter".to_string(),
            })?;
        
        let mut store = self.data_store.lock().await;
        store.insert(key.to_string(), value.clone());
        
        Ok(ToolResult {
            content: vec![ContentBlock::text(format!("Stored value at key: {}", key))],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

// Resource Handler: Read stored data
pub struct DataResource {
    data_store: Arc<Mutex<HashMap<String, Value>>>,
}

#[async_trait]
impl ResourceHandler for DataResource {
    async fn read(
        &self,
        uri: &str,
        params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        let store = self.data_store.lock().await;
        
        if uri == "data://all" {
            let data = serde_json::to_string_pretty(&*store)?;
            Ok(vec![ResourceContents::Text {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: data,
                meta: None,
            }])
        } else if let Some(key) = uri.strip_prefix("data://") {
            match store.get(key) {
                Some(value) => {
                    Ok(vec![ResourceContents::Text {
                        uri: uri.to_string(),
                        mime_type: Some("application/json".to_string()),
                        text: serde_json::to_string_pretty(value)?,
                        meta: None,
                    }])
                }
                None => Err(McpError::ResourceNotFound(uri.to_string()))
            }
        } else {
            Err(McpError::ResourceNotFound(uri.to_string()))
        }
    }
}

// Prompt Handler: Generate analysis prompts
pub struct AnalysisPrompt;

#[async_trait]
impl PromptHandler for AnalysisPrompt {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult> {
        let data = arguments.get("data")
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing 'data' parameter".to_string(),
            })?;
        
        let messages = vec![
            PromptMessage {
                role: Role::System,
                content: Content::text(
                    "You are a data analyst. Provide insights based on the provided data."
                ),
            },
            PromptMessage {
                role: Role::User,
                content: Content::text(format!(
                    "Analyze this data: {}",
                    serde_json::to_string_pretty(data)?
                )),
            },
        ];
        
        Ok(PromptResult {
            description: Some("Data analysis prompt".to_string()),
            messages,
        })
    }
}

// Completion Handler: Provide key suggestions
pub struct KeyCompletionHandler {
    data_store: Arc<Mutex<HashMap<String, Value>>>,
}

#[async_trait]
impl CompletionHandler for KeyCompletionHandler {
    async fn complete(
        &self,
        reference: &CompletionReference,
        argument: &CompletionArgument,
        _context: Option<&CompletionContext>,
    ) -> McpResult<Vec<String>> {
        if let CompletionReference::Tool { name } = reference {
            if name == "retrieve_data" && argument.name == "key" {
                let store = self.data_store.lock().await;
                let keys: Vec<String> = store.keys()
                    .filter(|k| k.starts_with(&argument.value))
                    .cloned()
                    .collect();
                return Ok(keys);
            }
        }
        Ok(vec![])
    }
}

// Export the plugin
plugin_export!(DataPlugin);
```

### Building the Plugin

```bash
# Development build
cargo build

# Release build
cargo build --release

# Output: target/release/libexample_plugin.so (Linux)
#         target/release/example_plugin.dll (Windows)
#         target/release/libexample_plugin.dylib (macOS)
```

## Distribution

### Distribution Channels

#### 1. Crates.io (Public Registry)

```bash
cargo publish
```

#### 2. Private Registry

```toml
[package]
registry = "https://private-registry.example.com"
```

#### 3. Git Repository

```toml
[dependencies]
my-plugin = { git = "https://github.com/org/plugin.git" }
```

#### 4. Binary Distribution

```bash
# Create distribution package
tar -czf plugin-v1.0.0-linux-x64.tar.gz \
    target/release/libplugin.so \
    plugin-manifest.json \
    README.md \
    LICENSE
```

### Plugin Manifest

```json
{
  "name": "example-plugin",
  "version": "1.0.0",
  "description": "Example plugin implementation",
  "author": "Your Name",
  "license": "MIT",
  "mcp_version": "2025-06-18",
  "sdk_version": "0.1.0",
  "entry_point": "libexample_plugin.so",
  "capabilities": {
    "tools": true,
    "resources": true,
    "prompts": true,
    "completions": true,
    "hot_reload": true
  },
  "requirements": {
    "min_sdk_version": "0.1.0",
    "max_sdk_version": "1.0.0"
  }
}
```

## Licensing

### SDK License Protection

The Prism MCP SDK is licensed under the MIT License, providing maximum flexibility for plugin developers. Plugin interfaces benefit from Apache 2.0 patent protection clauses.

### Plugin License Options

Plugins may be licensed under any license compatible with the SDK's MIT license:

- **Open Source**: MIT, Apache 2.0, BSD, MPL 2.0
- **Copyleft**: GPL (with linking exception), LGPL
- **Proprietary**: Commercial licenses with custom terms
- **Dual License**: Different licenses for different use cases

### License Header Example

```rust
// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Your Name
```

## Best Practices

### Error Handling

Implement comprehensive error handling with descriptive messages:

```rust
#[async_trait]
impl ToolHandler for MyTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Validate required parameters
        let param = arguments.get("required_param")
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing required parameter: required_param".to_string(),
            })?;
        
        // Handle operation errors
        match perform_operation(param) {
            Ok(result) => Ok(ToolResult::success(result)),
            Err(e) => Err(McpError::ToolExecution {
                tool: "my_tool".to_string(),
                error: e.to_string(),
            }),
        }
    }
}
```

### Resource Management

Properly manage plugin resources:

```rust
#[async_trait]
impl Plugin for MyPlugin {
    async fn initialize(&mut self) -> McpResult<()> {
        // Acquire resources
        self.connection = Some(Connection::establish().await?);
        Ok(())
    }
    
    async fn shutdown(&mut self) -> McpResult<()> {
        // Release resources
        if let Some(conn) = self.connection.take() {
            conn.close().await?;
        }
        Ok(())
    }
}
```

### Versioning

Implement semantic versioning and compatibility checks:

```rust
use semver::{Version, VersionReq};

impl Plugin for MyPlugin {
    fn is_compatible(&self, sdk_version: &str) -> bool {
        let req = VersionReq::parse(">=0.1.0, <2.0.0").unwrap();
        let version = Version::parse(sdk_version).unwrap();
        req.matches(&version)
    }
}
```

### Testing

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tool_execution() {
        let tool = MyTool::new();
        let mut args = HashMap::new();
        args.insert("param".to_string(), json!("value"));
        
        let result = tool.call(args).await;
        assert!(result.is_ok());
    }
}
```

#### Integration Tests

```rust
#[tokio::test]
async fn test_plugin_loading() {
    let manager = PluginManager::new();
    let result = manager.load_plugin("target/debug/libmy_plugin.so").await;
    assert!(result.is_ok());
    
    let capabilities = manager.get_capabilities("my_plugin");
    assert!(capabilities.provides_tools);
}
```

## API Reference

For detailed API documentation, refer to:

- [Plugin Module Documentation](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/plugin/)
- [Core Types Documentation](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/core/)
- [Protocol Types Documentation](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/protocol/)

## Support

- [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- [Discussion Forum](https://github.com/prismworks-ai/prism-mcp-rs/discussions)
- Email: sdk-support@prismworks.ai
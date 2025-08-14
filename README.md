# Prism MCP SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/prism-mcp-rs.svg)](https://crates.io/crates/prism-mcp-rs)
[![Downloads](https://img.shields.io/crates/d/prism-mcp-rs.svg)](https://crates.io/crates/prism-mcp-rs)
[![Documentation](https://docs.rs/prism-mcp-rs/badge.svg)](https://docs.rs/prism-mcp-rs)
[![CI](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml)
[![Security](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/prismworks-ai/prism-mcp-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/prismworks-ai/prism-mcp-rs)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://blog.rust-lang.org/2025/01/09/Rust-1.85.0.html)
[![deps.rs](https://deps.rs/repo/github/prismworks-ai/prism-mcp-rs/status.svg)](https://deps.rs/repo/github/prismworks-ai/prism-mcp-rs)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/prism-mcp-rs.svg?label=total%20downloads)](https://crates.io/crates/prism-mcp-rs)
[![API Stability](https://img.shields.io/badge/API-beta-orange.svg)](https://github.com/prismworks-ai/prism-mcp-rs/blob/main/CHANGELOG.md)

[![Contributors](https://img.shields.io/github/contributors/prismworks-ai/prism-mcp-rs.svg)](https://github.com/prismworks-ai/prism-mcp-rs/graphs/contributors)
[![GitHub last commit](https://img.shields.io/github/last-commit/prismworks-ai/prism-mcp-rs.svg)](https://github.com/prismworks-ai/prism-mcp-rs/commits/main)
[![GitHub release](https://img.shields.io/github/release/prismworks-ai/prism-mcp-rs.svg)](https://github.com/prismworks-ai/prism-mcp-rs/releases)
[![Discord](https://img.shields.io/discord/123456789?logo=discord&label=Discord)](https://discord.gg/prismworks)
[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-Support-pink.svg)](https://github.com/sponsors/prismworks-ai)

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

## Overview

The Prism MCP SDK provides a Rust implementation of the Model Context Protocol with a distinctive plugin architecture. This architecture enables runtime composition of MCP servers through dynamically loadable plugins, transforming how MCP tools are developed, distributed, and deployed.

## Documentation

- [Development Guide](DEVELOPMENT.md) - Build system, workflows, and contribution process
- [Plugin Development Guide](docs/PLUGIN_GUIDE.md) - Complete guide to building plugins
- [Plugin Types Reference](docs/PLUGIN_TYPES.md) - Detailed component specifications
- [API Documentation](https://docs.rs/prism-mcp-rs) - Complete API reference
- [Contributing Guide](CONTRIBUTING.md) - Code of conduct and submission guidelines

## Key Innovation: Plugin architecture

### Traditional MCP development challenges

Standard MCP implementations compile all tools directly into server binaries:

```rust
// Traditional approach: Monolithic server
struct MCPServer {
    tool1: Tool1,  // Compiled into binary
    tool2: Tool2,  // Compiled into binary
    // Adding new tools requires:
    // - Source code modification
    // - Complete recompilation  
    // - Server restart (downtime)
    // - Full redeployment
}
```

This approach creates several limitations:
- Tools are tightly coupled to server implementations
- Adding or updating tools requires server downtime
- All components share the same version and release cycle
- Tool developers must maintain entire server codebases
- Limited code reuse across different servers

### Plugin-based architecture

The Prism MCP SDK introduces plugins as composable units:

```rust
// Prism approach: Composable server
struct PrismServer {
    plugin_manager: PluginManager,
    // Adding tools through plugins:
    // - Drop plugin file into directory
    // - No compilation required
    // - No restart needed (hot reload)
    // - No redeployment
}
```

This architecture provides:
- Runtime plugin loading and unloading
- Independent versioning for each plugin
- Hot reload capability during development
- Simplified tool distribution and sharing
- Reduced barriers to contribution

## Architectural components

### Plugin system design

Plugins serve as containers for MCP components, bundling related functionality:

```
Plugin (Dynamic Library)
├── Tools (Executable functions)
├── Resources (Data providers)
├── Prompts (Message templates)
└── Completions (Autocomplete providers)
```

Each plugin operates independently with:
- Isolated namespace and state
- Independent lifecycle management
- Configurable capabilities
- Version compatibility checking

### Server composition model

Servers become lightweight hosts that compose functionality from plugins:

```yaml
# plugins.yaml - Declarative server configuration
plugins:
  - name: github-tools
    version: "1.2.0"
    enabled: true
    
  - name: database-tools  
    version: "2.0.1"
    enabled: true
    
  - name: api-tools
    version: "0.5.0"
    enabled: false  # Disabled without removal
```

## Technical capabilities

### Dynamic loading

The SDK uses Rust's `libloading` crate for safe dynamic library loading:

```rust
use prism_mcp_rs::plugin::PluginManager;

#[tokio::main]
async fn main() -> Result<()> {
    let mut manager = PluginManager::new();
    
    // Load plugin at runtime
    manager.load_plugin("./plugins/github-tools.so").await?;
    
    // Plugin immediately available
    let tools = manager.list_tools();
    
    // Support for hot reload
    manager.reload_plugin("github-tools").await?;
    
    Ok(())
}
```

### Plugin development

Plugins are standard Rust crates with specific traits:

```rust
use prism_mcp_rs::plugin::*;
use async_trait::async_trait;

pub struct MyPlugin {
    // Plugin state
}

#[async_trait]
impl Plugin for MyPlugin {
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            provides_tools: true,
            provides_resources: true,
            provides_prompts: true,
            supports_hot_reload: true,
        }
    }
    
    // Additional trait methods
}

// Export for dynamic loading
plugin_export!(MyPlugin);
```

### MCP protocol support

The SDK implements comprehensive MCP specification support:

#### Protocol features
- Tools, resources, prompts, and completions
- Audio content and multimodal support
- Bidirectional communication
- Argument autocompletion
- Resource templates and patterns
- Annotations and metadata

#### Transport layers
- **HTTP/2**: Server push and multiplexing support
- **WebSocket**: Automatic compression
- **Standard I/O**: CLI integration
- **Reconnection**: Exponential backoff strategy

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
prism-mcp-rs = "0.1.0"
```

With specific features:

```toml
[dependencies]
prism-mcp-rs = {
    version = "0.1.0",
    features = ["plugin", "websocket", "http"]
}
```

### Available features

| Feature | Description |
|---------|-------------|
| `plugin` | Plugin system support |
| `websocket` | WebSocket transport |
| `http` | HTTP/1.1 and HTTP/2 transport |
| `stdio` | Standard I/O transport |
| `auth` | Authentication mechanisms |
| `tls` | TLS/SSL support |
| `full` | All features enabled |
| `minimal` | Core functionality only |

## Usage examples

### Server with plugin support

```rust
use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> McpResult<()> {
    let mut server = McpServer::new("my-server", "1.0.0");
    
    // Initialize plugin manager
    let plugin_manager = PluginManager::new();
    
    // Load plugins from directory
    plugin_manager.load_from_directory("./plugins").await?;
    
    // Attach to server
    server.with_plugin_manager(plugin_manager);
    
    // Run with selected transport
    server.run_with_websocket("127.0.0.1:8080").await?;
    
    Ok(())
}
```

### Plugin development

```rust
use prism_mcp_rs::plugin::*;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct CalculatorPlugin;

#[async_trait]
impl Plugin for CalculatorPlugin {
    fn name(&self) -> &str {
        "calculator"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Basic arithmetic operations"
    }
    
    async fn initialize(&mut self) -> McpResult<()> {
        Ok(())
    }
    
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            provides_tools: true,
            provides_resources: false,
            provides_prompts: false,
            supports_hot_reload: true,
        }
    }
}

// Tool implementation
pub struct CalculatorTool;

#[async_trait]
impl ToolHandler for CalculatorTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let a = arguments.get("a")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let b = arguments.get("b")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let operation = arguments.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("add");
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" if b != 0.0 => a / b,
            _ => 0.0,
        };
        
        Ok(ToolResult {
            content: vec![ContentBlock::text(format!("Result: {}", result))],
        })
    }
}

// Export the plugin
plugin_export!(CalculatorPlugin);
```

### Client implementation

```rust
use prism_mcp_rs::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Connect to server
    let session = ClientSession::connect_with_websocket(
        "ws://localhost:8080",
        "my-client",
        "1.0.0"
    ).await?;
    
    // Initialize connection
    session.initialize().await?;
    
    // Call tool from plugin
    let result = session.call_tool(
        "calculator.add",
        json!({
            "a": 10,
            "b": 5,
            "operation": "multiply"
        })
    ).await?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}
```

## Development tools

### Related tools

For enhanced development experience with the Prism MCP SDK, consider using the companion developer tools available in the [mcp-rs-dev](https://github.com/prismworks-ai/mcp-rs-dev) repository. These tools provide CLI utilities for plugin development, testing, and deployment workflows.

### Testing

```bash
# Run all tests
cargo test

# Test with all features
cargo test --all-features

# Test specific module
cargo test plugin::

### CI/CD and quality metrics

The project includes comprehensive CI/CD with automatic reporting:

#### 📊 Automatic reports

- **Coverage reports**: Code coverage metrics with trends
- **Benchmark reports**: Performance metrics for all components
- **Format**: Markdown, viewable directly on GitHub
- **Location**: `reports/` directory

#### 🚀 For contributors

**No tokens needed!** All CI features work automatically:
- Testing, linting, and validation
- Coverage and benchmark generation
- PR checks and status updates

#### 📦 For maintainers

Only publishing to crates.io requires the `CRATES_IO_TOKEN` secret.

```bash
# Generate reports locally
make reports              # Both coverage and benchmarks
make report-coverage      # Coverage only
make report-bench         # Benchmarks only

# Or use Act to run GitHub Actions locally
act -j coverage    # Run coverage job from CI workflow
act push          # Run full CI pipeline
```

# Run integration tests
cargo test --test integration
```

### API usage patterns

The SDK provides comprehensive APIs for building MCP servers and clients:

#### Creating a basic MCP server

```rust
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::server::McpServer;
use prism_mcp_rs::transport::stdio::StdioTransport;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize server
    let mut server = McpServer::new("example-server", "1.0.0");
    
    // Add tool handler
    server.add_tool(
        Tool::new("example_tool")
            .with_description("An example tool")
            .with_handler(|args| async move {
                Ok(ToolResult::text("Tool executed successfully"))
            })
    )?;
    
    // Run with stdio transport
    let transport = StdioTransport::new();
    server.run(transport).await
}
```

#### Loading plugins dynamically

```rust
use prism_mcp_rs::plugin::{PluginManager, PluginConfig};

// Create plugin manager
let mut manager = PluginManager::new();

// Load individual plugin
manager.load_plugin("./plugins/my_plugin.so").await?;

// Load all plugins from directory
manager.load_from_directory("./plugins").await?;

// Configure plugin
let config = PluginConfig {
    enabled: true,
    hot_reload: true,
    settings: serde_json::json!({
        "api_key": "your-key",
        "timeout": 30
    }),
};
manager.configure_plugin("my_plugin", config).await?;

// List loaded plugins
for plugin in manager.list_plugins() {
    println!("Plugin: {} v{}", plugin.name(), plugin.version());
}
```

#### Building a plugin

```rust
use prism_mcp_rs::plugin::*;
use async_trait::async_trait;

// Define your plugin struct
pub struct MyPlugin {
    // Plugin state
}

// Implement the Plugin trait
#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my_plugin" }
    fn version(&self) -> &str { "1.0.0" }
    
    async fn initialize(&mut self) -> McpResult<()> {
        // Setup code here
        Ok(())
    }
    
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            provides_tools: true,
            provides_resources: true,
            provides_prompts: false,
            supports_hot_reload: true,
        }
    }
}

// Export plugin for dynamic loading
plugin_export!(MyPlugin);
```

#### Client session management

```rust
use prism_mcp_rs::client::{ClientSession, ClientConfig};
use prism_mcp_rs::transport::websocket::WebSocketTransport;

// Configure client
let config = ClientConfig {
    name: "my-client",
    version: "1.0.0",
    timeout: Duration::from_secs(30),
    retry_strategy: RetryStrategy::exponential_backoff(),
};

// Connect to server
let transport = WebSocketTransport::connect("ws://localhost:8080").await?;
let session = ClientSession::new(config, transport);

// Initialize connection
session.initialize().await?;

// List available tools
let tools = session.list_tools().await?;
for tool in tools {
    println!("Tool: {} - {}", tool.name, tool.description);
}

// Call a tool
let result = session.call_tool(
    "example_tool",
    serde_json::json!({ "param": "value" })
).await?;
```


## Plugin distribution

### Distribution channels

Plugins can be distributed through multiple channels:

1. **Crates.io**: Standard Rust package registry
2. **Git repositories**: Direct git dependencies
3. **Binary packages**: Compiled dynamic libraries
4. **Private registries**: Enterprise distribution

### Plugin manifest

```json
{
  "name": "example-plugin",
  "version": "1.0.0",
  "description": "Example plugin implementation",
  "mcp_version": "2025-06-18",
  "sdk_version": "0.1.0",
  "entry_point": "libexample_plugin.so",
  "capabilities": {
    "tools": true,
    "resources": true,
    "prompts": true,
    "completions": true,
    "hot_reload": true
  }
}
```

## Deployment patterns

### Domain-specific servers

Compose specialized servers from generic plugins:

```yaml
# devops-server.yaml
plugins:
  - kubernetes-tools
  - terraform-tools
  - aws-tools
  - monitoring-tools

# data-science-server.yaml  
plugins:
  - jupyter-tools
  - pandas-tools
  - visualization-tools
  - ml-tools
```

### Progressive enhancement

Add capabilities incrementally without modifying server code:

```rust
// Initial deployment: Basic server
let server = McpServer::new("server", "1.0.0");

// Later: Add plugins by dropping files into plugins/
// No code changes or recompilation required
```

### A/B testing

Test different tool implementations safely:

```rust
// Load multiple versions
manager.load_plugin("search-v1.so").await?;
manager.load_plugin("search-v2.so").await?;

// Route based on criteria
if client.is_beta_tester() {
    manager.enable_plugin("search-v2");
} else {
    manager.enable_plugin("search-v1");
}
```

## Performance characteristics

- **Plugin loading**: Sub-10ms per plugin
- **Hot reload**: Sub-50ms interruption
- **Memory isolation**: Per-plugin memory management
- **Concurrent execution**: Async plugin execution
- **Message throughput**: Optimized for high volume

## Enterprise integration

Organizations requiring enterprise features can leverage:

- Private plugin registries
- Plugin-level access control
- Audit logging for compliance
- Performance monitoring
- SLA-backed support

Contact contact@prismworks.ai for enterprise offerings.

## Examples

| Example | Description |
|---------|-------------|
| [Plugin Development](examples/plugins/) | Creating and loading plugins |
| [WebSocket Server](examples/server/websocket_server.rs) | WebSocket transport implementation |
| [HTTP/2 Server](examples/server/http2_server.rs) | HTTP/2 with server push |
| [Calculator Plugin](examples/plugins/calculator/) | Complete plugin example |
| [Database Server](examples/server/database_server.rs) | Resource management patterns |
| [Error Handling](examples/production_error_handling_demo.rs) | Production error patterns |
| [Performance Tests](examples/performance_benchmarks.rs) | Benchmarking utilities |
| [CLI Integration](examples/cli_integration/) | Using prism-cli |

## Licensing

The Prism MCP SDK is licensed under the MIT License. Plugins may choose any compatible license.

## Support

- [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- [GitHub Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions)
- Email: sdk@prismworks.ai

---

Built by [Prismworks AI](https://prismworks.ai)
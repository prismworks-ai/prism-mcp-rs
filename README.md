# Prism MCP SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/prism-mcp-rs.svg)](https://crates.io/crates/prism-mcp-rs)
[![Documentation](https://docs.rs/prism-mcp-rs/badge.svg)](https://docs.rs/prism-mcp-rs)
[![CI](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml)
[![Security](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/prismworks-ai/prism-mcp-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/prismworks-ai/prism-mcp-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.85%2B-blue.svg)](https://www.rust-lang.org)

Production-ready Rust implementation of the Model Context Protocol (MCP), providing comprehensive client and server implementations with multiple transport support and a revolutionary plugin architecture for tool development.

## Why Choose Prism MCP SDK

### Distinctive Features

#### Plugin Architecture for Tool Development
- **Decoupled Tools** - Tools are completely decoupled from MCP servers, enabling independent development and deployment
- **Dynamic Loading** - Add and remove tools at runtime without server restarts
- **Separate Crate Distribution** - Deploy tools as independent crates with their own versioning and release cycles
- **Apache 2.0 Protection** - Plugin developers maintain full control over licensing of their tools while benefiting from SDK protection
- **Tool Ecosystem Foundation** - Architecture ready for community plugin contributions and marketplace development

#### Unmatched Developer Experience
- **Prism CLI Integration** - Comprehensive CLI tools via [prism-cli](https://crates.io/crates/prism-cli) ([GitHub](https://github.com/prismworks-ai/mcp-rs-dev/tree/main/tools/prism-cli))
- **One-Shot Commands** - SDK convenience commands via `prs` command for rapid development
- **Project Scaffolding** - Generate complete MCP projects with a single command
- **Tool Testing Framework** - Built-in testing utilities for plugin development
- **Hot Reload Support** - Develop tools with instant feedback during development

#### Complete MCP 2025-06-18 Specification Support
- Full audio content support for multimodal interactions
- Advanced annotation system for tools and content metadata
- Argument autocompletion for improved developer experience
- File system roots for controlled resource access
- Bidirectional communication enabling server-initiated requests
- Resource templates with pattern-based discovery

#### Production Architecture
- Zero unsafe code - 100% safe Rust implementation
- 65%+ test coverage with comprehensive test suite
- Memory safety guaranteed through Rust's ownership system
- Production-tested implementations

#### Performance Characteristics
- 100,000+ messages/second throughput (WebSocket)
- Sub-millisecond latency for local operations
- 10MB base memory footprint
- Support for 10,000+ concurrent connections

#### Transport Layer Features
- HTTP/2 with server push and multiplexing
- WebSocket with automatic compression
- Intelligent reconnection with exponential backoff
- Transport-agnostic API design

### Feature Comparison

| Feature | Prism MCP SDK | Other Rust SDKs | TypeScript SDK |
|---------|---------------|-----------------|----------------|
| MCP 2025-06-18 Specification | Complete | Partial | Partial |
| **Plugin Architecture** | **Complete** | No | No |
| ├ Tool Handlers | Yes | Static only | Static only |
| ├ Resource Handlers | Yes | Static only | Static only |
| ├ Prompt Handlers | Yes | No | Limited |
| ├ Completion Handlers | Yes | No | Limited |
| └ Hot Reload Support | Yes | No | No |
| Dynamic Loading | Runtime | Compile-time | Limited |
| CLI Development Tools | Yes ([prism-cli](https://crates.io/crates/prism-cli)) | No | No |
| Audio Content Support | Yes | No | Limited |
| Bidirectional Communication | Yes | No | Yes |
| Zero Unsafe Code | Yes | No | N/A |
| HTTP/2 Server Push | Yes | No | No |
| WebSocket Compression | Yes | Limited | Yes |
| Test Coverage | 65%+ | ~40% | ~50% |
| Production Examples | 15+ | 3-5 | 5-7 |
| Component Isolation | Complete | None | Partial |
| Independent Versioning | Per-plugin | Monolithic | Monolithic |
| Apache 2.0 Plugin Protection | Yes | No | No |

## Core Features

### Protocol Implementation
- Complete MCP protocol compliance
- Support for WebSocket, HTTP/2, HTTP/1.1, and stdio transports
- Asynchronous operations using Tokio runtime
- Type-safe API with compile-time validation
- Zero unsafe code throughout the codebase

### Plugin System

#### Architecture Overview

Plugins are dynamically loadable libraries that extend MCP server capabilities without requiring server modifications or restarts. Each plugin can provide multiple component types:

- **Tools**: Executable functions that perform operations
- **Resources**: Data providers accessible via URI patterns
- **Prompts**: Template generators for LLM interactions
- **Completions**: Autocomplete providers for enhanced UX

#### Key Capabilities

- **Runtime Loading**: Load/unload plugins without server restart
- **Component Isolation**: Each plugin operates in its own namespace
- **Version Management**: Independent versioning per plugin
- **Hot Reload**: Update plugins while server is running
- **Distribution Flexibility**: Via crates.io, git, or binaries

### Developer Tools
- Integration with [prism-cli](https://crates.io/crates/prism-cli) for project management
- Built-in debugging and profiling support
- Extensive example collection
- Complete API documentation
- Testing utilities and mocks

## Installation

```toml
[dependencies]
prism-mcp-rs = "0.1.0"
```

With optional features:

```toml
[dependencies]
prism-mcp-rs = {
    version = "0.1.0",
    features = ["websocket", "http", "stdio", "plugin", "full"]
}
```

### Available Features

| Feature | Description |
|---------|-------------|
| `websocket` | WebSocket transport support |
| `http` | HTTP/1.1 and HTTP/2 transport |
| `stdio` | Standard I/O transport |
| `plugin` | Plugin system for dynamic tools |
| `auth` | Authentication mechanisms |
| `tls` | TLS/SSL support |
| `full` | All features enabled |
| `minimal` | Core functionality only |

## Quick Start

### Using Prism CLI (Recommended)

```bash
# Install the CLI (provides the 'prs' command)
cargo install prism-cli

# Use the 'prs' command for development
prs new plugin my-plugin  # Create new plugin project
prs build                  # Build the project
prs test                   # Run tests
prs dev                    # Run development server with hot reload
prs publish                # Publish to registry
```

### Manual Server Implementation

```rust
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;
use serde_json::{json, Value};

// Define a tool handler
struct CalculatorHandler;

#[async_trait]
impl ToolHandler for CalculatorHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let a = arguments.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b = arguments.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
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

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize server
    let mut server = McpServer::new("calculator-server", "1.0.0");
    
    // Register tool
    server.register_tool(
        "calculator",
        "Perform basic math operations",
        json!({
            "type": "object",
            "properties": {
                "a": {"type": "number"},
                "b": {"type": "number"},
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                }
            }
        }),
        Box::new(CalculatorHandler),
    )?;

    // Run with selected transport
    #[cfg(feature = "websocket")]
    server.run_with_websocket("127.0.0.1:8080").await?;
    
    #[cfg(feature = "http")]
    server.run_with_http("127.0.0.1:8080").await?;
    
    #[cfg(feature = "stdio")]
    server.run_with_stdio().await?;
    
    Ok(())
}
```

### Plugin Development

#### Traditional Approach vs Plugin Architecture

**Traditional MCP Implementation:**
```rust
// All components compiled into server binary
struct MyServer {
    tool1: Tool1,  // Compiled in
    tool2: Tool2,  // Compiled in
    // Adding new tool requires:
    // 1. Modify server code
    // 2. Recompile entire server
    // 3. Restart server (downtime)
    // 4. Redeploy everything
}
```

**Prism Plugin Architecture:**
```rust
// Server remains unchanged
struct MyServer {
    plugin_manager: PluginManager,
    // Adding new tool requires:
    // 1. Load plugin file
    // No compilation, no restart, no downtime
}

// Plugins are separate crates
use prism_mcp_rs::plugin::*;

pub struct MyPlugin {
    name: String,
    version: String,
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
}

// Export for dynamic loading
plugin_export!(MyPlugin);
```

### Client Implementation

```rust
use prism_mcp_rs::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Connect client
    #[cfg(feature = "websocket")]
    let session = ClientSession::connect_with_websocket(
        "ws://localhost:8080",
        "my-client",
        "1.0.0"
    ).await?;
    
    // Initialize connection
    session.initialize().await?;
    
    // Call tool
    let result = session.call_tool(
        "calculator",
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

## Transport Configuration

### WebSocket Transport

```rust
// Server
server.run_with_websocket("127.0.0.1:8080").await?;

// Client
let session = ClientSession::connect_with_websocket(
    "ws://localhost:8080",
    "client-name",
    "1.0.0"
).await?;
```

### HTTP/2 Transport

```rust
// Server configuration
let config = HttpServerConfig {
    enable_http2: true,
    enable_compression: true,
    ..Default::default()
};
server.run_with_http_config("127.0.0.1:8080", config).await?;

// Client connection
let session = ClientSession::connect_with_http(
    "http://localhost:8080",
    "client-name",
    "1.0.0"
).await?;
```

### Standard I/O Transport

```rust
// Server
server.run_with_stdio().await?;

// Client
let session = ClientSession::connect_with_stdio(
    "path/to/server",
    "client-name",
    "1.0.0"
).await?;
```

## Advanced Usage

### Dynamic Plugin Loading

```rust
use prism_mcp_rs::plugin::PluginManager;

// Initialize plugin manager
let mut plugin_manager = PluginManager::new();

// Load plugin from file
plugin_manager.load_plugin("path/to/plugin.so").await?;

// List loaded plugins
let plugins = plugin_manager.list_plugins();

// Unload plugin
plugin_manager.unload_plugin("plugin-name").await?;
```

### Resource Management

```rust
struct FileResourceHandler;

#[async_trait]
impl ResourceHandler for FileResourceHandler {
    async fn read(&self, uri: &str) -> McpResult<ResourceContents> {
        let content = tokio::fs::read_to_string(uri).await?;
        Ok(ResourceContents {
            uri: uri.to_string(),
            mime_type: Some("text/plain".to_string()),
            content: Content::text(content),
        })
    }
}

// Register resource handler
server.register_resource(
    "file:///configs/*.json",
    "Configuration files",
    Box::new(FileResourceHandler),
)?;
```

### Error Handling

```rust
use prism_mcp_rs::core::error::{McpError, ErrorCode};

match session.call_tool("tool_name", args).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(McpError::Protocol { code, message, .. }) => {
        eprintln!("Protocol error {}: {}", code, message);
    }
    Err(McpError::Transport(e)) => {
        eprintln!("Transport error: {}", e);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Project Structure

```
prism-mcp-rs/
├── core/           # Core protocol implementation
│   ├── protocol/   # MCP protocol types and messages
│   ├── error/      # Error types and handling
│   └── handlers/   # Tool and resource handlers
├── transport/      # Transport layer implementations
│   ├── websocket/  # WebSocket transport
│   ├── http/       # HTTP/1.1 and HTTP/2
│   └── stdio/      # Standard I/O transport
├── plugin/         # Plugin system
│   ├── loader/     # Dynamic loading mechanisms
│   ├── registry/   # Plugin registry and discovery
│   └── sandbox/    # Plugin isolation
├── server/         # Server implementation
├── client/         # Client implementation
└── examples/       # Example implementations
```

## Performance Metrics

| Metric | Value |
|--------|-------|
| Message Throughput | 100,000+ messages/second (WebSocket) |
| Response Latency | <1ms (local) |
| Memory Usage | ~10MB base footprint |
| Concurrent Connections | 10,000+ (with tuning) |
| Plugin Load Time | <100ms |
| Tool Execution Overhead | <1ms |

## Testing

```bash
# Run all tests
cargo test

# Run with all features
cargo test --all-features

# Run specific module tests
cargo test transport::

# Run plugin tests
cargo test plugin::

# Run integration tests
cargo test --test integration
```

## Enterprise Platform Integration

While the SDK provides core MCP functionality, enterprises requiring advanced features such as centralized authentication, audit logging, rate limiting, and metrics can leverage the Prismworks Platform. The platform extends SDK capabilities with:

- Centralized tool registry and marketplace
- Enterprise authentication and authorization
- Comprehensive audit trails and compliance reporting
- Advanced monitoring and observability
- Multi-tenant support and isolation

Contact contact@prismworks.ai for platform access.

## Licensing

### SDK License
The Prism MCP SDK is licensed under the MIT License, providing maximum flexibility for both open-source and commercial use.

### Plugin Developer Rights
The SDK's plugin architecture extends Apache 2.0 license protection to plugin developers, ensuring:
- Full control over plugin licensing terms
- Protection from patent claims when using SDK interfaces
- Freedom to distribute plugins under any compatible license
- Commercial deployment without SDK licensing restrictions

Plugin developers maintain complete ownership and licensing control of their tools while benefiting from the SDK's legal protections.

## Documentation

- [API Documentation](https://docs.rs/prism-mcp-rs) - Complete API reference
- [Prism CLI Documentation](https://github.com/prismworks-ai/mcp-rs-dev/tree/main/tools/prism-cli) - CLI tools and commands
- [Plugin Development Guide](https://github.com/prismworks-ai/prism-mcp-rs/blob/main/docs/PLUGIN_GUIDE.md) - Creating and distributing plugins
- [Examples](https://github.com/prismworks-ai/prism-mcp-rs/tree/main/examples) - Production examples
- [MCP Specification](https://modelcontextprotocol.io/specification) - Protocol specification
- [Contributing Guide](https://github.com/prismworks-ai/prism-mcp-rs/blob/main/CONTRIBUTING.md) - Contribution guidelines

### Example Applications

| Example | Description |
|---------|-------------|
| [Plugin Development](examples/plugin/) | Creating and loading dynamic tools |
| [WebSocket Server/Client](examples/server/websocket_server.rs) | Bidirectional real-time communication |
| [HTTP/2 Server](examples/server/http2_server.rs) | HTTP/2 with server push |
| [Calculator Tool](examples/plugins/calculator/) | Complete tool plugin example |
| [Database Server](examples/server/database_server.rs) | Resource management patterns |
| [Error Handling](examples/production_error_handling_demo.rs) | Production error handling |
| [Performance Benchmarks](examples/performance_benchmarks.rs) | Benchmarking utilities |
| [Advanced Features](examples/advanced_2025_features.rs) | Audio, annotations, completion |
| [CLI Integration](examples/cli_integration/) | Using prism-cli for development |

## Contributing

Contributions are welcome. Please refer to the [Contributing Guide](CONTRIBUTING.md) for detailed information on development setup, coding standards, and submission process.

### Development Setup

```bash
# Clone repository
git clone https://github.com/prismworks-ai/prism-mcp-rs.git
cd prism-mcp-rs

# Initialize development environment
make dev-setup

# Run tests
make test

# Verify changes before commit
make commit-ready
```

## Related Projects

### MCP Ecosystem
- [prism-cli](https://crates.io/crates/prism-cli) - Developer CLI with `prs` command
- [mcp-rs-dev](https://github.com/prismworks-ai/mcp-rs-dev) - Development tools and example plugins
- [mcp-rs-registry](https://github.com/prismworks-ai/mcp-rs-registry) - Plugin discovery and registry

## About Prismworks AI

Prismworks AI develops enterprise AI integration infrastructure. The Prism MCP SDK represents our commitment to providing production-grade tools for the Model Context Protocol ecosystem.

## Contact

- **Email**: contact@prismworks.ai
- **Issues**: [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions)

---

Built by [Prismworks AI](https://prismworks.ai)
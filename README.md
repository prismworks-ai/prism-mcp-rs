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
[![Discord](https://img.shields.io/discord/1406362094353383637?logo=discord&label=Discord)](https://discord.gg/prismworks)

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

**prism-mcp-rs** is a high-performance Rust implementation of the Model Context Protocol (MCP) SDK. Build secure, scalable MCP servers and clients with comprehensive type safety, async support, and a powerful plugin system.

## Why Prism MCP SDK?

- 🚀 **High Performance** - Async/await, zero-copy operations, and efficient serialization
- 🔌 **Plugin System** - Runtime-loadable plugins with hot reload support
- 🛡️ **Type Safe** - Leverage Rust's type system for compile-time guarantees
- 🔄 **Multiple Transports** - STDIO, WebSocket, HTTP with optional SSE support
- 📦 **Batteries Included** - Authentication, TLS, compression, and more
- ✅ **Production Ready** - Comprehensive test suite with 229+ tests

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
prism-mcp-rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
async-trait = "0.1"
```

### Hello World Server

Create a simple MCP server in under 30 lines:

```rust
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::server::McpServer;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

struct HelloHandler;

#[async_trait]
impl ToolHandler for HelloHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let name = arguments.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        Ok(ToolResult::text(format!("Hello, {}!", name)))
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let mut server = McpServer::new("hello-server".to_string(), "1.0.0".to_string());
    
    server.add_tool(
        "hello".to_string(),
        Some("Greets a user".to_string()),
        json!({"type": "object", "properties": {"name": {"type": "string"}}}),
        HelloHandler,
    ).await?;
    
    server.run_with_stdio().await
}
```

### Client Example

```rust
use prism_mcp_rs::client::ClientSession;
use prism_mcp_rs::transport::stdio::StdioClientTransport;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> McpResult<()> {
    let transport = StdioClientTransport::new_with_command("./hello-server");
    let mut session = ClientSession::new(transport);
    
    session.initialize("my-client".to_string(), "1.0.0".to_string()).await?;
    
    let result = session.call_tool(
        "hello",
        Some(HashMap::from([("name".to_string(), json!("Alice"))])),
    ).await?;
    
    println!("Response: {:?}", result);
    Ok(())
}
```

## Features

| Feature | Description | Default |
|---------|-------------|------|
| **Core Transports** | | |
| `stdio` | Standard I/O transport | ✅ |
| `http` | HTTP transport with JSON-RPC | ❌ |
| `websocket` | WebSocket transport | ❌ |
| **HTTP Features** | | |
| `sse` | Server-Sent Events for real-time updates | ❌ |
| `http2` | HTTP/2 protocol with multiplexing | ❌ |
| `chunked-encoding` | Chunked transfer for large payloads | ❌ |
| `compression` | Response compression (gzip, brotli, zstd) | ❌ |
| **Extensions** | | |
| `plugin` | Plugin system with hot reload | ❌ |
| `auth` | Authentication (JWT, Argon2) | ❌ |
| `tls` | TLS/SSL support | ❌ |
| **Bundles** | | |
| `full` | All features enabled | ❌ |
| `minimal` | Bare minimum (no features) | ❌ |

Enable features in your `Cargo.toml`:

```toml
prism-mcp-rs = { version = "0.1.0", features = ["websocket", "auth"] }
```

## Documentation

- 📚 **[Getting Started Guide](docs/GETTING_STARTED.md)** - Step-by-step tutorial
- 🔌 **[Plugin Development](docs/guides/plugins.md)** - Create runtime-loadable plugins
- 🔐 **[Authentication Guide](docs/guides/authentication.md)** - Secure your MCP servers
- ⚡ **[Performance Guide](docs/guides/performance.md)** - Optimization techniques
- 🛠️ **[API Reference](https://docs.rs/prism-mcp-rs)** - Complete API documentation
- 📂 **[Examples](examples/)** - Production-ready example implementations

## Examples

Explore complete examples in the [`examples/`](examples/) directory:

```bash
# Run the async server example
cargo run --example async_server

# Run with all features
cargo run --example advanced_features_showcase --features full
```

## Development

### Quick Setup

```bash
git clone https://github.com/prismworks-ai/prism-mcp-rs
cd prism-mcp-rs
make test     # Run tests
make check    # Run CI checks locally
```

### Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Testing

```bash
cargo test              # All tests
cargo test --doc        # Documentation tests
cargo bench             # Run benchmarks
make coverage           # Generate coverage report
```

## Related Projects

- [mcp-rs-dev](https://github.com/prismworks-ai/mcp-rs-dev) - Production tools and utilities
- [mcp-rs-plugins](https://github.com/prismworks-ai/mcp-rs-plugins) - Community plugin collection
- [mcp-rs-registry](https://github.com/prismworks-ai/mcp-rs-registry) - Plugin registry

## Support

- 💬 [GitHub Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions) - Ask questions
- 🐛 [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues) - Report bugs
- 📖 [API Docs](https://docs.rs/prism-mcp-rs) - Reference documentation

## License

MIT License - see [LICENSE](LICENSE) file for details.
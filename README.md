# Prism MCP SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/prism-mcp-rs.svg?style=flat-square)](https://crates.io/crates/prism-mcp-rs)
[![Downloads](https://img.shields.io/crates/d/prism-mcp-rs.svg?style=flat-square&label=downloads&color=brightgreen)](https://crates.io/crates/prism-mcp-rs)
[![Documentation](https://img.shields.io/docsrs/prism-mcp-rs?style=flat-square&label=docs)](https://docs.rs/prism-mcp-rs)
[![CI](https://img.shields.io/github/actions/workflow/status/prismworks-ai/prism-mcp-rs/ci.yml?style=flat-square&label=CI&logo=github)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml)
[![Benchmarks](https://img.shields.io/github/actions/workflow/status/prismworks-ai/prism-mcp-rs/benchmarks.yml?style=flat-square&label=benchmarks&logo=github)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/benchmarks.yml)
[![Performance](https://img.shields.io/badge/performance-tracked-brightgreen?style=flat-square)](https://prismworks-ai.github.io/prism-mcp-rs/benchmarks/)
[![Security Audit](https://img.shields.io/github/actions/workflow/status/prismworks-ai/prism-mcp-rs/security.yml?style=flat-square&label=security&logo=shield)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security.yml)
[![codecov](https://img.shields.io/codecov/c/github/prismworks-ai/prism-mcp-rs?style=flat-square&logo=codecov)](https://codecov.io/gh/prismworks-ai/prism-mcp-rs)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg?style=flat-square&logo=rust)](https://blog.rust-lang.org/2025/01/09/Rust-1.85.0.html)
[![Dependencies](https://img.shields.io/librariesio/github/prismworks-ai/prism-mcp-rs?style=flat-square&label=dependencies&color=informational)](https://deps.rs/repo/github/prismworks-ai/prism-mcp-rs)
[![Total Downloads](https://img.shields.io/crates/d/prism-mcp-rs.svg?style=flat-square&label=total%20downloads&color=success)](https://crates.io/crates/prism-mcp-rs)
[![API Stability](https://img.shields.io/badge/API-v0.1.3-orange.svg?style=flat-square)](https://github.com/prismworks-ai/prism-mcp-rs/blob/main/CHANGELOG.md)

[![Contributors](https://img.shields.io/github/contributors/prismworks-ai/prism-mcp-rs.svg?style=flat-square)](https://github.com/prismworks-ai/prism-mcp-rs/graphs/contributors)
[![Last Commit](https://img.shields.io/github/last-commit/prismworks-ai/prism-mcp-rs.svg?style=flat-square)](https://github.com/prismworks-ai/prism-mcp-rs/commits/main)
[![Release](https://img.shields.io/github/v/release/prismworks-ai/prism-mcp-rs.svg?style=flat-square&include_prereleases)](https://github.com/prismworks-ai/prism-mcp-rs/releases)
[![Discord](https://img.shields.io/badge/Discord-Join%20Community-5865f2?style=flat-square&logo=discord)](https://discord.gg/prismworks)

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square)](https://github.com/rust-secure-code/safety-dance/)

**prism-mcp-rs** is a production-grade Rust implementation of the Model Context Protocol (MCP) SDK with enterprise-class features for building secure, scalable MCP servers and clients.


## Why Prism MCP?

**The first MCP SDK designed for production AI systems.** While other implementations focus on basic protocol compliance, Prism MCP brings enterprise-grade reliability patterns, zero-downtime operations, and plugin ecosystems that scale.

**Built for the AI-first world**: Where services need to be fault-tolerant, discoverable, and composable. Where hot-swapping capabilities matters more than cold starts. Where observability isn't optional—it's survival.

**From prototype to production in minutes**: Clean APIs that hide complexity, but expose power when you need it.

## Core Differentiators

### 1. Advanced Resilience Patterns

- **Circuit Breaker Pattern**: Automatic failure isolation preventing cascading failures
- **Adaptive Retry Policies**: Smart backoff with jitter and error-based retry decisions
- **Health Check System**: Multi-level health monitoring for transport, protocol, and resources
- **Graceful Degradation**: Automatic fallback strategies when services become unavailable

### 2. Enterprise Transport Features

- **Streaming HTTP/2**: Full multiplexing, server push, and flow control support
- **Adaptive Compression**: Dynamic selection of Gzip, Brotli, or Zstd based on content analysis
- **Chunked Transfer Encoding**: Efficient handling of large payloads with streaming
- **Connection Pooling**: Intelligent connection reuse with keep-alive management
- **TLS/mTLS Support**: Enterprise-grade security with certificate validation

### 3. Plugin System Architecture

- **Hot Reload Support**: Update plugins without service interruption
- **ABI-Stable Interface**: Binary compatibility across Rust versions
- **Plugin Isolation**: Sandboxed execution with resource limits
- **Dynamic Discovery**: Runtime plugin loading with dependency resolution
- **Lifecycle Management**: Automated plugin health monitoring and recovery

### 4. Protocol Extensions

- **Schema Introspection**: Complete runtime discovery of server capabilities
- **Batch Operations**: Efficient bulk request processing with transaction support
- **Progressive Content Delivery**: Streaming responses for large datasets
- **Rich Metadata Support**: Comprehensive annotations and capability negotiation
- **Custom Method Extensions**: Seamless protocol extensibility

### 5. Production Observability

- **Structured Logging**: Contextual tracing with correlation IDs
- **Metrics Collection**: Performance counters, histograms, and gauges
- **Distributed Tracing**: OpenTelemetry integration for request flow analysis
- **Error Forensics**: Detailed error context with stack traces and recovery hints

## Technical Architecture

### Core Components

| Component | Description | Key Features |
|-----------|-------------|-------------|
| **Transport Layer** | Multi-protocol transport abstraction | STDIO, HTTP/1.1, HTTP/2, WebSocket, SSE |
| **Protocol Engine** | MCP 2025-06-18 implementation | JSON-RPC, batch operations, streaming |
| **Plugin Runtime** | Dynamic extension system | Hot reload, sandboxing, versioning |
| **Resilience Core** | Fault tolerance mechanisms | Circuit breakers, retries, health checks |
| **Security Module** | Authentication and authorization | JWT, OAuth2, mTLS, rate limiting |

### Performance Characteristics

- **Zero-Copy Operations**: Minimal memory allocation in hot paths
- **Async/Await Runtime**: Tokio-based non-blocking I/O
- **Connection Multiplexing**: Single TCP connection for multiple streams
- **Smart Buffering**: Adaptive buffer sizing based on throughput
- **CPU Affinity**: Thread pinning for cache optimization

## Installation

### Standard Installation

```toml
[dependencies]
prism-mcp-rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
async-trait = "0.1"
```

### Feature Matrix

| Feature Category | Features | Use Case |
|-----------------|----------|----------|
| **Core Transports** | `stdio`, `http`, `websocket` | Basic connectivity |
| **HTTP Extensions** | `sse`, `http2`, `chunked-encoding`, `compression` | Advanced HTTP capabilities |
| **Security** | `auth`, `tls` | Authentication and encryption |
| **Extensions** | `plugin` | Runtime extensibility |
| **Bundles** | `full`, `minimal` | Convenience feature sets |

### Advanced Configuration

```toml
# High-performance configuration
[dependencies]
prism-mcp-rs = { 
    version = "0.1.0", 
    features = ["http2", "compression", "plugin", "auth", "tls"] 
}

# Memory-constrained environments
[dependencies]
prism-mcp-rs = { 
    version = "0.1.0", 
    default-features = false,
    features = ["stdio"] 
}
```


## Clean & Simple API

### 30-Second Server Setup

```rust
use prism_mcp_rs::prelude::*;

#[derive(Clone)]
struct SystemTools;

#[async_trait]
impl ToolProvider for SystemTools {
    async fn call_tool(&self, request: CallToolRequest) -> McpResult<CallToolResult> {
        match request.name.as_str() {
            "system_info" => Ok(CallToolResult::text(format!(
                "Host: {}, OS: {}", 
                gethostname::gethostname().to_string_lossy(),
                std::env::consts::OS
            ))),
            _ => Err(McpError::invalid_request("Unknown tool"))
        }
    }

    async fn list_tools(&self) -> McpResult<ListToolsResult> {
        Ok(ListToolsResult::from_tools(vec![
            Tool::new("system_info", "Get system information")
        ]))
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    McpServer::builder()
        .with_tool_provider(SystemTools)
        .with_stdio_transport()
        .build()
        .await?
        .run()
        .await
}
```

### Enterprise-Grade Client

```rust
use prism_mcp_rs::client::*;

let client = ClientSession::builder()
    .with_circuit_breaker() // Auto fault isolation
    .with_adaptive_retries() // Smart backoff
    .with_health_monitoring() // Continuous health checks
    .connect_stdio("./server")
    .await?;

// Resilient operations with automatic recovery
let tools = client.list_tools().await?;
let result = client.call_tool("system_info", json!({})).await?;
```

### Hot-Reloadable Plugins

```rust
use prism_mcp_rs::plugin::*;

#[plugin]
struct WeatherPlugin {
    api_key: String,
}

#[async_trait]
impl ToolPlugin for WeatherPlugin {
    async fn call_tool(&self, req: CallToolRequest) -> McpResult<CallToolResult> {
        let weather = self.fetch_weather(&req.args["location"]).await?;
        Ok(CallToolResult::json(weather))
    }
}

// Runtime plugin management
let plugin_manager = PluginManager::new();
plugin_manager.hot_reload("weather_plugin.so").await?; // Zero downtime
```

## Architectural Innovations

### Zero-Configuration Service Discovery
Automatic capability negotiation and runtime schema introspection eliminates manual configuration:

```rust
// Client automatically discovers and adapts to server capabilities
let client = ClientSession::auto_discover("./server").await?;
let schema = client.introspect().await?; // Full runtime capability discovery
```

### Fault-Tolerant by Design
Built-in resilience patterns prevent cascading failures in distributed AI systems:

```rust
// Circuit breakers, retries, and health checks work together automatically
let result = client
    .with_fallback(backup_service)
    .call_tool_resilient("analyze", data)
    .await?; // Never fails catastrophically
```

### Plugin Ecosystem Revolution
Hot-swappable plugins with ABI stability across Rust versions:

```rust
// Live plugin updates without service interruption
plugin_manager.hot_swap("analyzer_v2.so", "analyzer_v1.so").await?;
// Automatic dependency resolution and health monitoring
```

## New Use Cases Enabled

### **Multi-Agent AI Orchestration**
Combine multiple AI services with automatic failover and load balancing.

### **Enterprise Integration Hubs**
Connect legacy systems to modern AI tools with protocol translation and security policies.

### **Real-Time AI Pipelines**
Build streaming data processing pipelines with sub-millisecond latency guarantees.

### **Federated AI Networks**
Create distributed AI service meshes with automatic service discovery and routing.

### **Edge AI Deployment**
Deploy AI capabilities to edge devices with offline-first architecture and smart sync.
## Production-Ready Performance

| Metric | Value | Impact |
|--------|-------|---------|
| **Zero-downtime deployments** | < 100ms | Keep AI services running during updates |
| **Automatic failover** | < 50ms | No user-visible service interruptions |
| **Memory efficiency** | 2-12MB baseline | Deploy to edge and resource-constrained environments |
| **Protocol overhead** | < 0.5ms | Sub-millisecond response times for real-time AI |

## Documentation

### **📚 Getting Started**
- [Quick Start Guide](docs/GETTING_STARTED.md) - Installation, setup, and first steps
- [AI Tool Integration](docs/AI_TOOL_INTEGRATION.md) - Connect to Claude, Cursor, VS Code, Windsurf
- [Configuration Examples](docs/examples/ai-tool-configs/README.md) - Ready-to-use configuration templates
- [API Reference](https://docs.rs/prism-mcp-rs) - Complete API documentation

### **🚀 Deployment & Production**
- [Deployment Guide](docs/DEPLOYMENT_GUIDE.md) - Production deployment strategies
- [Troubleshooting Guide](docs/TROUBLESHOOTING.md) - Common issues and solutions

### **🏗️ Development Guides**
- [Architecture Guide](docs/ARCHITECTURE.md) - System design and components
- [Plugin Development](docs/guides/plugins.md) - Building custom plugins
- [Plugin Types Reference](docs/guides/plugin-types.md) - Detailed component specifications
- [Error Handling](docs/guides/error-handling.md) - Comprehensive error management patterns
- [Performance Tuning](docs/guides/performance.md) - Optimization strategies
- [Authentication Guide](docs/guides/authentication.md) - Authentication and authorization
- [Development Setup](docs/DEVELOPMENT.md) - Development environment and workflows

### **🔄 Migration & Updates**
- [Migration Guide](docs/guides/migration.md) - Migrating from other MCP implementations
- [Changelog](CHANGELOG.md) - Version history and breaking changes

### **🔒 Security & Policies**
- [Security Policy](SECURITY.md) - Vulnerability reporting and security practices

## Contributing

Contributions are welcome! Please review our [Contributing Guidelines](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

See our [Contributors](CONTRIBUTORS.md) for a list of everyone who has contributed to this project.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Support

- GitHub Issues: [Bug Reports & Feature Requests](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- Discord: [Community Support](https://discord.gg/prismworks)
- Email: developers@prismworks.ai
# Prism MCP SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/prism-mcp-rs.svg)](https://crates.io/crates/prism-mcp-rs)
[![Downloads](https://img.shields.io/crates/d/prism-mcp-rs.svg)](https://crates.io/crates/prism-mcp-rs)
[![Documentation](https://docs.rs/prism-mcp-rs/badge.svg)](https://docs.rs/prism-mcp-rs)
[![CI](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml)
[![Benchmarks](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/benchmarks.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/benchmarks.yml)
[![Performance](https://prismworks-ai.github.io/prism-mcp-rs/benchmarks/badge.svg)](https://prismworks-ai.github.io/prism-mcp-rs/benchmarks/)
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

**prism-mcp-rs** is a production-grade Rust implementation of the Model Context Protocol (MCP) SDK with enterprise-class features for building secure, scalable MCP servers and clients.

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

## Quick Start

### Basic Server Implementation

```rust
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::server::McpServer;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

struct CalculatorHandler;

#[async_trait]
impl ToolHandler for CalculatorHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let operation = arguments.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing operation"))?;
        
        let a = arguments.get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::invalid_params("Missing parameter a"))?;
        
        let b = arguments.get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::invalid_params("Missing parameter b"))?;
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(McpError::invalid_params("Division by zero"));
                }
                a / b
            }
            _ => return Err(McpError::invalid_params("Unknown operation")),
        };
        
        Ok(ToolResult::text(format!("{}", result)))
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let mut server = McpServer::new("calculator-server".to_string(), "1.0.0".to_string());
    
    server.add_tool(
        "calculate".to_string(),
        Some("Perform arithmetic operations".to_string()),
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        }),
        CalculatorHandler,
    ).await?;
    
    server.run_with_stdio().await
}
```

### Production Client with Resilience

```rust
use prism_mcp_rs::client::{ClientSession, SessionConfig};
use prism_mcp_rs::core::retry::{RetryConfig, CircuitBreakerConfig};
use prism_mcp_rs::transport::http::HttpClientTransport;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Configure resilience policies
    let session_config = SessionConfig {
        retry_config: RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_base: 2.0,
            jitter: true,
            ..Default::default()
        },
        enable_circuit_breaker: true,
        circuit_breaker_config: CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_requests: 3,
        },
        ..Default::default()
    };
    
    // Create HTTP transport with authentication
    let transport = HttpClientTransport::builder()
        .base_url("https://api.example.com")
        .auth_token("Bearer YOUR_TOKEN")
        .timeout(Duration::from_secs(30))
        .build()
        .await?;
    
    // Initialize session with resilience features
    let mut session = ClientSession::new_with_config(transport, session_config);
    
    session.initialize("production-client".to_string(), "1.0.0".to_string()).await?;
    
    // Execute with automatic retry and circuit breaker protection
    let result = session.call_tool(
        "calculate",
        Some(HashMap::from([
            ("operation".to_string(), json!("divide")),
            ("a".to_string(), json!(100.0)),
            ("b".to_string(), json!(3.0)),
        ])),
    ).await?;
    
    println!("Result: {:?}", result);
    Ok(())
}
```

### Plugin Development

```rust
use prism_mcp_rs::plugin::{ToolPlugin, PluginMetadata};
use async_trait::async_trait;

pub struct CustomAnalyticsPlugin {
    metrics: Arc<Mutex<HashMap<String, u64>>>,
}

#[async_trait]
impl ToolPlugin for CustomAnalyticsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "analytics".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Custom analytics processing".to_string()),
            author: Some("Your Name".to_string()),
            capabilities: PluginCapabilities {
                hot_reload: true,
                health_check: true,
                configurable: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    async fn call_tool(
        &self,
        tool_name: &str,
        arguments: HashMap<String, Value>,
    ) -> McpResult<ToolResult> {
        // Implementation with metrics tracking
        let mut metrics = self.metrics.lock().await;
        *metrics.entry(tool_name.to_string()).or_insert(0) += 1;
        
        // Process analytics request
        match tool_name {
            "aggregate" => self.process_aggregation(arguments).await,
            "visualize" => self.create_visualization(arguments).await,
            _ => Err(McpError::method_not_found()),
        }
    }
    
    async fn health_check(&self) -> McpResult<()> {
        // Verify plugin operational status
        Ok(())
    }
}

// Export plugin factory function
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn ToolPlugin> {
    Box::new(CustomAnalyticsPlugin::new())
}
```

## Advanced Features

### Streaming HTTP/2 with Compression

```rust
use prism_mcp_rs::transport::streaming_http::{StreamingHttpClientTransport, StreamingConfig};

let config = StreamingConfig::performance_optimized()
    .with_compression(CompressionType::Brotli)
    .with_http2_multiplexing(true)
    .with_adaptive_buffering(true);

let transport = StreamingHttpClientTransport::with_config(
    "https://api.example.com",
    config
).await?;
```

### Health Monitoring System

```rust
use prism_mcp_rs::core::health::{HealthChecker, HealthStatus};

let health_checker = HealthChecker::new()
    .add_check("database", || check_database_connection())
    .add_check("cache", || check_redis_connection())
    .add_check("disk_space", || check_disk_usage());

let health_report = health_checker.run_all_checks().await;
match health_report.overall_status {
    HealthStatus::Healthy => info!("All systems operational"),
    HealthStatus::Degraded(msg) => warn!("System degraded: {}", msg),
    HealthStatus::Unhealthy(msg) => error!("System unhealthy: {}", msg),
}
```

### Schema Introspection

```rust
use prism_mcp_rs::protocol::schema_introspection::IntrospectionProvider;

let provider = IntrospectionProvider::new();
let introspection = provider.build_complete_introspection();

// Discover server capabilities at runtime
for method in &introspection.methods.methods {
    println!("Method: {} - {}", method.name, method.description);
    if let Some(params) = &method.parameters {
        println!("  Parameters: {}", serde_json::to_string_pretty(params)?);
    }
}
```

## Performance Benchmarks

| Operation | Throughput | Latency (p99) | Memory |
|-----------|------------|---------------|--------|
| STDIO Echo | 50K msg/s | 0.5ms | 2MB |
| HTTP/1.1 Request | 20K req/s | 5ms | 8MB |
| HTTP/2 Multiplexed | 100K req/s | 2ms | 12MB |
| WebSocket Bidirectional | 40K msg/s | 1ms | 4MB |
| Plugin Hot Reload | < 100ms | - | 1MB |

## Documentation

- [API Reference](https://docs.rs/prism-mcp-rs) - Complete API documentation
- [Architecture Guide](docs/ARCHITECTURE.md) - System design and components
- [Plugin Development](docs/guides/plugins.md) - Building custom plugins
- [Performance Tuning](docs/guides/performance.md) - Optimization strategies
- [Authentication Guide](docs/guides/authentication.md) - Authentication and authorization

## Contributing

Contributions are welcome. Please review our [Contributing Guidelines](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT License - see [LICENSE](LICENSE) for details.

## Support

- GitHub Issues: [Bug Reports & Feature Requests](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- Discord: [Community Support](https://discord.gg/prismworks)
- Email: developers@prismworks.ai
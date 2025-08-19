# Documentation

## Overview

This directory contains comprehensive technical documentation for the Prism MCP SDK. The documentation is organized hierarchically to facilitate both learning and reference.

## Documentation Structure

### Core Documentation

| Document | Purpose | Audience |
|----------|---------|----------|
| [Getting Started](GETTING_STARTED.md) | Initial setup and basic usage | New users |
| [Architecture](ARCHITECTURE.md) | System design and components | Architects, senior developers |
| [Development](DEVELOPMENT.md) | Development environment and workflows | Contributors |

### Technical Guides

| Guide | Focus Area | Use Case |
|-------|------------|----------|
| [Authentication](guides/authentication.md) | Security implementation | Secure deployments |
| [Performance](guides/performance.md) | Optimization techniques | High-throughput systems |
| [Plugins](guides/plugins.md) | Extension development | Custom functionality |
| [Plugin Types](guides/plugin-types.md) | Component specifications | Plugin architecture |
| [Error Handling](guides/error-handling.md) | Fault tolerance | Production systems |
| [Migration](guides/migration.md) | Version upgrades | System maintenance |

### API Documentation

#### Generating Local Documentation

```bash
# Generate and open API documentation
cargo doc --no-deps --open

# Generate with private items
cargo doc --no-deps --document-private-items --open

# Generate for specific features
cargo doc --no-deps --features "http2 compression plugin" --open
```

#### Online Documentation

- [Published API Docs](https://docs.rs/prism-mcp-rs) - Available after crates.io publication
- [GitHub Repository](https://github.com/prismworks-ai/prism-mcp-rs) - Source code and examples

## Quick Reference

### Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|-------------|
| `stdio` | Standard I/O transport | None (default) |
| `http` | HTTP/1.1 transport | reqwest, hyper, axum |
| `websocket` | WebSocket transport | tokio-tungstenite |
| `http2` | HTTP/2 support | h2 |
| `sse` | Server-Sent Events | tokio-stream |
| `compression` | Response compression | brotli, gzip, zstd |
| `plugin` | Plugin system | libloading, abi_stable |
| `auth` | Authentication | jsonwebtoken, argon2 |
| `tls` | TLS support | rustls, tokio-rustls |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|------|
| `RUST_LOG` | Logging level | `info` |
| `MCP_SERVER_PORT` | Server port | `8080` |
| `MCP_SERVER_HOST` | Server host | `127.0.0.1` |
| `MCP_MAX_CONNECTIONS` | Connection limit | `1000` |
| `MCP_REQUEST_TIMEOUT` | Request timeout (seconds) | `30` |
| `MCP_PLUGIN_DIR` | Plugin directory | `./plugins` |

### Performance Benchmarks

| Metric | Target | Achieved |
|--------|--------|----------|
| Latency (p99) | <10ms | 2-5ms |
| Throughput (HTTP/2) | >50K req/s | 100K req/s |
| Memory per connection | <1MB | 0.5MB |
| Startup time | <1s | 0.3s |
| Plugin hot reload | <500ms | 100ms |

## Learning Path

### Beginner

1. Read [Getting Started](GETTING_STARTED.md)
2. Run example applications
3. Build simple echo server
4. Implement basic client

### Intermediate

1. Study [Architecture](ARCHITECTURE.md)
2. Implement custom handlers
3. Add authentication
4. Configure transports

### Advanced

1. Review [Performance Guide](guides/performance.md)
2. Develop plugins
3. Implement custom transports
4. Deploy at scale

### Expert

1. Contribute to core SDK
2. Design distributed systems
3. Optimize for specific workloads
4. Security hardening

## Common Patterns

### Server Initialization

```rust
use prism_mcp_rs::server::McpServer;

let server = McpServer::builder()
    .name("production-server")
    .version("2.0.0")
    .with_auth(auth_config)
    .with_rate_limiting(rate_limit_config)
    .with_health_check(health_config)
    .build()?;
```

### Client with Resilience

```rust
use prism_mcp_rs::client::{ClientSession, SessionConfig};

let config = SessionConfig::production()
    .with_retry_policy(RetryConfig::exponential())
    .with_circuit_breaker(CircuitBreakerConfig::default())
    .with_timeout(Duration::from_secs(30));

let session = ClientSession::new_with_config(transport, config);
```

### Plugin Registration

```rust
use prism_mcp_rs::plugin::PluginManager;

let mut plugin_manager = PluginManager::new();
plugin_manager.load_plugin("./plugins/analytics.so").await?;
plugin_manager.register_all(&mut server).await?;
```

## Troubleshooting

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Connection refused | Server not running | Check server logs and port binding |
| Authentication failed | Invalid token | Verify token format and expiration |
| High latency | Network congestion | Enable compression, use HTTP/2 |
| Memory leak | Unclosed connections | Implement proper cleanup |
| Plugin load failure | ABI mismatch | Rebuild with same Rust version |

### Debug Commands

```bash
# Enable verbose logging
RUST_LOG=debug cargo run

# Profile memory usage
valgrind --leak-check=full ./target/release/mcp-server

# Trace system calls
strace -f ./target/release/mcp-server

# Monitor network traffic
tcpdump -i any -w mcp.pcap port 8080
```

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines.

## Support

- **GitHub Issues**: [Bug Reports](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- **Discord**: [Community Chat](https://discord.gg/prismworks)
- **Email**: developers@prismworks.ai

## License

MIT License - see [LICENSE](../LICENSE) for details.
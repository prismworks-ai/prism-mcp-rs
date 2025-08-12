#!/usr/bin/env python3
"""
Restructure documentation to eliminate duplicates and improve organization.
"""

import os
import shutil
from pathlib import Path

def backup_docs():
    """Create backup of current docs."""
    docs_dir = Path('docs')
    backup_dir = Path('docs-backup')
    
    if backup_dir.exists():
        shutil.rmtree(backup_dir)
    
    shutil.copytree(docs_dir, backup_dir)
    print(f"[x] Created backup in {backup_dir}")

def restructure_docs():
    """Restructure documentation files."""
    
    # Files to keep as-is
    keep_files = [
        'docs/SECURITY.md',
        'docs/integrations/claude-desktop.md',
        'docs/archive/*'
    ]
    
    # Files to update
    updates = {
        'README-new.md': 'README.md',
        'docs/error-handling-new.md': 'docs/error-handling.md',
    }
    
    # Apply updates
    for src, dst in updates.items():
        if Path(src).exists():
            shutil.copy(src, dst)
            print(f"[x] Updated {dst}")
            if src.endswith('-new.md'):
                os.remove(src)
    
    # Remove duplicate content files
    remove_files = [
        'docs/SDK_FEATURES.md',  # Merged into README
        'docs/PLUGIN_SYSTEM.md',  # Should be auto-generated
        'docs/HTTP_TRANSPORT_FEATURES.md',  # Merged into transports.md
    ]
    
    for file in remove_files:
        if Path(file).exists():
            # Move to archive instead of deleting
            archive_path = Path('docs/archive') / Path(file).name
            shutil.move(file, archive_path)
            print(f"Package: Archived {file}")

def update_transport_docs():
    """Consolidate transport documentation."""
    
    content = """# Transport Guide

> **Complete guide to MCP transport layers: STDIO, HTTP, and WebSocket**

This guide covers all transport options in the MCP Protocol SDK. For API details, see the [transport module documentation](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/transport/index.html).

## Quick Selection Guide

| Transport | Best For | Pros | Cons |
|-----------|----------|------|------|
| **STDIO** | CLI tools, Claude Desktop | Simple, universal | No network support |
| **HTTP** | Web services, REST APIs | Stateless, proxy-friendly | Overhead per request |
| **WebSocket** | Real-time, bidirectional | Persistent, low latency | Complex setup |

## STDIO Transport

The simplest transport using standard input/output streams.

### When to Use
- 🔗 Claude Desktop integration
- 💻 Command-line tools
- - Local development

### Implementation

```rust
use mcp_protocol_sdk::transport::StdioServerTransport;

let transport = StdioServerTransport::new();
server.run_with_transport(transport).await?;
```

### Configuration

```rust
let config = TransportConfig {
    timeout_ms: Some(30000),
    buffer_size: Some(8192),
    ..Default::default()
};
let transport = StdioServerTransport::new_with_config(config);
```

## HTTP Transport

RESTful HTTP transport with SSE support for notifications.

### When to Use
- 🌐 Web services
- Security: Behind proxies/firewalls
- Package: Stateless operations

### Features
- **SSE Notifications** - Server-sent events for real-time updates
- **CORS Support** - Cross-origin resource sharing
- **Health Checks** - Built-in `/health` endpoint
- **Request/Response** - Standard REST patterns

### Server Implementation

```rust
use mcp_protocol_sdk::transport::HttpServerTransport;

let transport = HttpServerTransport::new("127.0.0.1:3000")
    .with_cors(true)
    .with_health_check(true);

server.run_with_transport(transport).await?;
```

### Client Implementation

```rust
use mcp_protocol_sdk::transport::HttpClientTransport;

let transport = HttpClientTransport::new("http://localhost:3000")
    .with_sse_support(true);  // Enable notifications

client.connect_with_transport(transport).await?;
```

### complete Features

#### Streaming (HTTP/2)
```rust
let transport = HttpServerTransport::new("127.0.0.1:3000")
    .with_http2(true)
    .with_compression(CompressionType::Gzip);
```

#### Authentication
```rust
let transport = HttpClientTransport::new("https://api.example.com")
    .with_bearer_token("your-token")
    .with_custom_headers(headers);
```

## WebSocket Transport

Full-duplex WebSocket transport for real-time communication.

### When to Use
- ⏱️ Real-time updates
- 🔄 Bidirectional communication
- 📊 Low latency requirements

### Implementation

```rust
use mcp_protocol_sdk::transport::WebSocketServerTransport;

let transport = WebSocketServerTransport::new("127.0.0.1:8080");
server.run_with_transport(transport).await?;
```

### Features
- **Auto-reconnection** - Automatic reconnect on disconnect
- **Ping/Pong** - Keep-alive mechanism
- **Binary & Text** - Support both message types
- **Compression** - Per-message deflate

### Configuration

```rust
let config = WebSocketConfig {
    max_frame_size: Some(65536),
    max_message_size: Some(1048576),
    compression: true,
    ping_interval_ms: Some(30000),
};
```

## Transport Selection Matrix

| Feature | STDIO | HTTP | WebSocket |
|---------|-------|------|----------|
| Setup Complexity | ⭐ Simple | ⭐⭐ Moderate | ⭐⭐⭐ Complex |
| Network Support | [!] No | [x] Yes | [x] Yes |
| Bidirectional | [x] Yes | Warning: SSE only | [x] Yes |
| Stateless | [!] No | [x] Yes | [!] No |
| Proxy Support | [!] No | [x] Yes | Warning: Limited |
| Performance | ⭐⭐⭐ High | ⭐⭐ Medium | ⭐⭐⭐ High |

## Error Handling {#transport-errors}

All transports integrate with the SDK's error handling:

```rust
match transport.connect().await {
    Ok(_) => info!("Connected"),
    Err(McpError::Connection { .. }) => {
        // Automatic retry with exponential backoff
    },
    Err(e) => return Err(e),
}
```

For error handling patterns, see the [Error Handling Guide](./error-handling.md#retry-logic).

## Examples

- [STDIO Server](../examples/server/improved_echo_server.rs)
- [HTTP Server](../examples/server/http2_server.rs)
- [WebSocket Client](../examples/README.md#websocket)

## API Reference

- [`Transport` trait](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/transport/trait.Transport.html)
- [`StdioTransport`](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/transport/stdio/index.html)
- [`HttpTransport`](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/transport/http/index.html)
- [`WebSocketTransport`](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/transport/websocket/index.html)
"""
    
    with open('docs/transports.md', 'w') as f:
        f.write(content)
    
    print("[x] Updated docs/transports.md")

def update_health_monitoring():
    """Update health monitoring docs to remove duplication."""
    
    content = """# Health Monitoring

> **Observability and health checks for production MCP deployments**

This guide covers health monitoring capabilities in the MCP Protocol SDK. For implementation details, see the [health module documentation](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/core/health/index.html).

## Overview

The SDK provides multi-layer health monitoring:

1. **Transport Health** - Connection status and latency
2. **Protocol Health** - Message processing and compliance
3. **Resource Health** - Memory, CPU, and resource availability
4. **Application Health** - Custom health indicators

## Health Check Endpoints

### HTTP Transport

The HTTP transport includes a built-in `/health` endpoint:

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "checks": {
    "transport": "ok",
    "protocol": "ok",
    "resources": "ok"
  },
  "metrics": {
    "uptime_seconds": 3600,
    "requests_processed": 1000,
    "active_connections": 5
  }
}
```

### Programmatic Health Checks

```rust
use mcp_protocol_sdk::core::health::{HealthChecker, HealthStatus};

let checker = HealthChecker::new();
let status = checker.check_all().await;

match status {
    HealthStatus::Healthy => info!("All systems operational"),
    HealthStatus::Degraded(issues) => warn!("Degraded: {:?}", issues),
    HealthStatus::Unhealthy(errors) => error!("Unhealthy: {:?}", errors),
}
```

## Metrics Collection

The SDK collects metrics automatically:

```rust
use mcp_protocol_sdk::core::metrics::MetricsCollector;

let metrics = MetricsCollector::global();
let snapshot = metrics.snapshot();

println!("Request rate: {}/s", snapshot.request_rate);
println!("Error rate: {}%", snapshot.error_percentage);
println!("P99 latency: {}ms", snapshot.p99_latency_ms);
```

### Available Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `requests_total` | Counter | Total requests processed |
| `errors_total` | Counter | Total errors encountered |
| `request_duration_ms` | Histogram | Request processing time |
| `active_connections` | Gauge | Current active connections |
| `retry_attempts` | Counter | Number of retry attempts |
| `circuit_breaker_trips` | Counter | Circuit breaker activations |

## Integration with Error Handling {#error-integration}

Health status is affected by error rates:

```rust
// Errors automatically affect health status
if error_rate > 0.05 {  // 5% error rate
    health_status = HealthStatus::Degraded;
}

if error_rate > 0.20 {  // 20% error rate
    health_status = HealthStatus::Unhealthy;
}
```

For error handling details, see the [Error Handling Guide](./error-handling.md#circuit-breaker).

## Monitoring Best Practices {#monitoring-best-practices}

1. **Set up alerts** for health status changes
2. **Monitor trends** not just current values
3. **Correlate metrics** across different layers
4. **Test health checks** regularly
5. **Document thresholds** for each metric

## Examples

- [Health check implementation](../examples/README.md#health-monitoring)
- [Metrics dashboard](../examples/production_error_handling_demo.rs)
- [Integration tests](../tests/production_safety_tests.rs)

## API Reference

- [`HealthChecker`](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/core/health/struct.HealthChecker.html)
- [`MetricsCollector`](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/core/metrics/struct.MetricsCollector.html)
- [`HealthStatus`](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/core/health/enum.HealthStatus.html)
"""
    
    with open('docs/health-monitoring.md', 'w') as f:
        f.write(content)
    
    print("[x] Updated docs/health-monitoring.md")

def update_production_readiness():
    """Update production readiness guide."""
    
    content = """# Production Deployment Guide

> **Enterprise deployment guide for MCP Protocol SDK applications**

This guide covers production deployment best practices. For specific features:
- [Error Handling](./error-handling.md#best-practices)
- [Health Monitoring](./health-monitoring.md#monitoring-best-practices)
- [Transport Selection](./transports.md#transport-selection-matrix)

## Deployment Checklist

### [x] Pre-Production

- [ ] Run test suite: `cargo test --all-features`
- [ ] Check security: `cargo audit`
- [ ] Verify coverage: `cargo llvm-cov --html`
- [ ] Review dependencies: `cargo tree`
- [ ] Test all transports
- [ ] Load test with expected traffic

### [x] Configuration

```rust
use mcp_protocol_sdk::prelude::*;

// Production configuration
let config = McpConfig {
    // Timeouts
    request_timeout_ms: 30000,
    connection_timeout_ms: 10000,
    
    // Retry policy
    retry_config: RetryConfig {
        max_attempts: 3,
        initial_delay_ms: 100,
        max_delay_ms: 5000,
        exponential_base: 2.0,
        respect_recoverability: true,
    },
    
    // Circuit breaker
    circuit_breaker: CircuitBreakerConfig {
        failure_threshold: 5,
        recovery_timeout_ms: 30000,
        half_open_max_calls: 3,
    },
    
    // Logging
    log_level: "info",
    structured_logging: true,
};
```

## Error Handling {#error-handling}

Production error handling configuration:

```rust
// Set up global error handler
std::panic::set_hook(Box::new(|info| {
    error!("Panic occurred: {}", info);
    // Send to monitoring system
}));

// Use error context
operation()
    .await
    .context("Failed in production operation")?;
```

See [Error Handling Guide](./error-handling.md) for patterns.

## Metrics {#metrics}

Export metrics to monitoring systems:

```rust
// Prometheus exporter
let metrics = MetricsCollector::global();
let prometheus_data = metrics.export_prometheus();

// Custom exporter
metrics.register_exporter(|snapshot| {
    send_to_datadog(snapshot);
});
```

## Logging {#logging}

Structured logging configuration:

```rust
use tracing_subscriber::{
    EnvFilter,
    fmt::format::FmtSpan,
};

tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_span_events(FmtSpan::CLOSE)
    .json()  // JSON format for log aggregation
    .init();
```

## Security Considerations

1. **Use TLS** for network transports
2. **Validate input** at all boundaries
3. **Rotate credentials** regularly
4. **Audit dependencies** with `cargo audit`
5. **Monitor for CVEs** in dependencies

See [Security Policy](./SECURITY.md) for vulnerability reporting.

## Performance Tuning

### Connection Pooling

```rust
let transport = HttpClientTransport::new(url)
    .with_connection_pool(100)  // Max connections
    .with_keep_alive(true);
```

### Buffer Sizes

```rust
let config = TransportConfig {
    buffer_size: Some(65536),  // 64KB for high throughput
    ..Default::default()
};
```

## Deployment Architectures

### Container Deployment

```dockerfile
FROM rust:1.85 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --all-features

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/mcp-server /usr/local/bin/
EXPOSE 3000
CMD ["mcp-server"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-server
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: mcp-server
        image: mcp-server:latest
        ports:
        - containerPort: 3000
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
```

## Monitoring Setup

- **Prometheus** + **Grafana** for metrics
- **ELK Stack** for log aggregation
- **Jaeger** for distributed tracing
- **PagerDuty** for alerting

## Load Testing

```bash
# Using vegeta
echo "POST http://localhost:3000/mcp" | \
  vegeta attack -duration=60s -rate=100 | \
  vegeta report
```

## Troubleshooting

Common issues and solutions:

| Issue | Cause | Solution |
|-------|-------|----------|
| High memory usage | Large message buffers | Reduce buffer sizes |
| Connection drops | Timeout too short | Increase timeout values |
| Slow responses | No connection pooling | Enable connection pools |
| Circuit breaker trips | Too sensitive | Adjust thresholds |

## Support

- [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- [Documentation](https://docs.rs/mcp-protocol-sdk)
- [Security Reports](./SECURITY.md)
"""
    
    with open('docs/production-readiness.md', 'w') as f:
        f.write(content)
    
    print("[x] Updated docs/production-readiness.md")

def main():
    """Main function to restructure docs."""
    print("## Restructuring documentation...\n")
    
    # Backup first
    backup_docs()
    
    # Update specific docs
    update_transport_docs()
    update_health_monitoring()
    update_production_readiness()
    
    # Restructure files
    restructure_docs()
    
    print("\n[x] Documentation restructuring complete!")
    print("Run 'python3 scripts/check-docs-quality.py' to verify")

if __name__ == '__main__':
    main()

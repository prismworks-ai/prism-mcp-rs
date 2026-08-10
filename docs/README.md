# Prism MCP SDK Documentation

This is the canonical index for maintained project documentation. Public Rust APIs are documented on [docs.rs](https://docs.rs/prism-mcp-rs); examples that compile with the repository are indexed in [examples](../examples/README.md).

## Start here

| Document | Purpose |
|----------|---------|
| [Getting Started](GETTING_STARTED.md) | Install the crate and run a STDIO server |
| [Protocol Versions](PROTOCOL_VERSIONS.md) | Understand 2026 behavior, 2025 interoperability, and downgrade rules |
| [AI Tool Integration](AI_TOOL_INTEGRATION.md) | Configure Claude, Cursor, VS Code, and Windsurf |
| [Architecture](ARCHITECTURE.md) | Understand modules, dispatch, and trust boundaries |
| [Deployment Guide](DEPLOYMENT_GUIDE.md) | Package and operate an SDK-based service |
| [Production Controls](PRODUCTION_CONTROLS.md) | Configure RBAC, rate limits, mTLS, tracing, and failover |
| [Troubleshooting](TROUBLESHOOTING.md) | Diagnose build, transport, policy, and plugin failures |

## Focused guides

| Guide | Purpose |
|-------|---------|
| [Authentication](guides/authentication.md) | OAuth client primitives and application responsibilities |
| [Error Handling](guides/error-handling.md) | Error categories and handler behavior |
| [Migration](guides/migration.md) | Move from 2.x to the 3.x dual-protocol API |
| [Performance](guides/performance.md) | Benchmark and tune without unsupported guarantees |
| [Plugins](guides/plugins.md) | Load trusted native plugins and understand the security boundary |

Contributor setup and release workflow are maintained in the root [Contributing Guide](../CONTRIBUTING.md). Script-specific instructions live in [scripts/README.md](../scripts/README.md), and report provenance lives in [reports/README.md](../reports/README.md).

## Feature flags

| Feature | Enables | Notes |
|---------|---------|-------|
| `stdio` | STDIO transports | Default |
| `http` | HTTP client/server | Required by `auth`, `sse`, and HTTP enhancements |
| `websocket` | WebSocket transport | Optional |
| `sse` | Server-Sent Events | Implies `http` |
| `http2` | HTTP/2 support | Implies `http` |
| `chunked-encoding` | Proprietary Prism chunked endpoint helpers | Implies `http`; explicit legacy-only use |
| `compression` | Brotli, gzip, and zstd support | Implies `http` |
| `plugin` | Trusted native plugin loading | Not sandboxed |
| `auth` | OAuth/JWT/Argon2 primitives | Implies `http` through current module wiring |
| `tls` | TLS 1.3 mTLS types | Use with `http` |
| `otel` | OTLP/OpenTelemetry tracing | Installs or integrates with a tracing subscriber |
| `full` | All optional features | Useful for CI, not automatically best for production |
| `bench` | Criterion benchmark dependency | Development only |

The crate does not define application environment variables. Configuration names, defaults, secret sources, and precedence belong to the binary that embeds the SDK.

## Documentation verification

```bash
cargo test --doc --all-features
cargo doc --no-deps --all-features
python3 scripts/docs/check-docs-quality.py
```

When behavior and prose disagree, tested public APIs and source code are authoritative. Please open an issue or correction rather than copying stale examples.

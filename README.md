# Prism MCP Rust SDK

[![Crates.io](https://img.shields.io/crates/v/prism-mcp-rs.svg?style=flat-square)](https://crates.io/crates/prism-mcp-rs)
[![Documentation](https://docs.rs/prism-mcp-rs/badge.svg)](https://docs.rs/prism-mcp-rs)
[![CI](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml)
[![Security audit](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security-audit.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security-audit.yml)
[![License](https://img.shields.io/crates/l/prism-mcp-rs.svg?style=flat-square)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg?style=flat-square)](https://blog.rust-lang.org/2025/01/09/Rust-1.85.0.html)

`prism-mcp-rs` is an async Rust SDK for building Model Context Protocol (MCP) clients and servers. It implements MCP 2025-11-25 types and supports tools, resources, prompts, sampling, roots, completion, and multiple transports.

## Capability status

| Area | Status | Feature |
|------|--------|---------|
| STDIO client/server | Implemented; default | `stdio` |
| HTTP client/server | Implemented | `http` |
| WebSocket | Implemented | `websocket` |
| SSE notifications | Implemented | `sse` |
| HTTP/2 helpers | Implemented | `http2` |
| Compression | Implemented | `compression` |
| OAuth client primitives | Implemented; application integration required | `auth` with `http` |
| Fine-grained RBAC and rate limiting | Implemented; opt in | core |
| TLS 1.3 mutual authentication | Implemented for HTTP | `http,tls` |
| OTLP/OpenTelemetry tracing | Implemented; opt in | `otel` |
| Endpoint balancing and failover | Implemented; reactive and process-local | core plus chosen transports |
| Native plugins | Implemented for trusted code | `plugin` |
| Sandboxed plugins | Not implemented | — |
| CPU affinity | Host/deployment responsibility | — |
| Service discovery and active health probing | Not implemented | — |

The default request policy is backward-compatible and permits requests. Production services should authenticate at the transport boundary and install a deny-by-default policy. Native plugins execute in process and are not a security boundary.

## Install

```toml
[dependencies]
prism-mcp-rs = "2"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
serde_json = "1"
```

Enable only what the application uses:

```toml
prism-mcp-rs = {
    version = "2",
    features = ["http", "tls", "auth", "otel"]
}
```

`full` enables every optional feature. The minimum supported Rust version is 1.85.

## Minimal STDIO server

```rust,no_run
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

struct Echo;

#[async_trait]
impl ToolHandler for Echo {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default();

        Ok(ToolResult {
            content: vec![ContentBlock::text(message)],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServer::create("echo-server", "1.0.0");
    server
        .add_tool(
            "echo",
            Some("Echo a message"),
            json!({
                "type": "object",
                "properties": { "message": { "type": "string" } },
                "required": ["message"]
            }),
            Echo,
        )
        .await?;

    server
        .run_with_transport(StdioServerTransport::new())
        .await
}
```

STDIO servers must reserve stdout for MCP frames; write diagnostics through `tracing` or stderr.

## Production controls

The SDK provides shared request context, deny-by-default RBAC, per-principal/per-method token-bucket limiting, TLS 1.3 mTLS, W3C trace propagation with OTLP export, and conservative endpoint failover. These controls are opt-in because identity verification, certificates, quotas, and availability policy belong to the host application.

See [Production Controls](docs/PRODUCTION_CONTROLS.md) for integration examples and explicit trust boundaries.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo test --doc --all-features
cargo bench --features bench,plugin,http --bench all_benchmarks
```

Benchmark results depend on hardware, enabled features, handlers, payloads, and network conditions. The checked-in [benchmark report](reports/benchmark-report.md) is a development snapshot, not an SLA.

## Documentation

- [Documentation index](docs/README.md)
- [Getting started](docs/GETTING_STARTED.md)
- [AI tool integration](docs/AI_TOOL_INTEGRATION.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Deployment](docs/DEPLOYMENT_GUIDE.md)
- [Production controls](docs/PRODUCTION_CONTROLS.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Examples](examples/README.md)
- [API reference](https://docs.rs/prism-mcp-rs)

## Contributing and security

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a change. Report vulnerabilities privately according to [SECURITY.md](SECURITY.md); do not use a public issue for a suspected vulnerability.

## License

MIT. See [LICENSE](LICENSE).

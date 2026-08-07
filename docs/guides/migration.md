# Migrating to 2.x

Version 2.x targets MCP 2025-11-25 and requires Rust 1.85 or newer. Treat a migration as an API, wire-protocol, feature, and deployment review rather than only changing the dependency number.

## Dependency

```toml
[dependencies]
prism-mcp-rs = "2"
```

The default feature remains `stdio`. Optional transports and production integrations must be selected explicitly. `full` is convenient for CI but may add unnecessary dependencies to an application.

## Server pattern

Register handlers asynchronously and start the server with a concrete server transport:

```rust,no_run
use prism_mcp_rs::prelude::*;

# async fn run() -> McpResult<()> {
let server = McpServer::create("example", "2.0.0");
// server.add_tool(...).await?;
server
    .run_with_transport(StdioServerTransport::new())
    .await?;
# Ok(())
# }
```

Do not copy examples using the wrong transport `start` method, synchronous registration, the old generic `StdioTransport` name, or `Tool::new` argument lists from a different tool type.

## Type and error changes

- Tool handlers implement `call(&self, HashMap<String, Value>) -> McpResult<ToolResult>`.
- `ToolResult`/`CallToolResult` contains `content`, `is_error`, `structured_content`, and `meta`.
- `McpError` variants are tuple variants except structured variants such as `RateLimited { retry_after_ms }`.
- New production-policy errors include `Forbidden` and `RateLimited`.

Let the compiler guide call-site changes and avoid string-matching error display text.

## Production policy

2.x adds shared `RequestContext`, RBAC, and rate limiting. The default remains allow-all for compatibility. Network applications should:

1. authenticate at the transport/gateway boundary;
2. map verified identity to `Principal`;
3. install `RequestPolicy` on the server; and
4. route authenticated calls through `handle_request_with_context`.

A deny-by-default RBAC policy will reject the anonymous compatibility path unless explicitly allowed.

## Transport security and telemetry

- HTTP TLS/mTLS requires `http,tls`; it is not enabled by default.
- OTLP tracing requires `otel` and either the provided initializer or a host-owned tracing subscriber.
- Endpoint balancing/failover is reactive and process-local. It does not add discovery or active health checks.

Review [Production Controls](../PRODUCTION_CONTROLS.md) before enabling these features.

## Plugin migration

Native plugins remain optional and trusted. They are not ABI-stable across arbitrary compiler/crate changes and are not sandboxed. Rebuild and test every plugin against the exact SDK/toolchain used by the host. If the old deployment assumed resource isolation, move the extension to a separate process/container.

## Recommended migration sequence

1. Pin 2.x in a branch and run `cargo check --all-features`.
2. Update handler signatures and transport startup.
3. Rebuild all examples and plugins.
4. Test initialize/capability negotiation against each supported client.
5. Add explicit authentication, policy, limits, and TLS for network deployments.
6. Run the complete verification suite and representative load tests.
7. Roll out with a reversible canary and retain the previous artifact/configuration.

```bash
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo test --doc --all-features
cargo build --examples --all-features
```

For a migration from another language SDK, map its request handlers to Rust `ToolHandler`, `ResourceHandler`, and `PromptHandler` traits, then test wire behavior rather than assuming type names or convenience APIs are identical.

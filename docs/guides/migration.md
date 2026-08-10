# Migrating from 2.x to 3.x

Version 3 makes MCP 2026-07-28 the preferred protocol and preserves MCP 2025-11-25 interoperability. It requires Rust 1.85 or newer. Treat the upgrade as an API and wire-behavior change, even when a deployment initially remains pinned to 2025.

## Dependency

```toml
[dependencies]
prism-mcp-rs = "3"
```

The default feature remains `stdio`. Optional transports and production integrations must still be selected explicitly.

## Best migration path

Start with `ProtocolMode::LegacyOnly` to isolate Rust API changes from protocol changes. After tests pass against existing peers, switch to the default `Auto` mode in a canary. Auto mode attempts 2026 discovery and falls back only when the peer explicitly rejects `server/discover` as an unknown method.

```rust,no_run
use prism_mcp_rs::prelude::*;

let mut client = McpClient::new("migration-client".into(), "3.0.0".into());
client.set_protocol_mode(ProtocolMode::LegacyOnly);

let server = ServerBuilder::new()
    .name("migration-server")
    .version("3.0.0")
    .protocol_mode(ProtocolMode::LegacyOnly)
    .build();
```

Do not implement broad fallback on timeouts, authentication failures, unsupported-version errors, or malformed responses. That can conceal deployment failures and silently select weaker behavior.

## API changes

- `LATEST_PROTOCOL_VERSION`, `PROTOCOL_VERSION`, and `MCP_PROTOCOL_VERSION` now identify MCP 2026-07-28. Use `LEGACY_PROTOCOL_VERSION` when constructing an explicit 2025 initialization fixture.
- `McpClient::connect` returns `ConnectResult`. Its `protocol` records the selected era and version, while `server_info` is optional because 2026 server identity is optional.
- `ClientConfig` adds `protocol_mode` and `max_mrtr_rounds`. The defaults are `Auto` and ten rounds.
- `ServerBuilder::protocol_mode` and `McpServer::set_protocol_mode` pin server behavior when needed.
- `Implementation` includes optional `website_url` and `icons`; `Icon` supports string sizes and a typed theme.
- Client and server capabilities include extension maps.
- Modern successful responses require `resultType`. Modern clients reject missing or unknown discriminants.
- Stateful-only methods such as `initialize`, `ping`, logging-level mutation, and resource subscribe/unsubscribe are rejected in 2026 mode.
- `subscriptions/listen` replaces the modern resource subscribe RPC and is available through `McpClient::listen` on HTTP and STDIO.
- The official Tasks extension is opt-in. Call `enable_tasks_extension` (or builder `with_tasks_extension`) on clients and register server work with `add_task_tool`; use `add_task_tool_with_fallback` for tools that can also execute synchronously, or `add_composed_task_tool` for the standards-defined MRTR-to-Task transition.

The established `ToolHandler`, `ResourceHandler`, and `PromptHandler` traits remain available. Normal complete tool results do not require changes.

## Transport behavior

Modern HTTP requests carry standard routing headers. Tool schemas containing `x-mcp-header` cause the HTTP client to mirror primitive argument values into `Mcp-Param-*`; the server validates those headers against the body before dispatch. Register tools before starting an HTTP server so the transport receives the complete schema set.

Recommended transport selection now keeps all modern traffic on standards-track Streamable HTTP. The older Prism chunked/compressed endpoint helpers are proprietary, require explicit `LegacyOnly` mode, and must be selected only for a matching Prism deployment.

For STDIO auto-negotiation, v3 launches a disposable 2026 probe process. If the probe returns `Method not found`, the SDK starts a clean sibling process for 2025 initialization. Account for this short-lived process in process supervision and startup metrics.

## Recommended rollout

1. Pin 3.x with `LegacyOnly` and run `cargo check --all-features`.
2. Update code that consumes `ConnectResult` or protocol constants.
3. Run client/server contract tests against every supported 2025 peer.
4. Enable `Auto` in a canary and record the negotiated protocol.
5. Exercise discovery, list/read/call operations, HTTP header integrity, subscriptions, Tasks, and multi-round input where applicable.
6. Confirm authentication, policy, rate limits, and telemetry behave identically in both eras.
7. Roll out gradually with a reversible configuration change back to `LegacyOnly`.

```bash
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo test --doc --all-features
cargo build --examples --all-features
```

See [Protocol Versions](../PROTOCOL_VERSIONS.md) for the complete behavior matrix and extension negotiation rules.

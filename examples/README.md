# Examples

The canonical learning sequence is the numbered `examples/features/` set. Root-level examples include compatibility, corrected, and diagnostic variants retained for API regression coverage; similar filenames do not represent separate recommended approaches.

## Start here

| Example | Feature |
|---------|---------|
| `closure_handlers.rs` | Register tools with closures and handlers |
| `bidirectional_basic.rs` | Basic bidirectional client/server behavior |
| `custom_transport.rs` | Implement a custom transport |
| `features/01_mcp_tool_macro.rs` | Direct `ToolHandler` registration (despite the legacy filename) |
| `features/02_resources_api.rs` | Resources |
| `features/03_prompts_api.rs` | Prompts |
| `features/04_sampling_api.rs` | Sampling |
| `features/05_http_transport.rs` | HTTP types |
| `features/06_websocket_transport.rs` | WebSocket types |
| `features/07_authentication.rs` | Authentication concepts; illustrative only |
| `features/08_error_handling.rs` | Error handling |
| `features/09_configuration.rs` | Configuration |
| `features/10_plugin_system.rs` | In-process extension pattern; not the native plugin loader |
| `features/11_advanced_tools.rs` | Advanced tool patterns |
| `features/12_integration_patterns.rs` | Integration patterns |

Build every configured example:

```bash
cargo build --examples --all-features
```

Run a specific example with its required features:

```bash
cargo run --example closure_handlers
cargo run --example 05_http_transport --features http
```

Some examples demonstrate construction/registration and exit without starting a transport. Use [Getting Started](../docs/GETTING_STARTED.md) for a complete STDIO server loop and [Production Controls](../docs/PRODUCTION_CONTROLS.md) for network security.

## Generated examples

`examples/generated/` is produced from Rust documentation examples and indexed by [generated/README.md](generated/README.md). Do not use generated filenames as stable API names. Regenerate through the documentation-example tests rather than editing the index manually.

When adding an example, keep it focused, include it in `Cargo.toml` when feature gating is required, build it in CI, and update this table only if it is part of the recommended learning path.

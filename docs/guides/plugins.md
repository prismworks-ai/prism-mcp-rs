# Native Plugins

The optional plugin subsystem loads tool plugins from native dynamic libraries (`.so`, `.dylib`, or `.dll`) at runtime. Use it only for trusted code that must be loaded independently of the host binary.

## Security boundary

Native plugins execute in the server process with the server's OS permissions. They can read process memory, block executor threads, crash the process, and consume unbounded resources. The SDK does not sandbox them or enforce CPU/memory limits.

For untrusted, third-party, or failure-prone extensions, run a separate MCP server process/container under a restricted identity and connect over a transport. Sandboxed WebAssembly plugins are planned, not implemented.

## Enable plugins

```toml
[dependencies]
prism-mcp-rs = { version = "3", features = ["plugin"] }
```

A plugin is a `cdylib` implementing `plugin::ToolPlugin` and exporting the symbols produced by `export_plugin!`. Its `PluginMetadata` and `Tool` definition identify the tool registered by `PluginManager`.

The dynamic boundary is Rust-trait based and must not be treated as a stable C ABI across arbitrary compiler, crate-version, feature, or target changes. Build host and plugin with the same locked SDK version, compatible toolchain, target, and relevant features.

## Loading

```rust,ignore
use prism_mcp_rs::plugin::{PluginConfig, PluginManager};

let manager = PluginManager::new();
manager
    .load_plugin(
        PluginConfig::simple("reporting")
            .with_path("/opt/prism/plugins/libreporting.so"),
    )
    .await?;

let tools = manager.list_tools().await;
```

`PluginManager` supports load, unload, reload, enable/disable state, tool execution, listing, and directory/config-file helpers. `McpServer::load_plugins` can register loaded plugin tools with the server. Consult generated API documentation for exact type details because multiple legacy plugin type modules remain exposed in 3.x.

## Lifecycle

Loading resolves the library and factory, constructs the plugin, runs initialization/configuration, and registers its tool. Unloading unregisters the tool before releasing the library. Reload performs unload then load, so state is not automatically preserved.

Auto-reload watches files when configured, but it is an availability feature rather than isolation. A bad replacement can still fail initialization or crash the process. Use staged artifacts, atomic file replacement, and rollback.

## Operational checklist

- Allowlist absolute plugin paths; never load a path supplied by an untrusted request.
- Verify artifact provenance, digest/signature, target, and expected SDK/toolchain before load.
- Run the host as an unprivileged user with minimal filesystem/network access.
- Keep plugin configuration and secrets out of logs and source control.
- Apply timeouts in plugin code and move blocking/CPU-heavy work off Tokio workers.
- Test load, initialization failure, tool failure, reload, and unload.
- Monitor plugin errors and process resource use; define a fast disable/rollback procedure.
- Rebuild every plugin for SDK, compiler, or target upgrades.

## Component scope

The maintained dynamic loader registers `ToolPlugin` tools. The repository also contains general plugin/component data types for resources and prompts, but they do not imply that the native loader dynamically registers every component category. Treat generated API docs and tests as authoritative and avoid promising unsupported component loading.

## Distribution

Distribute the plugin artifact with a checksum, build metadata, supported host version/toolchain/target, license, configuration schema, and rollback instructions. Never advertise native plugins as sandboxed or ABI-stable.

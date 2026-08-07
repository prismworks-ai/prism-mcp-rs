# Getting Started

This guide builds a small STDIO MCP server with the default feature set.

## Requirements

- Rust 1.85 or newer
- Cargo
- An MCP client for end-to-end testing

## Create the project

```bash
cargo new hello-mcp
cd hello-mcp
```

Add dependencies:

```toml
[dependencies]
prism-mcp-rs = "2"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
serde_json = "1"
```

Replace `src/main.rs` with:

```rust,no_run
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

struct Greet;

#[async_trait]
impl ToolHandler for Greet {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let name = arguments
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("world");

        Ok(ToolResult {
            content: vec![ContentBlock::text(format!("Hello, {name}!"))],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServer::create("hello-mcp", "1.0.0");
    server
        .add_tool(
            "greet",
            Some("Greet a person"),
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                }
            }),
            Greet,
        )
        .await?;

    server
        .run_with_transport(StdioServerTransport::new())
        .await
}
```

Verify it:

```bash
cargo check
cargo test
```

Running `cargo run` starts the server and waits for newline-delimited JSON-RPC on stdin. This is expected. Configure the compiled command in an MCP client using [AI Tool Integration](AI_TOOL_INTEGRATION.md).

## Handler behavior

Use `McpResult::Err` for protocol, transport, authorization, or internal failures. For an expected tool-domain failure that should be returned as a successful JSON-RPC response, return `ToolResult` with `is_error: Some(true)` and explanatory content.

Never write human-readable output to stdout in a STDIO server because it corrupts the MCP stream. Use `tracing`, `eprintln!`, or a file sink.

## Add optional features

```toml
prism-mcp-rs = {
    version = "2",
    features = ["http", "tls", "otel"]
}
```

Feature selection is additive. Avoid `full` in production unless every feature is needed; smaller feature sets reduce compile time and dependency surface.

## Next steps

- Browse the maintained [examples](../examples/README.md).
- Add resources and prompts using the feature examples.
- Read [Production Controls](PRODUCTION_CONTROLS.md) before exposing a network transport.
- Use the [Troubleshooting Guide](TROUBLESHOOTING.md) if a client cannot start or communicate with the server.

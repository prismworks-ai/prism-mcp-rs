# API Reference

> **Auto-generated from Rust source code**  
> Last Updated: $(date +"%Y-%m-%d %H:%M:%S")

## Documentation

- [Online Documentation](https://docs.rs/prism-mcp-rs) (once published)
- [Local Documentation](../target/doc/prism_mcp_rs/index.html) (run `cargo doc --open`)
- [GitHub Repository](https://github.com/prismworks-ai/prism-mcp-rs)

## Core Modules

### Client
MCP client implementation for connecting to servers.
- [Local Docs](../target/doc/prism_mcp_rs/client/index.html)

### Server
MCP server implementation for handling requests.
- [Local Docs](../target/doc/prism_mcp_rs/server/index.html)

### Transport
Transport layer implementations (STDIO, HTTP, WebSocket, Streaming).
- [Local Docs](../target/doc/prism_mcp_rs/transport/index.html)

### Protocol
MCP protocol types and messages.
- [Local Docs](../target/doc/prism_mcp_rs/protocol/index.html)

### Core
Core utilities, error handling, and helper types.
- [Local Docs](../target/doc/prism_mcp_rs/core/index.html)

### Prelude
Commonly used types and traits for convenient imports.
- [Local Docs](../target/doc/prism_mcp_rs/prelude/index.html)

## Quick Start

```rust
use prism_mcp_rs::prelude::*;

// Create a server
let server = McpServer::new("my-server")
    .with_tool(my_tool)
    .with_resource(my_resource);

// Run with STDIO transport
server.run_stdio().await?;
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|--------|
| `stdio` | STDIO transport support | ✅ |
| `http` | HTTP transport with Axum server | ❌ |
| `websocket` | WebSocket transport | ❌ |
| `streaming-http` | Streaming HTTP support | ❌ |
| `validation` | JSON Schema validation | ❌ |
| `plugin` | Plugin system support | ❌ |
| `full` | All features enabled | ❌ |

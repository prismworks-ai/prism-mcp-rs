# API Reference

## Overview

The complete API documentation for the Prism MCP SDK is automatically generated and hosted on docs.rs when the crate is published. This page provides quick links to access the documentation.

## Documentation Links

- **[Online Documentation](https://docs.rs/prism-mcp-rs)** - Official API documentation (available after crate publication)
- **[Local Documentation](../target/doc/prism_mcp_rs/index.html)** - Generate locally with `cargo doc --open`
- **[GitHub Repository](https://github.com/prismworks-ai/prism-mcp-rs)** - Source code and examples

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

### Plugin
Plugin system for dynamic component loading.
- [Local Docs](../target/doc/prism_mcp_rs/plugin/index.html)
- Component Types: Tools, Resources, Prompts, Completions
- Dynamic loading and hot reload support

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

For a complete list of available features and their descriptions, see the [main README](../README.md#available-features).
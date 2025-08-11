<!-- 
═══════════════════════════════════════════════════════════════
## DOCUMENTATION METADATA
═══════════════════════════════════════════════════════════════
Type: User Guide (Manually Written)
Path: docs/api/README.md
Last Updated: 2025-08-11 12:22:37 UTC
Hash: 1b3fe660
Repository: https://github.com/prismworks-ai/mcp-protocol-sdk
═══════════════════════════════════════════════════════════════
-->

<div align="center">

### Note: Documentation Type: **Manually Written Guide**

[![2-Click Report →](https://img.shields.io/badge/2--Click%20Report%20→-red?style=for-the-badge)](https://github.com/prismworks-ai/mcp-protocol-sdk/issues/new?title=%23%23+Documentation+Issue%3A+README.md&labels=documentation%2Cgood+first+issue&body=%3C%21--+Thank+you+for+helping+us+improve%21+Your+report+helps+maintain+our+high+documentation+standards.+--%3E%0A%0A%23%23%23+%F0%9F%93%8D+Document+Details%0A-+%2A%2AFile%3A%2A%2A+%60docs%2Fapi%2FREADME.md%60%0A-+%2A%2AType%3A%2A%2A+Manually+Written%0A-+%2A%2AURL%3A%2A%2A+%5Bdocs%2Fapi%2FREADME.md%5D%28https%3A%2F%2Fgithub.com%2Fprismworks-ai%2Fmcp-protocol-sdk%2Fblob%2Fmain%2Fdocs%2Fapi%2FREADME.md%29%0A%0A%23%23%23+Bug%3A+Issue+Description%0A%3C%21--+Please+describe+what%27s+wrong+with+the+documentation+%28required%29+--%3E%0A%0A%0A%0A%23%23%23+Note%3A+Suggested+Fix+%28Optional%29%0A%3C%21--+If+you+know+how+to+fix+it%2C+please+share%21+--%3E%0A%0A%0A%0A---%0A%2AThank+you+for+helping+us+maintain+the+highest+documentation+standards%21+Thanks%2A%0A%2AThis+issue+was+created+using+the+2-click+reporting+system%2A)
[![Become a Contributor](https://img.shields.io/badge/Become%20a%20Contributor-blue?style=for-the-badge)](https://github.com/prismworks-ai/mcp-protocol-sdk/blob/main/CONTRIBUTING.md)

**Thank you for helping us maintain the highest documentation standards!**  
*Found an issue? Your 2-click report helps us improve. Want to do more? Join our contributors!*

</div>

---

<div align="center">
<sub>

**# MCP Protocol SDK** - The de facto industry standard for developing MCP clients and servers in Rust  
*Production-ready • 65%+ test coverage • Full protocol compliance • production-ready error handling*

</sub>
</div>

---

# API Documentation

> **Auto-generated from Rust source code**  
> Version: 0.1.0  
> Last updated: $(date '+%Y-%m-%d %H:%M:%S')

## Quick Links

- [Online Documentation](https://docs.rs/prism-mcp-rs)
- [Local Documentation](../../target/doc/prism_mcp_rs/index.html)
- [GitHub Repository](https://github.com/prismworks-ai/prism-mcp-rs)

## Core Modules

### [Client](./client.md)
MCP client implementation for connecting to servers.

### [Server](./server.md)
MCP server implementation for handling requests.

### [Transport](./transport.md)
Transport layer implementations:
- STDIO (default)
- HTTP
- WebSocket
- Streaming HTTP

### [Protocol](./protocol.md)
MCP protocol types and messages.

### [Core](./core.md)
Core utilities, error handling, and helper types.

### [Plugin](./plugin.md)
Plugin system for extending functionality.

### [Prelude](./prelude.md)
Commonly used types and traits for convenient imports.

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

## Examples

The SDK includes comprehensive examples in the `/examples` directory:

- **Basic Usage**: Simple client and server implementations
- **Transport Examples**: HTTP, WebSocket, and streaming demonstrations
- **Advanced Features**: Plugin system, validation, and performance optimization

## Getting Started

```rust
use prism_mcp_rs::prelude::*;

// Create a simple MCP server
let server = McpServer::new("my-server")
    .with_tool(my_tool)
    .with_resource(my_resource);

// Run with STDIO transport
server.run_stdio().await?;
```

## Documentation Standards

This documentation follows Prismworks AI documentation standards:
- Clear module organization
- Comprehensive examples
- Auto-generated from source code
- Regular updates with each build


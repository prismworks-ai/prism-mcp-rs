# Server Examples

Examples demonstrating how to build MCP servers using the SDK.

# Available Examples

# improved Echo Server
- **File**: `improved_echo_server.rs`
- **Features**: `stdio`, `tracing-subscriber`
- **Highlights**: New improved API with 90% less boilerplate code

# Database Server
- **File**: `database_server.rs`
- **Features**: `stdio`, `tracing-subscriber`
- **Description**: SQL operations and data persistence

# HTTP Server
- **File**: `http_server.rs`
- **Features**: `http`
- **Description**: RESTful API with HTTP transport

# WebSocket Server
- **File**: `websocket_server.rs`
- **Features**: `websocket`
- **Description**: Real-time bidirectional communication

# Running Examples

```bash
cargo run --example improved_echo_server --features "stdio,tracing-subscriber"
cargo run --example database_server --features "stdio,tracing-subscriber"
cargo run --example http_server --features http
cargo run --example websocket_server --features websocket
```

# Key Patterns

All server examples follow these core patterns:

1. **Import with prelude**: `use mcp_protocol_sdk::prelude::*;`
2. **Implement ToolHandler**: Required for all tools
3. **Server setup**: Create server and add tools
4. **Transport selection**: Choose STDIO, HTTP, or WebSocket
5. **One-line server start**: Use `server.run_with_stdio().await` for convenience

See individual example files for complete implementations.

For detailed API documentation, see the [Implementation Guide](../../docs/implementation-guide.md).

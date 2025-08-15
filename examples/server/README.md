# Server Examples

This directory contains examples demonstrating how to build MCP servers using different transport mechanisms.

## Available Examples

### Database Server (`database_server.rs`)
Production-ready server with database integration.

**Required Features:** None (uses default STDIO)

```bash
cargo run --example database_server
```

**Key Features:**
- SQLite database integration
- CRUD operations via tools
- Resource management
- Transaction support
- Error handling

### Enhanced Echo Server (`enhanced_echo_server.rs`)
Feature-rich echo server demonstrating various MCP capabilities.

**Required Features:** None (uses default STDIO)

```bash
cargo run --example enhanced_echo_server
```

**Key Features:**
- Multiple tool implementations
- Resource serving
- Progress notifications
- Metadata annotations
- Validation examples

### HTTP Server (`http_server.rs`)
HTTP-based MCP server with SSE for notifications.

**Required Features:** `http-server`

```bash
cargo run --example http_server --features "http-server"
```

**Key Features:**
- HTTP POST endpoint for requests
- Server-Sent Events for notifications
- CORS support
- Health check endpoint
- Graceful shutdown

### HTTP/2 Server (`http2_server.rs`)
High-performance HTTP/2 server with streaming.

**Required Features:** `http2-server`

```bash
cargo run --example http2_server --features "http2-server"
```

**Key Features:**
- HTTP/2 multiplexing
- Server push capabilities
- Stream prioritization
- Header compression
- Binary framing

### WebSocket Server (`websocket_server.rs`)
Real-time bidirectional WebSocket server.

**Required Features:** `websocket-server`

```bash
cargo run --example websocket_server --features "websocket-server"
```

**Key Features:**
- Full-duplex communication
- Real-time event broadcasting
- Connection management
- Ping/pong heartbeat
- Automatic reconnection support

## Server Architecture Patterns

### Tool Implementation
All servers demonstrate tool implementation patterns:
```rust
#[async_trait]
impl ToolHandler for MyTool {
    async fn handle(&self, input: Value) -> Result<Value> {
        // Tool logic here
    }
}
```

### Resource Serving
Examples show how to serve resources:
```rust
server.add_resource(Resource {
    uri: "file:///example.txt".to_string(),
    content: Content::Text("Example content".to_string()),
    ...
});
```

### Error Handling
Production-ready error handling patterns:
```rust
match operation() {
    Ok(result) => Ok(json!({ "success": true, "data": result })),
    Err(e) => Err(McpError::Internal(e.to_string()))
}
```

## Transport Selection Guide

| Transport | Best For | Pros | Cons |
|-----------|----------|------|------|
| STDIO | CLI tools, local apps | Zero config, secure | Local only |
| HTTP | Web apps, REST APIs | Universal, firewall-friendly | Higher latency |
| WebSocket | Real-time apps | Low latency, bidirectional | Complex setup |
| HTTP/2 | High-performance | Multiplexing, efficient | Limited browser support |

## Testing Your Server

### With STDIO Transport
```bash
# Direct testing
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run --example enhanced_echo_server
```

### With HTTP Transport
```bash
# Start server
cargo run --example http_server --features "http-server"

# Test with curl
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

### With WebSocket Transport
```bash
# Start server
cargo run --example websocket_server --features "websocket-server"

# Test with websocat
websocat ws://localhost:8080/mcp
```

## Production Deployment

For production deployment:
1. Enable appropriate logging
2. Configure timeouts and limits
3. Implement authentication
4. Add monitoring/metrics
5. Use environment variables for configuration
6. Implement graceful shutdown
7. Add health check endpoints

## Expected Output Examples

### Enhanced Echo Server
```
MCP Echo Server v1.0.0 starting...
Listening on STDIO transport
Registered tools: echo, reverse, uppercase
Registered resources: config://settings, data://info
Server ready for connections
```

### HTTP Server
```
HTTP MCP Server starting on 127.0.0.1:3000
SSE endpoint: /events
MCP endpoint: /mcp
Health check: /health
Server ready, press Ctrl+C to stop
```

## Troubleshooting

- **Port already in use**: Change port or kill existing process
- **Permission denied**: Check file system permissions for database_server
- **Feature not enabled**: Add required feature flag to cargo command
- **Slow performance**: Use release build with `--release` flag

## Contributing

When adding new server examples:
1. Follow existing patterns for consistency
2. Include comprehensive error handling
3. Add documentation and comments
4. Test with corresponding client examples
5. Update this README
6. Add expected output examples
7. Include troubleshooting tips
# Client Examples

This directory contains examples demonstrating how to build MCP clients using different transport mechanisms.

## Available Examples

### HTTP Client (`http_client.rs`)
Basic HTTP client with Server-Sent Events for notifications.

**Required Features:** `http-client`

```bash
cargo run --example http_client --features "http-client"
```

**Key Features:**
- HTTP POST for requests
- Server-Sent Events for real-time notifications
- Automatic reconnection
- Request/response pattern

### Advanced HTTP Client (`advanced_http_client.rs`)
HTTP client with streaming capabilities and advanced features.

**Required Features:** `http-client`

```bash
cargo run --example advanced_http_client --features "http-client"
```

**Key Features:**
- Streaming response handling
- Compression support
- Progress tracking
- Large payload optimization

### Conservative HTTP Demo (`conservative_http_demo.rs`)
Memory-efficient HTTP client implementation.

**Required Features:** `http-client`

```bash
cargo run --example conservative_http_demo --features "http-client"
```

**Key Features:**
- Minimal memory footprint
- Streaming processing
- Efficient resource management
- Suitable for constrained environments

### WebSocket Client (`websocket_client.rs`)
Real-time bidirectional WebSocket client.

**Required Features:** `websocket-client`

```bash
cargo run --example websocket_client --features "websocket-client"
```

**Key Features:**
- Full-duplex communication
- Lowest latency (<5ms)
- Automatic reconnection with backoff
- Real-time event handling

## Transport Comparison

| Transport | Latency | Use Case | Required Feature |
|-----------|---------|----------|------------------|
| HTTP | 10-50ms | Web apps, mobile | `http-client` |
| WebSocket | <5ms | Real-time apps | `websocket-client` |
| Streaming HTTP | 10-30ms | Large payloads | `streaming-http` |

## Common Patterns

All client examples demonstrate:
1. Connection establishment
2. Request/response handling
3. Error recovery
4. Resource cleanup
5. Notification handling

## Quick Start

1. Choose a transport based on your needs
2. Enable the required feature in Cargo.toml
3. Run the example
4. Adapt the code to your application

## Testing Against a Server

To test these clients, you need a corresponding MCP server:

```bash
# Terminal 1: Start a server
cargo run --example http_server --features "http-server"

# Terminal 2: Run the client
cargo run --example http_client --features "http-client"
```

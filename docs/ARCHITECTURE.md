# Prism MCP SDK Architecture

## Transport Layer Architecture

The Prism MCP SDK provides a modular transport layer with clear separation of concerns. Each transport protocol and feature is independently selectable through Cargo features.

## Core Transports

### STDIO Transport (Default)
- **Feature**: `stdio` (enabled by default)
- **Use Case**: Command-line tools, local process communication
- **Protocol**: Standard input/output streams
- **Characteristics**: Simple, synchronous, no network overhead

### HTTP Transport
- **Feature**: `http`
- **Use Case**: RESTful APIs, web services, microservices
- **Protocol**: HTTP/1.1 with JSON-RPC
- **Characteristics**: Request-response, stateless, widely compatible

### WebSocket Transport
- **Feature**: `websocket`
- **Use Case**: Real-time bidirectional communication
- **Protocol**: WebSocket (RFC 6455)
- **Characteristics**: Full-duplex, persistent connection, low latency

## HTTP Protocol Extensions

### Server-Sent Events (SSE)
- **Feature**: `sse`
- **Requires**: `http` feature
- **Dependencies**: `tokio-stream`, `futures-util`
- **Use Case**: Real-time server-to-client push notifications
- **Protocol**: EventSource (W3C Server-Sent Events)
- **Characteristics**:
  - Unidirectional (server → client)
  - Text-based event stream
  - Automatic reconnection
  - Browser-native support

**When to use SSE:**
- Progress updates for long-running operations
- Real-time log streaming
- Live data feeds (stock prices, metrics)
- Event notifications

**SSE Message Format:**
```
event: progress
id: 12345
data: {"percent": 50, "status": "Processing"}

event: notification
data: {"type": "alert", "message": "Task completed"}
```

### HTTP/2 Protocol
- **Feature**: `http2`
- **Requires**: `http` feature
- **Dependencies**: `h2`
- **Use Case**: High-performance communication with multiplexing
- **Protocol**: HTTP/2 (RFC 7540)
- **Characteristics**:
  - Binary protocol
  - Stream multiplexing
  - Server push capability
  - Header compression (HPACK)
  - Flow control

**When to use HTTP/2:**
- High-throughput applications
- Multiple concurrent API calls
- Reduced latency requirements
- Efficient resource loading

### Chunked Transfer Encoding
- **Feature**: `chunked-encoding`
- **Requires**: `http` feature
- **Use Case**: Streaming large payloads without buffering
- **Protocol**: HTTP/1.1 Transfer-Encoding: chunked
- **Characteristics**:
  - Memory-efficient for large data
  - Progressive data transfer
  - No Content-Length required

**When to use Chunked Encoding:**
- Large file uploads/downloads
- Streaming data processing
- Memory-constrained environments
- Unknown content size

### Response Compression
- **Feature**: `compression`
- **Requires**: `http` feature
- **Dependencies**: `brotli`, `flate2`, `zstd`
- **Use Case**: Bandwidth optimization
- **Supported Algorithms**:
  - Gzip (widely compatible)
  - Brotli (better compression)
  - Zstd (fastest compression)

**When to use Compression:**
- High bandwidth costs
- Large text/JSON payloads
- Mobile clients
- International users

## Feature Comparison

| Feature | Protocol | Direction | Use Case | Overhead |
|---------|----------|-----------|----------|----------|
| HTTP | HTTP/1.1 | Request-Response | REST APIs | Medium |
| SSE | HTTP/1.1 + EventSource | Server→Client | Real-time updates | Low |
| HTTP/2 | HTTP/2 | Bidirectional | High performance | Low |
| WebSocket | WebSocket | Bidirectional | Real-time chat | Very Low |
| Chunked | HTTP/1.1 | Both | Large transfers | Very Low |

## Architecture Decisions

### Why SSE is Separate from HTTP
SSE is an optional enhancement to HTTP that adds server-push capabilities. Not all HTTP servers need SSE support, and it requires additional dependencies (streaming libraries). Making it a separate feature allows users to opt-in only when needed.

### Why HTTP/2 is Separate from HTTP
HTTP/2 is a different protocol version with its own binary framing layer. While it's backwards-compatible at the application level, it requires different handling at the transport level. Separating it allows users to choose between HTTP/1.1 simplicity and HTTP/2 performance.

### Why Chunked Encoding is Separate
Chunked transfer encoding adds complexity to handle streaming data. Many applications only need simple request-response patterns. Making it optional keeps the core HTTP transport simple while allowing advanced users to enable streaming capabilities.

## Migration from Old Architecture

If you were using the old feature flags, here's how to migrate:

| Old Feature | New Feature(s) | Notes |
|------------|---------------|-------|
| `tokio-stream` | `sse` | SSE now explicitly named |
| `streaming-http` | `chunked-encoding` | Clearer purpose |
| `streaming-http2` | `http2` | Simplified naming |
| `streaming-compression` | `compression` | Simplified naming |

## Best Practices

1. **Start Simple**: Use basic `http` transport first
2. **Add Features as Needed**: Enable SSE, HTTP/2, etc. only when required
3. **Consider Bundle Features**: Use `full` for development, specific features for production
4. **Profile Performance**: Measure before enabling compression or HTTP/2
5. **Test Compatibility**: Ensure clients support your chosen features

## Example Configurations

### Basic HTTP API
```toml
[dependencies]
prism-mcp-rs = { version = "0.1", features = ["http"] }
```

### Real-time Updates with SSE
```toml
[dependencies]
prism-mcp-rs = { version = "0.1", features = ["http", "sse"] }
```

### High-Performance with HTTP/2
```toml
[dependencies]
prism-mcp-rs = { version = "0.1", features = ["http", "http2"] }
```

### Full-Featured Server
```toml
[dependencies]
prism-mcp-rs = { version = "0.1", features = ["full"] }
```

## Implementation Notes

### SSE Implementation
- Uses `axum::response::Sse` for server implementation
- Requires `tokio-stream` for async streaming
- Automatically handles reconnection with Last-Event-ID
- Keep-alive messages prevent connection timeout

### HTTP/2 Implementation
- Uses `h2` crate for protocol handling
- Manages stream multiplexing automatically
- Supports server push for proactive resource delivery
- Implements flow control and backpressure

### Chunked Encoding Implementation
- Uses `Transfer-Encoding: chunked` header
- Streams data in chunks without buffering entire payload
- Supports both request and response streaming
- Handles backpressure automatically

## Future Considerations

- **HTTP/3**: QUIC-based transport for improved performance
- **WebTransport**: Modern replacement for WebSockets
- **gRPC**: Protocol Buffers-based RPC
- **Custom Transports**: Plugin system for proprietary protocols

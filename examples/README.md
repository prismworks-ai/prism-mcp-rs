# MCP SDK Examples

Comprehensive examples demonstrating the capabilities of the prism-mcp-rs SDK.

## Quick Start

To run any example:
```bash
# For examples with default features
cargo run --example <example_name>

# For examples requiring specific features
cargo run --example <example_name> --features "<required_features>"
```

## Examples Overview

### Core Examples (No Additional Features Required)

These examples work with the default SDK configuration:

| Example | Description | Command |
|---------|-------------|---------|
| `advanced_2025_features` | Demonstrates MCP 2025-06-18 specification features | `cargo run --example advanced_2025_features` |
| `advanced_features_showcase` | Showcases bidirectional communication, completion API, resource templates | `cargo run --example advanced_features_showcase` |
| `bidirectional_communication_demo` | Server-to-client request patterns | `cargo run --example bidirectional_communication_demo` |
| `convenience_methods_demo` | One-line server setup methods | `cargo run --example convenience_methods_demo` |
| `performance_benchmarks` | Transport performance analysis and comparison | `cargo run --example performance_benchmarks` |
| `production_error_handling_demo` | Production-ready error handling patterns | `cargo run --example production_error_handling_demo` |
| `transport_selection_guide` | Interactive guide for choosing the right transport | `cargo run --example transport_selection_guide` |

### Server Examples

| Example | Required Features | Description | Command |
|---------|------------------|-------------|---------|
| `server/database_server` | None | Database-backed MCP server | `cargo run --example database_server` |
| `server/enhanced_echo_server` | None | Echo server with tools and resources | `cargo run --example enhanced_echo_server` |
| `server/http_server` | `http-server` | HTTP-based MCP server | `cargo run --example http_server --features "http-server"` |
| `server/http2_server` | `http2-server` | HTTP/2 server with streaming | `cargo run --example http2_server --features "http2-server"` |
| `server/websocket_server` | `websocket-server` | WebSocket MCP server | `cargo run --example websocket_server --features "websocket-server"` |

### Client Examples

| Example | Required Features | Description | Command |
|---------|------------------|-------------|---------|
| `client/http_client` | `http-client` | Basic HTTP client | `cargo run --example http_client --features "http-client"` |
| `client/advanced_http_client` | `http-client` | Advanced HTTP client with streaming | `cargo run --example advanced_http_client --features "http-client"` |
| `client/conservative_http_demo` | `http-client` | Memory-efficient HTTP client | `cargo run --example conservative_http_demo --features "http-client"` |
| `client/websocket_client` | `websocket-client` | WebSocket client with reconnection | `cargo run --example websocket_client --features "websocket-client"` |

### Streaming Examples

| Example | Required Features | Description | Command |
|---------|------------------|-------------|---------|
| `streaming_http_showcase` | `streaming-http, tracing-subscriber, chrono` | HTTP streaming with compression | `cargo run --example streaming_http_showcase --features "streaming-http tracing-subscriber chrono"` |
| `streaming_http2_showcase` | `streaming-http2, tracing-subscriber, chrono` | HTTP/2 streaming with server push | `cargo run --example streaming_http2_showcase --features "streaming-http2 tracing-subscriber chrono"` |

### Utility Examples

| Example | Required Features | Description | Command |
|---------|------------------|-------------|---------|
| `utilities/transport_benchmark` | `http-client, websocket-client, streaming-http` | Benchmark all transport types | `cargo run --example transport_benchmark --features "http-client websocket-client streaming-http"` |

## Feature Flags

The SDK uses feature flags to minimize dependencies. Here are the available features:

### Transport Features
- `stdio` (default) - STDIO transport for process communication
- `http-client` - HTTP client transport
- `http-server` - HTTP server transport
- `websocket-client` - WebSocket client transport
- `websocket-server` - WebSocket server transport
- `streaming-http` - HTTP with streaming and compression
- `streaming-http2` - HTTP/2 with streaming and server push

### Utility Features
- `tracing` - Structured logging support
- `tracing-subscriber` - Logging configuration for examples
- `chrono` - Time/date handling
- `testing` - Testing utilities

### Convenience Feature Sets
- `client-full` - All client transports
- `server-full` - All server transports
- `streaming-full` - All streaming features
- `full` - All features enabled

## Running All Examples

To build all examples with their required features:
```bash
# Build all examples with all features
cargo build --all-features --examples

# Run specific category
cargo build --features "client-full" --examples  # All client examples
cargo build --features "server-full" --examples  # All server examples
```

## Example Categories

### Getting Started
Start with these examples to understand the basics:
1. `convenience_methods_demo` - Simplest server setup
2. `enhanced_echo_server` - Basic server with tools
3. `transport_selection_guide` - Choose the right transport

### Building Applications
For building production applications:
1. `production_error_handling_demo` - Error handling patterns
2. `advanced_features_showcase` - Advanced MCP features
3. `database_server` - Persistent server example

### Advanced Features
For leveraging advanced MCP 2025 features:
1. `advanced_2025_features` - Latest specification features
2. `bidirectional_communication_demo` - Server-initiated requests
3. `streaming_http_showcase` - Large payload handling

### Performance & Testing
For optimization and benchmarking:
1. `performance_benchmarks` - Transport comparison
2. `transport_benchmark` - Detailed benchmarking
3. `conservative_http_demo` - Memory optimization

## Troubleshooting

### Common Issues

1. **Example fails to compile**
   - Check if required features are enabled (see table above)
   - Use `--all-features` to enable everything

2. **"Feature not found" error**
   - Ensure you're using the latest version of the SDK
   - Check Cargo.toml for available features

3. **Runtime errors**
   - Some examples require external services (database, etc.)
   - Check example source for setup requirements

## Contributing

When adding new examples:
1. Place in appropriate directory (client/, server/, utilities/, or root)
2. Add required features to example's Cargo.toml section
3. Update this README with the example details
4. Include comprehensive documentation in the example file

## License

All examples are part of the prism-mcp-rs SDK and follow the same license terms.
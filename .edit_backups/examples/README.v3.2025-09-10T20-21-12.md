# Prism MCP SDK Examples

This directory contains example applications demonstrating how to use the Prism MCP SDK.

## Running Examples

```bash
# Build all examples
cargo build --examples

# Run a specific example
cargo run --example server_builder_demo

# Run with specific features
cargo run --example sse_showcase --features sse
cargo run --example http2_showcase --features http2
cargo run --example chunked_encoding_showcase --features chunked-encoding
```

## Available Examples

### SDK Enhancement Examples

- **sdk_enhancements_demo.rs** - Comprehensive demonstration of SDK enhancements:
  - McpClientBuilder with full configuration
  - All session convenience methods (list_tools, call_tool, etc.)
  - ConnectionConfig and RetryConfig
  - Interactive request handler

- **client_with_info.rs** - Alternative client creation methods:
  - `McpClient::with_client_info()` constructor
  - StdioClientTransport with `with_env()` for environment variables
  - Direct transport configuration

- **client_with_anthropic.rs** - Production LLM integration:
  - ClientRequestHandler for Anthropic Claude
  - Handling sampling/createMessage requests
  - MCP ↔ Anthropic message format conversion

### Core Examples

- **server_builder_demo.rs** - Building and configuring MCP servers
- **bidirectional.rs** - Bidirectional communication patterns
- **bidirectional_communication_demo.rs** - Full bidirectional demo
- **custom_transport.rs** - Implementing custom transports

### Advanced Features

- **advanced_2025_features.rs** - Showcases cutting-edge 2025 MCP features
- **advanced_features_showcase.rs** - Advanced SDK capabilities
- **convenience_methods_demo.rs** - Using SDK convenience methods
- **transport_selection_guide.rs** - Choosing the right transport

### Advanced HTTP Features

- **sse_showcase.rs** - Server-Sent Events for real-time updates
- **http2_showcase.rs** - HTTP/2 protocol with multiplexing and server push
- **chunked_encoding_showcase.rs** - Chunked transfer encoding for large payloads
- **performance_benchmarks.rs** - Performance testing examples

### Best Practices

- **production_error_handling_demo.rs** - Production-grade error handling

## Documentation Examples

The SDK source code contains many small documentation examples that are tested with:

```bash
cargo test --doc
```

These appear in the generated documentation:

```bash
cargo doc --open
```

## Contributing Examples

When adding new examples:

1. Create a descriptive `.rs` file in this directory
2. Add an entry to `Cargo.toml`:
   ```toml
   [[example]]
   name = "your_example"
   path = "examples/your_example.rs"
   required-features = ["feature-name"]  # if needed
   ```
3. Include clear comments explaining what the example demonstrates
4. Test that it builds: `cargo build --example your_example`

## Learning Path

1. Start with `server_builder_demo.rs` for basic server setup
2. Review `bidirectional.rs` and `bidirectional_communication_demo.rs` for communication patterns
3. Explore `production_error_handling_demo.rs` for production patterns
4. Check transport examples for different communication methods
5. Study advanced features for extensibility
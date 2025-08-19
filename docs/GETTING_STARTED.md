# Getting Started

## Prerequisites

### System Requirements

- **Rust** 1.85.0 or later (MSRV)
- **Operating System** - Linux, macOS, Windows
- **Memory** - Minimum 512MB RAM
- **Network** - Required for HTTP/WebSocket transports

### Development Environment

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should show 1.85.0 or later
cargo --version
```

## Installation

### Creating a New Project

```bash
cargo new mcp-application
cd mcp-application
```

### Adding Dependencies

#### Minimal Configuration

```toml
# Cargo.toml
[dependencies]
prism-mcp-rs = "0.1.0"
tokio = { version = "1.34", features = ["rt-multi-thread", "macros"] }
serde_json = "1.0"
async-trait = "0.1"
```

#### Production Configuration

```toml
# Cargo.toml
[dependencies]
prism-mcp-rs = { 
    version = "0.1.0",
    features = ["http2", "compression", "auth", "tls", "plugin"]
}
tokio = { version = "1.34", features = ["full"] }
serde_json = "1.0"
async-trait = "0.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1.0"
```

## Building Your First Server

### Basic Implementation

```rust
// src/main.rs
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::server::McpServer;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

// Define a tool handler
struct EchoHandler;

#[async_trait]
impl ToolHandler for EchoHandler {
    async fn call(
        &self,
        arguments: HashMap<String, Value>
    ) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message provided");
        
        Ok(ToolResult::text(message.to_string()))
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    // Create server instance
    let mut server = McpServer::new(
        "echo-server".to_string(),
        "1.0.0".to_string()
    );
    
    // Register tool
    server.add_tool(
        "echo".to_string(),
        Some("Echoes the provided message".to_string()),
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo"
                }
            },
            "required": ["message"]
        }),
        EchoHandler,
    ).await?;
    
    // Start server with STDIO transport
    tracing::info!("Starting MCP server on STDIO");
    server.run_with_stdio().await
}
```

### Running the Server

```bash
# Build the application
cargo build --release

# Run the server
cargo run

# Or run the compiled binary
./target/release/mcp-application
```

## Building Your First Client

### Client Implementation

```rust
// examples/client.rs
use prism_mcp_rs::client::ClientSession;
use prism_mcp_rs::transport::stdio::StdioClientTransport;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create transport connected to server process
    let transport = StdioClientTransport::new_with_command(
        "./target/release/mcp-application"
    );
    
    // Initialize client session
    let mut session = ClientSession::new(transport);
    
    // Connect and initialize
    let init_result = session.initialize(
        "test-client".to_string(),
        "1.0.0".to_string()
    ).await?;
    
    println!("Connected to: {}", init_result.server_info.name);
    println!("Server version: {}", init_result.server_info.version);
    
    // List available tools
    let tools = session.list_tools().await?;
    for tool in &tools.tools {
        println!("Available tool: {}", tool.name);
        if let Some(desc) = &tool.description {
            println!("  Description: {}", desc);
        }
    }
    
    // Call the echo tool
    let arguments = HashMap::from([
        ("message".to_string(), json!("Hello, MCP!"))
    ]);
    
    let result = session.call_tool("echo", Some(arguments)).await?;
    println!("Echo response: {:?}", result);
    
    Ok(())
}
```

## Transport Configuration

### HTTP Transport

```rust
use prism_mcp_rs::transport::http::{HttpServerTransport, HttpClientTransport};

// Server configuration
let server_transport = HttpServerTransport::new("127.0.0.1:8080").await?;
server.run_with_transport(server_transport).await?;

// Client configuration
let client_transport = HttpClientTransport::new("http://localhost:8080").await?;
let mut session = ClientSession::new(client_transport);
```

### WebSocket Transport

```rust
use prism_mcp_rs::transport::websocket::{WebSocketServerTransport, WebSocketClientTransport};

// Server configuration
let server_transport = WebSocketServerTransport::new("127.0.0.1:9000").await?;
server.run_with_transport(server_transport).await?;

// Client configuration
let client_transport = WebSocketClientTransport::new("ws://localhost:9000").await?;
let mut session = ClientSession::new(client_transport);
```

## Advanced Features

### Enabling Resilience

```rust
use prism_mcp_rs::client::SessionConfig;
use prism_mcp_rs::core::retry::{RetryConfig, CircuitBreakerConfig};
use std::time::Duration;

let config = SessionConfig {
    retry_config: RetryConfig {
        max_attempts: 3,
        initial_delay_ms: 100,
        max_delay_ms: 5000,
        exponential_base: 2.0,
        jitter: true,
        ..Default::default()
    },
    enable_circuit_breaker: true,
    circuit_breaker_config: CircuitBreakerConfig {
        failure_threshold: 5,
        recovery_timeout: Duration::from_secs(30),
        half_open_max_requests: 3,
    },
    ..Default::default()
};

let mut session = ClientSession::new_with_config(transport, config);
```

### Adding Authentication

```rust
use prism_mcp_rs::auth::{AuthConfig, TokenValidator};

let auth_config = AuthConfig {
    require_auth: true,
    token_header: "Authorization".to_string(),
    token_prefix: "Bearer ".to_string(),
};

server.set_auth_config(auth_config);
server.set_token_validator(Box::new(MyTokenValidator));
```

### Implementing Resources

```rust
use prism_mcp_rs::core::resource::{Resource, ResourceHandler};

#[async_trait]
impl ResourceHandler for FileResourceHandler {
    async fn read(
        &self,
        uri: &str
    ) -> McpResult<ResourceContents> {
        let path = uri.strip_prefix("file://")
            .ok_or_else(|| McpError::invalid_params("Invalid URI"))?;
        
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| McpError::internal(format!("Failed to read file: {}", e)))?;
        
        Ok(ResourceContents::Text(TextContent {
            text: content,
            mime_type: Some("text/plain".to_string()),
        }))
    }
}

// Register resource handler
server.add_resource(
    "file".to_string(),
    Some("File system resource".to_string()),
    FileResourceHandler,
).await?;
```

## Error Handling

### Structured Error Management

```rust
use prism_mcp_rs::core::error::{McpError, McpResult};

#[async_trait]
impl ToolHandler for ValidatedHandler {
    async fn call(
        &self,
        arguments: HashMap<String, Value>
    ) -> McpResult<ToolResult> {
        // Input validation
        let value = arguments
            .get("value")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::invalid_params("Missing or invalid 'value' parameter"))?;
        
        // Business logic validation
        if value < 0.0 {
            return Err(McpError::invalid_params("Value must be non-negative"));
        }
        
        // Operation that might fail
        let result = self.process_value(value)
            .await
            .map_err(|e| McpError::internal(format!("Processing failed: {}", e)))?;
        
        Ok(ToolResult::text(format!("Result: {}", result)))
    }
}
```

## Testing

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_echo_handler() {
        let handler = EchoHandler;
        let mut args = HashMap::new();
        args.insert("message".to_string(), json!("test"));
        
        let result = handler.call(args).await.unwrap();
        
        match result {
            ToolResult::Text(text) => assert_eq!(text.text, "test"),
            _ => panic!("Expected text result"),
        }
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_server_client_integration() {
    // Start server in background
    let server_handle = tokio::spawn(async {
        let mut server = create_test_server();
        server.run_with_stdio().await
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect client
    let transport = StdioClientTransport::new_with_command("./test-server");
    let mut session = ClientSession::new(transport);
    
    // Test initialization
    let init = session.initialize("test".to_string(), "1.0".to_string()).await;
    assert!(init.is_ok());
    
    // Cleanup
    server_handle.abort();
}
```

## Performance Optimization

### Connection Pooling

```rust
use prism_mcp_rs::transport::http::HttpClientTransportBuilder;

let transport = HttpClientTransportBuilder::new()
    .base_url("http://localhost:8080")
    .connection_pool_size(10)
    .timeout(Duration::from_secs(30))
    .build()
    .await?;
```

### Request Batching

```rust
use prism_mcp_rs::protocol::batch::{BatchRequest, BatchOperation};

let batch = BatchRequest {
    operations: vec![
        BatchOperation {
            id: "1".to_string(),
            method: "tools/call".to_string(),
            params: json!({"name": "tool1", "arguments": {}}),
        },
        BatchOperation {
            id: "2".to_string(),
            method: "tools/call".to_string(),
            params: json!({"name": "tool2", "arguments": {}}),
        },
    ],
};

let results = session.execute_batch(batch).await?;
```

## Deployment

### Building for Production

```bash
# Optimized release build
cargo build --release --features production

# Strip debug symbols
strip target/release/mcp-application

# Verify binary
ldd target/release/mcp-application  # Linux
otool -L target/release/mcp-application  # macOS
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.85 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --features production

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mcp-application /usr/local/bin/
EXPOSE 8080
CMD ["mcp-application"]
```

## Next Steps

### Recommended Reading

1. [Architecture Guide](ARCHITECTURE.md) - System design and components
2. [Performance Guide](guides/performance.md) - Optimization techniques
3. [Plugin Development](guides/plugins.md) - Creating extensions
4. [Authentication Guide](guides/authentication.md) - Security implementation

### Advanced Topics

- Schema introspection for dynamic discovery
- Custom transport implementation
- Distributed deployment patterns
- Monitoring and observability setup

### Community Resources

- [GitHub Repository](https://github.com/prismworks-ai/prism-mcp-rs)
- [API Documentation](https://docs.rs/prism-mcp-rs)
- [Discord Community](https://discord.gg/prismworks)
- [Example Applications](https://github.com/prismworks-ai/prism-mcp-rs/tree/main/examples)
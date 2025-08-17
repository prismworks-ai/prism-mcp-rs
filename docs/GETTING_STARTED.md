# Getting Started with Prism MCP SDK

This guide walks you through installing the Prism MCP SDK, setting up your development environment, and building your first MCP application.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Your First MCP Server](#your-first-mcp-server)
4. [Your First MCP Client](#your-first-mcp-client)
5. [Understanding the Core Concepts](#understanding-the-core-concepts)
6. [Next Steps](#next-steps)

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.85 or later** - Install from [rustup.rs](https://rustup.rs/)
- **Cargo** - Included with Rust installation
- **Basic Rust knowledge** - Familiarity with async/await and trait concepts

## Installation

### Creating a New Project

1. Create a new Rust project:

```bash
mkdir my-mcp-project
cd my-mcp-project
cargo init
```

2. Add the Prism MCP SDK to your `Cargo.toml`:

```toml
[dependencies]
prism-mcp-rs = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
async-trait = "0.1"
```

For additional features like WebSocket or HTTP transport:

```toml
[dependencies]
prism-mcp-rs = {
    version = "0.1.0",
    features = ["websocket", "http"]
}
```

## Your First MCP Server

Let's create a simple MCP server that provides a "hello" tool.

### Step 1: Create the Tool Handler

Create `src/main.rs`:

```rust
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::server::McpServer;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

// Define a handler for our "hello" tool
struct HelloHandler;

#[async_trait]
impl ToolHandler for HelloHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Get the name argument, or use "World" as default
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
        
        // Return a greeting
        Ok(ToolResult::text(format!("Hello, {}!", name)))
    }
}
```

### Step 2: Create and Configure the Server

Add to your `main.rs`:

```rust
#[tokio::main]
async fn main() -> McpResult<()> {
    // Create a new MCP server
    let mut server = McpServer::new(
        "hello-server".to_string(),
        "1.0.0".to_string()
    );
    
    // Register the "hello" tool
    server.add_tool(
        "hello".to_string(),
        Some("Greets a user by name".to_string()),
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name to greet"
                }
            },
            "required": []
        }),
        HelloHandler,
    ).await?;
    
    // Start the server with STDIO transport
    println!("Starting Hello MCP Server...");
    server.run_with_stdio().await
}
```

### Step 3: Run the Server

```bash
cargo run
```

Your server is now running and listening for MCP commands via standard input/output!

## Your First MCP Client

Now let's create a client that can connect to and interact with MCP servers.

### Step 1: Create a Client Application

Create a new file `src/bin/client.rs`:

```rust
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::client::ClientSession;
use prism_mcp_rs::transport::stdio::StdioClientTransport;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create a transport to connect to the server
    // Replace "path/to/server" with the actual server executable path
    let transport = StdioClientTransport::new_with_command("./target/debug/my-mcp-project");
    
    // Create a client session
    let mut session = ClientSession::new(transport);
    
    // Initialize the connection
    session.initialize(
        "hello-client".to_string(),
        "1.0.0".to_string(),
    ).await?;
    
    println!("Connected to server!");
    
    // List available tools
    let tools = session.list_tools().await?;
    println!("Available tools:");
    for tool in &tools.tools {
        println!("  - {}: {:?}", tool.name, tool.description);
    }
    
    // Call the hello tool
    let result = session.call_tool(
        "hello",
        Some(HashMap::from([
            ("name".to_string(), json!("Alice")),
        ])),
    ).await?;
    
    println!("Server response: {:?}", result);
    
    Ok(())
}
```

### Step 2: Run the Client

Add a binary target to your `Cargo.toml`:

```toml
[[bin]]
name = "client"
path = "src/bin/client.rs"
```

Then run:

```bash
cargo run --bin client
```

## Understanding the Core Concepts

### Tools

**Tools** are executable functions that perform operations. They:
- Accept typed arguments (as JSON)
- Return results or errors
- Can be synchronous or asynchronous
- Are the primary way to expose functionality

### Resources

**Resources** provide access to data through URI-based addressing:
- Read-only access to content
- Support parameterized queries
- Return structured or unstructured data

Example resource handler:

```rust
use prism_mcp_rs::core::ResourceHandler;

struct FileResource;

#[async_trait]
impl ResourceHandler for FileResource {
    async fn read(
        &self,
        uri: &str,
        params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        // Implementation to read file content
        if let Some(path) = uri.strip_prefix("file://") {
            let content = tokio::fs::read_to_string(path).await?;
            Ok(vec![ResourceContents::Text {
                uri: uri.to_string(),
                mime_type: Some("text/plain".to_string()),
                text: content,
                meta: None,
            }])
        } else {
            Err(McpError::ResourceNotFound(uri.to_string()))
        }
    }
}
```

### Prompts

**Prompts** generate message templates for LLM interactions:
- Create structured conversation contexts
- Support dynamic parameter substitution
- Return role-based message sequences

### Transports

The SDK supports multiple transport mechanisms:

- **STDIO** (default) - Communication via standard input/output
- **WebSocket** - Real-time bidirectional communication
- **HTTP/SSE** - HTTP with Server-Sent Events for streaming

## Error Handling

The SDK provides comprehensive error handling:

```rust
use prism_mcp_rs::McpError;

#[async_trait]
impl ToolHandler for MyHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Validate required parameters
        let required = arguments.get("required_field")
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing required_field".to_string(),
            })?;
        
        // Handle operation errors
        match perform_operation(required) {
            Ok(result) => Ok(ToolResult::text(result)),
            Err(e) => Ok(ToolResult::error(format!("Operation failed: {}", e)))
        }
    }
}
```

## Configuration

### Transport Selection

Configure transport based on your deployment needs:

```rust
use prism_mcp_rs::server::McpServer;

#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServer::new("my-server".to_string(), "1.0.0".to_string());
    
    // Choose transport based on environment
    match std::env::var("MCP_TRANSPORT").as_deref() {
        Ok("websocket") => {
            #[cfg(feature = "websocket")]
            server.run_with_websocket("127.0.0.1:8080").await?
        }
        Ok("http") => {
            #[cfg(feature = "http")]
            server.run_with_http("127.0.0.1:3000").await?
        }
        _ => server.run_with_stdio().await?
    }
    
    Ok(())
}
```

### Logging

Enable logging for debugging:

```rust
use env_logger;

fn main() {
    env_logger::init();
    // Your application code
}
```

Run with logging:

```bash
RUST_LOG=debug cargo run
```

## Next Steps

### Explore Advanced Features

1. **Plugin Development** - See [docs/guides/plugins.md](guides/plugins.md)
2. **Authentication** - See [docs/guides/authentication.md](guides/authentication.md)
3. **Error Handling** - See [docs/guides/error-handling.md](guides/error-handling.md)
4. **Performance** - See [docs/guides/performance.md](guides/performance.md)

### Example Projects

Explore the [examples directory](../examples/) for complete implementations:

- `async_server.rs` - Async server with multiple handlers
- `bidirectional.rs` - Bidirectional client-server communication
- `custom_transport.rs` - Implementing custom transports
- `server_builder_demo.rs` - Advanced server configuration

### API Documentation

For detailed API documentation:

```bash
cargo doc --open
```

Or visit [docs.rs/prism-mcp-rs](https://docs.rs/prism-mcp-rs) after the crate is published.

### Community Resources

- [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues) - Report bugs or request features
- [GitHub Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions) - Ask questions and share ideas
- [Contributing Guide](../CONTRIBUTING.md) - Learn how to contribute to the project
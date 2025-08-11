# Client Examples

This directory contains examples demonstrating how to build MCP clients using the SDK.

# Examples

# Basic Client
- **File**: `basic_client.rs`
- **Features**: `stdio`
- **Description**: Simple MCP client using stdio transport

**Key Code Pattern:**
```rust
use mcp_protocol_sdk::{
 client::{McpClient, ClientSession},
 transport::stdio::StdioClientTransport,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 let client = McpClient::new("demo-client".to_string(), "1.0.0".to_string());
 let session = ClientSession::new(client);
 
 let transport = StdioClientTransport::new("./server-command".to_string()).await?;
 let init_result = session.connect(transport).await?;
 
 println!("Connected to: {} v{}", 
 init_result.server_info.name, 
 init_result.server_info.version
 );
 
 // Use client...
 Ok(())
}
```

# HTTP Client
- **File**: `http_client.rs`
- **Features**: `http`
- **Description**: MCP client using HTTP transport

**Key Code Pattern:**
```rust
use mcp_protocol_sdk::{
 client::{McpClient, ClientSession},
 transport::http::HttpClientTransport,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 let client = McpClient::new("http-client".to_string(), "1.0.0".to_string());
 let session = ClientSession::new(client);
 
 let transport = HttpClientTransport::new("http://localhost:3000/mcp".to_string()).await?;
 let init_result = session.connect(transport).await?;
 
 let client = session.client();
 let client_guard = client.lock().await;
 
 // List available tools
 let tools = client_guard.list_tools().await?;
 println!("Available tools: {:?}", tools.tools.len());
 
 Ok(())
}
```

# complete HTTP Client
- **File**: `complete_http_client.rs`
- **Features**: `http`, `tracing-subscriber`, `chrono`, `fastrand`
- **Description**: complete HTTP client with full feature set including retry logic and health checks

# WebSocket Client
- **File**: `websocket_client.rs`
- **Features**: `websocket`
- **Description**: MCP client using WebSocket transport

**Key Code Pattern:**
```rust
use mcp_protocol_sdk::{
 client::{McpClient, ClientSession},
 transport::websocket::WebSocketClientTransport,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 let client = McpClient::new("ws-client".to_string(), "1.0.0".to_string());
 let session = ClientSession::new(client);
 
 let transport = WebSocketClientTransport::new("ws://localhost:8080").await?;
 let init_result = session.connect(transport).await?;
 
 // Real-time communication ready
 Ok(())
}
```

# Conservative HTTP Demo
- **File**: `conservative_http_demo.rs`
- **Features**: `http`, `tracing-subscriber`
- **Description**: Conservative HTTP client implementation with production-ready patterns

# Running Examples

```bash
# Run basic client example
cargo run --example basic_client --features stdio

# Run HTTP client example
cargo run --example http_client --features http

# Run complete HTTP client example
cargo run --example complete_http_client --features "http,tracing-subscriber,chrono,fastrand"

# Run WebSocket client example
cargo run --example websocket_client --features websocket

# Run conservative HTTP demo
cargo run --example conservative_http_demo --features "http,tracing-subscriber"
```

# Common Client Patterns

# Tool Calling
```rust
let client = session.client();
let client_guard = client.lock().await;

let mut args = HashMap::new();
args.insert("message".to_string(), json!("Hello World"));

let result = client_guard.call_tool("echo".to_string(), Some(args)).await?;
println!("Tool result: {:?}", result);
```

# Resource Access
```rust
// List available resources
let resources = client_guard.list_resources(None).await?;
for resource in &resources.resources {
 println!("Resource: {} ({})", resource.name, resource.uri);
}

// Read a specific resource
let content = client_guard.read_resource("file://example.txt".to_string()).await?;
println!("Resource content: {:?}", content);
```

# Prompt Handling
```rust
// List available prompts
let prompts = client_guard.list_prompts(None).await?;
for prompt in &prompts.prompts {
 println!("Prompt: {} - {}", prompt.name, prompt.description.as_deref().unwrap_or("No description"));
}
```

For more details on building MCP clients, see the [Implementation Guide](../../docs/implementation-guide.md) and [Getting Started Guide](../../docs/getting-started.md).

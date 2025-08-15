# Prism MCP SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/prism-mcp-rs.svg)](https://crates.io/crates/prism-mcp-rs)
[![Downloads](https://img.shields.io/crates/d/prism-mcp-rs.svg)](https://crates.io/crates/prism-mcp-rs)
[![Documentation](https://docs.rs/prism-mcp-rs/badge.svg)](https://docs.rs/prism-mcp-rs)
[![CI](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml)
[![Security](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security.yml/badge.svg)](https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/prismworks-ai/prism-mcp-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/prismworks-ai/prism-mcp-rs)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://blog.rust-lang.org/2025/01/09/Rust-1.85.0.html)
[![deps.rs](https://deps.rs/repo/github/prismworks-ai/prism-mcp-rs/status.svg)](https://deps.rs/repo/github/prismworks-ai/prism-mcp-rs)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/prism-mcp-rs.svg?label=total%20downloads)](https://crates.io/crates/prism-mcp-rs)
[![API Stability](https://img.shields.io/badge/API-v0.1.0-orange.svg)](https://github.com/prismworks-ai/prism-mcp-rs/blob/main/CHANGELOG.md)

[![Contributors](https://img.shields.io/github/contributors/prismworks-ai/prism-mcp-rs.svg)](https://github.com/prismworks-ai/prism-mcp-rs/graphs/contributors)
[![GitHub last commit](https://img.shields.io/github/last-commit/prismworks-ai/prism-mcp-rs.svg)](https://github.com/prismworks-ai/prism-mcp-rs/commits/main)
[![GitHub release](https://img.shields.io/github/release/prismworks-ai/prism-mcp-rs.svg)](https://github.com/prismworks-ai/prism-mcp-rs/releases)
[![Discord](https://img.shields.io/discord/123456789?logo=discord&label=Discord)](https://discord.gg/prismworks)
[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-Support-pink.svg)](https://github.com/sponsors/prismworks-ai)

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

## Overview

The Prism MCP SDK provides a Rust implementation of the Model Context Protocol with comprehensive feature support. Build MCP servers and clients with type safety, high performance, and excellent developer experience.

## Documentation

- [API Documentation](https://docs.rs/prism-mcp-rs) - Complete API reference
- [Architecture Overview](ARCHITECTURE.md) - System design and technical architecture
- [Development Guide](DEVELOPMENT.md) - Build system, workflows, and contribution process
- [Contributing Guide](CONTRIBUTING.md) - Code of conduct and submission guidelines
- [Migration Guide](MIGRATION.md) - Version migration instructions
- [Changelog](CHANGELOG.md) - Release history and changes

## Key Features

- **Complete MCP Protocol Support** - Tools, Resources, Prompts, Completions
- **Multiple Transports** - STDIO, HTTP/SSE, WebSocket
- **Type Safety** - Leverages Rust's type system
- **High Performance** - Async/await with zero-copy where possible
- **Developer Friendly** - Intuitive APIs with helpful error messages

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
prism-mcp-rs = "0.1.0"
```

With specific features:

```toml
[dependencies]
prism-mcp-rs = {
    version = "0.1.0",
    features = ["websocket", "http"]
}
```

### Available features

| Feature | Description |
|---------|-------------|
| `plugin` | Plugin system support |
| `websocket` | WebSocket transport |
| `http` | HTTP/1.1 and HTTP/2 transport |
| `stdio` | Standard I/O transport (default) |
| `auth` | Authentication mechanisms |
| `tls` | TLS/SSL support |
| `full` | All features enabled |
| `minimal` | Core functionality only |

## Quick Start

### Creating a Simple MCP Server

```rust
use prism_mcp_rs::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

// Define a tool handler
struct EchoHandler;

#[async_trait]
impl ToolHandler for EchoHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let message = arguments
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello, World!");

        // Use the new convenience method
        Ok(ToolResult::text(message))
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create server
    let server = McpServer::new("echo-server".to_string(), "1.0.0".to_string());
    
    // Add a tool
    server.add_tool(
        "echo".to_string(),
        Some("Echo a message back".to_string()),
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The message to echo"
                }
            },
            "required": ["message"]
        }),
        EchoHandler,
    ).await?;
    
    // Run with STDIO transport (convenience method)
    server.run_with_stdio().await
}
```

### Creating a Tool with Error Handling

```rust
use prism_mcp_rs::prelude::*;

struct CalculatorHandler;

#[async_trait]
impl ToolHandler for CalculatorHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let a = arguments.get("a")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::validation("Missing parameter 'a'"))?;
        
        let b = arguments.get("b")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::validation("Missing parameter 'b'"))?;
        
        let operation = arguments.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("add");
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Ok(ToolResult::error("Cannot divide by zero"));
                }
                a / b
            }
            _ => return Ok(ToolResult::error(format!("Unknown operation: {}", operation)))
        };
        
        Ok(ToolResult::text(result.to_string()))
    }
}
```

### Resource Handler Example

```rust
use prism_mcp_rs::prelude::*;

struct FileResourceHandler;

#[async_trait]
impl ResourceHandler for FileResourceHandler {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        // Extract file path from URI
        let path = uri.strip_prefix("file://").ok_or_else(|| {
            McpError::validation("Invalid file URI")
        })?;
        
        // Read file content
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| McpError::internal(format!("Failed to read file: {}", e)))?;
        
        Ok(vec![ResourceContents::Text {
            uri: uri.to_string(),
            mime_type: Some("text/plain".to_string()),
            text: content,
            meta: None,
        }])
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServer::new("file-server".to_string(), "1.0.0".to_string());
    
    // Add a resource
    server.add_resource(
        "readme".to_string(),
        "file://README.md".to_string(),
        FileResourceHandler,
    ).await?;
    
    server.run_with_stdio().await
}
```

### Prompt Handler Example

```rust
use prism_mcp_rs::prelude::*;

struct CodeReviewPromptHandler;

#[async_trait]
impl PromptHandler for CodeReviewPromptHandler {
    async fn get(&self, arguments: HashMap<String, Value>) -> McpResult<PromptResult> {
        let language = arguments.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("rust");
        
        let code = arguments.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::validation("Missing 'code' parameter"))?;
        
        let prompt_text = format!(
            "Please review the following {} code:\n\n```{}\n{}\n```\n\nProvide feedback on:\
            1. Code quality\n2. Potential bugs\n3. Performance improvements\n4. Best practices",
            language, language, code
        );
        
        Ok(PromptResult {
            messages: vec![PromptMessage {
                role: Role::User,
                content: ContentBlock::text(prompt_text),
            }],
            meta: None,
        })
    }
}
```

### Running with Different Transports

```rust
use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> McpResult<()> {
    let server = McpServer::new("my-server".to_string(), "1.0.0".to_string());
    
    // Add your tools, resources, and prompts here
    
    // Choose transport based on configuration
    match std::env::var("TRANSPORT").as_deref() {
        Ok("http") => {
            #[cfg(feature = "http")]
            server.run_with_http("127.0.0.1:3000").await?
        }
        Ok("websocket") => {
            #[cfg(feature = "websocket")]
            server.run_with_websocket("127.0.0.1:8080").await?
        }
        _ => {
            // Default to STDIO
            server.run_with_stdio().await?
        }
    }
    
    Ok(())
}
```

### Client Example

```rust
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::client::ClientSession;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Connect to server
    #[cfg(feature = "stdio")]
    let transport = StdioClientTransport::new_with_command("path/to/server");
    
    let mut session = ClientSession::new(transport);
    
    // Initialize connection
    session.initialize(
        "my-client".to_string(),
        "1.0.0".to_string(),
    ).await?;
    
    // List available tools
    let tools = session.list_tools().await?;
    for tool in tools.tools {
        println!("Tool: {} - {:?}", tool.name, tool.description);
    }
    
    // Call a tool
    let result = session.call_tool(
        "echo",
        Some(HashMap::from([
            ("message".to_string(), json!("Hello from client!")),
        ])),
    ).await?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}
```

## Advanced Features

### Custom Transport Implementation

```rust
use prism_mcp_rs::transport::traits::ServerTransport;
use async_trait::async_trait;

struct CustomTransport {
    // Your transport implementation
}

#[async_trait]
impl ServerTransport for CustomTransport {
    async fn start(&mut self) -> McpResult<()> {
        // Start listening for connections
        Ok(())
    }
    
    async fn stop(&mut self) -> McpResult<()> {
        // Clean up resources
        Ok(())
    }
    
    async fn send_notification(&mut self, notification: JsonRpcNotification) -> McpResult<()> {
        // Send notification to client
        Ok(())
    }
    
    fn set_request_handler(&mut self, handler: ServerRequestHandler) {
        // Set the handler for incoming requests
    }
}
```

### Tool with Structured Output

```rust
struct DataAnalyzer;

#[async_trait]
impl ToolHandler for DataAnalyzer {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let data = arguments.get("data")
            .ok_or_else(|| McpError::validation("Missing 'data' parameter"))?;
        
        // Perform analysis
        let mean = calculate_mean(data);
        let median = calculate_median(data);
        let std_dev = calculate_std_dev(data);
        
        // Return both text and structured content
        Ok(ToolResult {
            content: vec![ContentBlock::text(format!(
                "Mean: {:.2}, Median: {:.2}, Std Dev: {:.2}",
                mean, median, std_dev
            ))],
            is_error: Some(false),
            structured_content: Some(json!({
                "statistics": {
                    "mean": mean,
                    "median": median,
                    "standard_deviation": std_dev,
                }
            })),
            meta: None,
        })
    }
}
```

## Testing

```bash
# Run all tests
cargo test

# Test with all features
cargo test --all-features

# Test specific module
cargo test server::

# Run integration tests
cargo test --test integration
```

## CI/CD and Quality Metrics

The project includes comprehensive CI/CD with automatic reporting:

### 📊 Automatic Reports

- **Coverage reports**: Code coverage metrics with trends
- **Benchmark reports**: Performance metrics for all components
- **Format**: Markdown, viewable directly on GitHub
- **Location**: `reports/` directory

### 🚀 For Contributors

**No tokens needed!** All CI features work automatically:
- Testing, linting, and validation
- Coverage and benchmark generation
- PR checks and status updates

### 📦 For Maintainers

Only publishing to crates.io requires the `CRATES_IO_TOKEN` secret.

```bash
# Generate reports locally
make reports              # Both coverage and benchmarks
make report-coverage      # Coverage only
make report-bench         # Benchmarks only

# Or use Act to run GitHub Actions locally
act -j coverage    # Run coverage job from CI workflow
act push          # Run full CI pipeline
```

## Performance Characteristics

- **Message parsing**: <0.1ms per message
- **Tool execution**: <1ms overhead
- **WebSocket round-trip**: <5ms
- **HTTP/2 multiplexing**: 100+ concurrent streams
- **Memory usage**: Minimal allocations with zero-copy where possible

## Examples

| Example | Description |
|---------|-------------|
| [Server Examples](examples/server/) | Various server implementations |
| [Client Examples](examples/client/) | Client usage patterns |
| [Advanced Features](examples/advanced_features_showcase.rs) | Advanced MCP features |
| [Error Handling](examples/production_error_handling_demo.rs) | Production error patterns |
| [Performance Tests](examples/performance_benchmarks.rs) | Benchmarking utilities |

## Common Issues and Solutions

### Issue: Compilation Errors with ToolHandler

**Solution**: The `ToolHandler` trait expects `HashMap<String, Value>` as arguments:

```rust
#[async_trait]
impl ToolHandler for MyHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // NOT: async fn call(&self, arguments: Value) -> ...
        // Implementation
    }
}
```

### Issue: Missing ContentBlock convenience methods

**Solution**: Update to latest version (0.1.0+) which includes:

```rust
// These convenience methods are now available:
ContentBlock::text("message")
ToolResult::text("result")
ToolResult::error("error message")
```

### Issue: Type confusion with ToolResult/CallToolResult

**Solution**: Use `ToolResult` consistently. It's a type alias for `CallToolResult`:

```rust
use prism_mcp_rs::prelude::*;
// Use ToolResult in your code
Ok(ToolResult::text("success"))
```

## Development Tools

### Related Tools

For enhanced development experience, consider using the companion developer tools available in the [mcp-rs-dev](https://github.com/prismworks-ai/mcp-rs-dev) repository.

## Support

- [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- [GitHub Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions)
- [Discord Community](https://discord.gg/prismworks)

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

Built by [Prismworks AI](https://prismworks.ai)

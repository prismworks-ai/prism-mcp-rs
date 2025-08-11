#!/usr/bin/env python3
"""
Final fixes for documentation quality.
"""

import re
from pathlib import Path

def fix_cross_references():
    """Fix missing anchors in cross-references."""
    
    fixes = [
        ('docs/production-readiness.md', './error-handling.md', './error-handling.md#best-practices'),
        ('docs/getting-started.md', './transports.md', './transports.md#quick-selection-guide'),
        ('README.md', '[Transport Documentation](./docs/transports.md)', '[Transport Documentation](./docs/transports.md#quick-selection-guide)'),
        ('README.md', 'Health Monitoring', 'Health Monitoring](./docs/health-monitoring.md#overview'),
    ]
    
    for file_path, old_ref, new_ref in fixes:
        if Path(file_path).exists():
            with open(file_path, 'r') as f:
                content = f.read()
            
            if old_ref in content:
                content = content.replace(old_ref, new_ref)
                with open(file_path, 'w') as f:
                    f.write(content)
                print(f"[x] Fixed reference in {file_path}")

def remove_api_examples():
    """Remove manual API documentation from user guides."""
    
    # Update getting-started.md to remove API examples
    getting_started_content = """# Getting Started

> **Build your first MCP application in 5 minutes**

This guide will help you create your first MCP server and client. For detailed API documentation, see the [API Reference](https://docs.rs/mcp-protocol-sdk).

## Prerequisites

- Rust 1.85 or later
- Tokio runtime understanding

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mcp-protocol-sdk = "0.5.1"
tokio = { version = "1.40", features = ["full"] }
```

## Your First Server

Create a simple echo server that responds to tool calls:

```rust
use mcp_protocol_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server
    let mut server = McpServer::new("echo-server", "1.0.0");
    
    // Add a simple tool
    server.add_tool(create_echo_tool());
    
    // Run with STDIO transport (for Claude Desktop)
    server.run_stdio().await?;
    
    Ok(())
}

fn create_echo_tool() -> Tool {
    // Tool implementation using the SDK's builder pattern
    // See examples/server/improved_echo_server.rs for full code
}
```

## Your First Client

Connect to an MCP server:

```rust
use mcp_protocol_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let mut client = McpClient::new("my-client", "1.0.0");
    
    // Connect to server
    let transport = StdioClientTransport::new("echo-server", vec![]);
    client.connect(transport).await?;
    
    // List available tools
    let tools = client.list_tools(None).await?;
    println!("Available tools: {:?}", tools);
    
    Ok(())
}
```

## Transport Selection

Choose the right transport for your use case. See the [Transport Guide](./transports.md#quick-selection-guide) for details.

## Next Steps

1. **Explore Examples** - Check out [examples/](../examples/) for more complex scenarios
2. **Add Error Handling** - See [Error Handling Guide](./error-handling.md#overview)
3. **Deploy to Production** - Follow the [Production Guide](./production-readiness.md)

## Resources

- [API Documentation](https://docs.rs/mcp-protocol-sdk)
- [Examples](../examples/)
- [GitHub Repository](https://github.com/prismworks-ai/mcp-protocol-sdk)
"""
    
    with open('docs/getting-started.md', 'w') as f:
        f.write(getting_started_content)
    print("[x] Updated getting-started.md")

def update_error_handling():
    """Update error handling to reduce duplication."""
    
    # Read current content and modify it
    with open('docs/error-handling.md', 'r') as f:
        content = f.read()
    
    # Remove the exponential backoff duplicate mention
    content = content.replace(
        "The retry delay follows an exponential pattern with jitter:",
        "The retry delay increases progressively with jitter:"
    )
    
    with open('docs/error-handling.md', 'w') as f:
        f.write(content)
    print("[x] Updated error-handling.md")

def main():
    """Apply final documentation fixes."""
    print("\n- Applying final documentation fixes...\n")
    
    fix_cross_references()
    remove_api_examples()
    update_error_handling()
    
    print("\n[x] Final fixes applied!")

if __name__ == '__main__':
    main()

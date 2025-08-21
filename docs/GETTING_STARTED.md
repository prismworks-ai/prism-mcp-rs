# Getting Started with prism-mcp-rs

## Overview

This guide walks you through the complete process of building an MCP server with prism-mcp-rs and integrating it with AI tools like Claude Desktop, Cursor, VS Code, and Windsurf. By the end, you'll have a working MCP server that your AI assistant can use.

## Prerequisites

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs/)
- **AI Tool**: At least one of:
  - [Claude Desktop](https://claude.ai/download)
  - [Cursor](https://cursor.sh/)
  - [VS Code](https://code.visualstudio.com/) with GitHub Copilot
  - [Windsurf](https://codeium.com/windsurf)
  - [Claude Code](https://docs.anthropic.com/en/docs/claude-code)

## Quick Start (5 Minutes)

Follow this path for the fastest setup:

### 1. Create Your First Server

```bash
# Create new project
cargo new my-mcp-server
cd my-mcp-server

# Add prism-mcp-rs dependency
cargo add prism-mcp-rs tokio --features full
```

Add to `src/main.rs`:

```rust
use prism_mcp_rs::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server
    let server = MCPServer::new("my-first-server", "1.0.0");
    
    // Add a simple tool
    server.add_tool(
        "hello",
        "Says hello to someone",
        |params| async move {
            let name = params.get("name").unwrap_or("World");
            Ok(format!("Hello, {}!", name))
        }
    );
    
    // Start server with STDIO transport
    server.start_stdio().await?;
    Ok(())
}
```

### 2. Build Your Server

```bash
# Build release version
cargo build --release

# Your binary is now at:
# ./target/release/my-mcp-server
```

### 3. Connect to AI Tool

Choose your AI tool and follow the quick setup:

#### Claude Desktop (Easiest)

1. **Open Claude Desktop Settings:**
   - Click Claude menu → Settings
   - Go to Developer section

2. **Edit Config:**
   - Click "Edit Config" in MCP Servers section
   - Add this configuration:

```json
{
  "mcpServers": {
    "my-first-server": {
      "command": "/full/path/to/your/project/target/release/my-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

3. **Restart Claude Desktop**

4. **Test:** Look for hammer (🔨) icon in chat input

#### Cursor (Popular Choice)

1. **Open Settings:** Cursor → Settings → Extensions → MCP

2. **Add Server:**
   - Click "Add new global MCP server"
   - Name: `my-first-server`
   - Command: `/full/path/to/your/project/target/release/my-mcp-server`

3. **Restart Cursor**

4. **Test:** Check Settings → MCP for green active status

### 4. Test Your Server

In your AI tool, try asking:
> "Use the hello tool to say hello to Alice"

You should see your server respond with "Hello, Alice!"

🎉 **Congratulations!** You've successfully created and connected your first MCP server.

## Complete End-to-End Workflow

For production deployment, follow this comprehensive workflow:

### Phase 1: Development

1. **Create project structure**
2. **Implement server with prism-mcp-rs**
3. **Add tools, resources, and prompts**
4. **Test locally with debug builds**

### Phase 2: Build & Package

1. **Create optimized release build**
2. **Cross-compile for target platforms**
3. **Package binaries for distribution**
4. **Create installation scripts**

### Phase 3: AI Tool Integration

1. **Configure AI tool of choice**
2. **Test integration thoroughly**
3. **Verify all tools work correctly**
4. **Document configuration for users**

### Phase 4: Distribution

1. **Create GitHub releases**
2. **Package as desktop extensions (.dxt)**
3. **Submit to package managers**
4. **Provide installation documentation**

### Phase 5: Production Deployment

1. **Deploy to production servers**
2. **Set up monitoring and logging**
3. **Configure security and permissions**
4. **Plan maintenance and updates**

## Real-World Examples

### Example 1: File System Server

```rust
use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = MCPServer::new("filesystem-server", "1.0.0");
    
    // Read file tool
    server.add_tool(
        "read_file",
        "Read contents of a file",
        |params| async move {
            let path = params.get("path")
                .ok_or("Missing 'path' parameter")?;
            
            match tokio::fs::read_to_string(path).await {
                Ok(content) => Ok(content),
                Err(e) => Err(format!("Failed to read file: {}", e).into())
            }
        }
    );
    
    // List directory tool
    server.add_tool(
        "list_directory",
        "List contents of a directory",
        |params| async move {
            let path = params.get("path")
                .ok_or("Missing 'path' parameter")?;
            
            let mut entries = tokio::fs::read_dir(path).await?;
            let mut files = Vec::new();
            
            while let Some(entry) = entries.next_entry().await? {
                files.push(entry.file_name().to_string_lossy().to_string());
            }
            
            Ok(files.join(", "))
        }
    );
    
    server.start_stdio().await?;
    Ok(())
}
```

### Example 2: API Client Server

```rust
use prism_mcp_rs::prelude::*;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut server = MCPServer::new("api-client", "1.0.0");
    
    // HTTP GET tool
    server.add_tool(
        "http_get",
        "Make HTTP GET request",
        move |params| {
            let client = client.clone();
            async move {
                let url = params.get("url")
                    .ok_or("Missing URL parameter")?;
                
                let response = client.get(url).send().await?;
                let text = response.text().await?;
                
                Ok(text)
            }
        }
    );
    
    server.start_stdio().await?;
    Ok(())
}
```

## Next Steps

After completing the quick start:

1. **Explore advanced features:**
   - [AI Tool Integration Guide](./AI_TOOL_INTEGRATION.md) - Complete integration documentation
   - [Deployment Guide](./DEPLOYMENT_GUIDE.md) - Production deployment strategies
   - [Architecture Guide](./ARCHITECTURE.md) - Understanding prism-mcp-rs design

2. **Build more complex servers:**
   - Database integrations
   - Web API clients
   - File system operations
   - Custom business logic

3. **Distribute your server:**
   - Create GitHub releases
   - Package as desktop extensions
   - Submit to package managers
   - Build user community

4. **Join the ecosystem:**
   - Contribute to prism-mcp-rs
   - Share your servers
   - Help improve documentation
   - Report bugs and suggest features

## Troubleshooting

If you encounter issues:

1. **Check the [Troubleshooting Guide](./TROUBLESHOOTING.md)** for common solutions
2. **Validate your configuration** with the provided tools
3. **Test your server independently** before integration
4. **Enable debug logging** to understand what's happening
5. **Seek help** from the community or create detailed bug reports

## Success Path Summary

The complete journey from development to production:

```
Development → Build → Integration → Distribution → Production
     ↓           ↓         ↓            ↓           ↓
  Code with   Release   Configure   Package &   Deploy &
 prism-mcp-rs  Binary    AI Tools   Distribute  Monitor
```

This guide provides the foundation for building powerful MCP servers that extend AI tools with custom capabilities. The end-to-end workflow ensures your servers are reliable, secure, and easy to use.

Happy building! 🚀
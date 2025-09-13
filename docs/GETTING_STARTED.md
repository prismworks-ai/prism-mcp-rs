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
use std::collections::HashMap;

#[derive(Clone)]
struct HelloToolHandler;

#[async_trait]
impl ToolHandler for HelloToolHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let name = arguments.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");
            
        Ok(ToolResult {
            content: vec![ContentBlock::text(format!("Hello, {}!", name))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Create server
    let mut server = McpServer::new(
        "my-first-server".to_string(), 
        "1.0.0".to_string()
    );
    
    // Add a simple hello tool
    let tool = Tool::new(
        "hello".to_string(),
        Some("Says hello to someone".to_string()),
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name to greet"
                }
            }
        }),
        HelloToolHandler,
    );
    
    server.add_tool(tool);
    
    // Start server
    server.start().await
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

## Tool Creation Methods

Prism MCP SDK provides the `add_tool()` method for adding tools to your server:

### `add_tool()` - Complete Tool Definition

The `add_tool()` method provides full control for defining tools with proper schemas:

```rust
#[derive(Clone)]
struct MyToolHandler;

#[async_trait]
impl ToolHandler for MyToolHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Extract parameters from arguments HashMap
        let param = arguments.get("param_name")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        
        // Return ToolResult
        Ok(ToolResult {
            content: vec![ContentBlock::text(format!("Result: {}", param))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

// Add tool to server using async method
server.add_tool(
    "tool_name",
    Some("Tool description"),
    json!({
        "type": "object",
        "properties": {
            "param_name": {"type": "string", "description": "Parameter description"}
        },
        "required": ["param_name"]
    }),
    MyToolHandler,
).await?;
```

**Key features:**
- **Complete control** - Define input schemas, descriptions, and behavior
- **Type-safe parameter extraction** - Use `.as_str()`, `.as_i64()`, etc. for parameters
- **Structured output** - Return `ToolResult` with content and optional structured data
- **Async support** - Full async/await support for complex operations

```rust
use prism_mcp_rs::prelude::*;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Clone)]
struct MyToolHandler;

#[async_trait]
impl ToolHandler for MyToolHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        // Your async tool logic here
        Ok(ToolResult {
            content: vec![ContentBlock::text("result".to_string())],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }
}

// Custom JSON schema
let schema = json!({
    "type": "object",
    "properties": {
        "param": {"type": "string", "description": "Parameter description"}
    },
    "required": ["param"]
});

let tool = Tool::new(
    "tool_name".to_string(),
    Some("Tool description".to_string()),
    schema,
    MyToolHandler
);

server.add_tool(tool);
```

**When to use:**
- **Complex async operations** that require proper async/await
- **Custom JSON schemas** with validation rules
- **Stateful tools** that need to maintain state between calls
- **Advanced error handling** with custom error types

## Real-World Examples

### Example 1: File System Server

```rust
use prism_mcp_rs::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
struct ReadFileHandler;

#[async_trait]
impl ToolHandler for ReadFileHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let path = arguments.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing 'path' parameter"))?;
        
        match std::fs::read_to_string(path) {
            Ok(content) => Ok(ToolResult {
                content: vec![ContentBlock::text(content)],
                is_error: Some(false),
                meta: None,
                structured_content: None,
            }),
            Err(e) => Err(McpError::internal_error(format!("Failed to read file: {}", e)))
        }
    }
}

#[derive(Clone)]
struct ListDirHandler;

#[async_trait]
impl ToolHandler for ListDirHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let path = arguments.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing 'path' parameter"))?;
        
        match std::fs::read_dir(path) {
            Ok(entries) => {
                let files: Result<Vec<_>, _> = entries
                    .map(|entry| entry.map(|e| e.file_name().to_string_lossy().to_string()))
                    .collect();
                
                match files {
                    Ok(file_list) => Ok(ToolResult {
                        content: vec![ContentBlock::text(file_list.join(", "))],
                        is_error: Some(false),
                        meta: None,
                        structured_content: None,
                    }),
                    Err(e) => Err(McpError::internal_error(format!("Failed to read directory: {}", e)))
                }
            },
            Err(e) => Err(McpError::internal_error(format!("Failed to read directory: {}", e)))
        }
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let mut server = McpServer::new(
        "filesystem-server".to_string(), 
        "1.0.0".to_string()
    );
    
    // Read file tool using closure helper
    let read_file_tool = closure_tool(
        "read_file".to_string(),
        Some("Read contents of a file".to_string()),
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "File path to read"}
            },
            "required": ["path"]
        }),
        |args| {
            let path = args.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::invalid_request("Missing 'path' parameter"))?;
            
            match std::fs::read_to_string(path) {
                Ok(content) => Ok(vec![ContentBlock::text(content)]),
                Err(e) => Err(McpError::internal_error(format!("Failed to read file: {}", e)))
            }
        }
    );
    
    // List directory tool
    let list_dir_tool = closure_tool(
        "list_directory".to_string(),
        Some("List contents of a directory".to_string()),
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Directory path to list"}
            },
            "required": ["path"]
        }),
        |args| {
            let path = args.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::invalid_request("Missing 'path' parameter"))?;
            
            match std::fs::read_dir(path) {
                Ok(entries) => {
                    let files: Result<Vec<_>, _> = entries
                        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().to_string()))
                        .collect();
                    
                    match files {
                        Ok(file_list) => Ok(vec![ContentBlock::text(file_list.join(", "))]),
                        Err(e) => Err(McpError::internal_error(format!("Failed to read directory: {}", e)))
                    }
                },
                Err(e) => Err(McpError::internal_error(format!("Failed to read directory: {}", e)))
            }
        }
    );
    
    server.add_tool(
        "read_file",
        Some("Read contents of a file"),
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "File path to read"}
            },
            "required": ["path"]
        }),
        ReadFileHandler,
    ).await?;
    
    server.add_tool(
        "list_directory", 
        Some("List contents of a directory"),
        json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Directory path to list"}
            },
            "required": ["path"]
        }),
        ListDirHandler,
    ).await?;
    
    let transport = StdioServerTransport::new();
    server.start(transport).await
}
```

### Example 2: API Client Server (Async Operations)

For operations that require async/await, use the full `ToolHandler` trait implementation:

```rust
use prism_mcp_rs::prelude::*;
use reqwest::Client;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Clone)]
struct HttpTool {
    client: Client,
}

#[async_trait]
impl ToolHandler for HttpTool {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let url = arguments.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_request("Missing URL parameter"))?;
        
        match self.client.get(url).send().await {
            Ok(response) => {
                let text = response.text().await
                    .map_err(|e| McpError::internal_error(format!("HTTP response error: {}", e)))?;
                
                Ok(ToolResult {
                    content: vec![ContentBlock::text(text)],
                    is_error: Some(false),
                    meta: None,
                    structured_content: None,
                })
            }
            Err(e) => Err(McpError::internal_error(format!("HTTP request failed: {}", e)))
        }
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    let mut server = McpServer::new(
        "api-client".to_string(), 
        "1.0.0".to_string()
    );
    
    // For async operations, create Tool with ToolHandler
    let schema = json!({
        "type": "object",
        "properties": {
            "url": {"type": "string", "description": "URL to fetch"}
        },
        "required": ["url"]
    });
    
    let http_tool = Tool::new(
        "http_get".to_string(),
        Some("Make HTTP GET request".to_string()),
        schema,
        HttpTool { client: Client::new() }
    );
    
    server.add_tool(http_tool);
    server.start().await
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
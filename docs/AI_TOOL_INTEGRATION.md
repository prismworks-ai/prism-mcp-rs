# AI Tool Integration Guide

## Overview

This guide covers how to connect your Rust MCP servers built with prism-mcp-rs to popular AI tools including Claude Desktop, Cursor, VS Code, Windsurf, and Claude Code. After building your server with `cargo build --release`, follow these platform-specific instructions to integrate with your AI assistant.

## Prerequisites

- **Completed server build**: Your Rust MCP server binary must be built and accessible
- **AI tool installed**: At least one of the supported AI tools must be installed
- **Node.js (optional)**: Some configurations may require Node.js for `npx` commands

## Quick Start Matrix

| AI Tool | Config Location | Format | Scope Options |
|---------|----------------|--------|---------------|
| **Claude Desktop** | `claude_desktop_config.json` | JSON | Global only |
| **Cursor** | `mcp.json` | JSON | Global, Project |
| **VS Code** | `mcp.json` or `settings.json` | JSON | Workspace, User |
| **Windsurf** | `mcp_config.json` | JSON | Global |
| **Claude Code** | `.mcp.json` or CLI | JSON/Command | Local, Project, User |

## Claude Desktop Integration

### Configuration File Location

**macOS:**
```bash
~/Library/Application Support/Claude/claude_desktop_config.json
```

**Windows:**
```bash
%APPDATA%\Claude\claude_desktop_config.json
```

### Configuration Format

```json
{
  "mcpServers": {
    "my-rust-server": {
      "command": "/path/to/your/rust/binary",
      "args": [],
      "env": {
        "RUST_LOG": "info",
        "API_KEY": "your-api-key-here"
      }
    }
  }
}
```

### Setup Steps

1. **Build your server:**
   ```bash
   cd your-server-project
   cargo build --release
   ```

2. **Locate the binary:**
   ```bash
   # Binary will be at:
   ./target/release/your-server-name
   ```

3. **Open Claude Desktop settings:**
   - Click Claude menu in menu bar → Settings
   - Navigate to Developer section
   - Click "Edit Config" in the MCP Servers section

4. **Add your server configuration:**
   ```json
   {
     "mcpServers": {
       "my-filesystem-server": {
         "command": "/Users/username/projects/my-server/target/release/filesystem-server",
         "args": ["/Users/username/Documents", "/Users/username/Downloads"],
         "env": {}
       }
     }
   }
   ```

5. **Restart Claude Desktop**

6. **Verify connection:**
   - Look for hammer (🔨) icon in chat input
   - Click to see available tools

## Cursor Integration

### Global Configuration

**Location:** `~/.cursor/mcp.json` (macOS/Linux) or `%USERPROFILE%\.cursor\mcp.json` (Windows)

```json
{
  "mcpServers": {
    "rust-server": {
      "command": "/path/to/target/release/your-server",
      "args": ["--port", "8080"],
      "env": {
        "SERVER_CONFIG": "/path/to/config.json"
      }
    }
  }
}
```

### Project-Specific Configuration

**Location:** `.cursor/mcp.json` in your project root

```json
{
  "mcpServers": {
    "project-tools": {
      "command": "./target/release/project-server",
      "args": ["--workspace", "."],
      "env": {
        "PROJECT_ROOT": "${workspaceFolder}"
      }
    }
  }
}
```

### Setup Steps

1. **Option A - UI Configuration:**
   - Open Cursor Settings
   - Navigate to Extensions → MCP
   - Click "Add new global MCP server"
   - Set command type and binary path

2. **Option B - Manual Configuration:**
   - Create/edit `~/.cursor/mcp.json`
   - Add server configuration
   - Restart Cursor

3. **Verify in Cursor:**
   - Check Settings → MCP for green active status
   - Look for tools in chat interface

## VS Code Integration

### Workspace Configuration

**Location:** `.vscode/mcp.json` in your workspace

```json
{
  "servers": {
    "RustMCPServer": {
      "type": "stdio",
      "command": "/path/to/target/release/your-server",
      "args": ["--mode", "vscode"],
      "env": {
        "VSCODE_WORKSPACE": "${workspaceFolder}"
      }
    }
  }
}
```

### User Settings Configuration

**Location:** User settings.json

```json
{
  "mcp.servers": {
    "global-rust-server": {
      "type": "stdio",
      "command": "/usr/local/bin/my-rust-server",
      "args": ["--global"],
      "env": {}
    }
  }
}
```

### Setup Steps

1. **Install GitHub Copilot extension** (required for MCP support)

2. **Create workspace MCP config:**
   ```bash
   mkdir -p .vscode
   touch .vscode/mcp.json
   ```

3. **Add server configuration** to the JSON file

4. **Enable MCP support:**
   - Open VS Code settings
   - Search for `chat.mcp.enabled`
   - Ensure it's set to `true`

5. **Restart VS Code**

6. **Verify:**
   - Open Command Palette
   - Run "MCP: List Servers"
   - Check for your server in the list

## Windsurf Integration

### Configuration File Location

```bash
~/.codeium/windsurf/mcp_config.json
```

### Configuration Format

```json
{
  "mcpServers": {
    "windsurf-rust-server": {
      "command": "/path/to/target/release/your-server",
      "args": ["--windsurf-mode"],
      "env": {
        "WINDSURF_INTEGRATION": "true",
        "LOG_LEVEL": "debug"
      }
    }
  }
}
```

### Setup Steps

1. **Open Windsurf settings:**
   - Navigate to Settings → Advanced Settings
   - Or use Command Palette: "Open Windsurf Settings Page"

2. **Configure MCP server:**
   - Scroll to Cascade section
   - Click "Add Server" → "Add custom server +"
   - Add configuration

3. **Alternative - Direct file edit:**
   ```bash
   # Edit the config file directly
   code ~/.codeium/windsurf/mcp_config.json
   ```

4. **Restart Windsurf**

5. **Verify in Cascade:**
   - Open Cascade assistant
   - Click hammer (🔨) icon
   - Click "Configure" to see active servers

## Claude Code Integration

### Project Configuration

**Location:** `.mcp.json` in project root

```json
{
  "mcpServers": {
    "local-rust-server": {
      "command": "./target/release/server",
      "args": ["--project-mode"],
      "env": {
        "PROJECT_PATH": "."
      }
    }
  }
}
```

### CLI-Based Setup

```bash
# Add a local-scoped server (default)
claude mcp add my-rust-server /path/to/target/release/server

# Add with arguments
claude mcp add my-rust-server --scope local /path/to/server -- --arg1 value1

# Add project-scoped server
claude mcp add shared-tools --scope project ./target/release/shared-server

# Add user-scoped server
claude mcp add global-utils --scope user /usr/local/bin/my-global-server
```

### Environment Variables

```bash
# Claude Code supports environment variable expansion
claude mcp add db-server --scope project /path/to/server -- --db-url $DATABASE_URL
```

### Verification

```bash
# List configured servers
claude mcp list

# Test server connection
claude mcp test my-rust-server
```

## Common Configuration Patterns

### Binary Path Configuration

**Absolute paths (recommended for production):**
```json
{
  "command": "/usr/local/bin/my-server",
  "args": []
}
```

**Relative paths (for development):**
```json
{
  "command": "./target/release/my-server",
  "args": []
}
```

**Using PATH:**
```json
{
  "command": "my-server",
  "args": []
}
```

### Environment Variables

**Development configuration:**
```json
{
  "env": {
    "RUST_LOG": "debug",
    "DEVELOPMENT": "true",
    "CONFIG_PATH": "./config/dev.json"
  }
}
```

**Production configuration:**
```json
{
  "env": {
    "RUST_LOG": "warn",
    "PRODUCTION": "true",
    "CONFIG_PATH": "/etc/myserver/config.json"
  }
}
```

### Arguments Patterns

**File system server:**
```json
{
  "args": [
    "--allowed-dirs", "/home/user/projects",
    "--read-only", "false"
  ]
}
```

**Database server:**
```json
{
  "args": [
    "--database-url", "postgresql://localhost/mydb",
    "--max-connections", "10"
  ]
}
```

**HTTP server:**
```json
{
  "args": [
    "--port", "8080",
    "--host", "127.0.0.1",
    "--tls-cert", "/path/to/cert.pem"
  ]
}
```

## Security Considerations

### File System Access

- **Limit directory access** with specific path arguments
- **Use read-only mode** when write access isn't needed
- **Validate paths** in your server implementation

### Network Access

- **Bind to localhost** only unless remote access is required
- **Use TLS** for network transport when possible
- **Implement authentication** for sensitive operations

### Environment Variables

- **Store secrets** in environment variables, not config files
- **Use system keychain** integration when available
- **Rotate API keys** regularly

### Example Secure Configuration

```json
{
  "mcpServers": {
    "secure-server": {
      "command": "/usr/local/bin/my-server",
      "args": [
        "--allowed-dirs", "/home/user/safe-directory",
        "--read-only",
        "--log-level", "warn"
      ],
      "env": {
        "API_KEY": "${SECRET_API_KEY}",
        "TLS_CERT_PATH": "/etc/ssl/certs/server.pem"
      }
    }
  }
}
```

## Troubleshooting

### Server Not Appearing

1. **Check configuration syntax:**
   ```bash
   # Validate JSON
   python -m json.tool ~/.cursor/mcp.json
   ```

2. **Verify binary path:**
   ```bash
   # Test binary execution
   /path/to/your/server --help
   ```

3. **Check permissions:**
   ```bash
   # Ensure binary is executable
   chmod +x /path/to/your/server
   ```

### Connection Issues

1. **Check logs:**
   - Most AI tools show MCP server logs in developer console
   - Enable debug logging with `RUST_LOG=debug`

2. **Test server independently:**
   ```bash
   # Run server directly to check for errors
   /path/to/your/server --stdio
   ```

3. **Verify transport mode:**
   - Ensure your server supports the transport mode (stdio/http)
   - Check that arguments match your server's expectations

### Performance Issues

1. **Monitor resource usage:**
   ```bash
   # Check if server is consuming too many resources
   top -p $(pgrep your-server)
   ```

2. **Optimize server startup:**
   - Reduce initialization time
   - Cache configuration
   - Use lazy loading for heavy operations

## Advanced Integration

### Multiple Server Configuration

```json
{
  "mcpServers": {
    "filesystem-server": {
      "command": "/path/to/filesystem-server",
      "args": ["/home/user/projects"]
    },
    "database-server": {
      "command": "/path/to/database-server",
      "args": ["--db-url", "postgresql://localhost/mydb"]
    },
    "api-server": {
      "command": "/path/to/api-server",
      "args": ["--endpoint", "https://api.example.com"]
    }
  }
}
```

### Conditional Configuration

**Development vs Production:**
```json
{
  "mcpServers": {
    "my-server": {
      "command": "/path/to/server",
      "args": ["--env", "${NODE_ENV:-development}"],
      "env": {
        "DATABASE_URL": "${DEV_DB_URL}",
        "API_ENDPOINT": "${DEV_API_ENDPOINT}"
      }
    }
  }
}
```

### Server Dependency Management

When servers depend on each other:

1. **Use startup delays:**
   ```json
   {
     "args": ["--startup-delay", "5000"]
   }
   ```

2. **Implement health checks:**
   ```rust
   // In your server
   async fn wait_for_dependency() {
       // Check if required service is available
   }
   ```

3. **Configure proper shutdown:**
   ```rust
   // Handle graceful shutdown
   tokio::signal::ctrl_c().await?;
   ```

## Next Steps

After successfully integrating your server:

1. **Test all tools** to ensure they work correctly
2. **Monitor performance** and optimize as needed
3. **Implement additional capabilities** like resources and prompts
4. **Create user documentation** for your specific server
5. **Consider packaging** as a desktop extension (.dxt) for easier distribution

For more advanced topics, see:
- [Deployment Guide](./DEPLOYMENT_GUIDE.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)
- [Plugin Development Guide](./guides/plugins.md)
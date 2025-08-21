# Example AI Tool Configuration Files

This directory contains example configuration files for integrating prism-mcp-rs servers with various AI tools. Copy and modify these examples for your specific use case.

## Directory Structure

```
examples/ai-tool-configs/
├── claude-desktop/
│   ├── basic-server.json
│   ├── filesystem-server.json
│   ├── database-server.json
│   └── multi-server.json
├── cursor/
│   ├── global-config.json
│   ├── project-config.json
│   └── development-config.json
├── vscode/
│   ├── workspace-mcp.json
│   ├── user-settings.json
│   └── enterprise-config.json
├── windsurf/
│   ├── basic-config.json
│   └── advanced-config.json
├── claude-code/
│   ├── project-mcp.json
│   ├── user-scope.json
│   └── cli-examples.sh
└── README.md (this file)
```

## Quick Reference

### Configuration File Locations by AI Tool

| AI Tool | Config Location | Format |
|---------|----------------|---------|
| **Claude Desktop** | `claude_desktop_config.json` | JSON |
| **Cursor** | `mcp.json` (global or project) | JSON |
| **VS Code** | `mcp.json` or `settings.json` | JSON |
| **Windsurf** | `mcp_config.json` | JSON |
| **Claude Code** | `.mcp.json` or CLI commands | JSON/CLI |

### Basic Template

All configurations follow this basic pattern:

```json
{
  "mcpServers": {
    "server-name": {
      "command": "/path/to/your/rust/binary",
      "args": ["--optional", "arguments"],
      "env": {
        "RUST_LOG": "info",
        "API_KEY": "your-api-key"
      }
    }
  }
}
```

## Example Configurations

### Claude Desktop - Basic Server

**Installation Path:**
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "my-rust-server": {
      "command": "/usr/local/bin/my-server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Cursor - Global Configuration

**Installation Path:** `~/.cursor/mcp.json`

```json
{
  "mcpServers": {
    "global-utilities": {
      "command": "/usr/local/bin/cursor-utils",
      "args": ["--global-mode"],
      "env": {
        "CURSOR_INTEGRATION": "true",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### VS Code - Workspace Configuration

**Installation Path:** `.vscode/mcp.json`

```json
{
  "servers": {
    "RustMCPServer": {
      "type": "stdio",
      "command": "/path/to/rust-server",
      "args": ["--workspace", "${workspaceFolder}"],
      "env": {
        "VSCODE_WORKSPACE": "${workspaceFolder}",
        "RUST_LOG": "info"
      }
    }
  }
}
```

### Windsurf - Basic Configuration

**Installation Path:** `~/.codeium/windsurf/mcp_config.json`

```json
{
  "mcpServers": {
    "windsurf-server": {
      "command": "/usr/local/bin/windsurf-mcp",
      "args": ["--windsurf-integration"],
      "env": {
        "WINDSURF_MODE": "true",
        "CASCADE_SUPPORT": "enabled"
      }
    }
  }
}
```

### Claude Code - CLI Examples

```bash
# Add local-scoped server (default)
claude mcp add my-rust-server /path/to/target/release/server

# Add with arguments
claude mcp add filesystem-server /usr/local/bin/fs-server -- --allowed-dirs /home/user/projects

# Add project-scoped server
claude mcp add project-tools --scope project ./target/release/project-server

# Add user-scoped server
claude mcp add global-utils --scope user /usr/local/bin/global-server

# List configured servers
claude mcp list

# Test server connection
claude mcp test filesystem-server
```

## Environment-Specific Examples

### Development Configuration

```json
{
  "mcpServers": {
    "dev-server": {
      "command": "./target/debug/server",
      "args": ["--dev", "--hot-reload"],
      "env": {
        "RUST_LOG": "debug",
        "DATABASE_URL": "sqlite:dev.db",
        "DEVELOPMENT": "true"
      }
    }
  }
}
```

### Production Configuration

```json
{
  "mcpServers": {
    "production-server": {
      "command": "/usr/local/bin/server",
      "args": ["--config", "/etc/server/production.json"],
      "env": {
        "RUST_LOG": "warn",
        "DATABASE_URL": "${DATABASE_URL}",
        "ENVIRONMENT": "production"
      }
    }
  }
}
```

## Security Best Practices

- **Never commit sensitive information** like API keys to version control
- **Use environment variables** for secrets and credentials
- **Set appropriate file permissions** (600) for configuration files
- **Use read-only mode** when write access isn't required
- **Limit directory access** to only what's necessary

## Usage Instructions

1. **Choose the appropriate configuration** for your AI tool and use case
2. **Copy the configuration** to the correct location for your platform
3. **Modify paths** to match your system and server binary locations
4. **Update environment variables** with your actual values
5. **Restart your AI tool** to load the new configuration
6. **Verify the server appears** in the AI tool's MCP server list

## Troubleshooting

If your configuration isn't working:

1. **Validate JSON syntax** using `python -m json.tool config.json`
2. **Check file paths** are absolute and correct
3. **Verify binary permissions** with `ls -la /path/to/server`
4. **Test server independently** with `/path/to/server --help`
5. **Check AI tool logs** for error messages
6. **Start with minimal configuration** and add complexity gradually

For more detailed troubleshooting, see the [Troubleshooting Guide](../../TROUBLESHOOTING.md).

## See Also

- [AI Tool Integration Guide](../../AI_TOOL_INTEGRATION.md) - Complete integration documentation
- [Deployment Guide](../../DEPLOYMENT_GUIDE.md) - Production deployment strategies
- [Getting Started Guide](../../GETTING_STARTED.md) - Quick start tutorial
- [Troubleshooting Guide](../../TROUBLESHOOTING.md) - Common issues and solutions
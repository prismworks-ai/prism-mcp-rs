# Troubleshooting Guide

## Overview

This guide helps diagnose and resolve common issues when deploying and using MCP servers built with prism-mcp-rs. Issues are organized by category with step-by-step debugging instructions.

## Table of Contents

- [Server Not Appearing in AI Tool](#server-not-appearing-in-ai-tool)
- [Connection Issues](#connection-issues)
- [Configuration Problems](#configuration-problems)
- [Performance Issues](#performance-issues)
- [Security and Permissions](#security-and-permissions)
- [Transport-Specific Issues](#transport-specific-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [Development vs Production Issues](#development-vs-production-issues)

## Server Not Appearing in AI Tool

### Symptoms
- MCP server doesn't show up in available tools
- No hammer (🔨) icon in AI tool interface
- Server status shows as "inactive" or "disconnected"

### Diagnostic Steps

#### 1. Verify Configuration File Location

**Claude Desktop:**
```bash
# macOS
ls -la ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Windows (PowerShell)
ls $env:APPDATA\Claude\claude_desktop_config.json
```

**Cursor:**
```bash
# Global config
ls -la ~/.cursor/mcp.json

# Project config  
ls -la .cursor/mcp.json
```

**VS Code:**
```bash
# Workspace config
ls -la .vscode/mcp.json

# Check settings.json for MCP configuration
code ~/.vscode/settings.json
```

#### 2. Validate JSON Syntax

```bash
# Test JSON validity
python -m json.tool ~/.cursor/mcp.json

# Or use jq
jq . ~/.cursor/mcp.json

# Or use online validator
cat ~/.cursor/mcp.json | curl -X POST -H "Content-Type: application/json" -d @- https://jsonlint.com/api/validate
```

#### 3. Check Binary Path and Permissions

```bash
# Verify binary exists and is executable
ls -la /path/to/your/server
file /path/to/your/server

# Test execution
/path/to/your/server --help

# Check permissions
stat /path/to/your/server
```

#### 4. Test Manual Server Startup

```bash
# Run server directly to check for errors
/path/to/your/server --stdio

# With debug logging
RUST_LOG=debug /path/to/your/server --stdio

# Test with MCP client simulation
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "1.0.0"}}}' | /path/to/your/server --stdio
```

### Common Solutions

#### Fix 1: Correct Configuration Format

**Correct format for Claude Desktop:**
```json
{
  "mcpServers": {
    "server-name": {
      "command": "/absolute/path/to/binary",
      "args": [],
      "env": {}
    }
  }
}
```

**Common mistakes:**
```json
// ❌ Wrong - missing mcpServers wrapper
{
  "server-name": {
    "command": "/path/to/binary"
  }
}

// ❌ Wrong - incorrect field names
{
  "mcpServers": {
    "server-name": {
      "executable": "/path/to/binary",  // Should be "command"
      "arguments": []                   // Should be "args"
    }
  }
}
```

#### Fix 2: Use Absolute Paths

```json
{
  "mcpServers": {
    "my-server": {
      "command": "/Users/username/projects/my-server/target/release/my-server",
      "args": [],
      "env": {}
    }
  }
}
```

#### Fix 3: Restart AI Tool

After configuration changes:
1. Save configuration file
2. Completely quit AI tool (not just close window)
3. Restart AI tool
4. Check for server in interface

## Connection Issues

### Symptoms
- Server appears but tools don't work
- "Connection refused" or "Connection timeout" errors
- Intermittent connectivity

### Diagnostic Steps

#### 1. Check Server Logs

Enable debug logging:
```bash
# Set environment variable
export RUST_LOG=debug

# Or in configuration
{
  "env": {
    "RUST_LOG": "debug"
  }
}
```

View logs:
```bash
# System logs (Linux)
journalctl -u your-server-name -f

# AI tool logs
# Claude Desktop: Help > Show Logs
# Cursor: Help > Toggle Developer Tools > Console
# VS Code: Output panel > MCP
```

#### 2. Network Connectivity

For HTTP transport:
```bash
# Test port accessibility
telnet localhost 8080

# Check if port is in use
netstat -an | grep :8080
lsof -i :8080

# Test HTTP endpoint
curl http://localhost:8080/health
```

#### 3. Process Status

```bash
# Check if server process is running
ps aux | grep your-server

# Monitor resource usage
top -p $(pgrep your-server)

# Check file descriptors
lsof -p $(pgrep your-server)
```

### Common Solutions

#### Fix 1: Transport Configuration

**STDIO Transport (recommended for local servers):**
```json
{
  "mcpServers": {
    "local-server": {
      "command": "/path/to/server",
      "args": ["--transport", "stdio"],
      "env": {}
    }
  }
}
```

**HTTP Transport:**
```json
{
  "mcpServers": {
    "http-server": {
      "command": "/path/to/server",
      "args": ["--transport", "http", "--port", "8080"],
      "env": {}
    }
  }
}
```

#### Fix 2: Firewall and Security

```bash
# Linux - check firewall
sudo ufw status
sudo iptables -L

# macOS - check firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# Windows - check firewall (PowerShell as Admin)
Get-NetFirewallProfile
```

#### Fix 3: Resource Limits

Increase limits if needed:
```bash
# Check current limits
ulimit -a

# Increase file descriptor limit
ulimit -n 4096

# For systemd services
[Service]
LimitNOFILE=4096
```

## Configuration Problems

### Symptoms
- Server starts but behaves unexpectedly
- Missing environment variables
- Wrong working directory

### Diagnostic Steps

#### 1. Validate Environment Variables

```bash
# Check if variables are set
printenv | grep MCP
env | grep API_KEY

# Test variable expansion
echo $DATABASE_URL
```

#### 2. Check Working Directory

Add debug logging to your server:
```rust
use tracing::info;

fn main() {
    info!("Starting server in directory: {:?}", std::env::current_dir());
    info!("Environment variables: {:?}", std::env::vars().collect::<Vec<_>>());
}
```

#### 3. Configuration Validation

Add configuration validation to your server:
```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    database_url: String,
    api_key: String,
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config = Config {
        database_url: std::env::var("DATABASE_URL")?,
        api_key: std::env::var("API_KEY")?,
    };
    
    tracing::info!("Loaded config: {:?}", config);
    Ok(config)
}
```

### Common Solutions

#### Fix 1: Environment Variable Configuration

```json
{
  "mcpServers": {
    "configured-server": {
      "command": "/path/to/server",
      "args": ["--config", "/path/to/config.json"],
      "env": {
        "DATABASE_URL": "postgresql://localhost/mydb",
        "API_KEY": "your-secret-key",
        "RUST_LOG": "info",
        "MCP_SERVER_PORT": "8080"
      }
    }
  }
}
```

## Quick Reference

### Essential Commands

```bash
# Configuration validation
python -m json.tool ~/.cursor/mcp.json

# Server testing
/path/to/server --help
/path/to/server --version

# Process monitoring
ps aux | grep server
top -p $(pgrep server)

# Network testing
telnet localhost 8080
curl http://localhost:8080/health

# Log monitoring
tail -f /var/log/server.log
journalctl -u server -f
```

### Configuration Templates

**Minimal working configuration:**
```json
{
  "mcpServers": {
    "test": {
      "command": "/bin/echo",
      "args": ["test"],
      "env": {}
    }
  }
}
```

**Production configuration:**
```json
{
  "mcpServers": {
    "production-server": {
      "command": "/usr/local/bin/server",
      "args": ["--config", "/etc/server/prod.json"],
      "env": {
        "RUST_LOG": "warn",
        "DATABASE_URL": "${DATABASE_URL}"
      }
    }
  }
}
```

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `command not found` | Binary path incorrect | Use absolute path |
| `permission denied` | File not executable | `chmod +x` |
| `connection refused` | Server not running | Check process status |
| `invalid JSON` | Syntax error | Validate JSON |
| `port in use` | Port conflict | Use different port |

For more detailed troubleshooting, see the full sections above or refer to the [AI Tool Integration Guide](./AI_TOOL_INTEGRATION.md) for setup-specific issues.
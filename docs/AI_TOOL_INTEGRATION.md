# AI Tool Integration

This guide configures a compiled STDIO server in common MCP clients. Client schemas change independently of this crate, so confirm details in the linked vendor documentation.

## Build and test the command

```bash
cargo build --release --locked
/absolute/path/to/target/release/your-server
```

The second command waits for MCP input; that is normal. Use an absolute executable path in client configuration. If the server requires files, either use absolute file paths or configure the client's working-directory mechanism when supported.

Do not put secrets directly in a committed project configuration. Prefer the client's environment expansion, OS keychain, or an external secret launcher.

## Claude Desktop

Open Claude Desktop's developer settings and edit its MCP configuration. A local STDIO entry uses the standard `mcpServers` shape:

```json
{
  "mcpServers": {
    "prism-example": {
      "command": "/absolute/path/to/your-server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

Restart the desktop app after changing the file. See Anthropic's [MCP documentation](https://docs.anthropic.com/en/docs/mcp) for the current UI and configuration location.

## Claude Code

Register a local server from the shell:

```bash
claude mcp add prism-example -- /absolute/path/to/your-server
```

Choose the appropriate local, project, or user scope for the intended audience. See the official [Claude Code MCP guide](https://docs.anthropic.com/en/docs/claude-code/mcp) and [CLI reference](https://docs.anthropic.com/en/docs/claude-code/cli-usage).

## Cursor

Use `.cursor/mcp.json` for project configuration or the documented global configuration location:

```json
{
  "mcpServers": {
    "prism-example": {
      "command": "/absolute/path/to/your-server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

See Cursor's [MCP documentation](https://docs.cursor.com/context/model-context-protocol).

## Visual Studio Code

Use `.vscode/mcp.json` in the workspace. VS Code uses a top-level `servers` object and an explicit transport type:

```json
{
  "servers": {
    "prism-example": {
      "type": "stdio",
      "command": "/absolute/path/to/your-server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

See the current [VS Code MCP server documentation](https://code.visualstudio.com/docs/agent-customization/mcp-servers).

## Windsurf

Windsurf uses an `mcpServers` configuration in its documented MCP configuration file:

```json
{
  "mcpServers": {
    "prism-example": {
      "command": "/absolute/path/to/your-server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

See Windsurf's [MCP configuration guide](https://docs.windsurf.com/windsurf/cascade/mcp).

## Remote HTTP servers

Only configure a remote URL when the client supports the same MCP HTTP transport and authentication method as the server. Use HTTPS, validate certificates, and follow the client's current remote-server schema. Do not translate an HTTP URL into a STDIO `command` entry.

## Diagnostics

If a client reports that the server disconnected:

1. run the exact absolute command outside the client;
2. confirm the binary matches the host architecture and is executable;
3. ensure stdout contains only MCP frames;
4. inspect client logs and server stderr/file logs;
5. verify every environment variable and referenced path;
6. use a minimal configuration with no arguments, then add settings back one at a time.

For protocol and policy errors, continue with [Troubleshooting](TROUBLESHOOTING.md).

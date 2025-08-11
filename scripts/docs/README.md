# MCP Protocol SDK Documentation

**Version:** 0.5.1  
**Protocol:** MCP 2025-06-18  
**Status:** Production Ready

## ## Documentation Structure

This documentation is organized for different audiences:

### # Quick Start
- **[Getting Started](./getting-started.md)** - Build your first MCP application in 5 minutes
- **[Examples](../examples/README.md)** - Working code examples for common use cases

### - User Guides
- **[Transport Selection](./transports.md)** - Choose between STDIO, HTTP, and WebSocket
- **[Error Handling](./error-handling.md#overview)** - Production-ready error recovery patterns
- **[Production Deployment](./production-readiness.md)** - Enterprise deployment guide

### - Feature Documentation
- **[HTTP Transport Features](./HTTP_TRANSPORT_FEATURES.md)** - complete HTTP capabilities
- **[Plugin System](./PLUGIN_SYSTEM.md)** - Dynamic tool loading architecture
- **[Health Monitoring](./health-monitoring.md)** - Observability and metrics

### ## API Reference
- **[API Documentation](./api-reference.md)** - Auto-generated API reference
- **[Rust Docs](https://docs.rs/mcp-protocol-sdk)** - Complete type documentation
- **[Protocol Types](https://docs.rs/mcp-protocol-sdk/latest/mcp_protocol_sdk/protocol/index.html)** - MCP protocol structures

### - Integrations
- **[Claude Desktop](./integrations/claude-desktop.md)** - Connect to Claude Desktop

### Security: Security & Policies
- **[Security Policy](./SECURITY.md)** - Vulnerability reporting and security practices

## ## Navigation Guide

| I want to... | Go to... |
|-------------|----------|
| Build my first MCP server | [Getting Started](./getting-started.md) |
| Understand error handling | [Error Handling Guide](./error-handling.md#smart-retry-logic) |
| Deploy to production | [Production Guide](./production-readiness.md) |
| Use HTTP transport | [HTTP Features](./HTTP_TRANSPORT_FEATURES.md) |
| View API docs | [API Reference](./api-reference.md) |
| Report a security issue | [Security Policy](./SECURITY.md) |

## Package: Feature Matrix

For a complete feature list, see the [SDK Features Overview](../README.md#features).

## 🔄 Documentation Updates

This documentation is automatically generated from source code. To update:

```bash
make docs        # Generate API documentation
make docs-sync   # Sync with source code
```

---

*For the latest updates, visit [GitHub](https://github.com/prismworks-ai/mcp-protocol-sdk) or [crates.io](https://crates.io/crates/mcp-protocol-sdk).*

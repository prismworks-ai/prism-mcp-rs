# Migration Guide

This guide helps developers migrate from other MCP implementations or understand the differences between prism-mcp-rs and the MCP specification.

## Table of Contents

- [Naming Differences](#naming-differences)
- [Type Separation](#type-separation)
- [API Improvements](#api-improvements)
- [Feature Flags](#feature-flags)
- [Common Migration Patterns](#common-migration-patterns)

## Naming Differences

prism-mcp-rs uses prefixed naming for clarity and to avoid namespace collisions:

| MCP Specification | prism-mcp-rs | Notes |
|------------------|--------------|-------|
| `Server` | `McpServer` | Prefixed to avoid confusion with generic server types |
| `Client` | `McpClient` | Prefixed for consistency |
| `Builder` | `ServerBuilder` / `McpClientBuilder` | Descriptive names for builders |
| `Handler` | `ToolHandler`, `ResourceHandler`, `PromptHandler` | Specific handler types |

### Type Aliases for Compatibility

For easier migration, you can use type aliases:

```rust
use prism_mcp_rs::server::McpServer as Server;
use prism_mcp_rs::client::McpClient as Client;
```

## Type Separation

prism-mcp-rs separates certain types for better type safety and error handling:

### JSON-RPC Messages

The library separates `JsonRpcResponse` and `JsonRpcError` as distinct types rather than using a single enum:

```rust
// prism-mcp-rs approach (type-safe)
use prism_mcp_rs::protocol::{JsonRpcResponse, JsonRpcError, JsonRpcMessage};

// Creating responses
let success = JsonRpcResponse::success_unchecked(id, result);
let error = JsonRpcError::method_not_found(id);

// Converting to JsonRpcMessage when needed
let msg_success: JsonRpcMessage = success.into();
let msg_error: JsonRpcMessage = error.into();
```

### Benefits of Type Separation

1. **Compile-time guarantees**: Can't accidentally treat an error as a success
2. **Clearer APIs**: Methods explicitly return success or error types
3. **Better IDE support**: Auto-completion shows relevant methods only
4. **Easier testing**: Can assert on specific types

## API Improvements

### 1. Builder Pattern

prism-mcp-rs provides fluent builders for complex types:

```rust
use prism_mcp_rs::server::ServerBuilder;
use prism_mcp_rs::client::McpClientBuilder;

// Server with builder
let server = ServerBuilder::new()
    .name("my-server")
    .version("1.0.0")
    .with_tools()
    .with_resources()
    .build();

// Client with builder
let client = McpClientBuilder::new()
    .name("my-client")
    .version("1.0.0")
    .with_retry()
    .build();
```

### 2. Error Convenience Methods

Quick error creation with built-in helpers:

```rust
use prism_mcp_rs::protocol::JsonRpcError;

// Standard JSON-RPC errors
let err1 = JsonRpcError::parse_error();
let err2 = JsonRpcError::method_not_found(request_id);
let err3 = JsonRpcError::invalid_params(request_id);

// MCP-specific errors
let err4 = JsonRpcError::tool_not_found(request_id, "unknown-tool");
let err5 = JsonRpcError::resource_not_found(request_id, "missing.txt");
```

### 3. Success Response Helpers

Two ways to create success responses:

```rust
use prism_mcp_rs::protocol::JsonRpcResponse;
use serde_json::json;

// Fallible (for serializable types)
let response1 = JsonRpcResponse::success(id, &my_struct)?;

// Infallible (for pre-serialized values)
let response2 = JsonRpcResponse::success_unchecked(id, json!({"key": "value"}));
```

### 4. Type Conversions

Seamless conversions between types:

```rust
use prism_mcp_rs::protocol::*;

// Into JsonRpcMessage
let msg1: JsonRpcMessage = request.into();
let msg2: JsonRpcMessage = response.into();
let msg3: JsonRpcMessage = error.into();
let msg4: JsonRpcMessage = notification.into();

// TryFrom JsonRpcMessage
let request: JsonRpcRequest = msg.try_into()?;
let response: JsonRpcResponse = msg.try_into()?;
let error: JsonRpcError = msg.try_into()?;
let notification: JsonRpcNotification = msg.try_into()?;
```

## Feature Flags

prism-mcp-rs uses feature flags for optional functionality:

```toml
[dependencies]
prism-mcp-rs = {
    version = "0.1.0",
    features = [
        "stdio",      # STDIO transport (default)
        "http",       # HTTP/SSE transport
        "websocket",  # WebSocket transport
        "full",       # All features
    ]
}
```

## Common Migration Patterns

### From TypeScript/JavaScript MCP

If migrating from the TypeScript MCP SDK:

#### Handler Pattern

```typescript
// TypeScript
server.setRequestHandler('tools/call', async (request) => {
    return { content: [{ type: 'text', text: 'Result' }] };
});
```

```rust
// Rust equivalent
use async_trait::async_trait;

struct MyToolHandler;

#[async_trait]
impl ToolHandler for MyToolHandler {
    async fn handle(&self, params: Option<Value>) 
        -> Result<Value, Box<dyn std::error::Error + Send + Sync>> 
    {
        Ok(json!({
            "content": [{"type": "text", "text": "Result"}]
        }))
    }
}
```

#### Async/Await

Both TypeScript and Rust use async/await, but Rust requires the `async_trait` macro:

```rust
use async_trait::async_trait;

#[async_trait]
impl YourTrait for YourStruct {
    async fn your_method(&self) -> Result<T, E> {
        // async code here
    }
}
```

### From Python MCP

If migrating from a Python MCP implementation:

#### Type Hints to Rust Types

```python
# Python
def handle_tool(params: Optional[Dict[str, Any]]) -> Dict[str, Any]:
    return {"result": "value"}
```

```rust
// Rust
fn handle_tool(params: Option<HashMap<String, Value>>) -> Result<Value, Box<dyn Error>> {
    Ok(json!({"result": "value"}))
}
```

#### Error Handling

```python
# Python
try:
    result = process_data()
except ValueError as e:
    raise McpError(f"Invalid value: {e}")
```

```rust
// Rust
let result = process_data()
    .map_err(|e| McpError::Protocol(format!("Invalid value: {}", e)))?;
```

### From Go MCP

If migrating from a Go MCP implementation:

#### Error Handling

```go
// Go
result, err := processData()
if err != nil {
    return nil, fmt.Errorf("processing failed: %w", err)
}
```

```rust
// Rust
let result = process_data()
    .map_err(|e| McpError::Protocol(format!("processing failed: {}", e)))?;
```

#### Interfaces to Traits

```go
// Go interface
type ToolHandler interface {
    Handle(params map[string]interface{}) (map[string]interface{}, error)
}
```

```rust
// Rust trait
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn handle(&self, params: Option<Value>) 
        -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
}
```

## Testing

prism-mcp-rs provides testing utilities:

```rust
#[cfg(test)]
mod tests {
    use prism_mcp_rs::test_utils::*;
    
    #[test]
    fn test_my_handler() {
        let request = mock_tool_call("my-tool", json!({"arg": "value"}));
        // Test your handler
        
        let response = mock_success(json!({"result": "ok"}));
        assert_response_contains(&response, &["result"]);
    }
}
```

## Getting Help

- **Documentation**: [docs.rs/prism-mcp-rs](https://docs.rs/prism-mcp-rs)
- **Examples**: See the `examples/` directory
- **Issues**: [GitHub Issues](https://github.com/PrismAI/prism-mcp-rs/issues)
- **Discord**: [Join our Discord](https://discord.gg/prism-ai)

## Compatibility Notes

- prism-mcp-rs implements MCP protocol version 2025-06-18
- All core MCP features are supported
- Additional convenience methods don't break protocol compatibility
- The SDK is wire-compatible with other MCP implementations
# Prism MCP SDK Examples

> 🚀 **Examples demonstrating core prism-mcp-rs SDK capabilities**

This directory contains example applications demonstrating how to use the Prism MCP SDK.

## 📚 Available Examples

| Example | Description |
|---------|-------------|
| **`bidirectional_basic.rs`** | Two-way communication between client and server |
| **`closure_handlers.rs`** | Using closures for handling tools and resources |
| **`custom_transport.rs`** | Implementing custom transport layers |

## 🚀 Running Examples

```bash
# Build all examples
cargo build --examples

# Run a specific example
cargo run --example bidirectional_basic
cargo run --example closure_handlers
cargo run --example custom_transport
```

## 📝 Documentation Examples

The SDK source code contains many small documentation examples that are tested with:

```bash
cargo test --doc
```

These appear in the generated documentation:

```bash
cargo doc --open
```

## 🎓 Learning Path

1. **Start with**: `closure_handlers` - Learn how to handle MCP operations
2. **Then try**: `bidirectional_basic` - Understand two-way communication
3. **Advanced**: `custom_transport` - Build your own transport layer

## 🤝 Contributing Examples

When adding new examples:

1. **Keep it simple**: One concept per example
2. **Use clear names**: Descriptive but concise
3. **Add comments**: Explain what the example demonstrates
4. **Test it**: `cargo build --example your_example`
5. **Update this README**: Add your example to the table above

## 🔗 Related Resources

- [SDK Documentation](../README.md)
- [API Reference](https://docs.rs/prism-mcp-rs)
- [MCP Specification](https://spec.modelcontextprotocol.io)

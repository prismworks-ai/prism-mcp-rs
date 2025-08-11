# Prism MCP SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/prism-mcp-rs.svg)](https://crates.io/crates/prism-mcp-rs)
[![Documentation](https://docs.rs/prism-mcp-rs/badge.svg)](https://docs.rs/prism-mcp-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Enterprise-grade Rust implementation of Anthropic's Model Context Protocol (MCP).

## Features

- 🚀 Full MCP protocol implementation
- 🔌 Extensible plugin system
- 🌐 Multiple transport layers (WebSocket, HTTP, stdio)
- ⚡ High-performance async runtime with Tokio
- 🛡️ Type-safe API with comprehensive error handling

## Installation

```toml
[dependencies]
prism-mcp-rs = "0.1.0"
```

## Quick Start

```rust
use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your MCP implementation here
    Ok(())
}
```

## Documentation

- [API Documentation](https://docs.rs/prism-mcp-rs)
- [Examples](./examples)
- [GitHub Repository](https://github.com/prismworks-ai/prism-mcp-rs)

## License

MIT License - See [LICENSE](LICENSE) file for details.

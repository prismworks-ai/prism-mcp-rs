#!/usr/bin/env bash

# Generate documentation for prism-mcp-rs
# This script is for local development and GitHub documentation
# It's separate from the crate build process

set -e

echo "🚀 Generating documentation for prism-mcp-rs..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Ensure we're in the project root
if [ ! -f "Cargo.toml" ]; then
    echo "${RED}Error: This script must be run from the project root${NC}"
    exit 1
fi

# Create necessary directories
echo "📁 Creating documentation directories..."
mkdir -p docs/api
mkdir -p .local/reports

# Clean up any stray profiling files
echo "🧹 Cleaning up stray files..."
for file in build_rs_cov.profraw default.profraw; do
    if [ -f "$file" ]; then
        mv "$file" .local/ 2>/dev/null || true
    fi
done

# Generate rustdoc
echo "📚 Generating rustdoc documentation..."
cargo doc --all-features --no-deps

if [ $? -eq 0 ]; then
    echo "${GREEN}✅ Rustdoc generated successfully${NC}"
else
    echo "${YELLOW}⚠️  Rustdoc generation had issues${NC}"
fi

# Generate API reference markdown
echo "📝 Generating API reference markdown..."
cat > docs/api-reference.md << 'EOF'
# API Reference

> **Auto-generated from Rust source code**  
> Last Updated: $(date +"%Y-%m-%d %H:%M:%S")

## Documentation

- [Online Documentation](https://docs.rs/prism-mcp-rs) (once published)
- [Local Documentation](../target/doc/prism_mcp_rs/index.html) (run `cargo doc --open`)
- [GitHub Repository](https://github.com/prismworks-ai/prism-mcp-rs)

## Core Modules

### Client
MCP client implementation for connecting to servers.
- [Local Docs](../target/doc/prism_mcp_rs/client/index.html)

### Server
MCP server implementation for handling requests.
- [Local Docs](../target/doc/prism_mcp_rs/server/index.html)

### Transport
Transport layer implementations (STDIO, HTTP, WebSocket, Streaming).
- [Local Docs](../target/doc/prism_mcp_rs/transport/index.html)

### Protocol
MCP protocol types and messages.
- [Local Docs](../target/doc/prism_mcp_rs/protocol/index.html)

### Core
Core utilities, error handling, and helper types.
- [Local Docs](../target/doc/prism_mcp_rs/core/index.html)

### Prelude
Commonly used types and traits for convenient imports.
- [Local Docs](../target/doc/prism_mcp_rs/prelude/index.html)

## Quick Start

```rust
use prism_mcp_rs::prelude::*;

// Create a server
let server = McpServer::new("my-server")
    .with_tool(my_tool)
    .with_resource(my_resource);

// Run with STDIO transport
server.run_stdio().await?;
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|--------|
| `stdio` | STDIO transport support | ✅ |
| `http` | HTTP transport with Axum server | ❌ |
| `websocket` | WebSocket transport | ❌ |
| `streaming-http` | Streaming HTTP support | ❌ |
| `validation` | JSON Schema validation | ❌ |
| `plugin` | Plugin system support | ❌ |
| `full` | All features enabled | ❌ |
EOF

echo "${GREEN}✅ API reference updated${NC}"

# Run the more advanced documentation script if it exists
if [ -f "scripts/docs/generate-docs-v2.sh" ]; then
    echo "📖 Running advanced documentation generation..."
    bash scripts/docs/generate-docs-v2.sh
fi

# Generate module-specific documentation if scripts exist
for module in client server transport protocol core; do
    if [ -f "scripts/docs/generate-${module}-docs.sh" ]; then
        echo "📄 Generating ${module} documentation..."
        bash "scripts/docs/generate-${module}-docs.sh"
    fi
done

echo ""
echo "${GREEN}✨ Documentation generation complete!${NC}"
echo ""
echo "📖 View documentation:"
echo "   - Run: cargo doc --open"
echo "   - Or open: target/doc/prism_mcp_rs/index.html"
echo ""
echo "📁 Documentation files:"
echo "   - API Reference: docs/api-reference.md"
echo "   - Module docs: docs/api/*.md"

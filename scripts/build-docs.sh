#!/bin/bash

# Comprehensive documentation generation script for prism-mcp-rs
# Integrates rustdoc with custom documentation processing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting comprehensive documentation build for prism-mcp-rs${NC}"

# Navigate to project root
cd "$(dirname "$0")/.."

# Clean previous docs
echo -e "${YELLOW}🧹 Cleaning previous documentation...${NC}"
rm -rf target/doc
rm -rf docs/api
mkdir -p docs/api

# Generate rustdoc with all features
echo -e "${BLUE}📚 Generating rustdoc API documentation...${NC}"
RUSTDOCFLAGS="--enable-index-page -Zunstable-options" cargo +nightly doc \
    --all-features \
    --no-deps \
    --document-private-items \
    2>/dev/null || cargo doc --all-features --no-deps --document-private-items

# Generate JSON documentation for processing
echo -e "${BLUE}🔧 Generating JSON documentation...${NC}"
cargo +nightly rustdoc \
    --lib \
    -- \
    -Z unstable-options \
    --output-format json \
    --document-private-items \
    2>/dev/null || true

# Extract module documentation to markdown
echo -e "${BLUE}📝 Extracting module documentation...${NC}"

# Core modules to document
MODULES=("client" "server" "transport" "protocol" "core" "plugin" "prelude")

for module in "${MODULES[@]}"; do
    echo -e "  Processing module: ${module}"
    
    # Create module-specific markdown file
    cat > "docs/api/${module}.md" << EOF
# ${module^} Module

> **Auto-generated from source code**  
> Last updated: $(date '+%Y-%m-%d %H:%M:%S')

For the complete API documentation, see the [rustdoc](../../target/doc/prism_mcp_rs/${module}/index.html).

## Overview

This module provides the ${module} functionality for the prism-mcp-rs SDK.

EOF

    # Try to extract module documentation from source
    if [ -f "src/${module}/mod.rs" ]; then
        # Extract module-level documentation
        awk '/^\/\/\//, /^[^/]/ {if (!/^[^/]/) print substr($0, 5)}' "src/${module}/mod.rs" >> "docs/api/${module}.md" 2>/dev/null || true
    elif [ -f "src/${module}.rs" ]; then
        awk '/^\/\/\//, /^[^/]/ {if (!/^[^/]/) print substr($0, 5)}' "src/${module}.rs" >> "docs/api/${module}.md" 2>/dev/null || true
    fi
done

# Generate main API index
echo -e "${BLUE}📋 Generating API index...${NC}"
cat > docs/api/README.md << 'EOF'
# API Documentation

> **Auto-generated from Rust source code**  
> Version: 0.1.0  
> Last updated: $(date '+%Y-%m-%d %H:%M:%S')

## Quick Links

- [Online Documentation](https://docs.rs/prism-mcp-rs)
- [Local Documentation](../../target/doc/prism_mcp_rs/index.html)
- [GitHub Repository](https://github.com/prismworks-ai/prism-mcp-rs)

## Core Modules

### [Client](./client.md)
MCP client implementation for connecting to servers.

### [Server](./server.md)
MCP server implementation for handling requests.

### [Transport](./transport.md)
Transport layer implementations:
- STDIO (default)
- HTTP
- WebSocket
- Streaming HTTP

### [Protocol](./protocol.md)
MCP protocol types and messages.

### [Core](./core.md)
Core utilities, error handling, and helper types.

### [Plugin](./plugin.md)
Plugin system for extending functionality.

### [Prelude](./prelude.md)
Commonly used types and traits for convenient imports.

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

## Examples

The SDK includes comprehensive examples in the `/examples` directory:

- **Basic Usage**: Simple client and server implementations
- **Transport Examples**: HTTP, WebSocket, and streaming demonstrations
- **Advanced Features**: Plugin system, validation, and performance optimization

## Getting Started

```rust
use prism_mcp_rs::prelude::*;

// Create a simple MCP server
let server = McpServer::new("my-server")
    .with_tool(my_tool)
    .with_resource(my_resource);

// Run with STDIO transport
server.run_stdio().await?;
```

## Documentation Standards

This documentation follows Prismworks AI documentation standards:
- Clear module organization
- Comprehensive examples
- Auto-generated from source code
- Regular updates with each build

EOF

# Apply documentation headers
if [ -f "scripts/docs/add-doc-headers-v3.py" ]; then
    echo -e "${BLUE}🏷️ Applying v3 documentation headers...${NC}"
    python3 scripts/docs/add-doc-headers-v3.py
fi

# Check documentation quality
if [ -f "scripts/docs/check-docs-quality.py" ]; then
    echo -e "${BLUE}🔍 Checking documentation quality...${NC}"
    python3 scripts/docs/check-docs-quality.py || true
fi

# Generate summary
echo -e "${GREEN}✅ Documentation generation complete!${NC}"
echo -e "${BLUE}📊 Summary:${NC}"
echo -e "  - Rustdoc generated at: target/doc/prism_mcp_rs/index.html"
echo -e "  - API docs generated at: docs/api/"
echo -e "  - Module docs created: ${#MODULES[@]}"
echo -e ""
echo -e "${YELLOW}To view documentation:${NC}"
echo -e "  - Run: cargo doc --open"
echo -e "  - Or open: target/doc/prism_mcp_rs/index.html"

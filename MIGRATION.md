# Migration Guide: Transport Feature Refactoring

## Overview

We've completed a major refactoring of the transport features in prism-mcp-rs to provide better clarity and separation of concerns. This guide will help you migrate from the old feature structure to the new one.

## Key Changes

### Feature Renaming

The following features have been renamed for clarity:

| Old Feature | New Feature | Purpose |
|------------|-------------|----------|
| `tokio-stream` | `sse` | Server-Sent Events support |
| `streaming-http` | `chunked-encoding` | HTTP chunked transfer encoding |
| `streaming-http2` | `http2` | HTTP/2 protocol support |
| `streaming-compression` | `compression` | Response compression (gzip, brotli, zstd) |

### Architectural Improvements

1. **SSE is now explicit**: Server-Sent Events is now a clear, named feature rather than being hidden behind a generic `tokio-stream` flag.

2. **HTTP/2 is separate**: HTTP/2 is now clearly distinguished from HTTP/1.1 features.

3. **Feature clarity**: Each feature now clearly describes what it enables.

## Migration Steps

### 1. Update Cargo.toml

**Before:**
```toml
[dependencies]
prism-mcp-rs = { version = "0.1", features = ["streaming-http", "tokio-stream"] }
```

**After:**
```toml
[dependencies]
prism-mcp-rs = { version = "0.1", features = ["chunked-encoding", "sse"] }
```

### 2. Update Feature Checks in Code

**Before:**
```rust
#[cfg(feature = "tokio-stream")]
fn setup_sse() { /* ... */ }

#[cfg(feature = "streaming-http2")]
fn setup_http2() { /* ... */ }
```

**After:**
```rust
#[cfg(feature = "sse")]
fn setup_sse() { /* ... */ }

#[cfg(feature = "http2")]
fn setup_http2() { /* ... */ }
```

### 3. Update Import Statements

The module structure remains the same, but feature gates have changed:

**Before:**
```rust
#[cfg(feature = "streaming-http")]
use prism_mcp_rs::transport::StreamingHttpClientTransport;
```

**After:**
```rust
#[cfg(feature = "chunked-encoding")]
use prism_mcp_rs::transport::StreamingHttpClientTransport;
```

## Feature Combinations

### Common Configurations

#### Basic HTTP with SSE
```toml
[dependencies]
prism-mcp-rs = { version = "0.1", features = ["http", "sse"] }
```

#### High-Performance HTTP/2
```toml
[dependencies]
prism-mcp-rs = { version = "0.1", features = ["http", "http2"] }
```

#### Full HTTP Stack
```toml
[dependencies]
prism-mcp-rs = { version = "0.1", features = [
    "http",
    "sse",
    "http2",
    "chunked-encoding",
    "compression"
] }
```

## Understanding the New Architecture

### Transport Hierarchy

```
Core Transports
├── stdio (default)
├── http
│   ├── sse (Server-Sent Events)
│   ├── http2 (HTTP/2 protocol)
│   ├── chunked-encoding (Transfer encoding)
│   └── compression (Response compression)
└── websocket
```

### Feature Dependencies

- `sse` requires `http`
- `http2` requires `http`
- `chunked-encoding` requires `http`
- `compression` requires `http`

## Breaking Changes

1. **Feature flags renamed**: All old feature flags must be updated.
2. **No automatic SSE**: SSE is no longer included when `http` is enabled.
3. **Explicit dependencies**: Features that were previously bundled are now separate.

## Benefits of the New Architecture

1. **Clarity**: Feature names now clearly indicate their purpose.
2. **Modularity**: Enable only the features you need.
3. **Smaller binaries**: Reduced code size when features aren't needed.
4. **Better documentation**: Clear separation makes it easier to understand.
5. **Type safety**: Proper feature gates prevent accidental dependencies.

## Examples Updated

The following examples have been renamed to match their purpose:

- `streaming_http_showcase.rs` → `chunked_encoding_showcase.rs`
- `streaming_http2_showcase.rs` → `http2_showcase.rs`
- NEW: `sse_showcase.rs` - Dedicated SSE example

## Troubleshooting

### Compilation Errors

If you see errors like:
```
error: cannot find value `handle_sse_events`
```

Make sure you've enabled the `sse` feature:
```toml
features = ["http", "sse"]
```

### Missing Types

If types like `Http2Config` are missing, enable the `http2` feature:
```toml
features = ["http", "http2"]
```

### Performance Issues

If you're seeing performance degradation, consider enabling:
- `compression` for bandwidth optimization
- `chunked-encoding` for large payloads
- `http2` for multiplexing

## Getting Help

For questions about the migration:
1. Check the [ARCHITECTURE.md](docs/ARCHITECTURE.md) document
2. Review the updated examples in the `examples/` directory
3. Open an issue on GitHub with the `migration` tag

## Timeline

- **v0.1.x**: Both old and new features work (with warnings)
- **v0.2.0**: Old features deprecated
- **v0.3.0**: Old features removed

We recommend migrating as soon as possible to avoid future breaking changes.

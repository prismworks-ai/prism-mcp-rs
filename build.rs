//! Build script for prism-mcp-rs SDK
//!
//! This script handles:
//! - Documentation generation with rustdoc
//! - API documentation extraction
//! - Documentation header standardization
//! - Build-time validation

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Only generate docs in release builds or when explicitly requested
    let should_generate_docs = env::var("GENERATE_DOCS").is_ok()
        || env::var("PROFILE").map(|p| p == "release").unwrap_or(false)
        || env::var("DOCS_RS").is_ok(); // Also generate on docs.rs

    if should_generate_docs {
        generate_api_docs();
    }

    // Ensure required directories exist
    ensure_directories();

    // Clean up any stray files
    cleanup_stray_files();

    // Tell Cargo to rerun if important files change
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=README.md");
}

fn ensure_directories() {
    let dirs = vec!["docs", "docs/api", ".local", ".local/reports"];

    for dir in dirs {
        let path = Path::new(dir);
        if !path.exists() {
            fs::create_dir_all(path).unwrap_or_else(|_| panic!("Failed to create {dir} directory"));
        }
    }
}

fn cleanup_stray_files() {
    // Move any profiling data to .local
    let stray_files = vec!["build_rs_cov.profraw", "default.profraw"];

    for file in stray_files {
        let source = Path::new(file);
        if source.exists() {
            let dest = Path::new(".local").join(file);
            fs::rename(source, dest).ok();
        }
    }
}

fn generate_api_docs() {
    println!("cargo:warning=Generating API documentation...");

    // Run the documentation generation script if it exists
    let doc_script = Path::new("scripts/build-docs.sh");
    if doc_script.exists() {
        let output = Command::new("bash").arg(doc_script).output();

        match output {
            Ok(result) if result.status.success() => {
                println!("cargo:warning=Documentation generated successfully");
            }
            Ok(result) => {
                // Don't fail the build for documentation issues
                eprintln!(
                    "Warning: Documentation generation had issues: {:?}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            Err(e) => {
                eprintln!("Warning: Could not run documentation script: {e}");
                // Fall back to basic rustdoc
                fallback_rustdoc();
            }
        }
    } else {
        // Fall back to basic rustdoc
        fallback_rustdoc();
    }

    // Create/update the main API reference file
    update_api_reference();
}

fn fallback_rustdoc() {
    // Basic rustdoc generation as fallback
    let output = Command::new("cargo")
        .args(["doc", "--all-features", "--no-deps"])
        .output();

    if let Ok(result) = output {
        if !result.status.success() {
            eprintln!("Warning: Basic rustdoc generation failed");
        }
    }
}

fn update_api_reference() {
    let api_ref_path = Path::new("docs/api-reference.md");

    // Always update the timestamp in the API reference
    let api_ref_content = format!(
        r#"# API Reference

> **Auto-generated from Rust source code**  
> Version: {}  

## Documentation

- [Online Documentation](https://docs.rs/prism-mcp-rs)
- [Local Documentation](../target/doc/prism_mcp_rs/index.html) (run `cargo doc --open`)
- [GitHub Repository](https://github.com/prismworks-ai/prism-mcp-rs)

## Core Modules

### Client
MCP client implementation for connecting to servers.
- [Online Docs](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/client/index.html)
- [Local Docs](../target/doc/prism_mcp_rs/client/index.html)

### Server
MCP server implementation for handling requests.
- [Online Docs](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/server/index.html)
- [Local Docs](../target/doc/prism_mcp_rs/server/index.html)

### Transport
Transport layer implementations (STDIO, HTTP, WebSocket, Streaming).
- [Online Docs](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/transport/index.html)
- [Local Docs](../target/doc/prism_mcp_rs/transport/index.html)

### Protocol
MCP protocol types and messages.
- [Online Docs](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/protocol/index.html)
- [Local Docs](../target/doc/prism_mcp_rs/protocol/index.html)

### Core
Core utilities, error handling, and helper types.
- [Online Docs](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/core/index.html)
- [Local Docs](../target/doc/prism_mcp_rs/core/index.html)

### Prelude
Commonly used types and traits for convenient imports.
- [Online Docs](https://docs.rs/prism-mcp-rs/latest/prism_mcp_rs/prelude/index.html)
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
"#,
        env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string())
    );

    fs::write(api_ref_path, api_ref_content).expect("Failed to write API reference");
}

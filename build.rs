//! Build script for prism-mcp-rs SDK
//!
//! This minimal build script only handles what's necessary for the crate build process.
//! Documentation generation is handled separately via scripts/generate-docs.sh

fn main() {
    // Tell Cargo to rerun if important files change
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=README.md");
    
    // Only print version information during build
    println!("cargo:rustc-env=PRISM_MCP_RS_VERSION={}", env!("CARGO_PKG_VERSION"));
}

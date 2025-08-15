//! Build script for prism-mcp-rs SDK
//!
//! This build script is intentionally minimal, following Rust best practices
//! for library crates. It only handles essential build-time configuration:
//!
//! - Sets environment variables for version information
//! - Configures rebuild triggers for important files
//! - Does NOT generate documentation (handled automatically by docs.rs)
//!
//! For development workflows and build commands, see:
//! - Makefile: Primary developer interface with convenient targets
//! - DEVELOPMENT.md: Complete development guide and workflow documentation
//! - scripts/ci/: CI simulation scripts that mirror GitHub Actions

fn main() {
    // Tell Cargo to rerun if important files change
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=README.md");

    // Only print version information during build
    println!(
        "cargo:rustc-env=PRISM_MCP_RS_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );
}

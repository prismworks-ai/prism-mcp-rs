# Utilities

This directory contains utility programs for testing and benchmarking various components of the prism-mcp-rs library.

## Benchmarks

The `benchmarks/` directory contains performance benchmarking utilities that can be run independently.

### Available Benchmarks

1. **Plugin Benchmarks** (`plugin_benchmarks.rs`)
   - Measures tool registration, execution, and plugin lifecycle management
   - Tests plugin configuration creation and state management
   - Benchmarks tool lookup performance with various registry sizes

2. **Client Benchmarks** (`client_benchmarks.rs`)
   - Measures transport layer performance
   - Tests request serialization and response deserialization
   - Benchmarks batch operations and client creation

3. **Server Benchmarks** (`server_benchmarks.rs`)
   - Measures request handling and routing efficiency
   - Tests concurrent request processing
   - Benchmarks middleware chain processing

### Usage

#### Method 1: Standalone Utilities (Recommended for Development)

Run individual benchmark utilities for detailed analysis:

```bash
# Run plugin benchmarks
cargo run --bin plugin_benchmarks --features bench

# Run client benchmarks
cargo run --bin client_benchmarks --features bench

# Run server benchmarks
cargo run --bin server_benchmarks --features bench
```

#### Method 2: Standard Cargo Bench (CI/CD Compatible)

For CI/CD workflows and compatibility with existing badges:

```bash
# Run all benchmarks (CI/CD compatible)
cargo bench --features bench

# Run specific benchmark suite
cargo bench --bench all_benchmarks --features bench
```

### Requirements

- The `bench` feature must be enabled when running benchmarks
- These utilities use the `criterion` crate for performance measurement
- Results will be displayed in the terminal and saved to `target/criterion/` directory

### CI/CD Integration

The project now supports both approaches:

- **CI/CD workflows** continue to work with `cargo bench --features bench` 
- **GitHub Actions benchmarks workflow** will function properly
- **Benchmark badges** in README.md will continue updating
- **Developer experience** is enhanced with standalone utilities

### Notes

This hybrid approach was implemented to resolve compilation issues with the criterion macro system while maintaining backward compatibility. The `benches/all_benchmarks.rs` file provides CI/CD compatibility, while the `utilities/benchmarks/` directory offers enhanced developer experience with detailed, standalone benchmark utilities.

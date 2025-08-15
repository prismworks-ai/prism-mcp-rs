# Project Structure

This document describes the organization of the prism-mcp-rs project.

## Directory Layout

```
prism-mcp-rs/
├── src/                    # Source code
│   ├── auth/              # Authentication modules
│   ├── client/            # MCP client implementation
│   ├── core/              # Core protocol types and traits
│   ├── plugin/            # Plugin system
│   ├── protocol/          # Protocol message definitions
│   ├── server/            # MCP server implementation
│   ├── transport/         # Transport layer implementations
│   ├── utils/             # Utility functions
│   └── lib.rs            # Library entry point
│
├── tests/                  # Integration tests
│   ├── plugin_fixtures/   # Test fixtures for plugins
│   └── *.rs              # Test modules
│
├── examples/              # Example applications
│   ├── client/           # Client examples
│   ├── server/           # Server examples
│   ├── utilities/        # Utility examples
│   └── *.rs             # Standalone examples
│
├── benches/               # Performance benchmarks
│   └── *.rs              # Benchmark suites
│
├── docs/                  # Documentation
│   ├── api-reference.md  # API documentation
│   ├── ci/               # CI documentation
│   ├── internal/         # Internal documentation
│   └── *.md             # Guide documents
│
├── scripts/               # Development and CI scripts
│   ├── ci/              # CI/CD scripts
│   ├── dev/             # Development setup scripts
│   ├── docs/            # Documentation generation
│   ├── test/            # Test runner scripts
│   └── utils/           # Utility scripts
│
├── .github/              # GitHub configuration
│   └── workflows/       # GitHub Actions workflows
│
├── supply-chain/         # Supply chain security
│   └── config.toml      # Cargo supply-chain config
│
├── reports/              # Generated reports
│   └── README.md        # Report documentation
│
├── coverage/             # Code coverage reports
│   └── html/           # HTML coverage reports
│
├── target/              # Build artifacts (git-ignored)
├── .local/              # Local development files (git-ignored)
└── .aicontext/          # AI context database (git-ignored)
```

## Key Files

### Configuration Files
- `Cargo.toml` - Rust package manifest
- `Cargo.lock` - Dependency lock file
- `rustfmt.toml` - Rust formatter configuration
- `clippy.toml` - Clippy linter configuration
- `deny.toml` - Cargo deny configuration
- `codecov.yml` - Code coverage configuration
- `.actrc` - Act (local CI) configuration
- `.gitignore` - Git ignore patterns

### Documentation
- `README.md` - Project readme and main documentation
- `LICENSE` - Apache 2.0 license
- `CONTRIBUTING.md` - Contribution guidelines
- `DEVELOPMENT.md` - Development setup guide
- `PROJECT_STRUCTURE.md` - This file

### Build & Development
- `build.rs` - Build script
- `Makefile` - Development tasks and shortcuts

## Development Workflow

### Running Tests
```bash
# All tests
cargo test --all-features

# Specific test suite
cargo test --test integration_e2e_stdio

# With coverage
cargo tarpaulin --all-features
```

### Running CI Locally
```bash
# Using native commands
./scripts/ci/run_ci_local.sh

# Using act (GitHub Actions emulator)
USE_ACT=true ./scripts/ci/run_ci_local.sh
```

### Building Documentation
```bash
# Generate docs
cargo doc --no-deps --all-features

# Open in browser
cargo doc --no-deps --all-features --open
```

### Running Examples
```bash
# List all examples
cargo build --examples

# Run specific example
cargo run --example database_server
```

### Code Quality
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-targets --all-features

# Security audit
cargo audit
```

## Best Practices

1. **No Backup Files**: Don't commit .bak files
2. **Use Scripts Directory**: Place all scripts in appropriate subdirectories under `scripts/`
3. **Local Development**: Use `.local/` for temporary development files
4. **Documentation**: Keep docs in `docs/` directory, organized by type
5. **Tests**: Integration tests in `tests/`, unit tests inline with code
6. **Examples**: Organize by category (client/server/utilities)

## Clean Build

To perform a clean build:
```bash
make clean
make build
```

Or manually:
```bash
cargo clean
cargo build --all-features
```

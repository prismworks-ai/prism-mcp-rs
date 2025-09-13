# Development Guide

This guide covers the development environment setup, build system, and workflows for the Prism MCP SDK.

## Environment Setup

### Prerequisites

- Rust 1.85+ (MSRV)
- Git
- Make (optional but recommended)

### Installation

```bash
git clone https://github.com/prismworks-ai/prism-mcp-rs
cd prism-mcp-rs

# Install development tools
make install-tools

# Or manually:
cargo install cargo-audit cargo-llvm-cov cargo-deny cargo-nextest
rustup component add rustfmt clippy
```

### Installing Act (GitHub Actions Runner)

Act runs GitHub Actions workflows locally for testing CI/CD changes.

#### macOS
```bash
brew install act
```

#### Linux
```bash
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

#### Windows
```powershell
choco install act-cli
# or
scoop install act
# or
winget install nektos.act
```

#### Verify Installation
```bash
act --version
act -l  # List available workflows
```

## Build System

### Makefile Targets

The Makefile provides convenient shortcuts for common tasks:

| Target | Description | When to Use |
|--------|-------------|-------------|
| `make quick` | Format, lint, basic tests | Before commits |
| `make check` | Standard CI checks | Before pushing |
| `make test` | Run all tests | After changes |
| `make coverage` | Generate coverage report | Check test coverage |
| `make docs` | Build documentation | After API changes |
| `make commit-ready` | Full validation | Before pull requests |
| `make local-ci` | Run CI locally with Act | Test CI changes |

### Cargo Commands

```bash
cargo build            # Build debug
cargo build --release  # Build optimized
cargo test             # Run all tests
cargo test --doc       # Documentation tests only
cargo test --lib       # Unit tests only
cargo test --tests     # Integration tests only
cargo bench            # Run benchmarks
cargo doc --open       # Build and view docs
```

## Code Quality & Pre-commit Hooks

### Automated Code Quality Checks

To prevent CI failures and maintain code quality, set up pre-commit hooks:

```bash
# Install pre-commit framework
pip install pre-commit

# Install hooks (includes formatting, linting, and basic checks)
pre-commit install

# Or use the project script
./scripts/install-pre-commit.sh
```

### Manual Code Quality Commands

```bash
# Format code (fixes rustfmt CI failures)
cargo fmt

# Check formatting without fixing
cargo fmt -- --check

# Lint code (checks for common issues)
cargo clippy --all-features -- -D warnings

# Fix linting warnings automatically
cargo fix --allow-dirty --all-features

# Run security audit
cargo audit

# Complete quality check (recommended before commits)
make quick
```

### Pre-commit Hook Features

The pre-commit configuration includes:

- **Code Formatting**: `cargo fmt` ensures consistent formatting
- **Linting**: `cargo clippy` catches common mistakes and suggests improvements
- **Basic Compilation**: `cargo check` verifies code compiles
- **File Cleanup**: Removes trailing whitespace, fixes line endings
- **Configuration Validation**: Checks YAML/TOML syntax

### Preventing Common CI Failures

| Issue | Prevention | Fix |
|-------|------------|-----|
| Formatting violations | Run `cargo fmt` before commit | `cargo fmt` |
| Compilation errors | Use `cargo check` frequently | Fix syntax errors |
| Clippy warnings | Run `cargo clippy` | Fix warnings or add `#[allow(clippy::...)]` |
| Test failures | Run `cargo test` before push | Fix failing tests |
| Example errors | Build examples with `cargo build --examples` | Fix example code |

### Code Quality Workflow

```bash
# Before making changes
git pull origin main

# During development
cargo check          # Quick compilation check
cargo test           # Run relevant tests

# Before committing (pre-commit hooks run automatically)
cargo fmt           # Format code
cargo clippy        # Check for issues
cargo test --all    # Run all tests

# Before pushing
make commit-ready   # Full validation
git push
```

## Testing

### Test Organization

```
tests/
├── integration/       # Integration tests
├── protocol_tests.rs  # Protocol tests
├── phase3_*.rs       # Error handling tests
└── phase4_*.rs       # Comprehensive tests
```

### Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test protocol_tests

# Specific test function
cargo test test_function_name

# With output
cargo test -- --nocapture

# Documentation tests
cargo test --doc  # 51 tests, all passing
```

### Documentation Tests

Code examples in documentation are tested automatically:

```rust
/// Example of a testable documentation comment
///
/// ```rust
/// use prism_mcp_rs::prelude::*;
/// 
/// let server = McpServer::new("test".to_string(), "1.0.0".to_string());
/// assert_eq!(server.name(), "test");
/// ```
pub fn example_function() {}
```

Use these attributes for doc tests:
- `rust` - Compile and run
- `rust,no_run` - Compile only (for I/O, servers)
- `rust,ignore` - Skip completely (avoid)

### Coverage

```bash
# Generate coverage report
make coverage

# View HTML report
open target/llvm-cov/html/index.html

# Coverage requirements: >65%
```

## Documentation

### Writing Documentation

#### Module Documentation

```rust
//! Module-level documentation goes here.
//!
//! # Examples
//!
//! ```rust,no_run
//! use prism_mcp_rs::module::*;
//! // Example code
//! ```

/// Function documentation.
///
/// # Arguments
///
/// * `param` - Parameter description
///
/// # Returns
///
/// Return value description
pub fn function(param: Type) -> ReturnType {
    // Implementation
}
```

#### Writing Examples

Examples should be placed in the `examples/` directory:

```rust
// examples/my_example.rs
use prism_mcp_rs::prelude::*;

fn main() {
    // Example code here
}
```

Add to `Cargo.toml`:
```toml
[[example]]
name = "my_example"
path = "examples/my_example.rs"
```

Test examples:
```bash
cargo build --examples
cargo run --example my_example
```

### Building Documentation

```bash
# Build documentation
cargo doc --no-deps

# Build and open
cargo doc --no-deps --open

# Build with all features
cargo doc --all-features --no-deps
```

## CI/CD

### GitHub Actions Workflows

| Workflow | Trigger | Purpose |
|----------|---------|------|
| `ci.yml` | Push, PR | Main CI pipeline |
| `ci-local.yml` | Manual | Local CI testing |
| `security.yml` | Schedule | Security audits |
| `release.yml` | Tag (v*) | Publishing releases |
| `dependencies.yml` | Schedule | Dependency updates |

### Running CI Locally

```bash
# Using Make
make local-ci     # Sequential CI (recommended)
make full         # Parallel CI (resource intensive)

# Using Act directly
act push          # Run push event workflows
act -j test       # Run specific job
act -l            # List all workflows

# Debugging
act -v push       # Verbose output
act -n push       # Dry run
```

### Pre-Push Validation

```bash
# Run before pushing
make commit-ready

# Or manually:
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo test --doc
make examples-validate
```

## Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b issue-123-feature-name
```

### 2. Make Changes

```bash
# Edit files
vim src/module.rs

# Test continuously
cargo test module::

# Check formatting
cargo fmt -- --check
```

### 3. Validate Changes

```bash
# Quick validation
make quick

# Full validation
make commit-ready
```

### 4. Commit

```bash
git add -A
git commit -m "feat(module): add new feature"
```

Commit message format:
- `feat(scope): description` - New feature
- `fix(scope): description` - Bug fix
- `docs(scope): description` - Documentation
- `test(scope): description` - Tests
- `refactor(scope): description` - Refactoring
- `perf(scope): description` - Performance
- `chore(scope): description` - Maintenance

### 5. Push and PR

```bash
git push origin issue-123-feature-name
```

Create pull request on GitHub with:
- Clear title: `feat(module): description (#123)`
- Link to issue: `Fixes #123`
- Description of changes
- Test results

## Project Structure

```
prism-mcp-rs/
├── src/
│   ├── lib.rs           # Library root
│   ├── core/            # Core abstractions
│   ├── protocol/        # Protocol types
│   ├── server/          # Server implementation
│   ├── client/          # Client implementation
│   ├── transport/       # Transport layers
│   └── plugin/          # Plugin system
├── tests/               # Integration tests
├── examples/            # Example implementations
├── docs/                # Additional documentation
├── scripts/             # Build scripts
├── Cargo.toml           # Package manifest
├── Makefile            # Build automation
└── .github/
    └── workflows/       # CI/CD workflows
```

## Debugging

### Common Issues

#### Compilation Errors
```bash
cargo clean
cargo build
```

#### Test Failures
```bash
cargo test -- --nocapture
RUST_BACKTRACE=1 cargo test
```

#### Documentation Build Errors
```bash
cargo doc --no-deps 2>&1 | grep warning
```

#### CI Failures
```bash
# Test locally with Act
act -j failing-job-name
```

### Performance Profiling

```bash
# CPU profiling
cargo build --release
perf record --call-graph=dwarf target/release/example
perf report

# Memory profiling  
valgrind --tool=massif target/release/example
ms_print massif.out.*
```

## Release Process

### 1. Update Version

Edit `Cargo.toml`:
```toml
version = "0.2.0"
```

### 2. Update Documentation

- Update `CHANGELOG.md` with release notes
- Review and update `README.md` for both GitHub and crates.io

### 3. Create Release

```bash
git checkout main
git pull origin main
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

GitHub Actions will:
- Run tests
- Build binaries
- Create GitHub release
- Publish to crates.io

### 4. Verify Release

- Check [crates.io](https://crates.io/crates/prism-mcp-rs)
- Verify [docs.rs](https://docs.rs/prism-mcp-rs)
- Review GitHub release page

## Tools Reference

### Required Tools

| Tool | Purpose | Installation |
|------|---------|-------------|
| rustc | Rust compiler | [rustup.rs](https://rustup.rs) |
| cargo | Build tool | Included with Rust |
| rustfmt | Code formatter | `rustup component add rustfmt` |
| clippy | Linter | `rustup component add clippy` |

### Optional Tools

| Tool | Purpose | Installation |
|------|---------|-------------|
| act | Local CI testing | See [Environment Setup](#installing-act-github-actions-runner) |
| cargo-audit | Security audit | `cargo install cargo-audit` |
| cargo-llvm-cov | Coverage | `cargo install cargo-llvm-cov` |
| cargo-nextest | Test runner | `cargo install cargo-nextest` |
| cargo-deny | License check | `cargo install cargo-deny` |

## Getting Help

- [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues) - Bug reports and features
- [GitHub Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions) - Questions and discussions
- [API Documentation](https://docs.rs/prism-mcp-rs) - Complete API reference
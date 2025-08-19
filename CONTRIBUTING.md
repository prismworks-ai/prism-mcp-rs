# Contributing Guidelines

## Overview

Contributions to the Prism MCP SDK follow a structured process to maintain code quality, consistency, and architectural integrity. This document outlines the technical requirements and procedures for contributing.

## Development Prerequisites

### Required Tools

| Tool | Version | Purpose |
|------|---------|------|
| Rust | ≥ 1.85.0 | Compiler and toolchain |
| Cargo | Latest | Build and dependency management |
| Git | ≥ 2.25 | Version control |
| Make | ≥ 3.81 | Build automation (optional) |

### Development Environment Setup

```bash
# Clone repository
git clone https://github.com/prismworks-ai/prism-mcp-rs
cd prism-mcp-rs

# Install development dependencies
make install-tools

# Alternatively, install manually
cargo install cargo-audit cargo-llvm-cov cargo-deny cargo-nextest
rustup component add rustfmt clippy
```

## Contribution Workflow

### 1. Issue Creation

All contributions must originate from a GitHub issue:

1. Search [existing issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
2. Create new issue with appropriate template
3. Wait for maintainer acknowledgment
4. Reference issue number in all commits

### 2. Branch Strategy

```bash
# Create feature branch from main
git checkout main
git pull origin main
git checkout -b issue-{number}-{brief-description}

# Examples:
# issue-123-circuit-breaker
# issue-456-http2-optimization
```

### 3. Development Process

#### Code Standards

| Requirement | Command | Acceptance Criteria |
|-------------|---------|--------------------|
| Formatting | `cargo fmt --all -- --check` | No formatting changes |
| Linting | `cargo clippy -- -D warnings` | Zero warnings |
| Testing | `cargo test --all-features` | 100% pass rate |
| Documentation | `cargo doc --no-deps` | No broken links |
| Coverage | `cargo llvm-cov --html` | ≥65% coverage |

#### Commit Convention

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Testing
- `chore`: Maintenance

**Example:**
```
feat(transport): Add HTTP/3 support

Implements QUIC-based HTTP/3 transport with connection migration
and 0-RTT support.

Closes #789
```

### 4. Pull Request Process

#### PR Checklist

- [ ] Issue linked in description
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] CI passes
- [ ] Code review addressed

#### PR Template

```markdown
## Description
Brief description of changes

## Motivation
Why these changes are needed

## Changes
- Change 1
- Change 2

## Testing
How changes were tested

## Checklist
- [ ] Tests pass
- [ ] Documentation updated
- [ ] Breaking changes documented

Closes #issue_number
```

## Code Quality Standards

### Documentation Requirements

All public APIs must include:

```rust
/// Brief description.
///
/// Detailed explanation of functionality, behavior, and usage patterns.
///
/// # Arguments
///
/// * `param_name` - Parameter description with constraints
///
/// # Returns
///
/// Description of return value and possible states
///
/// # Errors
///
/// Returns [`McpError`] when:
/// - Specific error condition
/// - Another error condition
///
/// # Examples
///
/// ```rust
/// use prism_mcp_rs::prelude::*;
///
/// let result = function_name(param).await?;
/// assert_eq!(result, expected);
/// ```
///
/// # Panics
///
/// Panics if:
/// - Panic condition (if applicable)
///
/// # Safety
///
/// Safety requirements (for unsafe functions)
pub async fn function_name(param: Type) -> McpResult<ReturnType> {
    // Implementation
}
```

### Testing Standards

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(input1, expected1)]
    #[case(input2, expected2)]
    #[tokio::test]
    async fn test_function_behavior(
        #[case] input: Type,
        #[case] expected: Type,
    ) {
        let result = function_under_test(input).await;
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_error_condition() {
        let result = function_under_test(invalid_input).await;
        assert!(matches!(result, Err(McpError::InvalidParams(_))));
    }
}
```

#### Integration Tests

```rust
// tests/integration_test.rs
use prism_mcp_rs::prelude::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_end_to_end_workflow() {
    let server = setup_test_server().await;
    let client = setup_test_client().await;
    
    // Test workflow
    let result = client.call_tool("test", args).await;
    assert!(result.is_ok());
    
    // Cleanup
    teardown(server, client).await;
}
```

### Performance Considerations

#### Benchmarking Requirements

Performance-critical changes must include benchmarks:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_new_feature(c: &mut Criterion) {
    c.bench_function("feature_name", |b| {
        b.iter(|| {
            // Benchmark code
            black_box(function_under_test(input))
        });
    });
}

criterion_group!(benches, benchmark_new_feature);
criterion_main!(benches);
```

### Security Requirements

#### Input Validation

```rust
pub async fn process_input(input: String) -> McpResult<String> {
    // Validate length
    if input.len() > MAX_INPUT_SIZE {
        return Err(McpError::invalid_params("Input exceeds maximum size"));
    }
    
    // Validate content
    if !input.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(McpError::invalid_params("Invalid characters in input"));
    }
    
    // Process validated input
    Ok(process_safe(input).await?)
}
```

#### Secure Defaults

- TLS enabled by default for network transports
- Authentication required for production configurations
- Rate limiting enabled with reasonable defaults
- Input sanitization for all user-provided data

## Architecture Guidelines

### Module Organization

```
src/
├── core/           # Core abstractions and traits
├── protocol/       # Protocol implementation
├── transport/      # Transport implementations
├── server/         # Server implementation
├── client/         # Client implementation
├── plugin/         # Plugin system
└── auth/           # Authentication/authorization
```

### Dependency Management

- Minimize external dependencies
- Audit dependencies regularly: `cargo audit`
- Use feature flags for optional dependencies
- Document dependency rationale in comments

### Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Operation failed: {context}")]
    OperationFailed {
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

// Convert to McpError
impl From<ComponentError> for McpError {
    fn from(err: ComponentError) -> Self {
        McpError::internal(err.to_string())
    }
}
```

## Review Process

### Code Review Criteria

1. **Correctness** - Logic is sound and bug-free
2. **Performance** - No unnecessary allocations or blocking calls
3. **Security** - Input validation and secure defaults
4. **Documentation** - Clear and comprehensive
5. **Testing** - Adequate coverage and edge cases
6. **Style** - Consistent with codebase conventions

### Review Response Time

| PR Size | Target Response | Maximum Wait |
|---------|----------------|-------------|
| Small (<100 lines) | 24 hours | 3 days |
| Medium (<500 lines) | 48 hours | 5 days |
| Large (>500 lines) | 72 hours | 7 days |

## Release Process

### Version Numbering

Follows [Semantic Versioning](https://semver.org/):

- **Major** (x.0.0): Breaking API changes
- **Minor** (0.x.0): New features, backward compatible
- **Patch** (0.0.x): Bug fixes, backward compatible

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Generate documentation
5. Create Git tag
6. Publish to crates.io
7. Create GitHub release

## Community Standards

### Code of Conduct

All contributors must adhere to the [Code of Conduct](CODE_OF_CONDUCT.md).

### Communication Channels

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General questions and ideas
- **Discord** - Real-time community support
- **Email** - Security issues (security@prismworks.ai)

### Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Annual contributor report

## Legal

### Contributor License Agreement

By submitting a pull request, you agree that:

1. Your contribution is your original work
2. You grant an MIT license for your contribution
3. Your contribution complies with the project license

### Copyright Header

```rust
// Copyright (c) 2025 Prismworks AI
// SPDX-License-Identifier: MIT
```

## Support

For contribution support:

- Review [existing PRs](https://github.com/prismworks-ai/prism-mcp-rs/pulls) for examples
- Ask questions in [GitHub Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions)
- Join [Discord](https://discord.gg/prismworks) for real-time help
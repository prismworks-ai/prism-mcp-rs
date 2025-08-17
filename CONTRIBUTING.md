# Contributing to Prism MCP SDK

Thank you for your interest in contributing to the Prism MCP SDK! This document provides the technical process for setting up a development environment, running tests, and submitting pull requests.

## Development Setup

### Prerequisites

- Rust 1.85+ (install from [rustup.rs](https://rustup.rs/))
- Git
- Make (optional but recommended)

### Setting Up Your Environment

1. **Fork and Clone**

```bash
# Fork on GitHub, then:
git clone https://github.com/YOUR_USERNAME/prism-mcp-rs
cd prism-mcp-rs
```

2. **Install Development Tools**

```bash
make install-tools

# Or manually:
cargo install cargo-audit cargo-llvm-cov cargo-deny cargo-nextest
rustup component add rustfmt clippy
```

3. **Create a Feature Branch**

```bash
git checkout -b issue-123-feature-description
```

## Development Process

### 1. Start with an Issue

All contributions must begin with a GitHub Issue:

- Search [existing issues](https://github.com/prismworks-ai/prism-mcp-rs/issues) first
- Create a new issue with the appropriate label (`bug`, `feature-request`, `enhancement`, `documentation`)
- Wait for discussion/approval before starting major work

### 2. Code Standards

| Check | Command | Requirement |
|-------|---------|-------------|
| Format | `cargo fmt` | Must pass |
| Lint | `cargo clippy -- -D warnings` | Zero warnings |
| Test | `cargo test` | All tests pass |
| Doc Test | `cargo test --doc` | All examples work |
| Coverage | `make coverage` | Maintain >65% |

### 3. Writing Code

#### Documentation

All public APIs must be documented:

```rust
/// Brief description of the function.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```rust,no_run
/// use prism_mcp_rs::prelude::*;
///
/// let result = function_name(param);
/// assert_eq!(result, expected);
/// ```
///
/// # Errors
///
/// Returns `McpError` if:
/// - Condition that causes error
pub fn function_name(param: Type) -> McpResult<ReturnType> {
    // Implementation
}
```

#### Testing

Write tests for all new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
    
    #[tokio::test]
    async fn test_async_feature() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### 4. Commit Guidelines

Use conventional commits:

```bash
git commit -m "type(scope): description"
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only
- `test` - Test additions/changes
- `refactor` - Code refactoring
- `perf` - Performance improvement
- `chore` - Maintenance tasks

**Examples:**
```bash
git commit -m "fix(transport): resolve websocket timeout issue"
git commit -m "feat(plugin): add hot reload support"
git commit -m "docs(readme): clarify installation steps"
```

### 5. Pre-Push Checklist

Run the full validation suite:

```bash
make commit-ready
```

Or manually:

```bash
cargo fmt              # Format code
cargo clippy           # Lint
cargo test             # Run tests
cargo test --doc       # Test documentation
cargo doc --no-deps    # Build docs
```

## Pull Request Process

### PR Title Format

`type(scope): description (#issue-number)`

Example: `fix(transport): resolve connection timeout (#123)`

### PR Template

```markdown
## Summary
Brief description of changes

## Related Issue
Fixes #123

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring

## Testing
- [ ] Tests pass locally (`cargo test`)
- [ ] Added new tests for changes
- [ ] Documentation updated
- [ ] Code follows style guidelines (`cargo fmt`, `cargo clippy`)
```

### Review Process

1. CI must pass (automated checks)
2. Code review by maintainers
3. Address feedback
4. Approval and merge

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test module_name::

# With output
cargo test -- --nocapture

# Documentation tests only
cargo test --doc
```

### Running CI Locally

Test CI pipeline locally using Act:

```bash
# Install Act (see docs/DEVELOPMENT.md for platform-specific instructions)
make local-ci

# Or specific job
act -j test
```

## What We're Looking For

### High Priority Contributions

- Bug fixes with tests
- Documentation improvements
- Performance optimizations with benchmarks
- Test coverage improvements

### Requires Discussion First

- Architecture changes
- New dependencies
- Breaking API changes
- Large features (>500 lines)

Please discuss in the issue before implementing.

### Not Accepted

- Changes without associated issues
- Features without tests
- Breaking changes without strong justification
- Code using `unsafe` without exceptional reason
- PRs that decrease test coverage

## Code Style

### Rust Guidelines

- Follow standard Rust naming conventions
- Use `rustfmt` defaults
- Keep functions small and focused
- Prefer composition over inheritance
- Document all public APIs
- Use semantic types over primitives

### Error Handling

- Use `Result<T, McpError>` for fallible operations
- Provide context in error messages
- Don't use `unwrap()` or `expect()` in library code
- Use the `?` operator for error propagation

### Performance

- Benchmark performance-critical code
- Use `async`/`await` for I/O operations
- Minimize allocations in hot paths
- Document performance characteristics

## Getting Help

- **Questions**: [GitHub Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions)
- **Bugs**: [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- **Security**: Email security@prismworks.ai

## Recognition

Contributors are recognized in:
- GitHub contributors page
- Release notes
- CHANGELOG.md for significant contributions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
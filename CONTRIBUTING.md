# Contributing to Prism MCP SDK

## Overview

Thank you for your interest in contributing to the Prism MCP SDK. This document provides guidelines and procedures for contributing to the project.

## Project Vision

The Prism MCP SDK aims to be the standard Rust implementation for the Model Context Protocol, providing:
- Production-ready components with enterprise reliability
- Complete protocol compliance with the MCP specification
- High-performance implementation leveraging Rust's capabilities
- Comprehensive documentation and examples

## Project Standards

### Code Quality Metrics
- Test coverage: 65% minimum (80% target for new code)
- Zero unsafe code policy
- Full transport support (STDIO, HTTP, WebSocket, HTTP/2)
- Complete error handling and recovery mechanisms
- Production-tested implementations

## Contribution Areas

### Documentation

Documentation improvements are valuable contributions:
- Correct typos or unclear explanations
- Add practical examples and use cases
- Improve API documentation clarity
- Contribute translations

### Bug Reports

When reporting bugs:
1. Check existing [Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
2. Create a new issue with:
   - Clear problem description
   - Steps to reproduce
   - Expected versus actual behavior
   - System specifications and versions

### Feature Development

For new features:
1. Review the [project roadmap](https://github.com/prismworks-ai/prism-mcp-rs/projects)
2. Discuss the feature in an issue before implementation
3. Follow coding standards outlined below
4. Submit a pull request with comprehensive tests

### Testing

Test contributions help ensure reliability:
- Write unit tests for new functionality
- Add integration tests for complex scenarios
- Create test fixtures for edge cases
- Verify cross-platform compatibility

### Design and Architecture

Architectural contributions:
- Propose improvements through issues
- Share use cases and requirements
- Review pull requests
- Participate in design discussions

## Development Environment

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Install development tools
cargo install cargo-watch cargo-edit cargo-audit

# For documentation generation
pip3 install --user -r scripts/requirements.txt
```

### Initial Setup

```bash
# Clone repository
git clone https://github.com/prismworks-ai/prism-mcp-rs.git
cd prism-mcp-rs

# Configure development environment
./scripts/dev/setup-dev.sh

# Verify installation
cargo test
```

### Development Workflow

```bash
# Initial setup (once)
make dev-setup

# During development
make quick         # Quick validation
cargo watch -x test  # Continuous testing

# Before committing
make commit-ready  # Format, lint, test

# Before pushing
make push-ready    # Complete CI validation
```

## Coding Standards

### Rust Guidelines

- Follow Rust naming conventions (RFC 430)
- Use `rustfmt` for consistent formatting
- Pass all `clippy` lints without warnings
- Document all public APIs with examples
- Write tests for all new functionality

### Quality Requirements

| Aspect | Requirement |
|--------|-------------|
| Test Coverage | 80% for new code, 65% overall |
| Documentation | All public items documented |
| Examples | Provided for complex features |
| Error Handling | Explicit error types with context |
| Performance | Benchmarked for critical paths |

### Commit Message Format

Follow conventional commits specification:

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

Example:
```
feat(transport): implement HTTP/2 server push

Adds server push capability for HTTP/2 transport with automatic
resource detection and priority handling.

Closes #123
```

## Pull Request Process

### 1. Preparation

```bash
git checkout -b feature/your-feature-name
```

### 2. Development

- Implement changes following coding standards
- Add tests achieving 80% coverage for new code
- Update relevant documentation
- Run `make push-ready` to validate

### 3. Submission

- Complete the pull request template
- Link related issues
- Ensure CI checks pass
- Request review from maintainers

### 4. Review Process

- Address feedback constructively
- Keep pull requests focused and atomic
- Update branch with main if needed
- Maintain clean commit history

### 5. Merge

- Squash commits if requested
- Ensure final CI validation passes

## Testing Guidelines

### Running Tests

```bash
# Complete test suite
cargo test

# Specific module tests
cargo test transport::

# With all features
cargo test --all-features

# Integration tests
./scripts/ci/integration_test.sh
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality() {
        // Arrange
        let input = prepare_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_functionality() {
        // Async test implementation
    }
}
```

## Documentation Standards

### API Documentation

- Use `///` for public item documentation
- Include code examples in documentation
- Add `# Examples` sections where appropriate
- Maintain clarity and conciseness

### Documentation Generation

```bash
# Generate documentation
make docs

# View locally
make docs-open

# Validate documentation
make docs-check
```

## Community Guidelines

### Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). 
All interactions should be respectful, constructive, and professional.

### Communication Channels

- **Documentation**: [docs.rs/prism-mcp-rs](https://docs.rs/prism-mcp-rs)
- **Discussions**: [GitHub Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions)
- **Issues**: [GitHub Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- **Email**: mcp-sdk@prismworks.ai

### Recognition

Contributors are recognized in:
- [Contributors Graph](https://github.com/prismworks-ai/prism-mcp-rs/graphs/contributors)
- Release notes
- Project documentation

## License Agreement

By contributing to this project, you agree that your contributions will be licensed under the MIT License, consistent with the project's licensing.

---

For questions or clarifications about the contribution process, please open a discussion or contact the maintainers directly.

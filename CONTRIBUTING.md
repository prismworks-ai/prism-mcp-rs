# Contributing to MCP Protocol SDK

<div align="center">

# Join Us in Building the Industry Standard

**Thank you for your interest in contributing to the MCP Protocol SDK!**

We're building the de facto industry standard for developing MCP clients and servers in Rust. 
Your contributions help us achieve and maintain the highest standards of quality.

[![GitHub stars](https://img.shields.io/github/stars/prismworks-ai/prism-mcp-rs?style=social)](https://github.com/prismworks-ai/prism-mcp-rs)
[![Contributors](https://img.shields.io/github/contributors/prismworks-ai/prism-mcp-rs)](https://github.com/prismworks-ai/prism-mcp-rs/graphs/contributors)
[![Discord](https://img.shields.io/discord/YOUR_DISCORD_ID?label=Discord&logo=discord)](https://discord.gg/YOUR_INVITE)

</div>

# Why Contribute?

# Project Vision
The MCP Protocol SDK aims to be the standard Rust implementation for the Model Context Protocol, providing:
- **Production-ready** components with production-ready reliability
- **100% protocol compliance** with the MCP specification
- **high-quality performance** leveraging Rust's strengths
- **complete documentation** that sets the standard

# ** Our Accomplishments
- **65%+ test coverage** with complete test suite
- **Zero unsafe code** for maximum safety
- **Full transport support** (STDIO, HTTP, WebSocket, HTTP/2)
- **Enterprise features** (auth, monitoring, error handling)
- **Active production use** in real-world applications

# * What You'll Gain
- **Recognition** as a contributor to a foundational Rust library
- **Experience** with production-grade Rust development
- **Knowledge** of the Model Context Protocol ecosystem
- **Community** with passionate developers building the future

# Ways to Contribute

# Note: Documentation
Help us maintain the highest documentation standards:
- Fix typos or unclear explanations (use our 2-click reporting!)
- Add examples and use cases
- Improve API documentation
- Translate documentation

# Bug: Bug Reports
Found a bug? We want to know:
1. Check if it's already reported in [Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
2. Create a new issue with:
 - Clear description
 - Reproduction steps
 - Expected vs actual behavior
 - System information

# Feature Development
Want to add a feature?
1. Check our [roadmap](https://github.com/prismworks-ai/prism-mcp-rs/projects)
2. Discuss in an issue first
3. Follow our coding standards
4. Submit a PR with tests

# Test: Testing
Help us reach 100% coverage:
- Write unit tests
- Add integration tests
- Create test fixtures
- Test on different platforms

# Note: Ideas & Feedback
Your input shapes the project:
- Suggest improvements
- Share use cases
- Review PRs
- Participate in discussions

# Getting Started

# Prerequisites
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Development tools
cargo install cargo-watch cargo-edit cargo-audit

# For documentation
pip3 install --user -r scripts/requirements.txt
```

# Setup
```bash
# Clone the repository
git clone https://github.com/prismworks-ai/prism-mcp-rs.git
cd prism-mcp-rs

# Set up development environment./scripts/dev/setup-dev.sh

# Run tests to verify setup
cargo test
```

# Development Workflow
```bash
# Before starting work
make dev-setup # One-time setup

# During development
make quick # Quick checks
cargo watch -x test # Auto-run tests

# Before committing
make commit-ready # Format, lint, test

# Before pushing
make push-ready # Full CI checks
```

# Coding Standards

# Rust Guidelines
- Follow standard Rust naming conventions
- Use `rustfmt` for formatting
- Pass `clippy` lints
- Document public APIs
- Write tests for new code

# Code Quality Standards
- **Test Coverage**: Aim for 80%+ on new code
- **Documentation**: All public items must be documented
- **Examples**: Add examples for complex features
- **Error Handling**: Use proper error types
- **Performance**: Profile and benchmark critical paths

# Commit Messages
Follow conventional commits:
```
type(scope): description

[optional body]

[optional footer(s)]
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

Example:
```
feat(transport): add HTTP/2 server push support

Implements server push capability for HTTP/2 transport
with automatic resource detection and priority handling.

Closes #123
```

# Pull Request Process

1. **Fork & Branch**
 ```bash
 git checkout -b feature/your-feature
 ```

2. **Develop**
 - Write code following our standards
 - Add tests (aim for 80%+ coverage)
 - Update documentation
 - Run `make push-ready`

3. **Submit PR**
 - Fill out the PR template
 - Link related issues
 - Ensure CI passes
 - Request review

4. **Review Process**
 - Address feedback promptly
 - Keep PR focused and small
 - Update branch with main if needed

5. **Merge**
 - Squash commits if requested
 - Celebrate your contribution! ## Testing

# Running Tests
```bash
# All tests
cargo test

# Specific module
cargo test transport::

# With all features
cargo test --all-features

# Integration tests./scripts/ci/integration_test.sh
```

# Writing Tests
```rust
#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_feature() {
 // Arrange
 let input = prepare_test_data();
 
 // Act
 let result = function_under_test(input);
 
 // Assert
 assert_eq!(result, expected_value);
 }

 #[tokio::test]
 async fn test_async_feature() {
 // Test async code
 }
}
```

# Documentation

# Writing Documentation
- Use `///` for public item documentation
- Include examples with ` ```rust` blocks
- Add `# Examples` sections
- Keep it clear and concise

# Generating Documentation
```bash
# Generate with improved headers
make docs

# View locally
make docs-open

# Check quality
make docs-check
```

# Community

# Code of Conduct
We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
Be kind, respectful, and constructive.

# Getting Help
## [Documentation](https://docs.rs/prism-mcp-rs)
- Chat: [Discussions](https://github.com/prismworks-ai/prism-mcp-rs/discussions)
- Bug: [Issues](https://github.com/prismworks-ai/prism-mcp-rs/issues)
- Email: Email: mcp-sdk@prismworks.ai

# Recognition
All contributors are recognized in:
- [Contributors](https://github.com/prismworks-ai/prism-mcp-rs/graphs/contributors)
- Release notes
- Project documentation

# License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT).

---

<div align="center">

**Thank you for helping us build the future of MCP in Rust!** Rust

Every contribution, no matter how small, makes a difference.

[Start Contributing](https://github.com/prismworks-ai/prism-mcp-rs/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) |
[Join Discussion](https://github.com/prismworks-ai/prism-mcp-rs/discussions) |
[View Roadmap](https://github.com/prismworks-ai/prism-mcp-rs/projects)

</div>

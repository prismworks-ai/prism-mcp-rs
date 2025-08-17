# Documentation

This directory contains comprehensive documentation for the Prism MCP SDK.

## User Documentation

- **[Getting Started](GETTING_STARTED.md)** - Step-by-step tutorial for new users
- **[API Reference](https://docs.rs/prism-mcp-rs)** - Auto-generated from code (or run `cargo doc --open` locally)
- **[Examples](../examples/)** - Runnable code examples demonstrating SDK features

## Guides

In-depth guides for advanced topics:

- **[Authentication](guides/authentication.md)** - Implementing authentication and authorization
- **[Error Handling](guides/error-handling.md)** - Best practices for error handling
- **[Migration](guides/migration.md)** - Migrating from other MCP implementations
- **[Performance](guides/performance.md)** - Optimization techniques and benchmarking
- **[Plugin Development](guides/plugins.md)** - Creating runtime-loadable plugins
- **[Plugin Types](guides/plugin-types.md)** - Detailed plugin component specifications

## Developer Documentation

- **[Development Guide](DEVELOPMENT.md)** - Complete development environment setup and workflows
- **[Contributing](../CONTRIBUTING.md)** - How to contribute to the project

## Internal Documentation

The `internal/` directory contains technical documentation for SDK maintainers:

- Implementation notes
- API design decisions
- CI/CD documentation
- Development tools guides

## Generating API Documentation

API documentation is auto-generated from code comments:

```bash
# Generate and open locally
cargo doc --open

# Generate with all features
cargo doc --all-features --no-deps --open
```

Once published, documentation will be available at [docs.rs/prism-mcp-rs](https://docs.rs/prism-mcp-rs).

## Documentation Standards

### For Code Documentation

- All public APIs must be documented
- Include examples in doc comments
- Examples should be tested (`cargo test --doc`)
- Use `rust,no_run` for examples that require I/O or servers

### For Markdown Documentation

- Use clear, concise language
- Include code examples where relevant
- Keep guides focused on a single topic
- Update when APIs change

## Contributing to Documentation

Documentation improvements are always welcome! See [CONTRIBUTING.md](../CONTRIBUTING.md) for the contribution process.
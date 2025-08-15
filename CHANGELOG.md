# Changelog

All notable changes to the Prism MCP SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Convenience methods for `ContentBlock::text()`, `ContentBlock::image()`, `ContentBlock::audio()`, `ContentBlock::resource_link()`
- Convenience methods for `ToolResult::text()`, `ToolResult::error()`, `ToolResult::with_content()`, `ToolResult::with_structured()`
- Comprehensive plugin architecture for runtime component loading
- Support for MCP 2025-06-18 specification
- Multiple transport implementations (STDIO, HTTP, WebSocket, HTTP/2)
- Dynamic plugin loading and hot reload capabilities
- Bidirectional communication support
- Streaming capabilities for large payloads
- Completion API for autocomplete functionality
- Resource templates and patterns
- Comprehensive test suite with 229+ tests
- Full documentation suite including plugin development guides
- CI/CD pipeline with GitHub Actions
- Code coverage reporting
- Performance benchmarking suite
- Example implementations for all major features

### Changed
- **BREAKING**: `ToolHandler` trait now consistently uses `HashMap<String, Value>` for arguments
- Updated README with accurate, tested examples only
- Improved developer experience with better error messages

### Deprecated
- N/A (initial release)

### Removed
- Removed obsolete/incorrect code examples from README
- Removed confusing type alias duplicates

### Fixed
- Fixed `ToolHandler` trait signature inconsistency
- Fixed missing convenience methods that were documented but not implemented
- Fixed compilation errors in example files
- Fixed duplicate method implementations causing compilation errors

### Security
- Implemented secure plugin loading mechanisms
- Added input validation for all protocol messages
- Enforced memory safety with no unsafe code

## [0.1.0] - 2025-01-14 (Pending Release)

### Added
- Initial release of the Prism MCP SDK
- Core MCP protocol implementation
- Plugin system architecture
- Transport layer abstractions
- Client and server implementations
- Tool, Resource, Prompt, and Completion handlers
- Comprehensive documentation
- Example applications
- Test coverage >80%

### Technical Specifications
- **Rust Version**: MSRV 1.85
- **MCP Version**: 2025-06-18
- **License**: MIT
- **Dependencies**: Minimal, with optional features for additional transports

### Contributors
- Prismworks AI Team

[Unreleased]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/prismworks-ai/prism-mcp-rs/releases/tag/v0.1.0
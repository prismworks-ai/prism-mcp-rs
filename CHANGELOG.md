# Changelog

All notable changes to the Prism MCP SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.2] - 2025-12-27

### Maintenance & Quality

- **Code Quality**: Full rustfmt and clippy pass with zero warnings
- **API Consistency**: Standardized ContentBlock::text() and ToolResult usage across docs
- **Example Fixes**: Fixed Type alias mismatches in prompts and transport examples
- **Test Coverage**: Documentation examples now compile and pass verification

## [1.1.1] - 2025-09-13

### 🔧 Maintenance & Documentation

#### Examples & Documentation Improvements
- **📚 Enhanced Examples**: Updated and fixed multiple example files
  - Corrected API usage in authentication, WebSocket, and plugin examples
  - Fixed resource handling and prompts API examples
  - Updated integration tests and working examples
- **📖 Documentation Updates**: Improved plugin guides and error handling documentation
  - Enhanced plugin-types.md with clearer examples
  - Updated plugins.md with latest API patterns
  - Better error handling documentation

#### Library Improvements
- **🛠️ API Refinements**: Minor improvements to lib.rs exports
- **✅ Test Coverage**: Added new test files for documentation examples
- **🔧 Transport Fixes**: Improved custom transport example

### 📊 Impact
- **Enhanced Developer Experience**: All examples now demonstrate correct API usage
- **Better Documentation**: Clearer guides for plugin development and error handling
- **Improved Reliability**: Better test coverage and working examples

## [1.1.0] - 2025-09-13

### 🚀 Major Improvements

#### GitHub Actions CI/CD Fixes
- **✅ Fixed Formatting Issues**: Resolved all rustfmt violations across the codebase
- **✅ Fixed Examples Compilation**: Replaced broken `examples/basic/fixed_example.rs` with working version
  - Fixed 32+ compilation errors due to syntax issues and wrong API usage
  - Corrected type imports and struct initialization
  - All examples now compile successfully
- **✅ Fixed Example Warnings**: Applied `cargo fix` to resolve compilation warnings in 15 files

#### Development Workflow Enhancements
- **🔧 Pre-commit Hooks**: Added comprehensive `.pre-commit-config.yaml`
  - Automated code formatting with `cargo fmt`
  - Linting with `cargo clippy`
  - Compilation checks with `cargo check`
  - File cleanup (trailing whitespace, line endings)
  - Configuration validation (YAML/TOML)
- **📚 Enhanced Documentation**: Updated `docs/DEVELOPMENT.md` with:
  - Code quality workflow section
  - Pre-commit hook setup instructions
  - CI failure prevention guide
  - Manual code quality commands reference

#### Infrastructure Fixes
- **🔐 Dependency Update Workflow**: Fixed permissions issues
  - Added `issues: write` permission
  - Use `PAT_TOKEN` for issue creation to avoid GitHub token limitations
  - Resolves "Resource not accessible by integration" errors
- **🛠️ Installation Scripts**: Updated `scripts/install-pre-commit.sh` for pre-commit framework support

### 🛡️ Quality & Reliability
- **Code Quality**: Established automated code quality checks preventing future CI failures
- **Developer Experience**: Clear setup instructions and automated tooling
- **Maintenance**: Robust development workflow with comprehensive validation

### 📊 Impact
- **Before**: 2 of 13 CI jobs failing, 32 compilation errors in examples
- **After**: All immediate issues resolved, automated prevention of future problems
- **Long-term**: Sustainable development workflow with quality gates

## [1.0.0] - 2025-09-13

### 🎉 Production Release
- **Production-ready MCP SDK**: Complete implementation of Model Context Protocol
- **Enterprise Features**: Advanced resilience patterns, circuit breakers, adaptive retry policies
- **Multiple Transports**: HTTP/2, WebSocket, Stdio with compression and authentication
- **Security**: Comprehensive supply chain security audit and cargo-vet integration
- **Performance**: Optimized for production workloads with benchmarking
- **Documentation**: Complete API documentation and usage examples

## [0.1.4] - 2025-08-25

### 🎨 Documentation & Presentation
- **Badge Improvements**: Complete overhaul of README badges with consistent flat-square styling
- **Visual Enhancement**: Added appropriate icons (GitHub, Rust, Discord, shield) for better visual clarity
- **Information Accuracy**: Updated badges to reflect current project state and remove inconsistencies
- **Professional Appearance**: Enhanced badge presentation for better first impressions

### 🔧 Badge Fixes
- **Security Badge**: Changed from failing status indicator to descriptive "Security Audit" label
- **API Stability**: Updated from generic "beta" label to specific version indicator
- **Performance Badge**: Enhanced with "performance-tracked" descriptor for clarity
- **Dependencies**: Improved tracking using libraries.io for better accuracy
- **Downloads**: Better color coding and labeling for download statistics
- **Discord**: Community-focused messaging with proper Discord branding
- **Release**: Added pre-release inclusion for more accurate release tracking

### 📊 Impact
- **Improved Discoverability**: More professional appearance increases project credibility
- **Better Information**: Users get accurate, up-to-date project status at a glance
- **Enhanced Branding**: Consistent styling aligns with professional open-source standards

## [0.1.3] - 2025-08-25

### 🚀 New Features
- **Benchmark Consolidation**: Unified benchmark suite with `all_benchmarks.rs` for comprehensive performance testing
- **Utilities Module**: Added structured utilities directory with dedicated README and benchmark organization
- **Enhanced CI/CD**: Completely rebuilt CI workflow to eliminate infrastructure failures and improve reliability

### 🔧 Improvements
- **Project Organization**: Moved benchmark files to `utilities/benchmarks/` for better project structure
- **Performance Optimization**: Critical CI performance optimizations to reduce build times and failure rates
- **Code Quality**: Enhanced rustfmt formatting and resolved compilation warnings
- **Dependencies**: Added cargo-vet audit entries for unvetted dependencies to improve security posture

### 🐛 Bug Fixes
- **CI Compilation**: Resolved benchmark compilation issues and PowerShell compatibility problems
- **Import Resolution**: Fixed missing `Instant` import in streaming HTTP transport
- **Unused Imports**: Cleaned up unused import and variable warnings across the codebase
- **Workflow Reliability**: Improved release workflow reliability with better timeout handling
- **Context Management**: Removed `.context` folder from version control and added to `.gitignore`

### 📁 File Changes
- **Added**: `benches/all_benchmarks.rs` - Consolidated benchmark suite
- **Added**: `utilities/README.md` - Documentation for utilities module
- **Moved**: Benchmark files to `utilities/benchmarks/` directory (client, plugin, server benchmarks)
- **Modified**: Multiple test files and transport modules for improved reliability
- **Updated**: CI/CD workflows and Cargo configuration

### 📊 Statistics
- **13 commits** since v0.1.2
- **22 files** modified or added
- **Enhanced security** with dependency auditing
- **Improved CI/CD** reliability and performance

## [0.1.2] - 2025-08-22

### Added
- CI workflow stability improvements
- Enhanced error handling in transport modules
- Better dependency management with cargo-vet

### Fixed
- Multiple CI/CD pipeline issues
- Compilation warnings and errors
- Release workflow reliability

## [0.1.1] - 2025-08-20

### Added
- **🎯 Comprehensive AI Tool Integration Documentation** - Complete guides for integrating with Claude Desktop, Cursor, VS Code, Windsurf, and Claude Code
- **📋 Production Deployment Guide** - Enterprise-grade deployment strategies, cross-platform builds, and distribution methods
- **🔧 Troubleshooting Guide** - Systematic issue resolution with platform-specific solutions and diagnostic scripts
- **📚 Configuration Examples Library** - Ready-to-use templates for all supported AI tools and environments
- **🚀 Automatic Crates.io Publication** - Streamlined release workflow with automatic package publishing

### Changed
- **💎 Streamlined README** - Refocused on innovation showcase and clean API examples rather than verbose quickstart content
- **📖 Enhanced Documentation Structure** - Improved navigation and cross-references between all documentation files
- **🏗️ Optimized CI/CD Pipeline** - Fixed benchmark workflows and enabled proper plugin feature testing

### Fixed
- **🔒 Security Validation Issues** - Resolved cargo-vet configuration and license validation failures
- **🔗 Documentation Cross-Links** - Fixed all broken internal documentation references
- **📊 Benchmark Infrastructure** - Corrected benchmark badge generation and plugin feature compilation
- **📦 Crates.io Compatibility** - Shortened package keywords to meet registry requirements

### Technical Improvements
- Added MPL-2.0 to allowed licenses for dependency compatibility
- Reinitialized cargo-vet supply chain with compatible version format
- Enhanced README to highlight architectural innovations and unique value proposition
- Integrated plugin documentation with AI tool deployment workflow
- Complete validation: 264 tests passing, no clippy warnings, all security checks green

## [Unreleased]

### Added
- Circuit breaker pattern for fault tolerance
- Adaptive retry policies with exponential backoff and jitter
- Multi-level health monitoring system
- Streaming HTTP/2 transport with multiplexing and server push
- Adaptive compression (Gzip, Brotli, Zstd) based on content analysis
- Schema introspection for runtime capability discovery
- Batch operations support for bulk request processing
- Hot-reloadable plugin system with ABI stability
- Production observability with structured logging and metrics
- Convenience methods for `ContentBlock::text()`, `ContentBlock::image()`, `ContentBlock::audio()`, `ContentBlock::resource_link()`
- Convenience methods for `ToolResult::text()`, `ToolResult::error()`, `ToolResult::with_content()`, `ToolResult::with_structured()`
- Comprehensive plugin architecture for runtime component loading
- Support for MCP 2025-06-18 specification
- Multiple transport implementations (STDIO, HTTP, WebSocket, HTTP/2)
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
- Complete documentation overhaul with professional, academic standards
- Restructured examples directory with production-ready implementations
- Enhanced error handling with structured error types
- Improved developer experience with better error messages

### Deprecated
- N/A (initial release)

### Removed
- Internal documentation files (moved to separate repository)
- Obsolete migration guides
- Duplicate and redundant example files
- Macro-related examples (moved to prism-mcp-rs-dev)

### Fixed
- Fixed `ToolHandler` trait signature inconsistency
- Fixed missing convenience methods that were documented but not implemented
- Fixed compilation errors in example files
- Fixed duplicate method implementations causing compilation errors
- Resolved all clippy warnings and formatting issues

### Security
- Implemented secure plugin loading mechanisms
- Added input validation for all protocol messages
- Enforced memory safety with no unsafe code
- TLS 1.3 support with mTLS capabilities
- JWT/OAuth2 authentication support
- Rate limiting and DDoS protection

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

[Unreleased]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v1.1.2...HEAD
[1.1.2]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v1.1.0...v1.1.1
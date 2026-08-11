# Changelog

All notable changes to the Prism MCP SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Historical entries describe what was announced at release time. They are not a
current capability or security contract. In particular, earlier references to
plugin sandboxing/ABI stability, DDoS protection, and a no-unsafe-code boundary
were inaccurate; current behavior and trust boundaries are documented in
[README.md](README.md) and [SECURITY.md](SECURITY.md).

## [Unreleased]

## [3.0.1] - 2026-08-11

### Changed

- Upgraded the opt-in OpenTelemetry dependency family as one compatible set: `opentelemetry`, `opentelemetry_sdk`, and `opentelemetry-otlp` to 0.32, plus `tracing-opentelemetry` to 0.33.
- Grouped future OpenTelemetry and WebSocket dependency updates so coupled crates are tested and reviewed together.

### Fixed

- Restored MCP conformance CI for this library crate by resolving dependencies without requiring an intentionally untracked `Cargo.lock`.
- Made crates.io publication idempotent, including exact-version API checks, duplicate-publication race recovery, required request identification, and post-publication verification.
- Enabled GitHub dependency-graph-backed pull-request dependency review.

## [3.0.0] - 2026-08-09

### Added

- Native MCP 2026-07-28 stateless discovery and per-request protocol, identity, and capability metadata.
- `ProtocolMode` with dual-stack, modern-only, and legacy-only client/server behavior.
- Typed negotiated-protocol, modern discovery, result discriminant, cache, and multi-round-trip result objects.
- Automatic, bounded client handling of `input_required` results with fresh request IDs and opaque request-state preservation.
- Continuation-aware server tool handlers with request-state, input-response, and client-capability validation.
- MCP 2026 standard HTTP routing headers and header/body integrity validation.
- `x-mcp-header` tool-schema support with safe parameter encoding and server-side validation.
- Modern extension capability maps and updated implementation/icon metadata.
- Dedicated dual-protocol integration tests and a maintained protocol compatibility guide.
- Standards-track `subscriptions/listen` for HTTP request-scoped SSE and cancellable STDIO streams, including strict filters and subscription correlation metadata.
- Standard Streamable HTTP client handling for JSON-RPC results returned as either JSON or POST-scoped SSE.
- Legacy `resources/subscribe` and `resources/unsubscribe` dispatch to registered resource handlers instead of returning no-op acknowledgements.
- The opt-in `io.modelcontextprotocol/tasks` extension with durable task tools, typed client operations, status notifications, caller binding, TTL, cancellation, and multi-round input.
- Standards-defined MRTR-to-Task composition through `add_composed_task_tool`.
- Pinned official MCP conformance adapters and CI for the 2026 server-stateless and selected client scenarios.

### Changed

- The crate version is now 3.0.0 and the latest protocol constant is MCP 2026-07-28.
- `McpClient::connect` returns revision-neutral `ConnectResult`; server identity is optional.
- Modern successful results include `resultType`, server identity metadata, and explicit conservative cache policy where required.
- STDIO automatic negotiation uses a disposable modern probe so legacy initialization starts on a clean process.
- Tool, prompt, resource, and template listings are deterministic.
- Recommended transport selection always chooses standards-track HTTP; Prism's proprietary chunked/compressed helpers now require explicit `LegacyOnly` mode.
- Modern version negotiation retries once when the peer reports the requested 2026 revision as mutually supported.

### Compatibility

- MCP 2025-11-25 initialization, wire result shapes, and stateful methods remain available unchanged in dual-stack and legacy-only modes.
- Automatic downgrade occurs only after JSON-RPC `Method not found` for `server/discover`; all other failures are surfaced.
- Modern servers advertise subscription behavior implemented by HTTP and STDIO, and advertise Tasks only when a task tool is registered.

### Security

- Modern HTTP requests with mismatched routing or tool parameter headers are rejected before application dispatch.
- Malformed discovery responses and unsupported versions cannot trigger a silent legacy downgrade.

## [2.0.2] - 2026-08-07

### Added
- Shared `RequestContext` and principal model for transport-independent policy enforcement.
- Deny-by-default fine-grained RBAC with method and resource patterns.
- Enforced per-principal/per-method token-bucket rate limiting.
- Optional OTLP/OpenTelemetry tracing with W3C HTTP trace propagation.
- TLS 1.3 mutual-authentication configuration for HTTP clients and servers.
- Round-robin endpoint pooling with circuit state and idempotency-aware failover.
- Criterion benchmarks for common server dispatch and recoverable endpoint failover.

### Changed
- Production documentation now distinguishes implemented controls from planned sandboxing and optional host-level CPU affinity.
- Consolidated project documentation around one maintained index; merged and removed redundant development, plugin, AI-configuration, CI-logging, HTML landing-page, and badge-status documents.
- Documentation validation now detects exact duplicate Markdown files, broken local links, and references to removed documents.
- Benchmark CI fails when Criterion output cannot be parsed instead of publishing fabricated fallback data.
- Dependency updates now arrive as reviewable Dependabot pull requests instead of direct scheduled commits.
- Dependency security checks are consolidated into one least-privilege workflow with retained lockfile, audit, and supply-chain evidence.
- Removed duplicate audit jobs and the misleading empty SARIF upload.

### Fixed
- HTTP server transports now retain and install the `McpServer` request handler before accepting traffic.
- Network-negative tests no longer assume that `localhost:3000` is unused.
- The quick-start doctest now compiles when default transport features are disabled.
- Schema completion handling passes current stable Rust Clippy checks.

## [2.0.1] - 2026-05-16

### Fixed
- Accept JSON-RPC notifications without an `id` on the HTTP `/mcp` endpoint, including `notifications/initialized`.
- Reject unknown HTTP JSON-RPC notifications with a clear `400 Bad Request` response.
- Clean up HTTP SSE test warnings so `cargo clippy --features http --lib --tests -- -D warnings` passes.

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
  - Enhanced the former standalone plugin component reference with clearer examples
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
- **📚 Enhanced Documentation**: Updated the former standalone development guide with:
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

## [2.0.0] - 2026-02-25

### Added
- Circuit breaker pattern for fault tolerance
- Adaptive retry policies with exponential backoff and jitter
- Multi-level health monitoring system
- Streaming HTTP/2 transport with multiplexing and server push
- Adaptive compression (Gzip, Brotli, Zstd) based on content analysis
- Schema introspection for runtime capability discovery
- Batch operations support for bulk request processing
- Hot-reloadable native plugin system (without a stable cross-toolchain ABI)
- Production observability with structured logging and metrics
- Convenience methods for `ContentBlock::text()`, `ContentBlock::image()`, `ContentBlock::audio()`, `ContentBlock::resource_link()`
- Convenience methods for `ToolResult::text()`, `ToolResult::error()`, `ToolResult::with_content()`, `ToolResult::with_structured()`
- Comprehensive plugin architecture for runtime component loading
- Support for MCP 2025-11-25 specification
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
- Added native plugin loading for trusted in-process extensions
- Added input validation for all protocol messages
- Kept core request paths in safe Rust; native plugin FFI uses unsafe code
- TLS 1.3 support with mTLS capabilities
- JWT/OAuth2 authentication support
- Added authentication and transport-security primitives; application-level protection remained required

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
- **MCP Version**: 2025-11-25
- **License**: MIT
- **Dependencies**: Minimal, with optional features for additional transports

### Contributors
- Prismworks AI Team

[Unreleased]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v3.0.1...HEAD
[3.0.1]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v2.0.2...v3.0.0
[2.0.2]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v2.0.1...v2.0.2
[2.0.1]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v1.1.2...v2.0.0
[1.1.2]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v0.1.4...v1.0.0
[0.1.4]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/prismworks-ai/prism-mcp-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/prismworks-ai/prism-mcp-rs/releases/tag/v0.1.0

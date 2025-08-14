# Makefile for Prism MCP SDK
#
# This Makefile serves as the primary developer interface, providing convenient
# targets for common development tasks. It wraps complex commands in simple,
# memorable targets.
#
# For detailed documentation about the build system and development workflow,
# see DEVELOPMENT.md in the project root.
#
# ACT FOR LOCAL CI:
#   Run GitHub Actions locally without pushing:
#   act push         - Run full CI pipeline
#   act -j test      - Run test job
#   act -j clippy    - Run clippy job
#   act -l           - List all available workflows
#
# Quick usage (requires local Rust installation):
#   make quick       - Run quick validation before committing
#   make check       - Run standard CI checks before pushing
#   make full        - Run complete CI pipeline (uses Act)
#   make help        - Show all available targets

.PHONY: help check quick full fmt clippy test test-all test-features examples docs security coverage coverage-clean clean setup-hooks

# Default target
help: ## Show this help message
	@echo "MCP Protocol SDK - Local CI Commands"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Quick validation before commit
quick: ## Quick validation (format, clippy, basic tests)
	@echo "🚀 Running quick validation..."
	@$(MAKE) fmt
	@$(MAKE) clippy
	@cargo check --all-features
	@cargo test --lib
	@echo "✅ Quick validation complete!"

# Standard CI checks
check: ## Run standard CI checks (mirrors GitHub Actions)
	@echo "🚀 Running standard CI pipeline..."
	@$(MAKE) fmt
	@$(MAKE) clippy
	@cargo check --all-features
	@$(MAKE) test-all
	@$(MAKE) examples

# Run local CI with Act (sequential execution)
local-ci: ## Run sequential CI locally with Act (won't run on GitHub)
	@echo "🚀 Running local sequential CI with Act..."
	@if ! command -v act &> /dev/null; then \
		echo "❌ Act is not installed. Run 'make install-act' first."; \
		exit 1; \
	fi
	@act -W .github/workflows/ci-local.yml push
	@$(MAKE) docs
	@echo "✅ Standard CI checks complete!"

# Full CI pipeline
full: ## Run full CI pipeline using Act (GitHub Actions locally)
	@echo "🚀 Running full CI pipeline with Act..."
	@echo "💡 This runs your actual GitHub Actions workflows locally"
	@act push

# Formatting
fmt: ## Check code formatting
	@echo "🎨 Checking code formatting..."
	@cargo fmt --all -- --check

fmt-fix: ## Fix code formatting
	@echo "🎨 Fixing code formatting..."
	@cargo fmt --all

# Linting
clippy: ## Run Clippy linter
	@echo "📎 Running Clippy linter..."
	@cargo clippy --all-features -- -W clippy::all -A unused_imports -A unused_variables -A dead_code -A unused_mut -A private_interfaces -A clippy::redundant_closure -A clippy::redundant_pattern_matching -A clippy::should_implement_trait -A clippy::manual_strip -A clippy::type_complexity

clippy-fix: ## Fix Clippy suggestions automatically
	@echo "📎 Fixing Clippy suggestions..."
	@cargo clippy --all-features --fix

# Testing
test: ## Run tests with default features
	@echo "🧪 Running tests (default features)..."
	@cargo test --verbose

test-all: ## Run all test combinations
	@echo "🧪 Running all test combinations..."
	@cargo test --verbose
	@cargo test --all-features --verbose
	@cargo test --no-default-features --verbose --lib

test-features: ## Run feature-specific tests
	@echo "🔬 Running feature-specific tests..."
	@cargo test --features stdio --verbose
	@cargo test --features http --verbose
	@cargo test --features websocket --verbose
	@cargo test --features validation --verbose
	@cargo test --features validation --verbose

# Examples
examples: ## Check all examples compile
	@echo "📚 Checking examples..."
	@cargo check --example simple_server
	@cargo check --example echo_server
	@cargo check --example basic_client
	@cargo check --example database_server
	@cargo check --example http_server --features http
	@cargo check --example http_client --features http
	@cargo check --example websocket_server --features websocket
	@cargo check --example websocket_client --features websocket

# Documentation
docs: ## Generate documentation
	@echo "📖 Generating documentation..."
	@cargo doc --all-features --no-deps
	@echo "✅ Documentation generated at target/doc/prism_mcp_rs/index.html"
	@echo "🌐 After publishing: https://docs.rs/prism-mcp-rs"

docs-open: ## Generate and open documentation
	@echo "📖 Generating and opening documentation..."
	@cargo doc --all-features --no-deps --document-private-items --open

docs-rustdoc: ## Generate only rustdoc documentation
	@echo "📚 Generating rustdoc..."
	@cargo doc --all-features --no-deps

docs-check: ## Check documentation quality
	@echo "🔍 Checking documentation quality..."
	@python3 scripts/docs/check-docs-quality.py

docs-sync: docs-rustdoc docs-check ## Generate rustdoc and check quality
	@echo "✅ Documentation generated and checked"

# Security
security: ## Run security audit
	@echo "🔒 Running security audit..."
	@cargo audit

audit-fix: ## Attempt to fix security vulnerabilities
	@echo "🔒 Attempting to fix security vulnerabilities..."
	@cargo audit fix

# Dependencies
deps: ## Analyze dependencies
	@echo "📦 Analyzing dependencies..."
	@cargo tree --duplicates || echo "No duplicate dependencies found"

deps-update: ## Update dependencies
	@echo "📦 Updating dependencies..."
	@cargo update

# Coverage
coverage: ## Generate code coverage report
	@echo "📊 Generating code coverage..."
	@mkdir -p .local/reports
	@cargo llvm-cov --all-features --workspace --html --output-dir .local/reports
	@cargo llvm-cov report --lcov --output-path .local/reports/lcov.info
	@cargo llvm-cov report --cobertura --output-path .local/reports/cobertura.xml
	@cargo llvm-cov report
	@echo "Coverage report generated in .local/reports/ directory"

# Reports
reports: ## Generate coverage and benchmark reports
	@echo "📊 Generating reports using Act:"
	@echo "   act -j coverage    # Coverage report"
	@echo "   act -j benchmark   # Benchmark report"
	@echo "Or with local tools:"
	@echo "   make coverage      # Coverage with cargo-llvm-cov"
	@echo "   make bench         # Benchmarks with cargo bench"


coverage-open: ## Generate and open coverage report
	@$(MAKE) coverage
	@open .local/reports/index.html || xdg-open .local/reports/index.html || echo "Open .local/reports/index.html manually"

coverage-clean: ## Clean coverage data
	@echo "🧹 Cleaning coverage data..."
	@cargo llvm-cov clean --workspace

# Benchmarks
bench: ## Run performance benchmarks
	@echo "⚡ Running benchmarks..."
	@cargo bench

# Clean
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf .local/reports/*.html .local/reports/*.xml
	@rm -f *.profraw

# Tool installation (Act is the primary tool needed)
install-act: ## Install Act for local CI
	@echo "📦 Installing Act..."
	@if ! command -v act &> /dev/null; then \
		if [ "$$(uname)" = "Darwin" ]; then \
			brew install act; \
		else \
			curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash; \
		fi; \
	else \
		echo "✅ Act is already installed: $$(act --version)"; \
	fi
# Git hooks
setup-hooks: ## Set up Git hooks for automatic CI
	@echo "🪝 Setting up Git hooks..."
	@cp scripts/ci/pre-push .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "✅ Pre-push hook installed!"
	@echo "   Now 'git push' will automatically run comprehensive CI checks"

remove-hooks: ## Remove Git hooks
	@echo "🪝 Removing Git hooks..."
	@rm -f .git/hooks/pre-push
	@echo "✅ Pre-push hook removed"

# Development workflow commands
dev-setup: setup-hooks ## Complete development environment setup
	@echo "🚀 Development environment setup complete!"
	@echo "   💡 Make sure Act is installed: brew install act (macOS)"

commit-ready: quick ## Check if code is ready to commit
	@echo "✅ Code is ready to commit!"

push-ready: check ## Check if code is ready to push
	@echo "✅ Code is ready to push!"

# CI simulation with Act
ci-local: ## Run CI locally with Act
	@echo "🚀 Running CI locally with Act..."
	@act push

ci-quick: ## Quick CI check with Act
	@echo "🚀 Running quick CI check with Act..."
	@act -j test

ci-full: ## Full CI pipeline with Act
	@echo "🚀 Running full CI pipeline with Act..."
	@act push --matrix os:ubuntu-latest
	@echo "Common Development Workflows:"
	@echo ""
	@echo "📝 Before committing:"
	@echo "   make commit-ready"
	@echo ""
	@echo "🚀 Before pushing:"
	@echo "   make push-ready"
	@echo ""
	@echo "🔍 Daily development:"
	@echo "   make quick          # Quick validation"
	@echo "   make fmt-fix        # Fix formatting"
	@echo "   make clippy-fix     # Fix linting issues"
	@echo ""
	@echo "🧪 Testing:"
	@echo "   make test           # Basic tests"
	@echo "   make test-all       # All test combinations"
	@echo "   make coverage       # Generate coverage report"
	@echo ""
	@echo "📚 Documentation:"
	@echo "   make docs-open      # Generate and open docs"
	@echo ""
	@echo "🐳 Act for Local CI (GitHub Actions locally!):"
	@echo "   act -l              # List available workflows"
	@echo "   act -j test         # Run test job"
	@echo "   act -j clippy       # Run clippy job"
	@echo "   act push            # Run full CI pipeline"
	@echo "   act -v push         # Verbose output for debugging"
	@echo ""
	@echo "🔒 Security:"
	@echo "   make security       # Security audit"
	@echo "   make deps           # Dependency analysis"
	@echo ""
	@echo "🎯 Full validation:"
	@echo "   make check          # Standard CI checks"
	@echo "   make full           # Complete CI with Act"
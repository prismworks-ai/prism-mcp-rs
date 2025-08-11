# Makefile for MCP Protocol SDK
# Provides easy commands to run CI checks locally before pushing

.PHONY: help check quick full fmt clippy test test-all test-features examples docs security coverage coverage-clean clean install-tools setup-hooks

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
	@$(MAKE) docs
	@echo "✅ Standard CI checks complete!"

# Full CI pipeline
full: ## Run full CI pipeline including all matrix combinations
	@echo "🚀 Running full CI pipeline..."
	@./scripts/local-ci.sh --full

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
docs: ## Generate documentation (separate from build process)
	@echo "📖 Generating documentation..."
	@bash scripts/generate-docs.sh

docs-open: ## Generate and open documentation
	@echo "📖 Generating and opening documentation..."
	@bash scripts/generate-docs.sh
	@cargo doc --all-features --no-deps --document-private-items --open

docs-rustdoc: ## Generate only rustdoc documentation
	@echo "📚 Generating rustdoc..."
	@cargo doc --all-features --no-deps

docs-check: ## Check documentation quality
	@echo "🔍 Checking documentation quality..."
	@python3 scripts/docs/check-docs-quality.py

docs-sync: docs docs-check ## Generate docs with v3 headers and check quality
	@echo "✅ Documentation synchronized with v3 headers"

docs-headers: ## Update documentation headers to v3
	@echo "🏷️ Updating documentation headers to v3..."
	@python3 scripts/docs/add-doc-headers-v3.py

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

# Tool installation
install-tools: ## Install required development tools
	@echo "🔧 Installing development tools..."
	@cargo install cargo-audit || echo "cargo-audit already installed"
	@cargo install cargo-llvm-cov || echo "cargo-llvm-cov already installed"
	@rustup component add llvm-tools-preview || echo "llvm-tools-preview already installed"
	@cargo install cargo-tree || echo "cargo-tree already installed"
	@cargo install cargo-license || echo "cargo-license already installed"
	@cargo install cargo-deny || echo "cargo-deny already installed"
	@rustup component add rustfmt clippy

# Git hooks
setup-hooks: ## Set up Git hooks for automatic CI
	@echo "🪝 Setting up Git hooks..."
	@cp scripts/pre-push .git/hooks/pre-push
	@chmod +x .git/hooks/pre-push
	@echo "✅ Pre-push hook installed!"
	@echo "   Now 'git push' will automatically run CI checks"

remove-hooks: ## Remove Git hooks
	@echo "🪝 Removing Git hooks..."
	@rm -f .git/hooks/pre-push
	@echo "✅ Pre-push hook removed"

# Development workflow commands
dev-setup: install-tools setup-hooks ## Complete development environment setup
	@echo "🚀 Development environment setup complete!"

commit-ready: quick ## Check if code is ready to commit
	@echo "✅ Code is ready to commit!"

push-ready: check ## Check if code is ready to push
	@echo "✅ Code is ready to push!"

# CI simulation
ci-local: ## Run exact same checks as GitHub Actions
	@echo "🚀 Running local CI (mirrors GitHub Actions)..."
	@./scripts/local-ci.sh

ci-quick: ## Quick CI check
	@echo "🚀 Running quick CI check..."
	@./scripts/local-ci.sh --quick

ci-full: ## Full CI pipeline with all matrix combinations
	@echo "🚀 Running full CI pipeline..."
	@./scripts/local-ci.sh --full

# Release preparation
release-check: ## Comprehensive check before release
	@echo "🚀 Running release preparation checks..."
	@$(MAKE) clean
	@$(MAKE) full
	@$(MAKE) security
	@$(MAKE) coverage
	@echo "✅ Release checks complete!"

# Help for common workflows
workflow-help: ## Show common development workflows
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
	@echo "🔒 Security:"
	@echo "   make security       # Security audit"
	@echo "   make deps           # Dependency analysis"
	@echo ""
	@echo "🎯 Full validation:"
	@echo "   make ci-local       # Mirror GitHub Actions"
	@echo "   make release-check  # Pre-release validation"
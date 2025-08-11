#!/usr/bin/env bash

# Development Environment Setup for MCP Rust SDK
# Installs all necessary tools for local CI checks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_status "Setting up development environment for MCP Rust SDK..."
echo ""

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    print_error "Rust is not installed!"
    print_status "Please install Rust from: https://rustup.rs/"
    exit 1
fi

print_success "Rust is installed: $(rustc --version)"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed!"
    exit 1
fi

print_success "Cargo is installed: $(cargo --version)"

# Install rustfmt if not present
if ! rustup component list --installed | grep -q "rustfmt"; then
    print_status "Installing rustfmt..."
    if rustup component add rustfmt; then
        print_success "rustfmt installed"
    else
        print_error "Failed to install rustfmt"
        exit 1
    fi
else
    print_success "rustfmt is already installed"
fi

# Install clippy if not present
if ! rustup component list --installed | grep -q "clippy"; then
    print_status "Installing clippy..."
    if rustup component add clippy; then
        print_success "clippy installed"
    else
        print_error "Failed to install clippy"
        exit 1
    fi
else
    print_success "clippy is already installed"
fi

# Install cargo-audit for security checks
if ! command -v cargo-audit &> /dev/null; then
    print_status "Installing cargo-audit for security checks..."
    if cargo install cargo-audit; then
        print_success "cargo-audit installed"
    else
        print_warning "Failed to install cargo-audit (not critical)"
    fi
else
    print_success "cargo-audit is already installed"
fi

# Install cargo-llvm-cov for code coverage
if ! command -v cargo-llvm-cov &> /dev/null; then
    print_status "Installing cargo-llvm-cov for code coverage..."
    if cargo install cargo-llvm-cov; then
        print_success "cargo-llvm-cov installed"
    else
        print_warning "Failed to install cargo-llvm-cov (not critical)"
    fi
else
    print_success "cargo-llvm-cov is already installed"
fi

# Install llvm-tools-preview component
if ! rustup component list --installed | grep -q "llvm-tools"; then
    print_status "Installing llvm-tools-preview..."
    if rustup component add llvm-tools-preview; then
        print_success "llvm-tools-preview installed"
    else
        print_warning "Failed to install llvm-tools-preview (required for coverage)"
    fi
else
    print_success "llvm-tools-preview is already installed"
fi

# Install cargo-deny for dependency checking
if ! command -v cargo-deny &> /dev/null; then
    print_status "Installing cargo-deny for dependency checks..."
    if cargo install cargo-deny; then
        print_success "cargo-deny installed"
    else
        print_warning "Failed to install cargo-deny (not critical)"
    fi
else
    print_success "cargo-deny is already installed"
fi

# Check if git hooks are set up
if [ ! -f ".git/hooks/pre-commit" ] || [ ! -x ".git/hooks/pre-commit" ]; then
    print_warning "Pre-commit hook is not set up or not executable"
    print_status "The pre-commit hook should have been created automatically"
else
    print_success "Pre-commit hook is set up and executable"
fi

echo ""
print_success " Development environment setup complete!"
echo ""
print_status "Available commands:"
print_status "  • Run full CI checks: ./scripts/ci-check.sh"
print_status "  • Run quick checks:   ./scripts/ci-check.sh --quick"
print_status "  • Format code:        cargo fmt --all"
print_status "  • Fix clippy issues:  cargo clippy --all-features --fix"
print_status "  • Run tests:          cargo test --all-features"
print_status "  • Security audit:     cargo audit"
print_status "  • Code coverage:      cargo llvm-cov --all-features --html"
echo ""
print_status "The pre-commit hook will automatically run essential checks before each commit."
print_status "For full CI validation, run: ./scripts/ci-check.sh"
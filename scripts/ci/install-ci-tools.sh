#!/bin/bash

# Install CI Tools Script
# Installs all tools required for complete local CI parity with GitHub Actions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[✓]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[⚠]${NC} $1"; }
print_error() { echo -e "${RED}[✗]${NC} $1"; }

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}      CI Tools Installation Script${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed. Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Function to install tool if not present
install_tool() {
    local tool_name=$1
    local package_name=${2:-$1}
    local extra_steps=$3
    
    if command -v "$tool_name" &> /dev/null; then
        print_success "$tool_name is already installed"
    else
        print_status "Installing $tool_name..."
        if cargo install "$package_name"; then
            print_success "$tool_name installed successfully"
            if [[ -n "$extra_steps" ]]; then
                eval "$extra_steps"
            fi
        else
            print_error "Failed to install $tool_name"
            return 1
        fi
    fi
}

# Function to install Rust toolchain
install_toolchain() {
    local toolchain=$1
    
    if rustup toolchain list | grep -q "$toolchain"; then
        print_success "Rust $toolchain is already installed"
    else
        print_status "Installing Rust $toolchain..."
        if rustup toolchain install "$toolchain"; then
            print_success "Rust $toolchain installed successfully"
        else
            print_warning "Failed to install Rust $toolchain"
        fi
    fi
}

echo "Installing essential CI tools..."
echo ""

# Essential tools for security and quality
print_status "Installing security and quality tools..."
install_tool "cargo-audit" "cargo-audit"
install_tool "cargo-deny" "cargo-deny"
install_tool "cargo-outdated" "cargo-outdated"
install_tool "cargo-vet" "cargo-vet"

# Coverage tools
print_status "Installing coverage tools..."
install_tool "cargo-llvm-cov" "cargo-llvm-cov" "rustup component add llvm-tools-preview"

# Binary analysis tools
print_status "Installing binary analysis tools..."
install_tool "cargo-bloat" "cargo-bloat"

# Optional but useful tools
print_status "Installing optional tools..."
install_tool "cargo-license" "cargo-license"
install_tool "cargo-tree" "cargo-tree" # Usually built-in with newer cargo
install_tool "cargo-machete" "cargo-machete" # Find unused dependencies

echo ""
echo "Installing Rust toolchains for multi-version testing..."
echo ""

# Install multiple Rust versions for testing
install_toolchain "stable"
install_toolchain "beta"
install_toolchain "nightly"

# Install MSRV if specified in Cargo.toml
if grep -q 'rust-version' Cargo.toml; then
    MSRV=$(grep 'rust-version' Cargo.toml | head -1 | cut -d'"' -f2)
    if [[ -n "$MSRV" ]]; then
        print_status "Installing MSRV: $MSRV"
        install_toolchain "$MSRV"
    fi
fi

echo ""
echo "Verifying installations..."
echo ""

# Verify all tools
TOOLS_OK=true

for tool in cargo-audit cargo-deny cargo-outdated cargo-llvm-cov cargo-bloat; do
    if command -v "$tool" &> /dev/null; then
        version=$($tool --version 2>/dev/null | head -1 || echo "unknown")
        print_success "$tool: $version"
    else
        print_warning "$tool: not installed"
        TOOLS_OK=false
    fi
done

echo ""
echo "Rust toolchains installed:"
rustup toolchain list

echo ""
if [[ "$TOOLS_OK" == "true" ]]; then
    print_success "All essential CI tools are installed!"
else
    print_warning "Some tools failed to install, but you can still run CI"
fi

echo ""
echo "You can now run:"
echo "  ./scripts/ci/local-ci-enhanced.sh       # Standard CI checks"
echo "  ./scripts/ci/local-ci-enhanced.sh --full # Complete testing"
echo "  ./scripts/ci/local-ci-full.sh           # Full GitHub Actions parity"
echo ""
echo "For workflow simulation:"
echo "  ./scripts/ci/local-ci-full.sh --simulate-workflow=ci"
echo "  ./scripts/ci/local-ci-full.sh --simulate-workflow=security"
echo "  ./scripts/ci/local-ci-full.sh --simulate-workflow=benchmarks"
echo "  ./scripts/ci/local-ci-full.sh --simulate-workflow=dependencies"

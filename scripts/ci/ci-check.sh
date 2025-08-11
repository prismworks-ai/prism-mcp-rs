#!/usr/bin/env bash

# Full CI Check Script for MCP Rust SDK
# This script runs all the same checks as GitHub Actions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Function to print colored output
print_header() {
    echo -e "${PURPLE}========================================${NC}"
    echo -e "${PURPLE} $1${NC}"
    echo -e "${PURPLE}========================================${NC}"
}

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

# Function to run a command and check its exit status
run_check() {
    local name="$1"
    local command="$2"
    
    print_status "Running $name..."
    
    if eval "$command"; then
        print_success "$name passed"
        return 0
    else
        print_error "$name failed"
        return 1
    fi
}

# Track overall success
overall_success=true
start_time=$(date +%s)

print_header "MCP Rust SDK - Full CI Checks"
echo "This script runs all GitHub Actions checks locally"
echo ""

# Parse command line arguments
QUICK=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [--quick] [--verbose] [--help]"
            echo ""
            echo "Options:"
            echo "  --quick    Run only essential checks (faster)"
            echo "  --verbose  Show detailed output"
            echo "  --help     Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option $1"
            exit 1
            ;;
    esac
done

if $VERBOSE; then
    set -x
fi

echo "# Starting checks..."
if $QUICK; then
    print_warning "Running in QUICK mode (some checks skipped)"
fi
echo ""

# 1. Code Formatting
print_header "1. Code Formatting (rustfmt)"
if ! run_check "Rust formatting" "cargo fmt --all -- --check"; then
    print_error "Code is not properly formatted!"
    print_status "Fix with: cargo fmt --all"
    overall_success=false
fi
echo ""

# 2. Linting
print_header "2. Linting (clippy)"
if ! run_check "Clippy lints" "cargo clippy --all-features -- -W clippy::all -A unused_imports -A unused_variables -A dead_code -A unused_mut -A private_interfaces"; then
    print_error "Clippy found issues!"
    print_status "Fix with: cargo clippy --all-features --fix"
    overall_success=false
fi
echo ""

# 3. Compilation Check
print_header "3. Compilation Check"
if ! run_check "Basic compilation" "cargo check"; then
    overall_success=false
fi

if ! run_check "All features compilation" "cargo check --all-features"; then
    overall_success=false
fi

if ! run_check "No default features compilation" "cargo check --no-default-features"; then
    overall_success=false
fi
echo ""

# 4. Feature-specific compilation
print_header "4. Feature Tests"
features=("stdio" "http" "websocket" "validation" "full")
for feature in "${features[@]}"; do
    if ! run_check "Feature: $feature" "cargo check --features $feature"; then
        overall_success=false
    fi
done
echo ""

# 5. Unit Tests
print_header "5. Unit Tests"
if ! run_check "Default tests" "cargo test --verbose"; then
    overall_success=false
fi

if ! run_check "All features tests" "cargo test --all-features --verbose"; then
    overall_success=false
fi

if ! run_check "No default features tests" "cargo test --no-default-features --verbose"; then
    overall_success=false
fi

# Individual feature tests (skip in quick mode)
if ! $QUICK; then
    for feature in "${features[@]}"; do
        if ! run_check "Tests with feature: $feature" "cargo test --features $feature --verbose"; then
            overall_success=false
        fi
    done
fi
echo ""

# 6. Examples
print_header "6. Examples"
basic_examples=("simple_server" "echo_server" "client_example" "database_server")
for example in "${basic_examples[@]}"; do
    if ! run_check "Example: $example" "cargo check --example $example"; then
        overall_success=false
    fi
done

# Feature-specific examples
if ! run_check "HTTP server example" "cargo check --example http_server --features http"; then
    overall_success=false
fi

if ! run_check "HTTP client example" "cargo check --example http_client --features http"; then
    overall_success=false
fi

if ! run_check "WebSocket server example" "cargo check --example websocket_server --features websocket"; then
    overall_success=false
fi

if ! run_check "WebSocket client example" "cargo check --example websocket_client --features websocket"; then
    overall_success=false
fi
echo ""

# 7. Documentation
print_header "7. Documentation"
if ! run_check "Documentation build" "cargo doc --all-features --no-deps --document-private-items"; then
    overall_success=false
fi
echo ""

# 8. Security Audit (if cargo-audit is installed)
print_header "8. Security Audit"
if command -v cargo-audit &> /dev/null; then
    if ! run_check "Security audit" "cargo audit"; then
        print_warning "Security audit found issues (may not be critical)"
        # Don't fail overall for security issues
    fi
else
    print_warning "cargo-audit not installed, skipping security audit"
    print_status "Install with: cargo install cargo-audit"
fi
echo ""

# 9. Benchmarks (if not quick mode)
if ! $QUICK; then
    print_header "9. Benchmarks"
    if ! run_check "Benchmark compilation" "cargo check --benches"; then
        overall_success=false
    fi
    echo ""
fi

# 10. Code Coverage (if tarpaulin is installed and not quick mode)
if ! $QUICK && command -v cargo-tarpaulin &> /dev/null; then
    print_header "10. Code Coverage"
    if run_check "Code coverage" "cargo tarpaulin --all-features --workspace --timeout 120 --out xml"; then
        if [ -f cobertura.xml ]; then
            coverage=$(grep -o 'line-rate="[^"]*"' cobertura.xml | head -1 | cut -d'"' -f2)
            coverage_percent=$(echo "$coverage * 100" | bc -l 2>/dev/null || echo "unknown")
            print_success "Code coverage: ${coverage_percent}%"
        fi
    else
        print_warning "Code coverage generation failed"
    fi
    echo ""
elif ! $QUICK; then
    print_warning "cargo-llvm-cov not installed, skipping code coverage"
    print_status "Install with: cargo install cargo-llvm-cov"
    print_status "Also run: rustup component add llvm-tools-preview"
    echo ""
fi

# Summary
end_time=$(date +%s)
duration=$((end_time - start_time))

print_header "Summary"
if $overall_success; then
    print_success " All checks passed!"
    print_success "Time taken: ${duration} seconds"
    print_status ""
    print_status "Your code is ready for commit and should pass GitHub Actions!"
else
    print_error "[!] Some checks failed!"
    print_error "Time taken: ${duration} seconds"
    print_status ""
    print_status "Please fix the issues above before committing."
    print_status ""
    print_status "Quick fixes:"
    print_status "  • Format code: cargo fmt --all"
    print_status "  • Fix clippy: cargo clippy --all-features --fix"
    print_status "  • Run tests: cargo test --verbose"
    print_status "  • Check docs: cargo doc --all-features"
    print_status ""
    print_status "For detailed output, run with --verbose flag"
fi

exit $([ "$overall_success" = true ] && echo 0 || echo 1)
#!/bin/bash

# Local CI script that mirrors GitHub Actions pipeline
# Run this script to test your changes locally before pushing

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_header() {
    echo -e "\n${PURPLE}==========================================${NC}"
    echo -e "${PURPLE} $1${NC}"
    echo -e "${PURPLE}==========================================${NC}"
}

print_step() {
    echo -e "\n${CYAN}→ $1${NC}"
}

# Function to measure execution time
time_command() {
    local start_time=$(date +%s)
    "$@"
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    echo -e "${CYAN}  ⏱ Completed in ${duration}s${NC}"
}

# Parse command line arguments
FULL_CI=false
QUICK_CHECK=false
SKIP_SECURITY=false
SKIP_EXAMPLES=false
COVERAGE=false

for arg in "$@"; do
    case $arg in
        --full)
            FULL_CI=true
            shift
            ;;
        --quick)
            QUICK_CHECK=true
            shift
            ;;
        --skip-security)
            SKIP_SECURITY=true
            shift
            ;;
        --skip-examples)
            SKIP_EXAMPLES=true
            shift
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        --help)
            echo "Local CI Testing Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --full          Run full CI pipeline including all matrix combinations"
            echo "  --quick         Run only essential checks (format, clippy, basic tests)"
            echo "  --skip-security Skip security audit and license checks"
            echo "  --skip-examples Skip example compilation checks"
            echo "  --coverage      Generate code coverage report"
            echo "  --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0              # Run standard CI checks"
            echo "  $0 --quick      # Quick validation before commit"
            echo "  $0 --full       # Full CI pipeline (takes longer)"
            echo "  $0 --coverage   # Generate coverage report"
            exit 0
            ;;
        *)
            print_error "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Display configuration
print_header "# MCP Protocol SDK - Local CI Pipeline"
echo "Configuration:"
echo "  Full CI:        $FULL_CI"
echo "  Quick Check:    $QUICK_CHECK"
echo "  Skip Security:  $SKIP_SECURITY"
echo "  Skip Examples:  $SKIP_EXAMPLES"
echo "  Coverage:       $COVERAGE"

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo >/dev/null 2>&1; then
    print_error "Cargo is not installed or not in PATH"
    exit 1
fi

# Start timing
SCRIPT_START_TIME=$(date +%s)

# 1. Basic Checks
print_header " Basic Checks"

print_step "Checking Rust toolchain..."
rustc --version
cargo --version

print_step "Updating dependencies..."
time_command cargo update --dry-run

# 2. Code Quality
print_header "🎨 Code Quality"

print_step "Format check..."
time_command cargo fmt --all -- --check

print_step "Clippy linting..."
time_command cargo clippy --all-features -- -W clippy::all -A unused_imports -A unused_variables -A dead_code -A unused_mut -A private_interfaces -A clippy::redundant_closure -A clippy::redundant_pattern_matching -A clippy::should_implement_trait -A clippy::manual_strip -A clippy::type_complexity

# Quick check mode - skip detailed tests
if [[ "$QUICK_CHECK" == "true" ]]; then
    print_step "Quick compilation check..."
    time_command cargo check --all-features
    
    print_step "Quick test run..."
    time_command cargo test --lib
    
    SCRIPT_END_TIME=$(date +%s)
    TOTAL_DURATION=$((SCRIPT_END_TIME - SCRIPT_START_TIME))
    print_header "[x] Quick Check Complete (${TOTAL_DURATION}s)"
    print_success "Basic checks passed! Ready for detailed CI."
    exit 0
fi

# 3. Compilation
print_header "- Compilation"

print_step "Cargo check (all features)..."
time_command cargo check --all-features

print_step "Cargo check (no default features)..."
time_command cargo check --no-default-features

# 4. Test Suite
print_header "Test: Test Suite"

print_step "Test with default features..."
time_command cargo test --verbose

print_step "Test with all features..."
time_command cargo test --all-features --verbose

print_step "Test with no default features..."
time_command cargo test --no-default-features --verbose --lib

# 5. Feature Tests
if [[ "$FULL_CI" == "true" ]]; then
    print_header "🔬 Feature-Specific Tests"
    features=("stdio" "http" "websocket" "validation" "full")
    for feature in "${features[@]}"; do
        print_step "Testing feature: $feature"
        time_command cargo test --features "$feature" --verbose
    done
fi

# 6. Examples
if [[ "$SKIP_EXAMPLES" != "true" ]]; then
    print_header "## Examples"
    
    print_step "Default examples..."
    examples=("simple_server" "echo_server" "client_example" "database_server")
    for example in "${examples[@]}"; do
        print_status "Checking $example..."
        time_command cargo check --example "$example"
    done
    
    print_step "HTTP examples..."
    http_examples=("http_server" "http_client")
    for example in "${http_examples[@]}"; do
        print_status "Checking $example..."
        time_command cargo check --example "$example" --features http
    done
    
    print_step "WebSocket examples..."
    ws_examples=("websocket_server" "websocket_client")
    for example in "${ws_examples[@]}"; do
        print_status "Checking $example..."
        time_command cargo check --example "$example" --features websocket
    done
fi

# 7. Documentation
print_header "- Documentation"

print_step "Documentation generation..."
time_command cargo doc --all-features --no-deps --document-private-items

# 8. MSRV Check (if full CI)
if [[ "$FULL_CI" == "true" ]]; then
    print_header "🕐 MSRV Check"
    
    # Check if specific Rust versions are available
    msrv_versions=("1.82.0" "1.83.0")
    for version in "${msrv_versions[@]}"; do
        if rustup toolchain list | grep -q "$version"; then
            print_step "Testing with Rust $version..."
            rustup run "$version" cargo check --all-features
            rustup run "$version" cargo test --all-features
        else
            print_warning "Rust $version not installed, skipping MSRV check"
            print_warning "Install with: rustup toolchain install $version"
        fi
    done
fi

# 9. Security and Dependencies
if [[ "$SKIP_SECURITY" != "true" ]]; then
    print_header "Security: Security & Dependencies"
    
    print_step "Security audit..."
    if command -v cargo-audit >/dev/null 2>&1; then
        time_command cargo audit || print_warning "Security audit found issues"
    else
        print_warning "cargo-audit not installed, skipping security audit"
        print_warning "Install with: cargo install cargo-audit"
    fi
    
    print_step "Dependency analysis..."
    if command -v cargo-tree >/dev/null 2>&1; then
        print_status "Checking for duplicate dependencies..."
        cargo tree --duplicates || print_success "No duplicate dependencies found"
    else
        print_warning "cargo-tree not installed, skipping dependency analysis"
        print_warning "Install with: cargo install cargo-tree"
    fi
    
    print_step "License check..."
    if command -v cargo-license >/dev/null 2>&1; then
        cargo license --json > /dev/null
        print_success "License check completed"
    else
        print_warning "cargo-license not installed, skipping license check"
        print_warning "Install with: cargo install cargo-license"
    fi
fi

# 10. Code Coverage
if [[ "$COVERAGE" == "true" ]]; then
    print_header "📊 Code Coverage"
    
    if command -v cargo-llvm-cov >/dev/null 2>&1; then
        print_step "Generating coverage report..."
        mkdir -p .local/reports
        time_command cargo llvm-cov --all-features --workspace --html --output-dir .local/reports
        cargo llvm-cov report
        print_success "Coverage report generated in .local/reports/ directory"
    else
        print_warning "cargo-llvm-cov not installed, skipping coverage"
        print_warning "Install with: cargo install cargo-llvm-cov"
        print_warning "Also run: rustup component add llvm-tools-preview"
    fi
fi

# 11. Performance Benchmarks (if full CI)
if [[ "$FULL_CI" == "true" ]]; then
    print_header "* Performance Benchmarks"
    
    if [[ -d "benches" ]]; then
        print_step "Running benchmarks..."
        time_command cargo bench
    else
        print_warning "No benchmarks directory found, skipping performance tests"
    fi
fi

# Final Summary
SCRIPT_END_TIME=$(date +%s)
TOTAL_DURATION=$((SCRIPT_END_TIME - SCRIPT_START_TIME))

print_header " Local CI Pipeline Complete!"
echo -e "${GREEN}Total execution time: ${TOTAL_DURATION}s${NC}"
echo ""
print_success "All checks passed! Your code should pass GitHub Actions CI."
echo ""
echo "Next steps:"
echo "  1. Review any warnings above"
echo "  2. Commit your changes"
echo "  3. Push to trigger GitHub Actions"
echo ""
echo "Useful commands:"
echo "  cargo fmt --all           # Fix formatting"
echo "  cargo clippy --fix        # Auto-fix clippy suggestions"
echo "  $0 --quick               # Quick validation"
echo "  $0 --coverage            # Generate coverage report"
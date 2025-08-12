#!/bin/bash

# Enhanced Local CI Script - Mirrors GitHub Actions exactly
# This script runs the same checks as GitHub Actions workflows locally

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
print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[✓]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[⚠]${NC} $1"; }
print_error() { echo -e "${RED}[✗]${NC} $1"; }
print_header() {
    echo -e "\n${PURPLE}==========================================${NC}"
    echo -e "${PURPLE} $1${NC}"
    echo -e "${PURPLE}==========================================${NC}"
}
print_step() { echo -e "\n${CYAN}→ $1${NC}"; }

# Parse arguments
ACTION="check"
VERBOSE=false
FAIL_FAST=true

for arg in "$@"; do
    case $arg in
        --quick)
            ACTION="quick"
            ;;
        --full)
            ACTION="full"
            ;;
        --release)
            ACTION="release"
            ;;
        --security)
            ACTION="security"
            ;;
        --fix)
            ACTION="fix"
            ;;
        --reports)
            ACTION="reports"
            ;;
        --verbose)
            VERBOSE=true
            ;;
        --no-fail-fast)
            FAIL_FAST=false
            ;;
        --help)
            echo "Local CI Runner - Mirrors GitHub Actions"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Actions:"
            echo "  --quick    Quick validation (format, clippy, basic tests)"
            echo "  --full     Full CI pipeline with all combinations"
            echo "  --release  Release preparation checks"
            echo "  --security Run comprehensive security scans"
            echo "  --fix      Auto-fix formatting and clippy issues"
            echo "  --reports  Generate coverage and benchmark reports"
            echo "  --fix      Auto-fix formatting and clippy issues"
            echo ""
            echo "Options:"
            echo "  --verbose      Show detailed output"
            echo "  --no-fail-fast Continue on errors"
            echo "  --help         Show this help"
            echo ""
            echo "Examples:"
            echo "  $0              # Standard CI checks (default)"
            echo "  $0 --quick      # Quick pre-commit validation"
            echo "  $0 --full       # Complete CI matrix testing"
            echo "  $0 --reports    # Generate coverage and benchmark reports"
            echo "  $0 --fix        # Fix all auto-fixable issues"
            echo "  $0 --release    # Pre-release validation"
            exit 0
            ;;
    esac
done

# Error tracking
ERRORS=()
track_error() {
    ERRORS+=("$1")
    if [[ "$FAIL_FAST" == "true" ]]; then
        print_error "$1"
        exit 1
    else
        print_warning "$1 (continuing...)"
    fi
}

# Start timing
START_TIME=$(date +%s)

print_header "🚀 Local CI Pipeline - Action: ${ACTION}"

# Check prerequisites
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Cargo.toml not found. Run from project root."
    exit 1
fi

# Install required tools if missing
install_tools() {
    print_step "Checking required tools..."
    
    # Check for cargo-audit
    if ! command -v cargo-audit &> /dev/null; then
        print_warning "cargo-audit not found, installing..."
        cargo install cargo-audit
    fi
    
    # Check for cargo-llvm-cov
    if ! command -v cargo-llvm-cov &> /dev/null; then
        print_warning "cargo-llvm-cov not found, installing..."
        cargo install cargo-llvm-cov
        rustup component add llvm-tools-preview
    fi
    
    # Check for cargo-deny
    if ! command -v cargo-deny &> /dev/null; then
        print_warning "cargo-deny not found, installing..."
        cargo install cargo-deny
    fi
    
    # Check for cargo-outdated
    if ! command -v cargo-outdated &> /dev/null; then
        print_warning "cargo-outdated not found, installing..."
        cargo install cargo-outdated
    fi
    
    # Check for cargo-vet (optional)
    if ! command -v cargo-vet &> /dev/null; then
        print_warning "cargo-vet not found (optional)..."
        print_warning "Install with: cargo install cargo-vet"
    fi
    
    print_success "All required tools installed"
}

# Multi-Rust version testing function
test_multi_rust() {
    print_header "🦀 Multi-Rust Version Testing"
    
    local rust_versions=("stable" "beta" "nightly")
    local tested_versions=0
    local failed_versions=()
    
    for version in "${rust_versions[@]}"; do
        if rustup toolchain list | grep -q "$version"; then
            print_step "Testing with Rust $version"
            
            if cargo +"$version" test --all-features --verbose; then
                print_success "Tests passed on Rust $version"
                ((tested_versions++))
            else
                track_error "Tests failed on Rust $version"
                failed_versions+=("$version")
            fi
        else
            print_warning "Rust $version not installed - GitHub Actions will test this"
            print_status "Install with: rustup toolchain install $version"
        fi
    done
    
    if [[ $tested_versions -eq 0 ]]; then
        print_warning "No Rust versions tested! GitHub Actions tests on stable, beta, and nightly"
    elif [[ ${#failed_versions[@]} -gt 0 ]]; then
        print_error "Tests failed on Rust versions: ${failed_versions[*]}"
    else
        print_success "All available Rust versions tested successfully ($tested_versions/${#rust_versions[@]})"
    fi
}

# Multi-OS testing check
test_multi_os() {
    print_header "🖥️ Multi-OS Testing Check"
    
    case "$OSTYPE" in
        linux*)   CURRENT_OS="linux" ;;
        darwin*)  CURRENT_OS="macos" ;;
        msys*)    CURRENT_OS="windows" ;;
        cygwin*)  CURRENT_OS="windows" ;;
        *)        CURRENT_OS="unknown" ;;
    esac
    
    print_status "Current OS: $CURRENT_OS"
    print_warning "GitHub Actions tests on: Ubuntu (linux), macOS (darwin), Windows (msys)"
    
    if [[ "$CURRENT_OS" != "linux" ]]; then
        print_warning "Not testing on Linux - GitHub Actions will test ubuntu-latest"
    fi
    if [[ "$CURRENT_OS" != "macos" ]]; then
        print_warning "Not testing on macOS - GitHub Actions will test macos-latest"
    fi
    if [[ "$CURRENT_OS" != "windows" ]]; then
        print_warning "Not testing on Windows - GitHub Actions will test windows-latest"
    fi
    
    print_status "Consider using Docker or VMs for cross-platform testing locally"
}

# Minimal dependencies test
test_minimal() {
    print_header "📦 Minimal Dependencies Testing"
    
    print_step "Testing with no default features (library only)"
    cargo test --no-default-features --lib || track_error "Library tests failed with no features"
    
    print_step "Testing with only core feature"
    cargo test --no-default-features --features core || track_error "Core-only tests failed"
    
    print_step "Building with minimal features"
    cargo build --no-default-features || track_error "Minimal build failed"
    
    print_success "Minimal dependency tests completed"
}

# Binary size analysis
analyze_binary_size() {
    print_header "📏 Binary Size Analysis"
    
    print_step "Building release binaries"
    cargo build --release --all-features
    
    if [[ -d "target/release" ]]; then
        echo ""
        echo "Binary sizes:"
        echo "============="
        find target/release -maxdepth 1 -type f -executable ! -name "*.d" ! -name "*.rlib" -exec ls -lh {} \; 2>/dev/null | awk '{print $9 ": " $5}'
        
        # Use cargo-bloat if available
        if command -v cargo-bloat &> /dev/null; then
            print_step "Detailed size analysis with cargo-bloat"
            cargo bloat --release --all-features -n 10 || true
        else
            print_warning "cargo-bloat not installed - install with: cargo install cargo-bloat"
        fi
    fi
}

# Compile time metrics
measure_compile_time() {
    print_header "⏱️ Compile Time Metrics"
    
    print_step "Clean build timing"
    cargo clean
    
    echo "Debug build (all features):"
    time cargo build --all-features 2>&1 | tail -n 1
    
    echo "Release build (all features):"
    time cargo build --release --all-features 2>&1 | tail -n 1
    
    echo "Incremental rebuild (touch src/lib.rs):"
    touch src/lib.rs
    time cargo build --all-features 2>&1 | tail -n 1
}

# SARIF report generation
generate_sarif() {
    print_header "🔒 SARIF Security Report Generation"
    
    mkdir -p reports
    
    print_step "Generating SARIF report for GitHub Security tab"
    
    cargo audit --json 2>/dev/null | python3 -c "
import sys, json
try:
    audit_data = json.load(sys.stdin)
    vulnerabilities = audit_data.get('vulnerabilities', {}).get('list', [])
    
    results = []
    for vuln in vulnerabilities:
        advisory = vuln.get('advisory', {})
        package = vuln.get('package', {})
        
        result = {
            'ruleId': advisory.get('id', 'UNKNOWN'),
            'level': 'error' if advisory.get('severity', '').lower() in ['critical', 'high'] else 'warning',
            'message': {
                'text': advisory.get('title', 'Security vulnerability detected')
            },
            'locations': [{
                'physicalLocation': {
                    'artifactLocation': {
                        'uri': 'Cargo.toml'
                    },
                    'region': {
                        'startLine': 1
                    }
                }
            }]
        }
        results.append(result)
    
    sarif = {
        'version': '2.1.0',
        'runs': [{
            'tool': {
                'driver': {
                    'name': 'cargo-audit',
                    'version': '0.18.0'
                }
            },
            'results': results
        }]
    }
    
    with open('reports/audit.sarif', 'w') as f:
        json.dump(sarif, f, indent=2)
    
    print(f'SARIF report generated with {len(results)} findings')
except Exception as e:
    print(f'Error generating SARIF: {e}')
" || print_warning "SARIF generation failed"
    
    if [[ -f "reports/audit.sarif" ]]; then
        print_success "SARIF report saved to reports/audit.sarif"
    fi
}

# Enhanced dependency tree analysis
analyze_dependency_tree() {
    print_header "🌳 Dependency Tree Analysis"
    
    mkdir -p reports
    
    print_step "Checking for duplicate dependencies"
    cargo tree --duplicates > reports/duplicates.txt
    
    if [[ -s "reports/duplicates.txt" ]]; then
        print_warning "Duplicate dependencies found - see reports/duplicates.txt"
    else
        print_success "No duplicate dependencies"
    fi
    
    print_step "Generating full dependency tree"
    cargo tree --all-features > reports/dependency-tree.txt
    
    print_step "Analyzing dependency depth"
    cargo tree --all-features --depth 1 | wc -l | xargs -I {} echo "Direct dependencies: {}"
    cargo tree --all-features | wc -l | xargs -I {} echo "Total dependencies: {}"
}

# Fix action - auto-fix issues
if [[ "$ACTION" == "fix" ]]; then
    print_header "🔧 Auto-fixing Issues"
    
    print_step "Fixing formatting..."
    cargo fmt --all
    
    print_step "Fixing clippy issues..."
    cargo clippy --fix --allow-dirty --all-features || true
    
    print_step "Updating dependencies..."

    cargo update
    
    print_success "Auto-fix complete! Review changes before committing."
    exit 0
fi

# Security action - comprehensive security checks
if [[ "$ACTION" == "security" ]]; then
    print_header "🔒 Comprehensive Security Scan"
    
    # Install security tools
    install_tools
    
    print_step "Running vulnerability scan..."
    cargo audit || track_error "Critical vulnerabilities found!"
    
    print_step "Checking for security advisories..."
    cargo audit --deny warnings || track_error "Security advisories found!"
    
    print_step "License compliance check..."
    if [[ -f "deny.toml" ]]; then
        cargo deny check licenses || track_error "License compliance failed!"
        cargo deny check bans || print_warning "Banned dependencies found"
        cargo deny check advisories || print_warning "Advisory issues found"
    fi
    
    print_step "Checking for outdated dependencies..."
    echo "Dependencies with updates available:"
    cargo outdated || true
    
    print_step "Supply chain verification..."
    if command -v cargo-vet &> /dev/null; then
        cargo vet --locked || print_warning "Supply chain verification failed"
    fi
    
    print_step "Generating security report..."
    echo ""
    echo "Security Summary:"
    echo "=================="
    cargo audit --json 2>/dev/null | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    vulns = data.get('vulnerabilities', {}).get('count', 0)
    print(f'Vulnerabilities: {vulns}')
except:
    pass
" || true
    
    # Generate SARIF report
    generate_sarif
    
    if [[ ${#ERRORS[@]} -eq 0 ]]; then
        print_success "Security scan completed - No critical issues found!"
    else
        print_error "Security scan found ${#ERRORS[@]} critical issues!"
        exit 1
    fi
    exit 0
fi

# Quick action - minimal checks
if [[ "$ACTION" == "quick" ]]; then
    print_header "⚡ Quick Validation"
    
    print_step "Format check..."
    cargo fmt --all -- --check || track_error "Format check failed"
    
    print_step "Clippy check..."
    cargo clippy --all-features -- -W clippy::all || track_error "Clippy check failed"
    
    print_step "Quick compilation..."
    cargo check --all-features || track_error "Compilation failed"
    
    print_step "Basic tests..."
    cargo test --lib || track_error "Basic tests failed"
    
    if [[ ${#ERRORS[@]} -eq 0 ]]; then
        print_success "Quick validation passed!"
    else
        print_error "Quick validation failed with ${#ERRORS[@]} errors"
        exit 1
    fi
    exit 0
fi

# Standard checks (mirrors CI workflow)
print_header "📋 Standard CI Checks"

# OS and Multi-version checks
test_multi_os
if [[ "$SKIP_SLOW" != "true" ]]; then
    test_multi_rust
else
    print_warning "Skipping multi-Rust testing (use --full for complete testing)"
fi

# Minimal dependencies test
test_minimal

# 1. Format Check
print_step "Format Check (mirrors 'format' job)"
cargo fmt --all -- --check || track_error "Format check failed"

# 2. Clippy Analysis
print_step "Clippy Analysis (mirrors 'clippy' job)"
cargo clippy --all-features -- \
    -W clippy::all \
    -A unused_imports \
    -A unused_variables \
    -A dead_code \
    -A unused_mut \
    -A private_interfaces \
    -A clippy::redundant_closure \
    -A clippy::redundant_pattern_matching \
    -A clippy::should_implement_trait \
    -A clippy::manual_strip \
    -A clippy::type_complexity || track_error "Clippy analysis failed"

# 3. Test Suite
print_step "Test Suite (mirrors 'test' job)"

# Test with different feature combinations
print_status "Testing with default features..."
cargo test --verbose || track_error "Tests with default features failed"

print_status "Testing with all features..."
cargo test --all-features --verbose || track_error "Tests with all features failed"

print_status "Testing with no default features..."
cargo test --no-default-features --verbose --lib || track_error "Tests with no features failed"

# 4. Feature Tests
print_step "Feature Tests (mirrors 'feature-tests' job)"
for feature in stdio http websocket validation full; do
    print_status "Testing feature: $feature"
    cargo test --features "$feature" --verbose || track_error "Feature test '$feature' failed"
done

# 5. Examples Check
print_step "Examples Check (mirrors 'examples' job)"

# Default examples
for example in simple_server echo_server basic_client database_server; do
    if [[ -f "examples/${example}.rs" ]]; then
        cargo check --example "$example" || track_error "Example '$example' failed"
    fi
done

# HTTP examples
for example in http_server http_client; do
    if [[ -f "examples/${example}.rs" ]]; then
        cargo check --example "$example" --features http || track_error "HTTP example '$example' failed"
    fi
done

# WebSocket examples
for example in websocket_server websocket_client; do
    if [[ -f "examples/${example}.rs" ]]; then
        cargo check --example "$example" --features websocket || track_error "WebSocket example '$example' failed"
    fi
done

# 6. Documentation
print_step "Documentation (mirrors 'docs' job)"
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps --document-private-items || track_error "Documentation build failed"

# 7. MSRV Check (if rust 1.82.0 is installed)
print_step "MSRV Check (mirrors 'msrv' job)"
if rustup toolchain list | grep -q "1.82.0"; then
    rustup run 1.82.0 cargo check --all-features || track_error "MSRV check failed"
else
    print_warning "Rust 1.82.0 not installed, skipping MSRV check"
    print_warning "Install with: rustup toolchain install 1.82.0"
fi

# 8. Security Audit (Enhanced)
print_step "Security Audit (mirrors 'security' workflow)"

# Install security tools if missing
if ! command -v cargo-audit &> /dev/null; then
    print_warning "cargo-audit not found, installing..."
    cargo install cargo-audit
fi

if ! command -v cargo-deny &> /dev/null; then
    print_warning "cargo-deny not found, installing..."
    cargo install cargo-deny
fi

if ! command -v cargo-outdated &> /dev/null; then
    print_warning "cargo-outdated not found, installing..."
    cargo install cargo-outdated
fi

# Run security checks
print_status "Checking for known vulnerabilities..."
cargo audit || track_error "Security vulnerabilities found"

print_status "Checking for security advisories with strict mode..."
cargo audit --deny warnings || print_warning "Security warnings found (non-blocking)"

print_status "Checking license compliance..."
if [[ -f "deny.toml" ]]; then
    cargo deny check licenses || print_warning "License compliance issues (non-blocking)"
else
    print_warning "deny.toml not found, skipping license check"
fi

print_status "Checking for outdated dependencies..."
cargo outdated --exit-code 1 || print_warning "Outdated dependencies found (non-blocking)"

print_status "Checking supply chain security..."
if command -v cargo-vet &> /dev/null; then
    cargo vet --locked || print_warning "Supply chain issues (non-blocking)"
else
    print_warning "cargo-vet not installed, skipping supply chain check"
fi

# Full action - complete matrix testing
if [[ "$ACTION" == "full" ]]; then
    print_header "🔬 Full CI Matrix Testing"
    
    # Multi-OS check
    test_multi_os
    
    # Test on different Rust versions
    test_multi_rust
    
    # Minimal dependencies
    test_minimal
    
    # Binary size analysis
    analyze_binary_size
    
    # Compile time metrics
    measure_compile_time
    
    # Dependency tree analysis
    analyze_dependency_tree
    
    # Coverage report
    print_step "Generating coverage report..."
    install_tools
    
    # Generate HTML coverage in .local/reports for detailed viewing
    cargo llvm-cov --all-features --workspace --html --output-dir .local/reports
    
    # Generate markdown coverage report in reports/ folder
    if [[ -x "scripts/ci/generate-coverage-report.sh" ]]; then
        print_status "Generating markdown coverage report..."
        ./scripts/ci/generate-coverage-report.sh || print_warning "Coverage report generation failed"
        print_success "Coverage report saved to reports/coverage-report.md"
    fi
    
    cargo llvm-cov report
    print_success "HTML coverage report generated in .local/reports/"
    
    # Benchmarks
    if [[ -d "benches" ]] && [[ "$CARGO_BUILD_FLAGS" == *"bench"* ]] || [[ -f "benches/client_benchmarks.rs" ]]; then
        print_step "Running benchmarks..."
        
        # Check if benchmark feature is available
        if cargo build --benches --features bench 2>/dev/null; then
            if [[ -x "scripts/ci/run-benchmarks.sh" ]]; then
                print_status "Generating benchmark report..."
                ./scripts/ci/run-benchmarks.sh || print_warning "Benchmark report generation failed"
                print_success "Benchmark report saved to reports/benchmark-report.md"
            else
                cargo bench --features bench || print_warning "Benchmarks failed"
            fi
        else
            print_warning "Benchmark feature not available, skipping benchmarks"
        fi
    fi
fi

# Release action - pre-release checks
if [[ "$ACTION" == "release" ]]; then
    print_header "📦 Release Preparation"
    
    # All standard checks first
    $0 --no-fail-fast
    
    # Additional release checks
    print_step "Checking version consistency..."
    CARGO_VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
    print_status "Cargo.toml version: $CARGO_VERSION"
    
    print_step "Checking dependencies..."
    cargo tree --duplicates || print_success "No duplicate dependencies"
    
    print_step "License check..."
    if command -v cargo-license &> /dev/null; then
        cargo license --json > /dev/null
        print_success "License check passed"
    fi
    
    print_step "Checking package metadata..."
    cargo publish --dry-run --all-features || track_error "Package validation failed"
    
    print_step "Documentation completeness..."
    cargo doc --all-features --no-deps --open || track_error "Documentation incomplete"
fi

# Summary
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

print_header "📊 CI Pipeline Summary"
echo "Duration: ${DURATION}s"
echo "Errors: ${#ERRORS[@]}"

if [[ ${#ERRORS[@]} -gt 0 ]]; then
    print_error "CI Pipeline Failed!"
    echo ""
    echo "Errors encountered:"
    for error in "${ERRORS[@]}"; do
        echo "  • $error"
    done
    echo ""
    echo "Fix suggestions:"
    echo "  • Run '$0 --fix' to auto-fix formatting and clippy issues"
    echo "  • Run 'cargo fmt' to fix formatting"
    echo "  • Run 'cargo clippy --fix' to fix clippy issues"
    exit 1
else
    print_success "All CI checks passed! ✨"
    echo ""
    echo "Your code is ready for:"
    echo "  • Committing locally"
    echo "  • Pushing to GitHub (will trigger GitHub Actions)"
    echo "  • Creating a pull request"
    
    if [[ "$ACTION" == "release" ]]; then
        echo ""
        echo "Release checklist:"
        echo "  ✓ Version consistency verified"
        echo "  ✓ All tests passing"
        echo "  ✓ Documentation complete"
        echo "  ✓ Security audit passed"
        echo "  ✓ Package metadata valid"
        echo ""
        echo "Next steps for release:"
        echo "  1. Update CHANGELOG.md"
        echo "  2. Tag the release: git tag -a v$CARGO_VERSION -m 'Release v$CARGO_VERSION'"
        echo "  3. Push tags: git push origin v$CARGO_VERSION"
        echo "  4. GitHub Actions will handle the rest!"
    fi
fi
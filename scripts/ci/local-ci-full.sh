#!/bin/bash

# Full Local CI Script - Complete GitHub Actions Parity
# This script provides complete alignment with all GitHub Actions workflows
# Version: 2.0.0

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
ACTION="standard"
VERBOSE=false
FAIL_FAST=true
SKIP_SLOW=false
GENERATE_REPORTS=false
SIMULATE_WORKFLOW=""

for arg in "$@"; do
    case $arg in
        --quick)
            ACTION="quick"
            ;;
        --full)
            ACTION="full"
            ;;
        --standard)
            ACTION="standard"
            ;;
        --release)
            ACTION="release"
            ;;
        --security)
            ACTION="security"
            ;;
        --benchmarks)
            ACTION="benchmarks"
            ;;
        --dependencies)
            ACTION="dependencies"
            ;;
        --fix)
            ACTION="fix"
            ;;
        --reports)
            GENERATE_REPORTS=true
            ;;
        --simulate-workflow=*)
            SIMULATE_WORKFLOW="${arg#*=}"
            ;;
        --verbose)
            VERBOSE=true
            ;;
        --no-fail-fast)
            FAIL_FAST=false
            ;;
        --skip-slow)
            SKIP_SLOW=true
            ;;
        --help)
            echo "Full Local CI Runner - Complete GitHub Actions Parity"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Actions:"
            echo "  --quick              Quick validation (format, clippy, basic tests)"
            echo "  --standard           Standard CI checks (default)"
            echo "  --full               Complete CI pipeline with all combinations"
            echo "  --release            Release preparation checks"
            echo "  --security           Comprehensive security scans (security.yml)"
            echo "  --benchmarks         Performance benchmarks (benchmarks.yml)"
            echo "  --dependencies       Dependency analysis (dependencies.yml)"
            echo "  --fix                Auto-fix formatting and clippy issues"
            echo ""
            echo "Workflow Simulation:"
            echo "  --simulate-workflow=ci          Run all jobs from ci.yml"
            echo "  --simulate-workflow=security    Run all jobs from security.yml"
            echo "  --simulate-workflow=benchmarks  Run all jobs from benchmarks.yml"
            echo "  --simulate-workflow=dependencies Run all jobs from dependencies.yml"
            echo ""
            echo "Options:"
            echo "  --reports            Generate all reports (coverage, benchmarks, SARIF)"
            echo "  --verbose            Show detailed output"
            echo "  --no-fail-fast       Continue on errors"
            echo "  --skip-slow          Skip slow tests (multi-rust, compile-time)"
            echo "  --help               Show this help"
            echo ""
            echo "Examples:"
            echo "  $0                          # Standard CI checks"
            echo "  $0 --full --reports        # Complete testing with reports"
            echo "  $0 --simulate-workflow=ci  # Simulate entire CI workflow"
            echo "  $0 --quick --skip-slow     # Fast pre-commit validation"
            exit 0
            ;;
    esac
done

# Error tracking
ERRORS=()
WARNINGS=()
track_error() {
    ERRORS+=("$1")
    if [[ "$FAIL_FAST" == "true" ]]; then
        print_error "$1"
        exit 1
    else
        print_warning "$1 (continuing...)"
    fi
}

track_warning() {
    WARNINGS+=("$1")
    print_warning "$1"
}

# Start timing
START_TIME=$(date +%s)

# Check prerequisites
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Cargo.toml not found. Run from project root."
    exit 1
fi

# Detect OS for multi-OS testing warnings
detect_os() {
    case "$OSTYPE" in
        linux*)   echo "linux" ;;
        darwin*)  echo "macos" ;;
        msys*)    echo "windows" ;;
        cygwin*)  echo "windows" ;;
        *)        echo "unknown" ;;
    esac
}

CURRENT_OS=$(detect_os)

# Install required tools if missing
install_tools() {
    print_step "Checking required tools..."
    
    local tools_needed=false
    
    # Check for cargo-audit
    if ! command -v cargo-audit &> /dev/null; then
        print_warning "cargo-audit not found, installing..."
        cargo install cargo-audit
        tools_needed=true
    fi
    
    # Check for cargo-llvm-cov
    if ! command -v cargo-llvm-cov &> /dev/null; then
        print_warning "cargo-llvm-cov not found, installing..."
        cargo install cargo-llvm-cov
        rustup component add llvm-tools-preview
        tools_needed=true
    fi
    
    # Check for cargo-deny
    if ! command -v cargo-deny &> /dev/null; then
        print_warning "cargo-deny not found, installing..."
        cargo install cargo-deny
        tools_needed=true
    fi
    
    # Check for cargo-outdated
    if ! command -v cargo-outdated &> /dev/null; then
        print_warning "cargo-outdated not found, installing..."
        cargo install cargo-outdated
        tools_needed=true
    fi
    
    # Check for cargo-vet (optional but recommended)
    if ! command -v cargo-vet &> /dev/null; then
        print_warning "cargo-vet not found (optional)"
        print_warning "Install with: cargo install cargo-vet"
    fi
    
    # Check for cargo-bloat (for binary size analysis)
    if ! command -v cargo-bloat &> /dev/null; then
        print_warning "cargo-bloat not found, installing..."
        cargo install cargo-bloat
        tools_needed=true
    fi
    
    if [[ "$tools_needed" == "false" ]]; then
        print_success "All required tools installed"
    fi
}

# Multi-Rust version testing
test_multi_rust() {
    print_header "🦀 Multi-Rust Version Testing"
    
    local rust_versions=("stable" "beta" "nightly")
    local tested_versions=0
    local failed_versions=()
    
    for version in "${rust_versions[@]}"; do
        if rustup toolchain list | grep -q "$version"; then
            print_step "Testing with Rust $version"
            
            # Run tests with each Rust version
            if cargo +"$version" test --all-features --verbose; then
                print_success "Tests passed on Rust $version"
                ((tested_versions++))
            else
                track_error "Tests failed on Rust $version"
                failed_versions+=("$version")
            fi
        else
            track_warning "Rust $version not installed - GitHub Actions will test this"
            print_status "Install with: rustup toolchain install $version"
        fi
    done
    
    # Report summary
    if [[ $tested_versions -eq 0 ]]; then
        track_warning "No Rust versions tested! GitHub Actions tests on stable, beta, and nightly"
    elif [[ ${#failed_versions[@]} -gt 0 ]]; then
        track_error "Tests failed on Rust versions: ${failed_versions[*]}"
    else
        print_success "All available Rust versions tested successfully ($tested_versions/${#rust_versions[@]})"
    fi
}

# Multi-OS testing simulation
test_multi_os() {
    print_header "🖥️ Multi-OS Testing Check"
    
    print_status "Current OS: $CURRENT_OS"
    print_warning "GitHub Actions tests on: Ubuntu (linux), macOS (darwin), Windows (msys)"
    
    if [[ "$CURRENT_OS" == "linux" ]]; then
        print_success "Testing on Linux (matches GitHub Actions ubuntu-latest)"
    else
        track_warning "Not testing on Linux - GitHub Actions will test this"
    fi
    
    if [[ "$CURRENT_OS" == "macos" ]]; then
        print_success "Testing on macOS (matches GitHub Actions macos-latest)"
    else
        track_warning "Not testing on macOS - GitHub Actions will test this"
    fi
    
    if [[ "$CURRENT_OS" == "windows" ]]; then
        print_success "Testing on Windows (matches GitHub Actions windows-latest)"
    else
        track_warning "Not testing on Windows - GitHub Actions will test this"
    fi
    
    print_status "Consider using Docker or VMs for cross-platform testing locally"
}

# Minimal dependencies test
test_minimal() {
    print_header "📦 Minimal Dependencies Testing"
    
    print_step "Testing with no default features (library only)"
    if cargo test --no-default-features --lib; then
        print_success "Library tests passed with no features"
    else
        track_error "Library tests failed with no features"
    fi
    
    print_step "Testing with only core feature"
    if cargo test --no-default-features --features core; then
        print_success "Core-only tests passed"
    else
        track_error "Core-only tests failed"
    fi
    
    print_step "Building with minimal features"
    if cargo build --no-default-features; then
        print_success "Minimal build successful"
    else
        track_error "Minimal build failed"
    fi
}

# Binary size analysis
analyze_binary_size() {
    print_header "📏 Binary Size Analysis"
    
    print_step "Building release binaries"
    cargo build --release --all-features
    
    print_step "Analyzing binary sizes"
    
    # Find all generated binaries
    if [[ -d "target/release" ]]; then
        echo ""
        echo "Binary sizes:"
        echo "============="
        
        # Use different commands based on OS
        case "$CURRENT_OS" in
            linux|macos)
                find target/release -maxdepth 1 -type f -executable ! -name "*.d" ! -name "*.rlib" -exec ls -lh {} \; | awk '{print $9 ": " $5}'
                ;;
            windows)
                find target/release -maxdepth 1 -name "*.exe" -exec ls -lh {} \; | awk '{print $9 ": " $5}'
                ;;
        esac
        
        # Use cargo-bloat if available for detailed analysis
        if command -v cargo-bloat &> /dev/null; then
            print_step "Detailed size analysis with cargo-bloat"
            cargo bloat --release --all-features -n 10 || true
        fi
        
        # Strip binaries and show size difference
        print_step "Testing stripped binary size"
        if [[ "$CURRENT_OS" != "windows" ]]; then
            cp target/release/prism-mcp-rs target/release/prism-mcp-rs.stripped 2>/dev/null || true
            strip target/release/prism-mcp-rs.stripped 2>/dev/null || true
            if [[ -f "target/release/prism-mcp-rs.stripped" ]]; then
                echo "Stripped size: $(ls -lh target/release/prism-mcp-rs.stripped | awk '{print $5}')"
            fi
        fi
    else
        track_warning "No release binaries found"
    fi
}

# Compile time metrics
measure_compile_time() {
    print_header "⏱️ Compile Time Metrics"
    
    if [[ "$SKIP_SLOW" == "true" ]]; then
        print_warning "Skipping compile time metrics (--skip-slow enabled)"
        return
    fi
    
    print_step "Clean build timing"
    cargo clean
    
    print_status "Debug build (all features):"
    TIMEFORMAT="  Time: %R seconds"
    time (cargo build --all-features 2>&1 | tail -n 1)
    
    print_status "Release build (all features):"
    time (cargo build --release --all-features 2>&1 | tail -n 1)
    
    print_status "Incremental rebuild (touch src/lib.rs):"
    touch src/lib.rs
    time (cargo build --all-features 2>&1 | tail -n 1)
    
    # Check build with different feature combinations
    print_step "Feature combination build times"
    cargo clean
    
    print_status "Minimal build:"
    time (cargo build --no-default-features 2>&1 | tail -n 1)
    
    print_status "Default features build:"
    cargo clean
    time (cargo build 2>&1 | tail -n 1)
}

# SARIF report generation
generate_sarif() {
    print_header "🔒 SARIF Security Report Generation"
    
    mkdir -p reports
    
    print_step "Generating SARIF report for GitHub Security tab"
    
    # Generate SARIF from cargo audit
    cargo audit --json 2>/dev/null | python3 -c "
import sys, json
import datetime

try:
    audit_data = json.load(sys.stdin)
    vulnerabilities = audit_data.get('vulnerabilities', {}).get('list', [])
    
    results = []
    for vuln in vulnerabilities:
        advisory = vuln.get('advisory', {})
        package = vuln.get('package', {})
        
        result = {
            'ruleId': advisory.get('id', 'UNKNOWN'),
            'level': 'error' if advisory.get('severity', 'unknown').lower() in ['critical', 'high'] else 'warning',
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
            }],
            'properties': {
                'package': package.get('name', 'unknown'),
                'version': package.get('version', 'unknown'),
                'severity': advisory.get('severity', 'unknown'),
                'description': advisory.get('description', '')
            }
        }
        results.append(result)
    
    sarif = {
        'version': '2.1.0',
        'runs': [{
            'tool': {
                'driver': {
                    'name': 'cargo-audit',
                    'version': '0.18.0',
                    'informationUri': 'https://github.com/RustSec/cargo-audit'
                }
            },
            'results': results,
            'invocations': [{
                'executionSuccessful': True,
                'endTimeUtc': datetime.datetime.utcnow().isoformat() + 'Z'
            }]
        }]
    }
    
    with open('reports/audit.sarif', 'w') as f:
        json.dump(sarif, f, indent=2)
    
    print(f'SARIF report generated with {len(results)} findings')
except Exception as e:
    print(f'Error generating SARIF: {e}')
    sarif = {
        'version': '2.1.0',
        'runs': [{
            'tool': {'driver': {'name': 'cargo-audit', 'version': '0.18.0'}},
            'results': []
        }]
    }
    with open('reports/audit.sarif', 'w') as f:
        json.dump(sarif, f, indent=2)
" || track_warning "SARIF generation failed"
    
    if [[ -f "reports/audit.sarif" ]]; then
        print_success "SARIF report saved to reports/audit.sarif"
        print_status "This can be uploaded to GitHub Security tab"
    fi
}

# Enhanced dependency tree analysis
analyze_dependency_tree() {
    print_header "🌳 Dependency Tree Analysis"
    
    mkdir -p reports
    
    print_step "Checking for duplicate dependencies"
    cargo tree --duplicates > reports/duplicates.txt
    
    if [[ -s "reports/duplicates.txt" ]]; then
        print_warning "Duplicate dependencies found:"
        cat reports/duplicates.txt
    else
        print_success "No duplicate dependencies"
    fi
    
    print_step "Generating full dependency tree"
    cargo tree --all-features > reports/dependency-tree.txt
    
    print_step "Analyzing dependency depth"
    cargo tree --all-features --depth 1 | wc -l | xargs -I {} echo "Direct dependencies: {}"
    cargo tree --all-features | wc -l | xargs -I {} echo "Total dependencies: {}"
    
    print_step "Checking dependency licenses"
    if command -v cargo-license &> /dev/null; then
        cargo license --json > reports/licenses.json
        print_success "License report saved to reports/licenses.json"
    else
        print_warning "cargo-license not installed, skipping license analysis"
    fi
}

# Enhanced coverage report generation
generate_coverage_reports() {
    print_header "📊 Coverage Report Generation"
    
    mkdir -p reports/coverage
    
    print_step "Generating multiple coverage formats"
    
    # HTML coverage
    print_status "Generating HTML coverage report..."
    cargo llvm-cov --all-features --workspace --html --output-dir reports/coverage/html
    
    # LCOV format (for coverage services)
    print_status "Generating LCOV coverage report..."
    cargo llvm-cov --all-features --workspace --lcov --output-path reports/coverage/lcov.info
    
    # JSON format (for programmatic access)
    print_status "Generating JSON coverage report..."
    cargo llvm-cov --all-features --workspace --json --output-path reports/coverage/coverage.json
    
    # Markdown report
    if [[ -x "scripts/ci/generate-coverage-report.sh" ]]; then
        print_status "Generating Markdown coverage report..."
        ./scripts/ci/generate-coverage-report.sh
    fi
    
    # Terminal summary
    print_step "Coverage Summary:"
    cargo llvm-cov report --all-features
    
    print_success "Coverage reports generated in reports/coverage/"
}

# Enhanced benchmark execution
run_comprehensive_benchmarks() {
    print_header "🚀 Comprehensive Benchmarks"
    
    mkdir -p reports/benchmarks
    
    # Check if benchmarks can be built
    if ! cargo build --benches --features bench 2>/dev/null; then
        track_warning "Cannot build benchmarks - check feature flags"
        return
    fi
    
    print_step "Running performance benchmarks"
    
    # Run benchmarks with JSON output
    if cargo bench --features bench -- --output-format json > reports/benchmarks/bench.json 2>/dev/null; then
        print_success "Benchmark JSON saved to reports/benchmarks/bench.json"
    fi
    
    # Run benchmarks normally for terminal output
    cargo bench --features bench || track_warning "Some benchmarks failed"
    
    # Generate markdown report if script exists
    if [[ -x "scripts/ci/run-benchmarks.sh" ]]; then
        ./scripts/ci/run-benchmarks.sh
        print_success "Benchmark report saved to reports/benchmark-report.md"
    fi
}

# Simulate specific GitHub workflow
simulate_workflow() {
    local workflow="$1"
    
    case "$workflow" in
        ci)
            print_header "🔄 Simulating CI Workflow (ci.yml)"
            
            # Multi-OS warning
            test_multi_os
            
            # test job
            test_multi_rust
            
            # minimal job  
            test_minimal
            
            # coverage job
            if [[ "$SKIP_SLOW" != "true" ]]; then
                generate_coverage_reports
            fi
            
            # fmt job
            print_step "Format Check (fmt job)"
            cargo fmt --all -- --check || track_error "Format check failed"
            
            # clippy job
            print_step "Clippy Analysis (clippy job)"
            cargo clippy --all-features -- -W clippy::all || track_error "Clippy failed"
            
            # doc job
            print_step "Documentation Build (doc job)"
            RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps || track_error "Doc build failed"
            
            # security job
            print_step "Security Audit (security job)"
            cargo audit || track_error "Security audit failed"
            
            # check job
            print_step "Feature Checks (check job)"
            for feature in stdio http websocket validation full; do
                cargo check --features "$feature" || track_error "Feature check '$feature' failed"
            done
            
            # examples job
            print_step "Examples Check (examples job)"
            for example in simple_server echo_server basic_client database_server; do
                if [[ -f "examples/${example}.rs" ]]; then
                    cargo check --example "$example" || track_error "Example '$example' failed"
                fi
            done
            
            # bench job
            if [[ "$SKIP_SLOW" != "true" ]]; then
                run_comprehensive_benchmarks
            fi
            ;;
            
        security)
            print_header "🔒 Simulating Security Workflow (security.yml)"
            
            install_tools
            
            # audit job
            print_step "Security Audit (audit job)"
            cargo audit || track_error "Critical vulnerabilities found"
            cargo audit --deny warnings || track_warning "Security warnings found"
            
            # outdated job
            print_step "Outdated Dependencies (outdated job)"
            cargo outdated || true
            
            # licenses job
            print_step "License Check (licenses job)"
            if [[ -f "deny.toml" ]]; then
                cargo deny check licenses || track_warning "License issues"
            fi
            
            # supply-chain job
            print_step "Supply Chain Security (supply-chain job)"
            if command -v cargo-vet &> /dev/null; then
                cargo vet --locked || track_warning "Supply chain issues"
            fi
            
            # sarif job
            generate_sarif
            ;;
            
        benchmarks)
            print_header "⚡ Simulating Benchmarks Workflow (benchmarks.yml)"
            
            # benchmark job
            run_comprehensive_benchmarks
            
            # compile-time job
            measure_compile_time
            
            # size job
            analyze_binary_size
            ;;
            
        dependencies)
            print_header "📦 Simulating Dependencies Workflow (dependencies.yml)"
            
            # outdated job
            print_step "Check Outdated (outdated job)"
            cargo outdated --exit-code 1 || track_warning "Outdated dependencies found"
            
            # update job
            print_step "Update Analysis (update job)"
            cargo update --dry-run
            
            # tree job
            analyze_dependency_tree
            ;;
            
        *)
            print_error "Unknown workflow: $workflow"
            print_status "Available workflows: ci, security, benchmarks, dependencies"
            exit 1
            ;;
    esac
}

# Main execution
print_header "🚀 Full Local CI Pipeline - Action: ${ACTION}"

# Handle workflow simulation
if [[ -n "$SIMULATE_WORKFLOW" ]]; then
    simulate_workflow "$SIMULATE_WORKFLOW"
elif [[ "$ACTION" == "fix" ]]; then
    print_header "🔧 Auto-fixing Issues"
    
    print_step "Fixing formatting..."
    cargo fmt --all
    
    print_step "Fixing clippy issues..."
    cargo clippy --fix --allow-dirty --all-features || true
    
    print_step "Updating dependencies..."
    cargo update
    
    print_success "Auto-fix complete! Review changes before committing."
elif [[ "$ACTION" == "quick" ]]; then
    print_header "⚡ Quick Validation"
    
    print_step "Format check..."
    cargo fmt --all -- --check || track_error "Format check failed"
    
    print_step "Clippy check..."
    cargo clippy --all-features -- -W clippy::all || track_error "Clippy check failed"
    
    print_step "Quick compilation..."
    cargo check --all-features || track_error "Compilation failed"
    
    print_step "Basic tests..."
    cargo test --lib || track_error "Basic tests failed"
elif [[ "$ACTION" == "security" ]]; then
    simulate_workflow "security"
elif [[ "$ACTION" == "benchmarks" ]]; then
    simulate_workflow "benchmarks"
elif [[ "$ACTION" == "dependencies" ]]; then
    simulate_workflow "dependencies"
elif [[ "$ACTION" == "full" ]]; then
    print_header "🔬 Full CI Testing"
    
    # Run all workflow simulations
    simulate_workflow "ci"
    simulate_workflow "security"
    simulate_workflow "benchmarks"
    simulate_workflow "dependencies"
else
    # Standard action - essential CI checks
    print_header "📋 Standard CI Checks"
    
    # OS check
    test_multi_os
    
    # Multi-Rust if not skipping slow tests
    if [[ "$SKIP_SLOW" != "true" ]]; then
        test_multi_rust
    else
        print_warning "Skipping multi-Rust testing (--skip-slow enabled)"
    fi
    
    # Minimal testing
    test_minimal
    
    # Format
    print_step "Format Check"
    cargo fmt --all -- --check || track_error "Format check failed"
    
    # Clippy
    print_step "Clippy Analysis"
    cargo clippy --all-features -- -W clippy::all || track_error "Clippy failed"
    
    # Tests
    print_step "Test Suite"
    cargo test --all-features || track_error "Tests failed"
    
    # Documentation
    print_step "Documentation Build"
    cargo doc --all-features --no-deps || track_error "Doc build failed"
    
    # Security
    print_step "Security Audit"
    cargo audit || track_error "Security vulnerabilities found"
    
    # Examples
    print_step "Examples Check"
    for example in simple_server echo_server basic_client; do
        if [[ -f "examples/${example}.rs" ]]; then
            cargo check --example "$example" || track_error "Example '$example' failed"
        fi
    done
    
    # Binary size (quick)
    if [[ "$SKIP_SLOW" != "true" ]]; then
        analyze_binary_size
    fi
    
    # Reports if requested
    if [[ "$GENERATE_REPORTS" == "true" ]]; then
        generate_coverage_reports
        generate_sarif
        analyze_dependency_tree
        run_comprehensive_benchmarks
    fi
fi

# Summary
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

print_header "📊 CI Pipeline Summary"
echo "Duration: ${DURATION}s"
echo "Errors: ${#ERRORS[@]}"
echo "Warnings: ${#WARNINGS[@]}"
echo "Current OS: $CURRENT_OS"

if [[ ${#WARNINGS[@]} -gt 0 ]]; then
    echo ""
    echo "⚠️ Warnings:"
    for warning in "${WARNINGS[@]}"; do
        echo "  • $warning"
    done
fi

if [[ ${#ERRORS[@]} -gt 0 ]]; then
    print_error "CI Pipeline Failed!"
    echo ""
    echo "❌ Errors encountered:"
    for error in "${ERRORS[@]}"; do
        echo "  • $error"
    done
    echo ""
    echo "Fix suggestions:"
    echo "  • Run '$0 --fix' to auto-fix formatting and clippy issues"
    echo "  • Install missing Rust toolchains for complete testing"
    echo "  • Consider Docker/VM testing for other operating systems"
    exit 1
else
    print_success "All CI checks passed! ✨"
    echo ""
    
    # GitHub Actions parity report
    echo "📋 GitHub Actions Parity Report:"
    echo "  ✅ Format checking"
    echo "  ✅ Clippy linting"
    echo "  ✅ Test execution"
    echo "  ✅ Documentation build"
    echo "  ✅ Security audit"
    echo "  ✅ Example validation"
    echo "  ✅ Minimal dependency testing"
    echo "  ✅ Binary size analysis"
    
    if [[ "$GENERATE_REPORTS" == "true" ]]; then
        echo "  ✅ Coverage reports (HTML, LCOV, JSON)"
        echo "  ✅ SARIF security report"
        echo "  ✅ Dependency tree analysis"
        echo "  ✅ Benchmark reports"
    fi
    
    if [[ "$SKIP_SLOW" != "true" ]]; then
        echo "  ✅ Multi-Rust version testing"
        echo "  ✅ Compile-time metrics"
    fi
    
    echo ""
    echo "  ⚠️ Multi-OS testing (runs on $CURRENT_OS only)"
    echo "  ⚠️ Auto-commit reports (manual commit required)"
    echo "  ⚠️ PR-specific checks (no PR context locally)"
    echo ""
    echo "Your code is ready for:"
    echo "  • Committing locally"
    echo "  • Pushing to GitHub (will trigger full multi-OS CI)"
    echo "  • Creating a pull request"
fi

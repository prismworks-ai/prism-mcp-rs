#!/bin/bash
# Run CI pipeline locally using act or native commands
# This script mimics the GitHub Actions CI workflow locally

set -e

echo "🚀 Running Local CI Pipeline for prism-mcp-rs"
echo "========================================"

# Check if we should use act or run natively
if command -v act &> /dev/null && [ "$USE_ACT" != "false" ]; then
    echo "📦 Using 'act' to run GitHub Actions locally"
    echo ""
    
    # Run the CI workflow with act
    act -W .github/workflows/ci.yml \
        --platform ubuntu-latest=catthehacker/ubuntu:act-latest \
        --reuse \
        --pull=false
else
    echo "🔧 Running CI steps natively"
    echo ""
    
    # Colors for output
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    NC='\033[0m' # No Color
    
    # Track overall status
    FAILED=0
    
    # Helper function to run a step
    run_step() {
        local step_name="$1"
        local command="$2"
        
        echo -e "${YELLOW}▶ Running: $step_name${NC}"
        if eval "$command"; then
            echo -e "${GREEN}✅ $step_name passed${NC}\n"
        else
            echo -e "${RED}❌ $step_name failed${NC}\n"
            FAILED=1
        fi
    }
    
    # Run CI steps
    echo "📋 Step 1/8: Format Check"
    run_step "Format Check" "cargo fmt -- --check"
    
    echo "📋 Step 2/8: Clippy Lints"
    run_step "Clippy" "cargo clippy --all-targets --all-features -- -D warnings"
    
    echo "📋 Step 3/8: Build Check"
    run_step "Build" "cargo check --all-features"
    
    echo "📋 Step 4/8: Run Tests"
    run_step "Tests" "cargo test --all-features"
    
    echo "📋 Step 5/8: Check Minimal Dependencies"
    run_step "Minimal Dependencies" "cargo check -Z minimal-versions --all-features || true"
    
    echo "📋 Step 6/8: Documentation"
    run_step "Documentation" "cargo doc --no-deps --all-features"
    
    echo "📋 Step 7/8: Examples"
    run_step "Examples" "cargo build --examples --all-features"
    
    echo "📋 Step 8/8: Security Audit"
    if command -v cargo-audit &> /dev/null; then
        run_step "Security Audit" "cargo audit"
    else
        echo -e "${YELLOW}⚠️  cargo-audit not installed, skipping security audit${NC}"
        echo "   Install with: cargo install cargo-audit"
    fi
    
    # Summary
    echo "\n========================================"
    if [ $FAILED -eq 0 ]; then
        echo -e "${GREEN}✅ All CI checks passed!${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some CI checks failed. Please fix the issues above.${NC}"
        exit 1
    fi
fi

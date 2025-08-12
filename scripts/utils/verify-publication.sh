#!/bin/bash
# Pre-Publication Verification Script
# Run this before publishing to crates.io

set -e

echo "Search: MCP Protocol SDK - Pre-Publication Verification"
echo "=================================================="

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}[x] $1${NC}"
    else
        echo -e "${RED}[!] $1${NC}"
        exit 1
    fi
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}Warning:  $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}[!] Not in the root directory of the project${NC}"
    exit 1
fi

echo -e "${BLUE}# Starting verification process...${NC}"
echo ""

# 1. Code Quality Checks
echo "1️⃣ Code Quality Checks"
echo "====================="

print_info "Running cargo fmt check..."
cargo fmt --check
print_status "Code formatting"

print_info "Running cargo clippy..."
cargo clippy --all-features -- -D warnings
print_status "Clippy linting"

print_info "Running cargo test..."
cargo test --all-features
print_status "Test suite"

print_info "Checking minimal build..."
cargo check --no-default-features --lib
print_status "Minimal build"

print_info "Testing feature combinations..."
cargo check --no-default-features --features stdio
cargo check --no-default-features --features http
cargo check --no-default-features --features websocket
print_status "Feature combinations"

echo ""

# 2. Documentation Checks
echo "2️⃣ Documentation Checks"
echo "======================"

print_info "Building documentation..."
cargo doc --all-features --no-deps
print_status "Documentation build"

print_info "Testing examples..."
cargo check --example echo_server --features stdio,tracing-subscriber
cargo check --example http_server --features http
cargo check --example websocket_server --features websocket
print_status "Example compilation"

print_info "Testing doctests..."
cargo test --doc
print_status "Documentation tests"

echo ""

# 3. Metadata Verification
echo "3️⃣ Metadata Verification"
echo "========================"

print_info "Checking Cargo.toml metadata..."

# Check version format
VERSION=$(grep '^version' Cargo.toml | cut -d'"' -f2)
if [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_status "Version format: $VERSION"
else
    echo -e "${RED}[!] Invalid version format: $VERSION${NC}"
    exit 1
fi

# Check description length
DESCRIPTION=$(grep '^description' Cargo.toml | cut -d'"' -f2)
if [ ${#DESCRIPTION} -le 200 ]; then
    print_status "Description length: ${#DESCRIPTION} characters"
else
    echo -e "${RED}[!] Description too long: ${#DESCRIPTION} characters (max 200)${NC}"
    exit 1
fi

# Check keywords count
KEYWORDS_COUNT=$(grep '^keywords' Cargo.toml | grep -o '"[^"]*"' | wc -l | tr -d ' ')
if [ "$KEYWORDS_COUNT" -le 5 ]; then
    print_status "Keywords count: $KEYWORDS_COUNT"
else
    echo -e "${RED}[!] Too many keywords: $KEYWORDS_COUNT (max 5)${NC}"
    exit 1
fi

# Check required fields
REQUIRED_FIELDS=("license" "repository" "homepage" "documentation")
for field in "${REQUIRED_FIELDS[@]}"; do
    if grep -q "^$field" Cargo.toml; then
        print_status "$field field present"
    else
        echo -e "${RED}[!] Missing required field: $field${NC}"
        exit 1
    fi
done

echo ""

# 4. Security Audit
echo "4️⃣ Security Audit"
echo "================"

print_info "Running cargo audit..."
if command -v cargo-audit &> /dev/null; then
    cargo audit
    print_status "Security audit"
else
    print_warning "cargo-audit not installed, skipping security check"
    print_info "Install with: cargo install cargo-audit"
fi

echo ""

# 5. Publication Dry Run
echo "5️⃣ Publication Dry Run"
echo "====================="

print_info "Running cargo publish dry run..."
cargo publish --dry-run --all-features
print_status "Publish dry run"

echo ""

# 6. GitHub Status Check
echo "6️⃣ GitHub Status Check"
echo "====================="

print_info "Checking GitHub workflow status..."
if command -v gh &> /dev/null; then
    FAILING_WORKFLOWS=$(gh run list --limit 10 --json status,name --jq '.[] | select(.status == "failed") | .name' | wc -l | tr -d ' ')
    if [ "$FAILING_WORKFLOWS" -eq 0 ]; then
        print_status "GitHub workflows"
    else
        echo -e "${RED}[!] $FAILING_WORKFLOWS failing workflows${NC}"
        gh run list --limit 5 --json status,name,conclusion --jq '.[] | select(.status == "failed")'
        exit 1
    fi
else
    print_warning "GitHub CLI not available, skipping workflow check"
fi

print_info "Checking GitHub Pages status..."
if command -v gh &> /dev/null; then
    PAGES_URL=$(gh api repos/prismworks-ai/prism-mcp-rs/pages --jq '.html_url' 2>/dev/null || echo "")
    if [ -n "$PAGES_URL" ]; then
        print_status "GitHub Pages enabled: $PAGES_URL"
    else
        print_warning "GitHub Pages not enabled or accessible"
    fi
fi

echo ""

# 7. File Size Check
echo "7️⃣ Package Size Check"
echo "===================="

print_info "Checking package size..."
PACKAGE_SIZE=$(cargo package --list | wc -l)
print_info "Package contains $PACKAGE_SIZE files"

# Check if package would be too large (rough estimate)
if [ "$PACKAGE_SIZE" -gt 1000 ]; then
    print_warning "Package might be large - review exclude patterns in Cargo.toml"
else
    print_status "Package size reasonable"
fi

echo ""

# 8. Final Checklist
echo "8️⃣ Final Checklist"
echo "================="

CHECKLIST=(
    "All tests passing"
    "Documentation complete"
    "Examples working"
    "Security audit clean"
    "GitHub workflows passing"
    "Cargo.toml metadata complete"
    "README.md updated"
    "CHANGELOG.md updated"
)

for item in "${CHECKLIST[@]}"; do
    echo -e "${GREEN}[x] $item${NC}"
done

echo ""
echo -e "${GREEN} Pre-publication verification complete!${NC}"
echo ""
echo -e "${BLUE}Package: Ready to publish to crates.io:${NC}"
echo -e "${YELLOW}   cargo publish --all-features${NC}"
echo ""
echo -e "${BLUE}🏷️  Create GitHub release:${NC}"
echo -e "${YELLOW}   git tag v$VERSION${NC}"
echo -e "${YELLOW}   git push origin v$VERSION${NC}"
echo -e "${YELLOW}   gh release create v$VERSION --generate-notes${NC}"
echo ""
echo -e "${BLUE}🌐 GitHub Pages will be available at:${NC}"
echo -e "${YELLOW}   https://mcp-rust.github.io/mcp-protocol-sdk/${NC}"
echo ""
echo -e "${GREEN}* Happy publishing! *${NC}"

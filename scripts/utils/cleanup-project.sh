#!/bin/bash
# Project cleanup script for prism-mcp-rs
# Removes temporary files, build artifacts, and organizes the project structure

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🧹 Cleaning up prism-mcp-rs project${NC}"
echo "========================================"

# Function to print status
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠️${NC}  $1"
}

# Navigate to project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "\n${CYAN}📦 Step 1: Removing backup files${NC}"
find . -name "*.bak" -type f -delete 2>/dev/null || true
find . -name "*.backup" -type f -delete 2>/dev/null || true
print_status "Backup files removed"

echo -e "\n${CYAN}📦 Step 2: Removing OS-specific files${NC}"
find . -name ".DS_Store" -type f -delete 2>/dev/null || true
find . -name "._*" -type f -delete 2>/dev/null || true
find . -name "Thumbs.db" -type f -delete 2>/dev/null || true
print_status "OS-specific files removed"

echo -e "\n${CYAN}📦 Step 3: Cleaning temporary files${NC}"
find . -name "*.tmp" -type f -delete 2>/dev/null || true
find . -name "*.temp" -type f -delete 2>/dev/null || true
find . -name "*.log" -type f -delete 2>/dev/null || true
print_status "Temporary files removed"

echo -e "\n${CYAN}📦 Step 4: Cleaning build artifacts${NC}"
if [ -d ".local" ]; then
    find .local -name "*.profraw" -type f -delete 2>/dev/null || true
    find .local -name "*.prof" -type f -delete 2>/dev/null || true
    print_status "Local build artifacts cleaned"
fi

echo -e "\n${CYAN}📦 Step 5: Organizing structure${NC}"

# Move any stray shell scripts to scripts directory
for script in *.sh; do
    if [ -f "$script" ]; then
        print_warning "Found stray script: $script"
        echo "   Consider moving to scripts/ directory"
    fi
done

# Check for files that should be in docs
for doc in *_GUIDE.md *_REFERENCE.md; do
    if [ -f "$doc" ]; then
        print_warning "Found documentation in root: $doc"
        echo "   Consider moving to docs/ directory"
    fi
done

print_status "Structure check complete"

echo -e "\n${CYAN}📦 Step 6: Optional - Clean Cargo cache${NC}"
echo "Run 'cargo clean' to remove target directory? (y/N)"
read -r response
if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    cargo clean
    print_status "Cargo build artifacts cleaned"
else
    echo "Skipped cargo clean"
fi

echo -e "\n${CYAN}📦 Step 7: Git status check${NC}"
if [ -d ".git" ]; then
    echo "Untracked files:"
    git ls-files --others --exclude-standard | head -10
    
    UNTRACKED_COUNT=$(git ls-files --others --exclude-standard | wc -l)
    if [ "$UNTRACKED_COUNT" -gt 10 ]; then
        print_warning "... and $(($UNTRACKED_COUNT - 10)) more untracked files"
    fi
    
    if [ "$UNTRACKED_COUNT" -eq 0 ]; then
        print_status "No untracked files"
    fi
fi

echo -e "\n========================================"
echo -e "${GREEN}✅ Cleanup complete!${NC}"
echo ""
echo "Project statistics:"
echo "  📁 Source files: $(find src -name "*.rs" | wc -l)"
echo "  🧪 Test files: $(find tests -name "*.rs" | wc -l)"
echo "  📚 Examples: $(find examples -name "*.rs" | wc -l)"
echo "  📦 Total size: $(du -sh . | cut -f1)"

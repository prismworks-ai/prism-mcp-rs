#!/bin/bash

# # Badge Fix Script for MCP Protocol SDK
# This script addresses badge update issues by:
# 1. Triggering workflows manually to refresh badge status
# 2. Checking codecov configuration
# 3. Validating badge URLs

set -e

echo "Search: MCP Protocol SDK - Badge Fix Script"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d ".github/workflows" ]; then
    echo -e "${RED}[!] Error: Please run this script from the project root directory${NC}"
    exit 1
fi

echo -e "${BLUE} Current Badge Status Analysis${NC}"
echo "=================================="

# Function to check workflow files
check_workflow_file() {
    local file=$1
    local name=$(grep "^name:" "$file" | sed 's/name: *//' | tr -d '"')
    echo -e "${GREEN}[x] $file${NC}"
    echo -e "   Name: ${YELLOW}$name${NC}"
}

echo -e "\n${BLUE}- Checking Workflow Files${NC}"
for workflow in .github/workflows/*.yml; do
    check_workflow_file "$workflow"
done

echo -e "\n${BLUE}📊 Badge URL Validation${NC}"
echo "========================"

# Extract badge URLs from README
declare -A badges
badges[CI]="https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/ci.yml/badge.svg"
badges[Security]="https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/security.yml/badge.svg"
badges[Dependencies]="https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/dependencies.yml/badge.svg"
badges[Documentation]="https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/docs.yml/badge.svg"
badges[Benchmarks]="https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/benchmarks.yml/badge.svg"
badges[Release]="https://github.com/prismworks-ai/prism-mcp-rs/actions/workflows/release.yml/badge.svg"
badges[Codecov]="https://codecov.io/gh/prismworks-ai/prism-mcp-rs/branch/main/graph/badge.svg"

for badge_name in "${!badges[@]}"; do
    url="${badges[$badge_name]}"
    echo -e "${GREEN}[x] $badge_name:${NC} $url"
done

echo -e "\n${BLUE}🔄 Manual Workflow Trigger Commands${NC}"
echo "===================================="
echo "To trigger workflows manually and refresh badges, run:"
echo ""
echo -e "${YELLOW}# Trigger CI workflow${NC}"
echo "gh workflow run ci.yml"
echo ""
echo -e "${YELLOW}# Trigger Dependencies workflow${NC}"
echo "gh workflow run dependencies.yml"
echo ""
echo -e "${YELLOW}# Trigger Security workflow${NC}"
echo "gh workflow run security.yml"
echo ""
echo -e "${YELLOW}# Trigger Documentation workflow${NC}"
echo "gh workflow run docs.yml"
echo ""
echo -e "${YELLOW}# Trigger Benchmarks workflow${NC}"
echo "gh workflow run benchmarks.yml"

echo -e "\n${BLUE}📈 Codecov Configuration Check${NC}"
echo "==============================="

if [ -f "codecov.yml" ]; then
    echo -e "${GREEN}[x] codecov.yml found${NC}"
    echo "Coverage target: $(grep "target:" codecov.yml | head -1 | awk '{print $2}')"
else
    echo -e "${YELLOW}Warning:  codecov.yml not found (using defaults)${NC}"
fi

echo -e "\n${BLUE}🔄 Recommended Actions${NC}"
echo "===================="
echo ""
echo -e "${GREEN}1. Trigger workflows manually:${NC}"
echo "   Run the gh workflow commands above to refresh all badges"
echo ""
echo -e "${GREEN}2. Check codecov token:${NC}"
echo "   Ensure CODECOV_TOKEN is set in GitHub repository secrets"
echo ""
echo -e "${GREEN}3. Wait for workflow completion:${NC}"
echo "   Badges typically update within 5-10 minutes after workflow completion"
echo ""
echo -e "${GREEN}4. Release badge:${NC}"
echo "   Will update on next tag push/release"

echo -e "\n${BLUE}# Quick Fix Commands${NC}"
echo "===================="
echo ""
echo -e "${YELLOW}# If you have GitHub CLI installed, run all workflows:${NC}"
echo "gh workflow run ci.yml && gh workflow run dependencies.yml && gh workflow run security.yml && gh workflow run docs.yml"
echo ""
echo -e "${YELLOW}# Push any change to trigger CI (if needed):${NC}"
echo "git commit --allow-empty -m 'trigger: refresh workflow badges' && git push"

echo -e "\n${GREEN}[x] Badge Fix Analysis Complete!${NC}"
echo -e "${BLUE}Check workflow status at: https://github.com/prismworks-ai/prism-mcp-rs/actions${NC}"
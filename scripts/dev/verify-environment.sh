#!/bin/bash

# Environment Setup Verification Script
# Checks if required secrets and configurations are properly set up

set -e

echo "🔐 Environment Setup Verification"
echo "================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "[x]" ]; then
        echo -e "${GREEN}$status $message${NC}"
    elif [ "$status" = "[!]" ]; then
        echo -e "${RED}$status $message${NC}"
    elif [ "$status" = "Warning:" ]; then
        echo -e "${YELLOW}$status $message${NC}"
    else
        echo -e "${BLUE}$status $message${NC}"
    fi
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d ".github/workflows" ]; then
    print_status "[!]" "Not in project root directory"
    echo "Please run this script from the mcp-protocol-sdk root directory"
    exit 1
fi

print_status "📍" "Checking project structure..."

# Check workflow files exist
WORKFLOWS=("ci.yml" "security.yml" "docs.yml" "release.yml" "benchmarks.yml" "dependencies.yml")
for workflow in "${WORKFLOWS[@]}"; do
    if [ -f ".github/workflows/$workflow" ]; then
        print_status "[x]" "Workflow file: $workflow"
    else
        print_status "[!]" "Missing workflow file: $workflow"
    fi
done

echo

# Check codecov.yml configuration
print_status "📍" "Checking codecov.yml configuration..."

if [ -f "codecov.yml" ]; then
    if grep -q "token:" codecov.yml && ! grep -q "# Token configured in GitHub Secrets" codecov.yml; then
        print_status "Warning:" "codecov.yml contains hardcoded token (security risk)"
        echo "    Run: Remove hardcoded token from codecov.yml"
    else
        print_status "[x]" "codecov.yml properly configured"
    fi
else
    print_status "[!]" "codecov.yml not found"
fi

echo

# Check for secret references in workflows  
print_status "📍" "Checking secret references in workflows..."

if grep -r "CODECOV_TOKEN" .github/workflows/ >/dev/null 2>&1; then
    print_status "[x]" "CODECOV_TOKEN referenced in workflows"
else
    print_status "[!]" "CODECOV_TOKEN not found in workflows"
fi

if grep -r "CARGO_REGISTRY_TOKEN" .github/workflows/ >/dev/null 2>&1; then
    print_status "[x]" "CARGO_REGISTRY_TOKEN referenced in workflows"  
else
    print_status "[!]" "CARGO_REGISTRY_TOKEN not found in workflows"
fi

echo

# Check workflow syntax (basic)
print_status "📍" "Checking workflow syntax..."

for workflow in .github/workflows/*.yml; do
    if [ -f "$workflow" ]; then
        if python3 -c "import yaml; yaml.safe_load(open('$workflow'))" 2>/dev/null; then
            print_status "[x]" "$(basename "$workflow"): Valid YAML"
        else
            print_status "[!]" "$(basename "$workflow"): Invalid YAML syntax"
        fi
    fi
done

echo

# Manual checks that need to be done in GitHub UI
print_status "📍" "Manual setup required (GitHub repository)..."
print_status "Warning:" "CODECOV_TOKEN: Must be set in GitHub Secrets"
print_status "Warning:" "CARGO_REGISTRY_TOKEN: Must be set in GitHub Secrets"

echo
echo "- Setup Instructions:"
echo "======================"
echo
echo "1. Go to your GitHub repository"
echo "2. Navigate to Settings → Secrets and variables → Actions"
echo "3. Add these secrets:"
echo "   • CODECOV_TOKEN (get from https://codecov.io/)"
echo "   • CARGO_REGISTRY_TOKEN (get from https://crates.io/me)"
echo
echo "4. Test the setup:"
echo "   • Create a test PR"
echo "   • Check that CI workflow runs successfully"
echo "   • Verify codecov integration works"
echo

# Summary
echo "📊 Summary:"
echo "==========="
echo
if [ -f "codecov.yml" ] && ! grep -q "token:" codecov.yml; then
    print_status "[x]" "Configuration files are secure"
else
    print_status "Warning:" "Some configuration files need attention"
fi

if [ -d ".github/workflows" ] && [ "$(ls -1 .github/workflows/*.yml 2>/dev/null | wc -l)" -eq 6 ]; then
    print_status "[x]" "All workflow files present"
else
    print_status "Warning:" "Some workflow files may be missing"
fi

print_status "Warning:" "GitHub Secrets need manual setup"

echo
echo "# Next Steps:"
echo "=============="
echo "1. Set up GitHub Secrets (see instructions above)"
echo "2. Remove any hardcoded tokens from config files"
echo "3. Test with a small PR"
echo "4. Monitor workflow execution logs"
echo
echo "Note: For detailed instructions, see the Environment Setup Analysis document"

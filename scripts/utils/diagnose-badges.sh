#!/bin/bash

# Badge Diagnosis Script
# complete analysis of badge status and workflow health

set -e

echo "Search: Badge Diagnosis Report"
echo "====================================="
echo ""

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
    echo "[!] GitHub CLI (gh) is not installed"
    exit 1
fi

echo "📊 Recent Workflow Runs:"
echo "--------------------"
gh run list --limit 10 --json status,conclusion,workflowName,displayTitle,createdAt | jq -r '.[] | "\(.workflowName): \(.status) (\(.conclusion // "running"))"'

echo ""
echo "## CI Workflow Status:"
echo "------------------"
gh run list --workflow="CI" --limit 5 --json status,conclusion,displayTitle,createdAt | jq -r '.[] | "\(.displayTitle): \(.status) (\(.conclusion // "running")) - \(.createdAt)"'

echo ""
echo "📊 Badge URL Analysis:"
echo "-------------------"

# Test badge URLs
BADGES=(
    "CI:https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/ci.yml/badge.svg"
    "Security:https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/security.yml/badge.svg"
    "Dependencies:https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/dependencies.yml/badge.svg"
    "Docs:https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/docs.yml/badge.svg"
    "Benchmarks:https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/benchmarks.yml/badge.svg"
    "Release:https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/release.yml/badge.svg"
    "Codecov:https://codecov.io/gh/prismworks-ai/mcp-protocol-sdk/branch/main/graph/badge.svg"
)

for badge in "${BADGES[@]}"; do
    name=$(echo "$badge" | cut -d: -f1)
    url=$(echo "$badge" | cut -d: -f2-)
    
    echo "Testing $name badge..."
    
    # Get badge content to see what it says
    badge_content=$(curl -s "$url" | grep -o 'passing\|failing\|unknown\|error' | head -1 || echo "N/A")
    status_code=$(curl -s -w "%{http_code}" "$url" -o /dev/null)
    
    echo "  URL: $url"
    echo "  Status: $status_code"
    echo "  Content: $badge_content"
    echo ""
done

echo "- Potential Issues:"
echo "----------------"

# Check for cancelled workflows
cancelled_count=$(gh run list --limit 20 --json conclusion | jq '[.[] | select(.conclusion == "cancelled")] | length')
failed_count=$(gh run list --limit 20 --json conclusion | jq '[.[] | select(.conclusion == "failure")] | length')

if [ "$cancelled_count" -gt 0 ]; then
    echo "Warning:  Found $cancelled_count cancelled workflow(s) in recent runs"
fi

if [ "$failed_count" -gt 0 ]; then
    echo "Warning:  Found $failed_count failed workflow(s) in recent runs"
fi

# Check for successful recent runs
success_count=$(gh run list --limit 10 --json conclusion | jq '[.[] | select(.conclusion == "success")] | length')
echo "[x] Found $success_count successful workflow(s) in recent runs"

echo ""
echo "🔗 Solutions:"
echo "----------"
echo "1. Cancel any queued/running workflows that aren't needed"
echo "2. Re-run the most recent successful workflow to update badges"
echo "3. Wait for current workflows to complete"
echo "4. Check repository secrets for CODECOV_TOKEN"
echo "5. Consider using branch-specific badge URLs if needed"

echo ""
echo " Actions to take:"
echo "gh run cancel <run_id>  # Cancel unwanted runs"
echo "gh run rerun <run_id>   # Re-run successful runs"
echo "gh run list --workflow=CI --limit 1 --json databaseId  # Get latest CI run ID"

echo ""
echo "[x] Diagnosis complete!"

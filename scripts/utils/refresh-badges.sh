#!/bin/bash

# Badge Refresh Script
# This script forces refresh of GitHub Action badges by adding cache-busting parameters

set -e

echo "🔄 Refreshing GitHub Action badges..."

# Array of workflow badge URLs to refresh
BADGE_URLS=(
    "https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/ci.yml/badge.svg"
    "https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/security.yml/badge.svg"
    "https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/dependencies.yml/badge.svg"
    "https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/docs.yml/badge.svg"
    "https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/benchmarks.yml/badge.svg"
    "https://github.com/prismworks-ai/mcp-protocol-sdk/actions/workflows/release.yml/badge.svg"
    "https://codecov.io/gh/prismworks-ai/mcp-protocol-sdk/branch/main/graph/badge.svg"
)

echo "📊 Testing badge URLs..."

for url in "${BADGE_URLS[@]}"; do
    echo "Testing: $url"
    
    # Test with cache-busting timestamp
    timestamp=$(date +%s)
    cache_bust_url="${url}?t=${timestamp}"
    
    status_code=$(curl -s -w "%{http_code}" "$cache_bust_url" -o /dev/null)
    
    if [ "$status_code" = "200" ]; then
        echo "  [x] OK (200)"
    else
        echo "  [!] Failed ($status_code)"
    fi
    
    # Brief pause to avoid rate limiting
    sleep 0.5
done

echo ""
echo "## Recommendations:"
echo "1. GitHub Action badges reflect the latest workflow run status"
echo "2. If badges show 'failing', check the most recent workflow runs"
echo "3. CodeCov badge requires CODECOV_TOKEN to be set in repository secrets"
echo "4. Badges may take 5-10 minutes to update after workflow completion"
echo "5. Browser cache may show old badge status - try hard refresh (Ctrl+F5)"

echo ""
echo "🔗 Quick Links:"
echo "- Actions: https://github.com/prismworks-ai/mcp-protocol-sdk/actions"
echo "- CodeCov: https://codecov.io/gh/prismworks-ai/mcp-protocol-sdk"
echo "- Repository: https://github.com/prismworks-ai/mcp-protocol-sdk"

echo ""
echo "[x] Badge refresh complete!"

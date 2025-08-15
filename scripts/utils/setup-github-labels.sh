#!/bin/bash

# Setup GitHub Issue Labels for Prism MCP SDK
# This script creates the standard issue labels used for contributions
#
# Usage: ./scripts/utils/setup-github-labels.sh [owner/repo] [token]
#
# You need a GitHub personal access token with 'repo' scope
# Create one at: https://github.com/settings/tokens

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default repository
REPO="${1:-prismworks-ai/prism-mcp-rs}"
TOKEN="${2:-$GITHUB_TOKEN}"

if [ -z "$TOKEN" ]; then
    echo -e "${RED}Error: GitHub token required${NC}"
    echo "Usage: $0 [owner/repo] [github-token]"
    echo "Or set GITHUB_TOKEN environment variable"
    exit 1
fi

echo -e "${BLUE}Setting up GitHub labels for $REPO${NC}"

# Function to create or update a label
create_label() {
    local name="$1"
    local color="$2"
    local description="$3"
    
    echo -n "Creating label '$name'... "
    
    # Try to create the label
    response=$(curl -s -o /dev/null -w "%{http_code}" \
        -X POST \
        -H "Authorization: token $TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/$REPO/labels" \
        -d "{\"name\":\"$name\",\"color\":\"$color\",\"description\":\"$description\"}")
    
    if [ "$response" = "201" ]; then
        echo -e "${GREEN}created${NC}"
    elif [ "$response" = "422" ]; then
        # Label exists, try to update it
        response=$(curl -s -o /dev/null -w "%{http_code}" \
            -X PATCH \
            -H "Authorization: token $TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/repos/$REPO/labels/$name" \
            -d "{\"color\":\"$color\",\"description\":\"$description\"}")
        
        if [ "$response" = "200" ]; then
            echo -e "${YELLOW}updated${NC}"
        else
            echo -e "${RED}failed (HTTP $response)${NC}"
        fi
    else
        echo -e "${RED}failed (HTTP $response)${NC}"
    fi
}

# Delete default labels that we don't use
echo -e "\n${YELLOW}Removing unused default labels...${NC}"
for label in "invalid" "help wanted" "good first issue"; do
    echo -n "Removing '$label'... "
    response=$(curl -s -o /dev/null -w "%{http_code}" \
        -X DELETE \
        -H "Authorization: token $TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/$REPO/labels/$label")
    
    if [ "$response" = "204" ]; then
        echo -e "${GREEN}removed${NC}"
    elif [ "$response" = "404" ]; then
        echo -e "${YELLOW}not found${NC}"
    else
        echo -e "${RED}failed (HTTP $response)${NC}"
    fi
done

# Create our standard labels
echo -e "\n${BLUE}Creating standard labels...${NC}"

# Issue type labels
create_label "bug" "d73a4a" "Something isn't working"
create_label "feature-request" "a2eeef" "New feature proposal"
create_label "enhancement" "84b6eb" "Improvement to existing functionality"
create_label "documentation" "0075ca" "Documentation improvements"
create_label "question" "d876e3" "Questions about the project"

# Status labels
create_label "good-first-issue" "7057ff" "Good for newcomers"
create_label "help-wanted" "008672" "Extra attention needed"
create_label "blocked" "e11d21" "Blocked by another issue or external factor"
create_label "needs-discussion" "fbca04" "Needs discussion before proceeding"

# Resolution labels
create_label "wontfix" "ffffff" "This will not be worked on"
create_label "duplicate" "cfd3d7" "This issue or pull request already exists"
create_label "invalid" "e4e669" "This doesn't seem right"

# Component labels
create_label "transport" "bfdadc" "Transport layer (HTTP, WebSocket, stdio)"
create_label "plugin" "c5def5" "Plugin system"
create_label "protocol" "d4c5f9" "MCP protocol implementation"
create_label "client" "fef2c0" "Client implementation"
create_label "server" "bfd4f2" "Server implementation"

# Priority labels (optional, can be added if needed)
# create_label "priority-high" "b60205" "High priority"
# create_label "priority-medium" "ff9800" "Medium priority"
# create_label "priority-low" "0e8a16" "Low priority"

echo -e "\n${GREEN}Label setup complete!${NC}"
echo -e "View labels at: https://github.com/$REPO/labels"
echo ""
echo "Next steps:"
echo "1. Review the labels at the URL above"
echo "2. Add any project-specific labels if needed"
echo "3. Update issue templates to use these labels"
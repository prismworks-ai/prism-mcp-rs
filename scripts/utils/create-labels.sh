#!/bin/bash

# GitHub Labels for MCP Protocol SDK
# Run this script to create useful labels for the repository

set -e

REPO="your-username/mcp-protocol-sdk"

echo "Creating GitHub labels for $REPO..."

# Priority Labels
gh label create --repo $REPO "priority-critical" --color "b60205" --description "Critical priority requiring immediate attention" || true
gh label create --repo $REPO "priority-high" --color "d73a4a" --description "High priority items" || true
gh label create --repo $REPO "priority-medium" --color "fbca04" --description "Medium priority items" || true
gh label create --repo $REPO "priority-low" --color "0e8a16" --description "Low priority items" || true

# Type Labels
gh label create --repo $REPO "bug" --color "ee0701" --description "Something isn't working" || true
gh label create --repo $REPO "enhancement" --color "84b6eb" --description "New feature or request" || true
gh label create --repo $REPO "documentation" --color "1d76db" --description "Improvements or additions to documentation" || true
gh label create --repo $REPO "maintenance" --color "660e7a" --description "Code maintenance and refactoring" || true

# Status Labels
gh label create --repo $REPO "help-wanted" --color "159818" --description "Extra attention is needed" || true
gh label create --repo $REPO "good-first-issue" --color "7057ff" --description "Good for newcomers" || true
gh label create --repo $REPO "wontfix" --color "ffffff" --description "This will not be worked on" || true
gh label create --repo $REPO "duplicate" --color "cccccc" --description "This issue or pull request already exists" || true

# Component Labels
gh label create --repo $REPO "transport-stdio" --color "ff9500" --description "STDIO transport related" || true
gh label create --repo $REPO "transport-http" --color "ff9500" --description "HTTP transport related" || true
gh label create --repo $REPO "transport-websocket" --color "ff9500" --description "WebSocket transport related" || true
gh label create --repo $REPO "core-tools" --color "0366d6" --description "Tool system related" || true
gh label create --repo $REPO "core-resources" --color "0366d6" --description "Resource system related" || true
gh label create --repo $REPO "core-prompts" --color "0366d6" --description "Prompt system related" || true

# Integration Labels
gh label create --repo $REPO "integration-claude" --color "5319e7" --description "Claude Desktop integration" || true
gh label create --repo $REPO "integration-cursor" --color "5319e7" --description "Cursor IDE integration" || true
gh label create --repo $REPO "integration-vscode" --color "5319e7" --description "VS Code integration" || true
gh label create --repo $REPO "integration-other" --color "5319e7" --description "Other client integrations" || true

# Effort Labels
gh label create --repo $REPO "effort-small" --color "c2e0c6" --description "Small effort, 1-2 hours" || true
gh label create --repo $REPO "effort-medium" --color "fef2c0" --description "Medium effort, 1-2 days" || true
gh label create --repo $REPO "effort-large" --color "f9d0c4" --description "Large effort, 1+ weeks" || true

# Quality Labels
gh label create --repo $REPO "performance" --color "ff6b6b" --description "Performance improvements" || true
gh label create --repo $REPO "security" --color "d93f0b" --description "Security related" || true
gh label create --repo $REPO "testing" --color "0052cc" --description "Testing improvements" || true
gh label create --repo $REPO "ci-cd" --color "ededed" --description "Continuous integration and deployment" || true

# Feature Labels
gh label create --repo $REPO "feature-flag" --color "bfd4f2" --description "Feature flag related" || true
gh label create --repo $REPO "breaking-change" --color "b60205" --description "Introduces breaking changes" || true
gh label create --repo $REPO "backward-compatible" --color "0e8a16" --description "Backward compatible changes" || true

echo "[x] All labels created successfully!"
echo ""
echo " Available labels:"
gh label list --repo $REPO

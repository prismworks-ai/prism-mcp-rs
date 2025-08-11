#!/bin/bash

# Generate documentation with proper headers and issue reporting links

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "## Generating documentation..."

# Change to project root
cd "$PROJECT_ROOT"

# Generate API documentation from source code
echo "  - Generating API documentation from Rust source..."
cargo doc --no-deps --all-features

# Convert rustdoc to markdown if the script exists
if [ -f "$SCRIPT_DIR/rustdoc-to-markdown.py" ]; then
    echo "  Note: Converting rustdoc to markdown..."
    python3 "$SCRIPT_DIR/rustdoc-to-markdown.py"
fi

# Add documentation headers with issue reporting links
echo "  🏷️ Adding documentation headers and issue reporting links..."
python3 "$SCRIPT_DIR/add-doc-headers.py"

# Update documentation index
if [ -f "$SCRIPT_DIR/update-docs-index.py" ]; then
    echo "  📇 Updating documentation index..."
    python3 "$SCRIPT_DIR/update-docs-index.py"
fi

# Check documentation quality
if [ -f "$SCRIPT_DIR/check-docs-quality.py" ]; then
    echo "  [x] Checking documentation quality..."
    python3 "$SCRIPT_DIR/check-docs-quality.py"
fi

echo "* Documentation generation complete!"
echo "   All docs now include:"
echo "   - Clear manual/auto-generated labels"
echo "   - 2-click issue reporting links"
echo "   - Proper metadata headers"

#!/bin/bash

# improved documentation generation with v2 headers and 2-click issue reporting

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "## improved Documentation Generation v2"
echo "=====================================\n"

# Change to project root
cd "$PROJECT_ROOT"

# Ensure .local directory exists for metadata
mkdir -p .local

# Step 1: Generate API documentation from source code
echo "- Step 1: Generating API documentation from Rust source..."
cargo doc --no-deps --all-features --document-private-items

if [ $? -eq 0 ]; then
    echo "  [x] Rust documentation generated successfully"
else
    echo "  Warning:  Warning: Rust documentation generation had issues"
fi

# Step 2: Convert rustdoc to markdown (if script exists)
if [ -f "$SCRIPT_DIR/rustdoc-to-markdown.py" ]; then
    echo "\nNote: Step 2: Converting rustdoc to markdown..."
    python3 "$SCRIPT_DIR/rustdoc-to-markdown.py"
    if [ $? -eq 0 ]; then
        echo "  [x] Conversion complete"
    else
        echo "  Warning:  Conversion had issues"
    fi
else
    echo "\n⏭️  Step 2: Skipping rustdoc conversion (script not found)"
fi

# Step 3: Add improved v2 documentation headers
echo "\n🏷️  Step 3: Adding improved documentation headers..."
python3 "$SCRIPT_DIR/add-doc-headers-v2.py"

# Step 4: Update documentation index
if [ -f "$SCRIPT_DIR/update-docs-index.py" ]; then
    echo "\n📇 Step 4: Updating documentation index..."
    python3 "$SCRIPT_DIR/update-docs-index.py"
    if [ $? -eq 0 ]; then
        echo "  [x] Index updated"
    fi
else
    echo "\n⏭️  Step 4: Skipping index update (script not found)"
fi

# Step 5: Check documentation quality
if [ -f "$SCRIPT_DIR/check-docs-quality.py" ]; then
    echo "\n[x] Step 5: Checking documentation quality..."
    python3 "$SCRIPT_DIR/check-docs-quality.py"
else
    echo "\n⏭️  Step 5: Skipping quality check (script not found)"
fi

# Step 6: Generate documentation report
echo "\n📊 Step 6: Generating documentation report..."

# Count documentation files
MANUAL_COUNT=$(find docs -name "*.md" -exec grep -l "Manually Written" {} \; 2>/dev/null | wc -l | tr -d ' ')
GENERATED_COUNT=$(find docs -name "*.md" -exec grep -l "Auto-Generated" {} \; 2>/dev/null | wc -l | tr -d ' ')
TOTAL_COUNT=$(find docs -name "*.md" 2>/dev/null | wc -l | tr -d ' ')

echo ""
echo "📈 Documentation Statistics:"
echo "  • Total documentation files: $TOTAL_COUNT"
echo "  • Manually written: $MANUAL_COUNT"
echo "  • Auto-generated: $GENERATED_COUNT"
echo "  • Without headers: $((TOTAL_COUNT - MANUAL_COUNT - GENERATED_COUNT))"

echo ""
echo "* Documentation generation complete!"
echo ""
echo "## All documentation now includes:"
echo "  [x] Clear manual/auto-generated labels"
echo "  [x] 2-click issue reporting system"
echo "  [x] clean badge-based UI"
echo "  [x] Metadata tracking for changes"
echo ""
echo "Note: Issue Reporting Flow:"
echo "  1. User clicks the 'Report Issue' button"
echo "  2. GitHub issue form opens with all metadata pre-filled"
echo "  3. User just describes the issue and submits"
echo ""
echo "Note: Tip: Run 'make docs' to regenerate documentation"

#!/bin/bash

# Install pre-commit hook for doc-driven examples validation

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
HOOK_PATH="$PROJECT_ROOT/.git/hooks/pre-commit"

echo "🔧 Installing pre-commit hook for doc-driven examples..."

# Check if we're in a git repository
if [ ! -d "$PROJECT_ROOT/.git" ]; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$PROJECT_ROOT/.git/hooks"

# Check if pre-commit hook already exists
if [ -f "$HOOK_PATH" ]; then
    echo "⚠️  Warning: pre-commit hook already exists"
    echo "Current content:"
    echo "---"
    head -5 "$HOOK_PATH"
    echo "..."
    echo "---"
    echo ""
    read -p "Do you want to append the examples validation? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "🚫 Installation cancelled"
        exit 0
    fi
    
    # Append to existing hook
    echo "" >> "$HOOK_PATH"
    echo "# Doc-driven examples validation" >> "$HOOK_PATH"
    echo "$SCRIPT_DIR/pre-commit-examples.sh" >> "$HOOK_PATH"
    echo "if [ $? -ne 0 ]; then" >> "$HOOK_PATH"
    echo "    exit 1" >> "$HOOK_PATH"
    echo "fi" >> "$HOOK_PATH"
else
    # Create new hook
    cat > "$HOOK_PATH" << 'EOF'
#!/bin/bash

# Pre-commit hook
set -e

# Doc-driven examples validation
EOF
    echo "$SCRIPT_DIR/pre-commit-examples.sh" >> "$HOOK_PATH"
    echo "if [ \$? -ne 0 ]; then" >> "$HOOK_PATH"
    echo "    exit 1" >> "$HOOK_PATH"
    echo "fi" >> "$HOOK_PATH"
fi

# Make hooks executable
chmod +x "$HOOK_PATH"
chmod +x "$SCRIPT_DIR/pre-commit-examples.sh"

echo "✅ Pre-commit hook installed successfully!"
echo ""
echo "The hook will validate documentation examples before each commit."
echo "To bypass the hook (not recommended), use: git commit --no-verify"
echo ""
echo "To uninstall, run:"
echo "  rm $HOOK_PATH"

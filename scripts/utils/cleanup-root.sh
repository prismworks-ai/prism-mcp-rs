#!/bin/bash

# Clean up root folder and organize files according to Rust SDK best practices

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🧹 Cleaning up project root folder"
echo "==================================\n"

cd "$PROJECT_ROOT"

# Create .local subdirectories if they don't exist
echo " Setting up .local directory structure..."
mkdir -p .local/{dev,temp,logs,build-artifacts,docs-backup,drafts,notes,scratch}

# Move temporary and development files to .local
echo "\n🚚 Moving temporary files to .local/temp..."
find . -maxdepth 1 -type f \( -name "*.tmp" -o -name "*.bak" -o -name "*.swp" -o -name "*.old" \) -exec mv {} .local/temp/ \; 2>/dev/null || true

# Move log files
echo " Moving log files to .local/logs..."
find . -maxdepth 1 -type f -name "*.log" -exec mv {} .local/logs/ \; 2>/dev/null || true

# Move TODO and NOTES files
echo "Note: Moving notes and todo files to .local/notes..."
find . -maxdepth 1 -type f \( -name "TODO*" -o -name "NOTES*" -o -name "IDEAS*" \) -exec mv {} .local/notes/ \; 2>/dev/null || true

# Check for non-standard files in root
echo "\nSearch: Checking for non-standard files in root..."
NON_STANDARD_FILES=$(find . -maxdepth 1 -type f ! \( \
    -name "Cargo.toml" -o \
    -name "Cargo.lock" -o \
    -name "README.md" -o \
    -name "LICENSE" -o \
    -name "CHANGELOG.md" -o \
    -name "CONTRIBUTING.md" -o \
    -name "CODE_OF_CONDUCT.md" -o \
    -name "Makefile" -o \
    -name "build.rs" -o \
    -name "deny.toml" -o \
    -name "rustfmt.toml" -o \
    -name "clippy.toml" -o \
    -name "codecov.yml" -o \
    -name ".gitignore" -o \
    -name ".rustfmt.ignore" -o \
    -name ".*" \) | sort)

if [ -n "$NON_STANDARD_FILES" ]; then
    echo "Warning:  Found non-standard files in root:"
    echo "$NON_STANDARD_FILES"
    echo ""
    read -p "Move these to .local/scratch? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "$NON_STANDARD_FILES" | while read -r file; do
            if [ -f "$file" ]; then
                mv "$file" .local/scratch/
                echo "  Moved: $file -> .local/scratch/"
            fi
        done
    fi
else
    echo "  [x] No non-standard files found"
fi

# Organize scripts folder
echo "\n Organizing scripts folder..."
mkdir -p scripts/{ci,docs,dev,utils}

# Move CI-related scripts
for script in ci-check.sh local-ci.sh integration_test.sh validate-workflows.sh pre-push; do
    if [ -f "scripts/$script" ]; then
        mv "scripts/$script" "scripts/ci/" 2>/dev/null || true
        echo "  Moved $script to scripts/ci/"
    fi
done

# Move documentation scripts
for script in add-doc-headers.py add-doc-headers-v2.py generate-docs.sh generate-docs-v2.sh rustdoc-to-markdown.py update-docs-index.py check-docs-quality.py fix-final-docs.py restructure-docs.py; do
    if [ -f "scripts/$script" ]; then
        mv "scripts/$script" "scripts/docs/" 2>/dev/null || true
        echo "  Moved $script to scripts/docs/"
    fi
done

# Move development scripts
for script in setup-dev.sh verify-environment.sh; do
    if [ -f "scripts/$script" ]; then
        mv "scripts/$script" "scripts/dev/" 2>/dev/null || true
        echo "  Moved $script to scripts/dev/"
    fi
done

# Move utility scripts
for script in badge-manager.sh diagnose-badges.sh fix-badges.sh refresh-badges.sh create-labels.sh verify-publication.sh cleanup-root.sh; do
    if [ -f "scripts/$script" ]; then
        mv "scripts/$script" "scripts/utils/" 2>/dev/null || true
        echo "  Moved $script to scripts/utils/"
    fi
done

# Update .gitignore to ensure .local is ignored
echo "\nNote: Updating .gitignore..."
if ! grep -q "^\.local/" .gitignore 2>/dev/null; then
    echo "\n# Local development files" >> .gitignore
    echo ".local/" >> .gitignore
    echo "  Added .local/ to .gitignore"
else
    echo "  .local/ already in .gitignore"
fi

# Create a .local/README.md
echo "\n📄 Creating .local/README.md..."
cat > .local/README.md << 'EOF'
# Local Development Directory

This directory contains local development files that should not be committed to the repository.

## Directory Structure

- `dev/` - Development scripts and tools
- `temp/` - Temporary files (.tmp, .bak, .swp)
- `logs/` - Log files from builds and tests
- `build-artifacts/` - Build outputs and artifacts
- `docs-backup/` - Documentation backups
- `drafts/` - Work in progress documents
- `notes/` - Personal notes, TODOs, ideas
- `scratch/` - Miscellaneous files
- `docs-metadata.json` - Documentation generation metadata

## Usage

All files in this directory are git-ignored and local to your development environment.

To clean this directory:
```bash
rm -rf .local/temp/* .local/logs/*
```

To backup important notes:
```bash
tar -czf local-backup-$(date +%Y%m%d).tar.gz .local/notes .local/drafts
```
EOF

echo "  Created .local/README.md"

# Summary
echo "\n* Cleanup complete!"
echo "\n📊 Project structure now follows Rust SDK best practices:"
echo "  [x] Root folder contains only standard Rust project files"
echo "  [x] Scripts organized into categories (ci/, docs/, dev/, utils/)"
echo "  [x] Local development files moved to .local/"
echo "  [x] .gitignore updated to exclude .local/"
echo "\nNote: Tips:"
echo "  • Use .local/ for any temporary or personal development files"
echo "  • Run 'make clean' to clean build artifacts"
echo "  • Run 'rm -rf .local/temp/*' to clean temporary files"

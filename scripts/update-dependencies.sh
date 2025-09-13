#!/bin/bash
# Dependency Update Script
# Safely update dependencies with backup and verification

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATE=$(date +%Y-%m-%d)
BACKUP_DIR="${PROJECT_ROOT}/.dependency-backups"

# Create backup directory
mkdir -p "${BACKUP_DIR}"

echo "🔄 Dependency Update Process - ${DATE}"
echo "======================================"

# 1. Create backup
echo "1. Creating backup..."
cp Cargo.lock "${BACKUP_DIR}/Cargo.lock.${DATE}.backup"
echo "   📋 Backup saved: ${BACKUP_DIR}/Cargo.lock.${DATE}.backup"

# 2. Check current status
echo "2. Checking current status..."
echo "   📊 Current dependency count: $(wc -l < Cargo.lock) entries"

# 3. Preview updates
echo "3. Previewing available updates..."
if cargo update --dry-run; then
    echo "   ✅ Updates available"
    read -p "   🤔 Continue with updates? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "   ❌ Update cancelled"
        exit 0
    fi
else
    echo "   ℹ️ No updates available"
    exit 0
fi

# 4. Apply updates
echo "4. Applying updates..."
if cargo update; then
    echo "   ✅ Dependencies updated successfully"
else
    echo "   ❌ Update failed - restoring backup"
    cp "${BACKUP_DIR}/Cargo.lock.${DATE}.backup" Cargo.lock
    echo "   🔙 Backup restored"
    exit 1
fi

# 5. Verify build
echo "5. Verifying build..."
if cargo check --all-features; then
    echo "   ✅ Build verification passed"
else
    echo "   ❌ Build failed - restoring backup"
    cp "${BACKUP_DIR}/Cargo.lock.${DATE}.backup" Cargo.lock
    echo "   🔙 Backup restored"
    exit 1
fi

# 6. Run security checks
echo "6. Running security checks..."
if cargo deny check all; then
    echo "   ✅ Security checks passed"
else
    echo "   ⚠️ Security check issues found"
    read -p "   🤔 Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "   ❌ Rolling back due to security concerns"
        cp "${BACKUP_DIR}/Cargo.lock.${DATE}.backup" Cargo.lock
        echo "   🔙 Backup restored"
        exit 1
    fi
fi

# 7. Update supply chain exemptions if needed
echo "7. Checking supply chain status..."
if ! cargo vet check; then
    echo "   ⚠️ New dependencies need vetting"
    echo "   📄 Run 'cargo vet check' to see details"
    echo "   📄 Run 'cargo vet certify' to complete audits"
else
    echo "   ✅ Supply chain verification passed"
fi

# 8. Generate update report
echo "8. Generating update report..."
cat > "${BACKUP_DIR}/update-report-${DATE}.md" << EOF
# Dependency Update Report - ${DATE}

## Summary
Dependencies updated on $(date)

## Changes
\`\`\`
$(git diff "${BACKUP_DIR}/Cargo.lock.${DATE}.backup" Cargo.lock || echo "No git repository or changes too large")
\`\`\`

## Verification
- Build check: ✅ Passed
- Security check: $(cargo deny check all >/dev/null 2>&1 && echo "✅ Passed" || echo "⚠️ Issues found")
- Supply chain: $(cargo vet check >/dev/null 2>&1 && echo "✅ Passed" || echo "⚠️ Needs attention")

## Next Steps
1. Run tests: \`cargo test\`
2. Update documentation if needed
3. Consider updating CI/CD if new features used
4. Monitor for any runtime issues

## Rollback Instructions
If issues are found:
\`\`\`bash
cp "${BACKUP_DIR}/Cargo.lock.${DATE}.backup" Cargo.lock
cargo check
\`\`\`
EOF

echo "   📋 Update report: ${BACKUP_DIR}/update-report-${DATE}.md"
echo ""
echo "✅ Dependency update completed successfully!"
echo ""
echo "📝 Next steps:"
echo "   1. Run tests: cargo test"
echo "   2. Review changes: git diff"
echo "   3. Commit changes: git add Cargo.lock && git commit -m 'Update dependencies'"
echo "   4. Monitor for issues in development"

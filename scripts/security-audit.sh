#!/bin/bash
# Supply Chain Security Audit Script
# Run weekly or on-demand for security monitoring

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATE=$(date +%Y-%m-%d)
REPORT_DIR="${PROJECT_ROOT}/reports/security"

# Create reports directory
mkdir -p "${REPORT_DIR}"

echo "🔍 Running Supply Chain Security Audit - ${DATE}"
echo "================================================"

# 1. Security Vulnerability Scan
echo "1. Scanning for security vulnerabilities..."
if cargo audit --version >/dev/null 2>&1; then
    echo "   ✅ Running cargo-audit"
    cargo audit --json > "${REPORT_DIR}/audit-${DATE}.json" 2>/dev/null || {
        echo "   ⚠️  Vulnerabilities found or network issues"
        cargo audit > "${REPORT_DIR}/audit-${DATE}.txt" || true
    }
    echo "   📄 Report saved to: ${REPORT_DIR}/audit-${DATE}.json"
else
    echo "   ❌ cargo-audit not installed"
fi

# 2. Supply Chain Policy Check
echo "2. Checking supply chain policies..."
if cargo deny --version >/dev/null 2>&1; then
    echo "   ✅ Running cargo-deny"
    if cargo deny check all; then
        echo "   ✅ All policy checks passed"
    else
        echo "   ⚠️  Policy violations detected"
        cargo deny check all > "${REPORT_DIR}/deny-${DATE}.txt" 2>&1 || true
    fi
else
    echo "   ❌ cargo-deny not installed"
fi

# 3. Supply Chain Verification
echo "3. Verifying supply chain..."
if cargo vet --version >/dev/null 2>&1; then
    echo "   ✅ Running cargo-vet"
    if cargo vet check; then
        echo "   ✅ Supply chain verification passed"
    else
        echo "   ⚠️  Unvetted dependencies found"
        cargo vet check > "${REPORT_DIR}/vet-${DATE}.txt" 2>&1 || true
    fi
else
    echo "   ❌ cargo-vet not installed"
fi

# 4. Dependency Analysis
echo "4. Analyzing dependencies..."
echo "   📊 Dependency tree analysis"
cargo tree --duplicates > "${REPORT_DIR}/duplicates-${DATE}.txt" 2>/dev/null || true

if cargo outdated --version >/dev/null 2>&1; then
    echo "   📊 Checking for outdated dependencies"
    cargo outdated > "${REPORT_DIR}/outdated-${DATE}.txt" 2>/dev/null || true
fi

# 5. Unsafe Code Analysis
echo "5. Analyzing unsafe code usage..."
echo "   🔍 Scanning for unsafe blocks"
grep -r "unsafe" src/ --include="*.rs" > "${REPORT_DIR}/unsafe-${DATE}.txt" 2>/dev/null || {
    echo "No unsafe blocks found" > "${REPORT_DIR}/unsafe-${DATE}.txt"
}

# 6. Generate Summary Report
echo "6. Generating summary report..."
cat > "${REPORT_DIR}/summary-${DATE}.md" << EOF
# Security Audit Summary - ${DATE}

## Overview
This report summarizes the security audit performed on $(date).

## Tools Used
- cargo-audit: Security vulnerability scanning
- cargo-deny: License and policy enforcement
- cargo-vet: Supply chain verification
- cargo-outdated: Dependency freshness analysis

## Files Generated
$(ls -la "${REPORT_DIR}"/*-${DATE}.* | sed 's/^/- /')

## Next Steps
1. Review any vulnerabilities found in audit-${DATE}.json
2. Address policy violations in deny-${DATE}.txt
3. Complete audits for unvetted dependencies in vet-${DATE}.txt
4. Consider updating outdated dependencies in outdated-${DATE}.txt

## Recommendations
- Schedule next audit in 1 week
- Monitor for new advisories
- Keep dependencies updated
EOF

echo "📋 Summary report: ${REPORT_DIR}/summary-${DATE}.md"
echo ""
echo "✅ Security audit completed!"
echo "📁 Reports saved in: ${REPORT_DIR}/"
echo ""
echo "🔄 Schedule next run with: crontab -e"
echo "   Add: 0 9 * * 1 cd ${PROJECT_ROOT} && ./scripts/security-audit.sh"

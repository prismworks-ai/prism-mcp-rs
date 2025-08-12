#!/bin/bash
# Generate markdown coverage report from cargo-llvm-cov output
# This script is called from GitHub Actions after coverage runs

set -e

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPORT_DIR="reports"
COVERAGE_FILE="$REPORT_DIR/coverage-report.md"
DATE=$(date -u +"%Y-%m-%d %H:%M:%S UTC")
COMMIT_SHA=${GITHUB_SHA:-$(git rev-parse HEAD)}
COMMIT_SHORT=${COMMIT_SHA:0:8}
BRANCH=${GITHUB_REF_NAME:-$(git branch --show-current)}
RUN_ID=${GITHUB_RUN_ID:-"local"}

echo -e "${GREEN}📊 Generating Coverage Report${NC}"

# Ensure reports directory exists
mkdir -p "$REPORT_DIR"

# Run coverage with all output formats
echo "Running cargo-llvm-cov..."
cargo llvm-cov --all-features --workspace \
    --lcov --output-path target/coverage.lcov

# Also generate text output for parsing
cargo llvm-cov --all-features --workspace \
    --text > target/coverage.txt

# Generate JSON if possible (for detailed parsing)
cargo llvm-cov --all-features --workspace \
    --json > target/coverage.json 2>/dev/null || true

# Generate HTML for browsing
cargo llvm-cov --all-features --workspace \
    --html --output-dir target/coverage-html

# Extract coverage percentage from text output
if [[ -f target/coverage.txt ]]; then
    COVERAGE_PCT=$(grep -E "^TOTAL" target/coverage.txt | awk '{print $(NF-1)}' | sed 's/%//' | head -1)
else
    # Fallback - run again to get text output
    cargo llvm-cov report --all-features --workspace 2>/dev/null | grep -E "^TOTAL" | awk '{print $(NF-1)}' | sed 's/%//' > /tmp/coverage_pct.txt
    COVERAGE_PCT=$(cat /tmp/coverage_pct.txt 2>/dev/null || echo "0")
fi

# Ensure we have a valid percentage
if [[ -z "$COVERAGE_PCT" ]] || [[ "$COVERAGE_PCT" == "" ]]; then
    COVERAGE_PCT="0"
fi

# Determine coverage status badge color
if (( $(echo "$COVERAGE_PCT >= 80" | bc -l) )); then
    BADGE_COLOR="brightgreen"
    STATUS_EMOJI="✅"
elif (( $(echo "$COVERAGE_PCT >= 60" | bc -l) )); then
    BADGE_COLOR="yellow"
    STATUS_EMOJI="⚠️"
else
    BADGE_COLOR="red"
    STATUS_EMOJI="❌"
fi

# Parse JSON for detailed metrics if available
if [[ -f target/coverage.json ]] && command -v jq &> /dev/null; then
    LINES_COVERED=$(jq '.data[0].totals.lines.covered' target/coverage.json 2>/dev/null || echo "0")
    LINES_TOTAL=$(jq '.data[0].totals.lines.count' target/coverage.json 2>/dev/null || echo "0")
    FUNCTIONS_COVERED=$(jq '.data[0].totals.functions.covered' target/coverage.json 2>/dev/null || echo "0")
    FUNCTIONS_TOTAL=$(jq '.data[0].totals.functions.count' target/coverage.json 2>/dev/null || echo "0")
    BRANCHES_COVERED=$(jq '.data[0].totals.branches.covered // 0' target/coverage.json 2>/dev/null || echo "0")
    BRANCHES_TOTAL=$(jq '.data[0].totals.branches.count // 0' target/coverage.json 2>/dev/null || echo "0")
else
    # Fallback to parsing text output or use placeholder values
    if [[ -f target/coverage.txt ]]; then
        TOTAL_LINE=$(grep "^TOTAL" target/coverage.txt | head -1)
        if [[ -n "$TOTAL_LINE" ]]; then
            # Try to extract numbers from TOTAL line
            LINES_COVERED=$(echo "$TOTAL_LINE" | awk '{print $2}' | tr -d ',')
            LINES_TOTAL=$(echo "$TOTAL_LINE" | awk '{print $4}' | tr -d ',')
        fi
    fi
    
    # Use defaults if not found
    LINES_COVERED=${LINES_COVERED:-"N/A"}
    LINES_TOTAL=${LINES_TOTAL:-"N/A"}
    FUNCTIONS_COVERED="N/A"
    FUNCTIONS_TOTAL="N/A"
    BRANCHES_COVERED="N/A"
    BRANCHES_TOTAL="N/A"
fi

# Generate markdown report
cat > "$COVERAGE_FILE" << EOF
# Coverage Report

> Generated: $DATE  
> Branch: \`$BRANCH\`  
> Commit: \`$COMMIT_SHORT\`  
> Run ID: $RUN_ID

## Summary

![Coverage Badge](https://img.shields.io/badge/coverage-${COVERAGE_PCT}%25-${BADGE_COLOR})

| Metric | Coverage | Status |
|--------|----------|--------|
| **Overall** | ${COVERAGE_PCT}% | $STATUS_EMOJI |
| **Lines** | $LINES_COVERED / $LINES_TOTAL | $(if [[ "$LINES_TOTAL" != "N/A" ]] && [[ "$LINES_TOTAL" != "0" ]]; then echo "scale=1; $LINES_COVERED * 100 / $LINES_TOTAL" | bc 2>/dev/null || echo "N/A"; else echo "N/A"; fi)% |
| **Functions** | $FUNCTIONS_COVERED / $FUNCTIONS_TOTAL | $(if [[ "$FUNCTIONS_TOTAL" != "N/A" ]] && [[ "$FUNCTIONS_TOTAL" != "0" ]]; then echo "scale=1; $FUNCTIONS_COVERED * 100 / $FUNCTIONS_TOTAL" | bc 2>/dev/null || echo "N/A"; else echo "N/A"; fi)% |
| **Branches** | $BRANCHES_COVERED / $BRANCHES_TOTAL | $(if [[ "$BRANCHES_TOTAL" != "N/A" ]] && [[ "$BRANCHES_TOTAL" != "0" ]]; then echo "scale=1; $BRANCHES_COVERED * 100 / $BRANCHES_TOTAL" | bc 2>/dev/null || echo "N/A"; else echo "N/A"; fi)% |

## Module Coverage

| Module | Lines | Functions | Coverage |
|--------|-------|-----------|----------|
EOF

# Parse per-module coverage from text output
grep -E "^src/" target/coverage.txt | while read -r line; do
    MODULE=$(echo "$line" | awk '{print $1}')
    MODULE_LINES=$(echo "$line" | awk '{print $2 "/" $4}')
    MODULE_PCT=$(echo "$line" | awk '{print $(NF-1)}')
    echo "| \`$MODULE\` | $MODULE_LINES | - | $MODULE_PCT |" >> "$COVERAGE_FILE"
done

# Add uncovered files section
cat >> "$COVERAGE_FILE" << EOF

## Uncovered Files

Files with 0% coverage:

EOF

# Find files with 0% coverage
grep -E "^src/.*0\.00%" target/coverage.txt | while read -r line; do
    FILE=$(echo "$line" | awk '{print $1}')
    echo "- \`$FILE\`" >> "$COVERAGE_FILE"
done

# Add coverage trends (if history exists)
if [[ -f "$REPORT_DIR/coverage-history.json" ]]; then
    cat >> "$COVERAGE_FILE" << EOF

## Coverage Trend

\`\`\`mermaid
xychart-beta
    title "Coverage Trend (Last 10 Runs)"
    x-axis [$(tail -10 "$REPORT_DIR/coverage-history.json" | jq -r '.date' | tr '\n' ',' | sed 's/,$//')]
    y-axis "Coverage %" 0 --> 100
    line [$(tail -10 "$REPORT_DIR/coverage-history.json" | jq -r '.coverage' | tr '\n' ',' | sed 's/,$//')]
\`\`\`
EOF
fi

# Add links section
cat >> "$COVERAGE_FILE" << EOF

## Additional Reports

- [HTML Coverage Report](../target/coverage-html/index.html)
- [LCOV Report](../target/coverage.lcov)
- [JSON Report](../target/coverage.json)

## Thresholds

| Type | Threshold | Current | Status |
|------|-----------|---------|--------|
| Line Coverage | 60% | ${COVERAGE_PCT}% | $(if (( $(echo "$COVERAGE_PCT >= 60" | bc -l) )); then echo "✅ Pass"; else echo "❌ Fail"; fi) |
| Function Coverage | 60% | $(if [[ "$FUNCTIONS_TOTAL" != "N/A" ]]; then echo "scale=1; $FUNCTIONS_COVERED * 100 / $FUNCTIONS_TOTAL" | bc; else echo "N/A"; fi)% | $(if [[ "$FUNCTIONS_TOTAL" != "N/A" ]] && (( $(echo "$FUNCTIONS_COVERED * 100 / $FUNCTIONS_TOTAL >= 60" | bc -l) )); then echo "✅ Pass"; else echo "⚠️ Check"; fi) |

---

*Generated by cargo-llvm-cov with prism-mcp-rs CI/CD pipeline*
EOF

# Append to history (for trends)
if command -v jq &> /dev/null; then
    echo "{\"date\": \"$(date +%Y-%m-%d)\", \"coverage\": $COVERAGE_PCT, \"commit\": \"$COMMIT_SHORT\"}" >> "$REPORT_DIR/coverage-history.json"
fi

echo -e "${GREEN}✅ Coverage report generated: $COVERAGE_FILE${NC}"
echo -e "${GREEN}   Coverage: ${COVERAGE_PCT}% ${STATUS_EMOJI}${NC}"

# Exit with failure if coverage is below threshold (optional)
if (( $(echo "$COVERAGE_PCT < 60" | bc -l) )); then
    echo -e "${RED}❌ Coverage ${COVERAGE_PCT}% is below minimum threshold of 60%${NC}"
    # Uncomment to fail the build on low coverage:
    # exit 1
fi
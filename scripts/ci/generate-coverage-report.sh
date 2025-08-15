#!/bin/bash

# Generate Coverage Report Script
# This script generates a markdown coverage report from lcov.info file
# and prepares it for upload to codecov

set -e

echo "📊 Generating coverage report..."

# Check if lcov.info exists
if [ ! -f "lcov.info" ]; then
    echo "❌ Error: lcov.info not found. Running cargo llvm-cov to generate it..."
    cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
fi

# Create reports directory if it doesn't exist
mkdir -p reports

# Generate HTML report for detailed viewing (optional)
if command -v cargo-llvm-cov &> /dev/null; then
    echo "📈 Generating HTML coverage report..."
    cargo llvm-cov --all-features --workspace --html --output-dir target/llvm-cov/html
fi

# Parse lcov.info to generate markdown report
echo "📝 Creating markdown coverage report..."

# Initialize coverage report
cat > reports/coverage-report.md << 'EOF'
# Coverage Report

## Summary

Generated on: $(date)

EOF

date >> reports/coverage-report.md
echo "" >> reports/coverage-report.md

# Calculate total coverage percentage from lcov.info
if [ -f "lcov.info" ]; then
    # Extract line coverage
    LINES_FOUND=$(grep -E '^LF:' lcov.info | awk -F: '{sum += $2} END {print sum}')
    LINES_HIT=$(grep -E '^LH:' lcov.info | awk -F: '{sum += $2} END {print sum}')
    
    if [ -n "$LINES_FOUND" ] && [ "$LINES_FOUND" -gt 0 ]; then
        COVERAGE=$(echo "scale=2; $LINES_HIT * 100 / $LINES_FOUND" | bc)
        echo "### Overall Coverage: ${COVERAGE}%" >> reports/coverage-report.md
        echo "" >> reports/coverage-report.md
        echo "- **Lines Found:** $LINES_FOUND" >> reports/coverage-report.md
        echo "- **Lines Hit:** $LINES_HIT" >> reports/coverage-report.md
        echo "" >> reports/coverage-report.md
        
        # Add coverage badge
        if (( $(echo "$COVERAGE >= 80" | bc -l) )); then
            BADGE_COLOR="green"
        elif (( $(echo "$COVERAGE >= 60" | bc -l) )); then
            BADGE_COLOR="yellow"
        else
            BADGE_COLOR="red"
        fi
        
        echo "![Coverage](https://img.shields.io/badge/coverage-${COVERAGE}%25-${BADGE_COLOR})" >> reports/coverage-report.md
        echo "" >> reports/coverage-report.md
    fi
    
    # Module-level coverage
    echo "## Module Coverage" >> reports/coverage-report.md
    echo "" >> reports/coverage-report.md
    echo "| Module | Coverage | Lines |" >> reports/coverage-report.md
    echo "|--------|----------|-------|" >> reports/coverage-report.md
    
    # Parse source files from lcov.info
    while IFS= read -r line; do
        if [[ $line == SF:* ]]; then
            SOURCE_FILE=${line#SF:}
            MODULE_NAME=$(basename "$SOURCE_FILE" .rs)
            
            # Get the next LF and LH values for this file
            LF_LINE=$(grep -A 100 "SF:$SOURCE_FILE" lcov.info | grep -m 1 '^LF:' | head -1)
            LH_LINE=$(grep -A 100 "SF:$SOURCE_FILE" lcov.info | grep -m 1 '^LH:' | head -1)
            
            if [ -n "$LF_LINE" ] && [ -n "$LH_LINE" ]; then
                LF=${LF_LINE#LF:}
                LH=${LH_LINE#LH:}
                
                if [ "$LF" -gt 0 ]; then
                    MODULE_COV=$(echo "scale=1; $LH * 100 / $LF" | bc)
                    echo "| $MODULE_NAME | ${MODULE_COV}% | $LH/$LF |" >> reports/coverage-report.md
                fi
            fi
        fi
    done < lcov.info
    
    echo "" >> reports/coverage-report.md
fi

# Add test execution summary if available
if [ -f "target/nextest/ci/junit.xml" ]; then
    echo "## Test Execution Summary" >> reports/coverage-report.md
    echo "" >> reports/coverage-report.md
    # Parse junit.xml for test results (simplified)
    echo "Test results available in junit.xml" >> reports/coverage-report.md
    echo "" >> reports/coverage-report.md
fi

# Instructions for codecov
echo "## Codecov Integration" >> reports/coverage-report.md
echo "" >> reports/coverage-report.md
echo "Coverage data has been generated in lcov.info format and is ready for upload to Codecov." >> reports/coverage-report.md
echo "" >> reports/coverage-report.md
echo "To view detailed coverage:" >> reports/coverage-report.md
echo "1. Check the Codecov dashboard after CI completes" >> reports/coverage-report.md
echo "2. View HTML report in \`target/llvm-cov/html/index.html\` (if generated locally)" >> reports/coverage-report.md
echo "" >> reports/coverage-report.md

echo "✅ Coverage report generated successfully!"
echo "📄 Markdown report: reports/coverage-report.md"
echo "📊 LCOV data: lcov.info (ready for Codecov upload)"

# Verify lcov.info exists for codecov upload
if [ ! -f "lcov.info" ]; then
    echo "⚠️  Warning: lcov.info not found - Codecov upload may fail"
    exit 1
fi

exit 0
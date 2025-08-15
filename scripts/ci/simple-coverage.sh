#!/bin/bash

# Simple Coverage Report Fallback Script
# Used when the main coverage script fails

set -e

echo "📊 Running simple coverage report (fallback)..."

# Create reports directory
mkdir -p reports

# Generate basic coverage with cargo llvm-cov
if command -v cargo-llvm-cov &> /dev/null; then
    echo "Generating coverage with cargo-llvm-cov..."
    cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
    
    # Generate a simple text report
    cargo llvm-cov --all-features --workspace --text > reports/coverage.txt
    
    # Create a basic markdown report
    cat > reports/coverage-report.md << 'EOF'
# Coverage Report (Simplified)

Generated on: $(date)

## Coverage Summary

EOF
    date >> reports/coverage-report.md
    echo "" >> reports/coverage-report.md
    echo "\`\`\`" >> reports/coverage-report.md
    tail -20 reports/coverage.txt >> reports/coverage-report.md
    echo "\`\`\`" >> reports/coverage-report.md
    echo "" >> reports/coverage-report.md
    echo "Full coverage data available in lcov.info for Codecov upload." >> reports/coverage-report.md
    
    echo "✅ Simple coverage report generated"
else
    echo "❌ cargo-llvm-cov not found. Installing would provide better coverage reports."
    
    # Fallback to basic test output
    echo "# Coverage Report (Test Output Only)" > reports/coverage-report.md
    echo "" >> reports/coverage-report.md
    echo "Generated on: $(date)" >> reports/coverage-report.md
    echo "" >> reports/coverage-report.md
    echo "## Test Results" >> reports/coverage-report.md
    echo "" >> reports/coverage-report.md
    echo "\`\`\`" >> reports/coverage-report.md
    cargo test --all-features --workspace 2>&1 | tail -50 >> reports/coverage-report.md
    echo "\`\`\`" >> reports/coverage-report.md
    echo "" >> reports/coverage-report.md
    echo "Note: Install cargo-llvm-cov for detailed coverage metrics." >> reports/coverage-report.md
    
    # Create empty lcov.info to prevent codecov upload failure
    touch lcov.info
    echo "⚠️  Warning: No actual coverage data generated (cargo-llvm-cov not available)"
fi

exit 0
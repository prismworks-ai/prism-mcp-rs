#!/bin/bash

# Run Benchmarks and Generate Report Script
# This script runs cargo benchmarks and creates a markdown report

set -e

echo "🚀 Running benchmarks..."

# Create reports directory if it doesn't exist
mkdir -p reports
mkdir -p target/criterion

# Initialize benchmark report
cat > reports/benchmark-report.md << 'EOF'
# Benchmark Report

## Summary

Generated on: $(date)

EOF

date >> reports/benchmark-report.md
echo "" >> reports/benchmark-report.md

# Check if we have benchmark features
if cargo build --benches --features bench 2>/dev/null; then
    echo "📊 Running benchmarks with criterion..."
    
    # Run benchmarks and capture output
    BENCH_OUTPUT=$(cargo bench --features bench -- --output-format bencher 2>&1 || true)
    
    # Add benchmark results to report
    echo "## Benchmark Results" >> reports/benchmark-report.md
    echo "" >> reports/benchmark-report.md
    
    # Parse criterion output if available
    if [ -d "target/criterion" ]; then
        echo "### Performance Metrics" >> reports/benchmark-report.md
        echo "" >> reports/benchmark-report.md
        echo "| Benchmark | Time | Throughput | Iterations |" >> reports/benchmark-report.md
        echo "|-----------|------|------------|------------|" >> reports/benchmark-report.md
        
        # Find and parse criterion JSON files
        for json_file in target/criterion/*/base/estimates.json; do
            if [ -f "$json_file" ]; then
                BENCH_NAME=$(basename $(dirname $(dirname "$json_file")))
                # Extract mean time (simplified parsing)
                if command -v jq &> /dev/null; then
                    MEAN_TIME=$(jq -r '.mean.point_estimate' "$json_file" 2>/dev/null || echo "N/A")
                    if [ "$MEAN_TIME" != "N/A" ] && [ "$MEAN_TIME" != "null" ]; then
                        # Convert nanoseconds to appropriate unit
                        if (( $(echo "$MEAN_TIME < 1000" | bc -l) )); then
                            TIME_STR="${MEAN_TIME} ns"
                        elif (( $(echo "$MEAN_TIME < 1000000" | bc -l) )); then
                            TIME_US=$(echo "scale=2; $MEAN_TIME / 1000" | bc)
                            TIME_STR="${TIME_US} µs"
                        else
                            TIME_MS=$(echo "scale=2; $MEAN_TIME / 1000000" | bc)
                            TIME_STR="${TIME_MS} ms"
                        fi
                        echo "| $BENCH_NAME | $TIME_STR | - | - |" >> reports/benchmark-report.md
                    fi
                fi
            fi
        done
        echo "" >> reports/benchmark-report.md
    fi
    
    # Add raw benchmark output
    echo "### Raw Output" >> reports/benchmark-report.md
    echo "" >> reports/benchmark-report.md
    echo "\`\`\`" >> reports/benchmark-report.md
    echo "$BENCH_OUTPUT" | tail -100 >> reports/benchmark-report.md
    echo "\`\`\`" >> reports/benchmark-report.md
    echo "" >> reports/benchmark-report.md
    
    # Add criterion HTML report link if generated
    if [ -f "target/criterion/report/index.html" ]; then
        echo "### Detailed Report" >> reports/benchmark-report.md
        echo "" >> reports/benchmark-report.md
        echo "Detailed HTML report available at: \`target/criterion/report/index.html\`" >> reports/benchmark-report.md
        echo "" >> reports/benchmark-report.md
    fi
else
    echo "⚠️  No benchmarks configured or bench feature not available"
    echo "## No Benchmarks Available" >> reports/benchmark-report.md
    echo "" >> reports/benchmark-report.md
    echo "Benchmarks are not configured for this project or the \`bench\` feature is not enabled." >> reports/benchmark-report.md
    echo "" >> reports/benchmark-report.md
fi

# Add system information
echo "## System Information" >> reports/benchmark-report.md
echo "" >> reports/benchmark-report.md
echo "- **OS:** $(uname -s)" >> reports/benchmark-report.md
echo "- **Arch:** $(uname -m)" >> reports/benchmark-report.md
echo "- **CPU:** $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "N/A") cores" >> reports/benchmark-report.md
echo "- **Rust:** $(rustc --version)" >> reports/benchmark-report.md
echo "- **Cargo:** $(cargo --version)" >> reports/benchmark-report.md
echo "" >> reports/benchmark-report.md

# Add notes
echo "## Notes" >> reports/benchmark-report.md
echo "" >> reports/benchmark-report.md
echo "- Benchmarks are run with \`cargo bench --features bench\`" >> reports/benchmark-report.md
echo "- Results may vary based on system load and configuration" >> reports/benchmark-report.md
echo "- For consistent results, run benchmarks on dedicated hardware" >> reports/benchmark-report.md
echo "" >> reports/benchmark-report.md

echo "✅ Benchmark report generated successfully!"
echo "📄 Report saved to: reports/benchmark-report.md"

exit 0
#!/bin/bash

# Test script to verify all examples compile and use correct API

set -e  # Exit on error

echo "Testing all examples in prism-mcp-rs SDK"
echo "========================================"

# List of all examples to test
examples=(
    "advanced_2025_features"
    "advanced_features_showcase"
    "bidirectional_communication_demo"
    "convenience_methods_demo"
    "performance_benchmarks"
    "production_error_handling_demo"
    "streaming_http_showcase"
    "streaming_http2_showcase"
    "transport_selection_guide"
    "client/advanced_http_client"
    "client/conservative_http_demo"
    "client/http_client"
    "client/websocket_client"
    "server/database_server"
    "server/enhanced_echo_server"
    "server/http2_server"
    "server/http_server"
    "server/websocket_server"
    "utilities/transport_benchmark"
)

total=${#examples[@]}
passed=0
failed=0
failed_examples=()

echo "Found $total examples to test"
echo ""

# Test each example
for example in "${examples[@]}"; do
    echo -n "Testing $example... "
    
    # Try to build the example
    if cargo build --example "$(basename $example)" 2>/dev/null 1>&2; then
        echo "✅ PASSED"
        ((passed++))
    else
        echo "❌ FAILED"
        ((failed++))
        failed_examples+=("$example")
    fi
done

echo ""
echo "========================================"
echo "Test Results:"
echo "  ✅ Passed: $passed/$total"
echo "  ❌ Failed: $failed/$total"

if [ $failed -gt 0 ]; then
    echo ""
    echo "Failed examples:"
    for example in "${failed_examples[@]}"; do
        echo "  - $example"
    done
    echo ""
    echo "To debug a specific example, run:"
    echo "  cargo build --example <example_name>"
    exit 1
else
    echo ""
    echo "🎉 All examples compile successfully!"
    echo "All examples are using the correct API."
fi

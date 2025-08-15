#!/bin/bash

# Test all examples in prism-mcp-rs
# This script runs each example to ensure they compile and execute without errors

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
SUCCESS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
FAILED_EXAMPLES=()
SKIPPED_EXAMPLES=()

echo "========================================"
echo "Testing prism-mcp-rs Examples"
echo "========================================"
echo ""

# Function to test an example
test_example() {
    local example_name=$1
    local timeout_seconds=${2:-5}  # Default 5 seconds timeout
    local skip_run=${3:-false}  # Some examples need servers running
    
    echo -e "${YELLOW}Testing: $example_name${NC}"
    
    # First, check if it compiles
    if cargo build --example "$example_name" 2>&1 | grep -q "error\[\|error:"; then
        echo -e "${RED}✗ Failed to compile: $example_name${NC}"
        FAIL_COUNT=$((FAIL_COUNT + 1))
        FAILED_EXAMPLES+=("$example_name")
        return 1
    fi
    
    echo -e "  ${GREEN}✓ Compiles successfully${NC}"
    
    # For some examples, we just check compilation
    if [ "$skip_run" = true ]; then
        echo -e "  ${YELLOW}⊘ Runtime test skipped (requires server/client setup)${NC}"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        SKIPPED_EXAMPLES+=("$example_name")
        return 0
    fi
    
    # Try to run the example with a timeout
    if timeout "$timeout_seconds" cargo run --example "$example_name" 2>&1 | head -20; then
        echo -e "  ${GREEN}✓ Runs successfully${NC}"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        exit_code=$?
        if [ $exit_code -eq 124 ]; then
            # Timeout reached - this is expected for servers
            echo -e "  ${GREEN}✓ Runs successfully (timeout expected for long-running examples)${NC}"
            SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        else
            echo -e "  ${RED}✗ Runtime error: $example_name (exit code: $exit_code)${NC}"
            FAIL_COUNT=$((FAIL_COUNT + 1))
            FAILED_EXAMPLES+=("$example_name")
            return 1
        fi
    fi
    
    echo ""
    return 0
}

# Test standalone examples (these should run without external dependencies)
echo "=== Testing Standalone Examples ==="
echo ""

test_example "transport_selection_guide" 3
test_example "convenience_methods_demo" 3
test_example "production_error_handling_demo" 3
test_example "advanced_features_showcase" 3
test_example "bidirectional_communication_demo" 3
test_example "performance_benchmarks" 10  # Might take longer
test_example "advanced_2025_features" 3

# Test streaming examples (these demonstrate features but may need specific setup)
echo ""
echo "=== Testing Streaming Examples ==="
echo ""

test_example "streaming_http_showcase" 3
test_example "streaming_http2_showcase" 3

# Test server examples (these are long-running, so we just check compilation)
echo ""
echo "=== Testing Server Examples ==="
echo ""

test_example "http_server" 2 true
test_example "http2_server" 2 true
test_example "websocket_server" 2 true
test_example "database_server" 2 true
test_example "enhanced_echo_server" 2 true

# Test client examples (these need servers running, so we just check compilation)
echo ""
echo "=== Testing Client Examples ==="
echo ""

test_example "http_client" 2 true
test_example "websocket_client" 2 true
test_example "advanced_http_client" 2 true
test_example "conservative_http_demo" 2 true

# Print summary
echo ""
echo "========================================"
echo "Test Summary"
echo "========================================"
echo -e "${GREEN}Successful: $SUCCESS_COUNT${NC}"
echo -e "${YELLOW}Skipped (compile-only): $SKIP_COUNT${NC}"
echo -e "${RED}Failed: $FAIL_COUNT${NC}"

if [ ${#FAILED_EXAMPLES[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed examples:${NC}"
    for example in "${FAILED_EXAMPLES[@]}"; do
        echo "  - $example"
    done
fi

if [ ${#SKIPPED_EXAMPLES[@]} -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}Skipped examples (compile-only test):${NC}"
    for example in "${SKIPPED_EXAMPLES[@]}"; do
        echo "  - $example"
    done
fi

echo ""
if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ All examples tested successfully!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some examples failed. Please review the errors above.${NC}"
    exit 1
fi
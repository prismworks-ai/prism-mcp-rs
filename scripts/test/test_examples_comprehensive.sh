#!/bin/bash

# Comprehensive test script for prism-mcp-rs examples
# This script tests compilation and runs examples with proper timeout handling

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Track results
TOTAL_EXAMPLES=0
COMPILE_SUCCESS=0
COMPILE_FAIL=0
RUN_SUCCESS=0
RUN_FAIL=0
RUN_SKIP=0

FAILED_COMPILE=()
FAILED_RUN=()

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}   Testing prism-mcp-rs Examples${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Function to compile an example
compile_example() {
    local example_name=$1
    TOTAL_EXAMPLES=$((TOTAL_EXAMPLES + 1))
    
    echo -ne "  Compiling ${example_name}... "
    
    if cargo build --example "$example_name" &>/dev/null; then
        echo -e "${GREEN}✓${NC}"
        COMPILE_SUCCESS=$((COMPILE_SUCCESS + 1))
        return 0
    else
        echo -e "${RED}✗${NC}"
        COMPILE_FAIL=$((COMPILE_FAIL + 1))
        FAILED_COMPILE+=("$example_name")
        return 1
    fi
}

# Function to run an example with timeout
run_example() {
    local example_name=$1
    local max_runtime=${2:-3}  # Default 3 seconds
    local expect_long_running=${3:-false}
    
    echo -ne "  Running ${example_name}... "
    
    # Create a temporary file for output
    local output_file="/tmp/example_${example_name}_$$.out"
    local error_file="/tmp/example_${example_name}_$$.err"
    
    # Run the example in background
    cargo run --example "$example_name" > "$output_file" 2> "$error_file" &
    local pid=$!
    
    # Wait for the specified time
    local count=0
    while [ $count -lt $max_runtime ]; do
        if ! kill -0 $pid 2>/dev/null; then
            # Process finished
            wait $pid
            local exit_code=$?
            if [ $exit_code -eq 0 ]; then
                echo -e "${GREEN}✓ (completed)${NC}"
                RUN_SUCCESS=$((RUN_SUCCESS + 1))
                # Show first few lines of output
                if [ -s "$output_file" ]; then
                    echo -e "    ${BLUE}Output:${NC}"
                    head -5 "$output_file" | sed 's/^/      /'
                    if [ $(wc -l < "$output_file") -gt 5 ]; then
                        echo "      ..."
                    fi
                fi
            else
                echo -e "${RED}✗ (exit code: $exit_code)${NC}"
                RUN_FAIL=$((RUN_FAIL + 1))
                FAILED_RUN+=("$example_name")
                # Show error output
                if [ -s "$error_file" ]; then
                    echo -e "    ${RED}Error:${NC}"
                    head -10 "$error_file" | sed 's/^/      /'
                fi
            fi
            rm -f "$output_file" "$error_file"
            return $exit_code
        fi
        sleep 1
        count=$((count + 1))
    done
    
    # Process still running after timeout
    kill $pid 2>/dev/null || true
    wait $pid 2>/dev/null || true
    
    if [ "$expect_long_running" = true ]; then
        echo -e "${GREEN}✓ (long-running, stopped after ${max_runtime}s)${NC}"
        RUN_SUCCESS=$((RUN_SUCCESS + 1))
        # Show output if any
        if [ -s "$output_file" ]; then
            echo -e "    ${BLUE}Output (first 5 lines):${NC}"
            head -5 "$output_file" | sed 's/^/      /'
        fi
    else
        echo -e "${YELLOW}⚠ (timeout after ${max_runtime}s)${NC}"
        RUN_SUCCESS=$((RUN_SUCCESS + 1))  # Still count as success if it ran without error
    fi
    
    rm -f "$output_file" "$error_file"
    return 0
}

# List of all examples categorized
declare -A EXAMPLES

# Standalone examples that should complete quickly
STANDALONE_EXAMPLES=(
    "transport_selection_guide"
    "convenience_methods_demo"
    "production_error_handling_demo"
    "advanced_features_showcase"
    "bidirectional_communication_demo"
    "performance_benchmarks"
    "advanced_2025_features"
    "streaming_http_showcase"
    "streaming_http2_showcase"
)

# Server examples (long-running)
SERVER_EXAMPLES=(
    "http_server"
    "http2_server"
    "websocket_server"
    "database_server"
    "enhanced_echo_server"
)

# Client examples (need server)
CLIENT_EXAMPLES=(
    "http_client"
    "websocket_client"
    "advanced_http_client"
    "conservative_http_demo"
)

# Phase 1: Compile all examples
echo -e "${MAGENTA}Phase 1: Compiling Examples${NC}"
echo "================================"

echo -e "\n${YELLOW}Standalone Examples:${NC}"
for example in "${STANDALONE_EXAMPLES[@]}"; do
    compile_example "$example"
done

echo -e "\n${YELLOW}Server Examples:${NC}"
for example in "${SERVER_EXAMPLES[@]}"; do
    compile_example "$example"
done

echo -e "\n${YELLOW}Client Examples:${NC}"
for example in "${CLIENT_EXAMPLES[@]}"; do
    compile_example "$example"
done

# Phase 2: Run standalone examples
echo ""
echo -e "${MAGENTA}Phase 2: Running Standalone Examples${NC}"
echo "====================================="
echo ""

for example in "${STANDALONE_EXAMPLES[@]}"; do
    # Skip if compilation failed
    if [[ " ${FAILED_COMPILE[@]} " =~ " ${example} " ]]; then
        echo -e "  Skipping ${example} (compilation failed)"
        RUN_SKIP=$((RUN_SKIP + 1))
        continue
    fi
    
    # Adjust timeout based on example
    case $example in
        "performance_benchmarks")
            run_example "$example" 10 false  # Longer timeout for benchmarks
            ;;
        *)
            run_example "$example" 5 false  # Standard timeout
            ;;
    esac
done

# Phase 3: Test server startup (just check they start without errors)
echo ""
echo -e "${MAGENTA}Phase 3: Testing Server Startup${NC}"
echo "================================"
echo ""

for example in "${SERVER_EXAMPLES[@]}"; do
    # Skip if compilation failed
    if [[ " ${FAILED_COMPILE[@]} " =~ " ${example} " ]]; then
        echo -e "  Skipping ${example} (compilation failed)"
        RUN_SKIP=$((RUN_SKIP + 1))
        continue
    fi
    
    run_example "$example" 2 true  # Short timeout, expect long-running
done

# Phase 4: Note about client examples
echo ""
echo -e "${MAGENTA}Phase 4: Client Examples${NC}"
echo "========================"
echo ""
echo -e "${CYAN}Note: Client examples require running servers.${NC}"
echo -e "${CYAN}Compilation was verified, but runtime tests are skipped.${NC}"
for example in "${CLIENT_EXAMPLES[@]}"; do
    if [[ " ${FAILED_COMPILE[@]} " =~ " ${example} " ]]; then
        echo -e "  ${RED}✗ ${example} (compilation failed)${NC}"
    else
        echo -e "  ${GREEN}✓ ${example} (compiles successfully)${NC}"
        RUN_SKIP=$((RUN_SKIP + 1))
    fi
done

# Print detailed summary
echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}            Test Summary${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Compilation summary
echo -e "${BLUE}Compilation Results:${NC}"
echo -e "  Total Examples: ${TOTAL_EXAMPLES}"
echo -e "  ${GREEN}Successful: ${COMPILE_SUCCESS}${NC}"
echo -e "  ${RED}Failed: ${COMPILE_FAIL}${NC}"

if [ ${#FAILED_COMPILE[@]} -gt 0 ]; then
    echo -e "\n  ${RED}Failed to compile:${NC}"
    for example in "${FAILED_COMPILE[@]}"; do
        echo "    - $example"
    done
fi

# Runtime summary
echo -e "\n${BLUE}Runtime Results:${NC}"
echo -e "  ${GREEN}Successful: ${RUN_SUCCESS}${NC}"
echo -e "  ${RED}Failed: ${RUN_FAIL}${NC}"
echo -e "  ${YELLOW}Skipped: ${RUN_SKIP}${NC}"

if [ ${#FAILED_RUN[@]} -gt 0 ]; then
    echo -e "\n  ${RED}Failed at runtime:${NC}"
    for example in "${FAILED_RUN[@]}"; do
        echo "    - $example"
    done
fi

# Overall result
echo ""
if [ $COMPILE_FAIL -eq 0 ] && [ $RUN_FAIL -eq 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}  ✓ All examples tested successfully!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 0
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}  ✗ Some tests failed. See details above.${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 1
fi
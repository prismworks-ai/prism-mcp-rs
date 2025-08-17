#!/bin/bash

# Local CI Runner Script
# This script runs the local CI workflow using Act

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}==================================${NC}"
echo -e "${BLUE}    Local CI Pipeline Runner      ${NC}"
echo -e "${BLUE}==================================${NC}"
echo ""

# Check if Act is installed
if ! command -v act &> /dev/null; then
    echo -e "${RED}❌ Act is not installed!${NC}"
    echo ""
    echo "Please install Act first:"
    echo "  macOS:    brew install act"
    echo "  Linux:    curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash"
    echo "  Windows:  choco install act-cli"
    echo ""
    echo "See: https://github.com/nektos/act for more information"
    exit 1
fi

# Parse command line arguments
MODE="full"
VERBOSE=""
DRY_RUN=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --quick|-q)
            MODE="quick"
            shift
            ;;
        --verbose|-v)
            VERBOSE="-v"
            shift
            ;;
        --dry-run|-n)
            DRY_RUN="-n"
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --quick, -q     Run quick validation only"
            echo "  --verbose, -v   Show verbose output"
            echo "  --dry-run, -n   Show what would run without executing"
            echo "  --help, -h      Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0              # Run full CI pipeline"
            echo "  $0 --quick      # Run quick validation only"
            echo "  $0 --verbose    # Run with verbose output"
            echo "  $0 --dry-run    # Show what would run"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Determine which event to trigger
if [ "$MODE" = "quick" ]; then
    EVENT="workflow_dispatch"
    echo -e "${YELLOW}🚀 Running QUICK validation...${NC}"
else
    EVENT="push"
    echo -e "${GREEN}🔧 Running FULL CI pipeline...${NC}"
fi

echo ""
echo -e "${BLUE}Configuration:${NC}"
echo "  Mode:    $MODE"
echo "  Event:   $EVENT"
if [ -n "$VERBOSE" ]; then
    echo "  Verbose: enabled"
fi
if [ -n "$DRY_RUN" ]; then
    echo "  Dry run: enabled"
fi
echo ""

# Run Act with the local CI workflow
echo -e "${BLUE}Starting Act...${NC}"
echo "--------------------------------------"

act $EVENT \
    --workflows .github/workflows/ci-local.yml \
    $VERBOSE \
    $DRY_RUN

EXIT_CODE=$?

echo "--------------------------------------"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ Local CI completed successfully!${NC}"
else
    echo -e "${RED}❌ Local CI failed with exit code $EXIT_CODE${NC}"
    exit $EXIT_CODE
fi

echo ""
echo -e "${BLUE}Tips:${NC}"
echo "  • Use 'make quick' for faster local validation"
echo "  • Use 'make check' for standard CI checks"
echo "  • Use 'make test' to run all tests"
echo "  • Use 'make commit-ready' for full validation before committing"
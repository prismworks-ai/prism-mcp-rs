#!/bin/bash

# Force bash execution for associative array support
if [ "$BASH_VERSION" = "" ]; then
    exec bash "$0" "$@"
fi

# 🏷️ improved Badge Manager for MCP Protocol SDK
# complete badge management with automated workflow triggers
# and smart badge status monitoring

set -e

# Colors for improved output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
REPO_OWNER="mcp-rust"
REPO_NAME="mcp-protocol-sdk"
REPO_FULL="${REPO_OWNER}/${REPO_NAME}"
MAIN_BRANCH="main"

# Badge URLs mapping
declare -A BADGE_URLS=(
    ["ci"]="https://github.com/${REPO_FULL}/actions/workflows/ci.yml/badge.svg"
    ["security"]="https://github.com/${REPO_FULL}/actions/workflows/security.yml/badge.svg"
    ["dependencies"]="https://github.com/${REPO_FULL}/actions/workflows/dependencies.yml/badge.svg"
    ["documentation"]="https://github.com/${REPO_FULL}/actions/workflows/docs.yml/badge.svg"
    ["benchmarks"]="https://github.com/${REPO_FULL}/actions/workflows/benchmarks.yml/badge.svg"
    ["release"]="https://github.com/${REPO_FULL}/actions/workflows/release.yml/badge.svg"
    ["codecov"]="https://codecov.io/gh/${REPO_FULL}/branch/main/graph/badge.svg"
    ["crates-version"]="https://img.shields.io/crates/v/mcp-protocol-sdk.svg"
    ["crates-downloads"]="https://img.shields.io/crates/d/mcp-protocol-sdk.svg"
    ["docs-rs"]="https://docs.rs/mcp-protocol-sdk/badge.svg"
    ["license"]="https://img.shields.io/badge/License-MIT-yellow.svg"
)

# Workflow file mapping
declare -A WORKFLOW_FILES=(
    ["ci"]="ci.yml"
    ["security"]="security.yml"
    ["dependencies"]="dependencies.yml"
    ["documentation"]="docs.yml"
    ["benchmarks"]="benchmarks.yml"
    ["codecov"]="codecov-refresh.yml"
)

# Function to print headers
print_header() {
    echo -e "${BLUE}=================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=================================${NC}"
}

# Function to print section headers
print_section() {
    echo -e "\n${CYAN}📊 $1${NC}"
    echo -e "${CYAN}$(printf '=%.0s' {1..50})${NC}"
}

# Function to check prerequisites
check_prerequisites() {
    print_section "Checking Prerequisites"
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ] || [ ! -d ".github/workflows" ]; then
        echo -e "${RED}[!] Error: Please run this script from the project root directory${NC}"
        exit 1
    fi
    
    # Check for GitHub CLI
    if command -v gh &> /dev/null; then
        echo -e "${GREEN}[x] GitHub CLI detected${NC}"
        GH_CLI_AVAILABLE=true
        
        # Check authentication
        if gh auth status &> /dev/null; then
            echo -e "${GREEN}[x] GitHub CLI authenticated${NC}"
        else
            echo -e "${YELLOW}Warning: GitHub CLI not authenticated - manual triggers unavailable${NC}"
            GH_CLI_AVAILABLE=false
        fi
    else
        echo -e "${YELLOW}Warning: GitHub CLI not found - install for workflow triggers${NC}"
        echo -e "${YELLOW}   Install: https://cli.github.com/${NC}"
        GH_CLI_AVAILABLE=false
    fi
    
    # Check for curl
    if command -v curl &> /dev/null; then
        echo -e "${GREEN}[x] curl available for badge status checks${NC}"
        CURL_AVAILABLE=true
    else
        echo -e "${YELLOW}Warning: curl not found - badge validation limited${NC}"
        CURL_AVAILABLE=false
    fi
}

# Function to validate badge accessibility
validate_badge_url() {
    local badge_name=$1
    local url=$2
    
    if [ "$CURL_AVAILABLE" = true ]; then
        if curl -s --head "$url" | head -n 1 | grep -q "200 OK"; then
            echo -e "${GREEN}[x] $badge_name${NC}"
            return 0
        else
            echo -e "${RED}[!] $badge_name - Not accessible${NC}"
            return 1
        fi
    else
        echo -e "${BLUE}🔗 $badge_name${NC}: $url"
        return 0
    fi
}

# Function to check badge status
check_badge_status() {
    print_section "Badge Status Validation"
    
    local failed_badges=()
    
    for badge_name in "${!BADGE_URLS[@]}"; do
        if ! validate_badge_url "$badge_name" "${BADGE_URLS[$badge_name]}"; then
            failed_badges+=("$badge_name")
        fi
    done
    
    if [ ${#failed_badges[@]} -gt 0 ]; then
        echo -e "\n${YELLOW}Warning: Failed badges: ${failed_badges[*]}${NC}"
        return 1
    else
        echo -e "\n${GREEN}[x] All badges are accessible${NC}"
        return 0
    fi
}

# Function to trigger workflow
trigger_workflow() {
    local workflow_type=$1
    local workflow_file=${WORKFLOW_FILES[$workflow_type]}
    
    if [ "$GH_CLI_AVAILABLE" = true ]; then
        echo -e "${BLUE}# Triggering $workflow_type workflow...${NC}"
        
        if gh workflow run "$workflow_file" --ref "$MAIN_BRANCH"; then
            echo -e "${GREEN}[x] $workflow_type workflow triggered successfully${NC}"
            return 0
        else
            echo -e "${RED}[!] Failed to trigger $workflow_type workflow${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}Warning: Cannot trigger $workflow_type - GitHub CLI not available${NC}"
        return 1
    fi
}

# Function to trigger badge update workflow
trigger_badge_update() {
    local badge_type=${1:-"all"}
    
    print_section "Triggering Badge Update Workflow"
    
    if [ "$GH_CLI_AVAILABLE" = true ]; then
        echo -e "${BLUE}# Triggering badge-update workflow for: $badge_type${NC}"
        
        if gh workflow run "badge-update.yml" --ref "$MAIN_BRANCH" -f badge_type="$badge_type"; then
            echo -e "${GREEN}[x] Badge update workflow triggered successfully${NC}"
            echo -e "${BLUE}🔗 Monitor progress: https://github.com/${REPO_FULL}/actions${NC}"
            return 0
        else
            echo -e "${RED}[!] Failed to trigger badge update workflow${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}Warning: Cannot trigger badge update - GitHub CLI not available${NC}"
        return 1
    fi
}

# Function to show workflow status
show_workflow_status() {
    print_section "Recent Workflow Status"
    
    if [ "$GH_CLI_AVAILABLE" = true ]; then
        echo -e "${BLUE}Last 5 workflow runs:${NC}"
        gh run list --limit 5 --json status,conclusion,name,createdAt --template '
{{range .}}• {{.name}}: {{.status}} ({{.conclusion}}) - {{timeago .createdAt}}
{{end}}'
    else
        echo -e "${YELLOW}Install GitHub CLI to view workflow status${NC}"
        echo -e "${BLUE}View manually: https://github.com/${REPO_FULL}/actions${NC}"
    fi
}

# Function to display help
show_help() {
    echo -e "${BLUE}Badge Manager - Usage${NC}"
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  check         - Validate all badge URLs and status"
    echo "  update [type] - Trigger badge updates (default: all)"
    echo "  status        - Show recent workflow status"
    echo "  list          - List all badges and their URLs"
    echo "  help          - Show this help message"
    echo ""
    echo "Badge types for update:"
    echo "  all          - Update all badges (default)"
    echo "  ci           - CI workflow badge"
    echo "  security     - Security audit badge"
    echo "  deps         - Dependencies badge"
    echo "  docs         - Documentation badge"
    echo "  benchmarks   - Benchmarks badge"
    echo "  codecov      - Code coverage badge"
    echo ""
    echo "Examples:"
    echo "  $0 check                    # Check all badge status"
    echo "  $0 update                   # Update all badges"
    echo "  $0 update codecov           # Update only codecov badge"
    echo "  $0 status                   # Show workflow status"
}

# Function to list all badges
list_badges() {
    print_section "Available Badges"
    
    echo -e "${BLUE}Badge Name${NC} | ${BLUE}URL${NC}"
    echo "$(printf '=%.0s' {1..80})"
    
    for badge_name in "${!BADGE_URLS[@]}"; do
        printf "%-15s | %s\n" "$badge_name" "${BADGE_URLS[$badge_name]}"
    done
}

# Function to show codecov specific help
show_codecov_help() {
    print_section "Codecov Badge Troubleshooting"
    
    echo -e "${YELLOW}Common codecov badge issues:${NC}"
    echo "1. CODECOV_TOKEN not configured in repository secrets"
    echo "2. Coverage report upload failures"
    echo "3. Badge caching issues"
    echo ""
    echo -e "${GREEN}Solutions:${NC}"
    echo "1. Add CODECOV_TOKEN to GitHub repository secrets"
    echo "2. Run: $0 update codecov"
    echo "3. Check codecov dashboard: https://codecov.io/gh/${REPO_FULL}"
    echo "4. Trigger codecov-refresh workflow manually"
}

# Function to generate summary report
generate_summary() {
    print_section "Badge Management Summary"
    
    echo -e "${GREEN}Repository:${NC} $REPO_FULL"
    echo -e "${GREEN}Total Badges:${NC} ${#BADGE_URLS[@]}"
    echo -e "${GREEN}Workflow Files:${NC} ${#WORKFLOW_FILES[@]}"
    echo ""
    
    if [ "$GH_CLI_AVAILABLE" = true ]; then
        echo -e "${GREEN}[x] GitHub CLI available - can trigger workflows${NC}"
    else
        echo -e "${YELLOW}Warning: GitHub CLI not available - limited functionality${NC}"
    fi
    
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. Run '$0 check' to validate badge status"
    echo "2. Run '$0 update' to refresh all badges"
    echo "3. Run '$0 status' to monitor workflow progress"
}

# Main script logic
main() {
    print_header "🏷️ Badge Manager for MCP Protocol SDK"
    
    case "${1:-help}" in
        "check")
            check_prerequisites
            check_badge_status
            ;;
        "update")
            check_prerequisites
            trigger_badge_update "$2"
            ;;
        "status")
            check_prerequisites
            show_workflow_status
            ;;
        "list")
            list_badges
            ;;
        "codecov")
            show_codecov_help
            ;;
        "summary")
            check_prerequisites
            generate_summary
            ;;
        "help")
            show_help
            ;;
        *)
            echo -e "${RED}Unknown command: $1${NC}"
            show_help
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"

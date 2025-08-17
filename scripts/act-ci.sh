#!/bin/bash
# Optimized Act CI runner with proper caching
# This script runs Act with all the caching optimizations enabled

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Running optimized Act CI with caching enabled${NC}"

# Enable Docker BuildKit for better caching
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# Clean up any stale containers from previous runs
echo -e "${YELLOW}🧹 Cleaning up stale Act containers...${NC}"
docker ps -a | grep act | awk '{print $1}' | xargs -r docker rm -f 2>/dev/null || true

# Create cargo cache directory if it doesn't exist
mkdir -p ~/.cargo/{registry,git}

# Optional: Pull the latest Act image to ensure we're up to date
echo -e "${YELLOW}📦 Ensuring Act image is up to date...${NC}"
docker pull catthehacker/ubuntu:act-latest

# Run Act with all optimizations
echo -e "${GREEN}🎬 Starting Act with workflow: ${1:-ci-local.yml}${NC}"

# Default to ci-local.yml if no workflow specified
WORKFLOW=${1:-ci-local.yml}

# Run Act with:
# - BuildKit enabled for better caching
# - Container reuse for faster subsequent runs
# - Cargo registry bind mount for dependency caching
# - Verbose output for debugging
DOCKER_BUILDKIT=1 act \
  -W ".github/workflows/${WORKFLOW}" \
  push \
  --reuse \
  --bind "$HOME/.cargo/registry:/github/home/.cargo/registry" \
  --bind "$HOME/.cargo/git:/github/home/.cargo/git" \
  --platform "ubuntu-latest=catthehacker/ubuntu:act-latest" \
  -v

echo -e "${GREEN}✅ Act CI run complete${NC}"
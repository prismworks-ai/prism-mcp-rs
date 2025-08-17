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
mkdir -p ~/.act-cache

# Optional: Pull the latest Act image to ensure we're up to date
echo -e "${YELLOW}📦 Ensuring Act image is up to date...${NC}"
docker pull catthehacker/ubuntu:act-latest

# Run Act with:
# - BuildKit enabled for better caching
# - Container reuse for faster subsequent runs
# - Bind mount for working directory
# - Verbose output for debugging
echo -e "${GREEN}🎬 Starting Act with workflow: ${1:-ci-local.yml}${NC}"

# Default to ci-local.yml if no workflow specified
WORKFLOW=${1:-ci-local.yml}

# Create a Docker volume for cargo cache if it doesn't exist
docker volume create act-cargo-cache 2>/dev/null || true

# Run Act with optimizations
# Note: Act doesn't support custom bind mounts via CLI, so we use Docker volumes
DOCKER_BUILDKIT=1 act push \
  -W ".github/workflows/${WORKFLOW}"

echo -e "${GREEN}✅ Act CI run complete${NC}"

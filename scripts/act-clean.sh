#!/bin/bash
# Clean up Act Docker resources
# Use this when you want to force a completely fresh run

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🧹 Cleaning up Act Docker resources...${NC}"

# Stop and remove all Act containers
echo "Removing Act containers..."
docker ps -a | grep act | awk '{print $1}' | xargs -r docker rm -f 2>/dev/null || echo "No Act containers to remove"

# Remove Act volumes
echo "Removing Act volumes..."
docker volume ls | grep act | awk '{print $2}' | xargs -r docker volume rm 2>/dev/null || echo "No Act volumes to remove"

# Optional: Remove Act images (commented out by default as they take time to download)
# echo "Removing Act images..."
# docker images | grep act | awk '{print $3}' | xargs -r docker rmi -f 2>/dev/null || echo "No Act images to remove"

# Clear Act cache
echo "Clearing Act cache..."
rm -rf ~/.cache/act/* 2>/dev/null || echo "No Act cache to clear"

# Prune Docker system (optional, more aggressive)
if [ "$1" = "--prune" ]; then
    echo -e "${YELLOW}Running Docker system prune...${NC}"
    docker system prune -f
fi

echo -e "${GREEN}✅ Act cleanup complete${NC}"
echo -e "${GREEN}You can now run 'scripts/act-ci.sh' for a fresh CI run${NC}"

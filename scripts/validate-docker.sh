#!/bin/bash
# Docker Environment Validation Script
# This script validates that the Docker setup is working correctly

set -e

echo -e "\033[32m Validating Docker Environment for Web x86 Emulator...\033[0m"

# Check if Docker is running
echo -e "\n\033[33m1. Checking Docker daemon...\033[0m"
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version)
    echo -e "\033[32m Docker is running: $DOCKER_VERSION\033[0m"
else
    echo -e "\033[31m Docker is not running or not installed\033[0m"
    echo -e "\033[31mPlease install Docker Desktop and ensure it's running\033[0m"
    exit 1
fi

# Check if Docker Compose is available
echo -e "\n\033[33m2. Checking Docker Compose...\033[0m"
if command -v docker-compose &> /dev/null; then
    COMPOSE_VERSION=$(docker-compose --version)
    echo -e "\033[32m Docker Compose is available: $COMPOSE_VERSION\033[0m"
else
    echo -e "\033[31m Docker Compose is not available\033[0m"
    echo -e "\033[31mPlease install Docker Compose\033[0m"
    exit 1
fi

# Check if required files exist
echo -e "\n\033[33m3. Checking required files...\033[0m"
REQUIRED_FILES=(
    "Dockerfile"
    "docker-compose.yml"
    ".dockerignore"
    "scripts/docker-dev.ps1"
    "scripts/docker-dev.sh"
)

ALL_FILES_EXIST=true
for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "\033[32m $file exists\033[0m"
    else
        echo -e "\033[31m $file is missing\033[0m"
        ALL_FILES_EXIST=false
    fi
done

if [ "$ALL_FILES_EXIST" = false ]; then
    echo -e "\n\033[31m Some required files are missing. Please check the setup.\033[0m"
    exit 1
fi

# Test Docker build
echo -e "\n\033[33m4. Testing Docker build...\033[0m"
echo -e "\033[36mBuilding Docker image (this may take a few minutes)...\033[0m"
if docker-compose build web-x86-dev; then
    echo -e "\033[32m Docker build successful\033[0m"
else
    echo -e "\033[31m Docker build failed\033[0m"
    exit 1
fi

# Test container startup
echo -e "\n\033[33m5. Testing container startup...\033[0m"
echo -e "\033[36mStarting container in background...\033[0m"
if docker-compose up -d web-x86-dev; then
    # Wait a moment for container to start
    sleep 10
    
    # Check if container is running
    CONTAINER_STATUS=$(docker-compose ps web-x86-dev)
    if echo "$CONTAINER_STATUS" | grep -q "Up"; then
        echo -e "\033[32m Container started successfully\033[0m"
    else
        echo -e "\033[31m Container failed to start\033[0m"
        echo -e "\033[31m Container status: $CONTAINER_STATUS\033[0m"
    fi
else
    echo -e "\033[31m Container startup failed\033[0m"
fi

# Clean up
echo -e "\n\033[33m6. Cleaning up test containers...\033[0m"
docker-compose down
echo -e "\033[ Cleanup complete\033[0m"

echo -e "\n\033[32m Docker environment validation complete!\033[0m"
echo -e "\n\033[36mNext steps:\033[0m"
echo -e "\033[37m1. Run: ./scripts/docker-dev.sh start\033[0m"
echo -e "\033[37m2. Open: http://localhost:5173\033[0m"
echo -e "\033[37m3. Start developing!\033[0m"

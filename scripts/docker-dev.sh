#!/bin/bash
# Docker Development Script for Web x86 Emulator
# This script provides easy commands for Docker development

set -e

COMMAND=${1:-help}

show_help() {
    echo -e "\033[32mWeb x86 Emulator - Docker Development Commands\033[0m"
    echo ""
    echo -e "\033[33mAvailable commands:\033[0m"
    echo "  start     - Start development environment with hot reload"
    echo "  stop      - Stop all containers"
    echo "  restart   - Restart development environment"
    echo "  build     - Build Docker images"
    echo "  logs      - Show logs from development container"
    echo "  shell     - Open shell in development container"
    echo "  clean     - Clean up containers and images"
    echo "  prod      - Start production build"
    echo "  tools     - Start development tools container"
    echo "  help      - Show this help message"
    echo ""
    echo -e "\033[36mExamples:\033[0m"
    echo "  ./scripts/docker-dev.sh start"
    echo "  ./scripts/docker-dev.sh logs"
    echo "  ./scripts/docker-dev.sh shell"
}

start_dev() {
    echo -e "\033[32m🚀 Starting Web x86 Emulator Development Environment...\033[0m"
    docker-compose up --build web-x86-dev
}

stop_dev() {
    echo -e "\033[33m🛑 Stopping development environment...\033[0m"
    docker-compose down
}

restart_dev() {
    echo -e "\033[33m🔄 Restarting development environment...\033[0m"
    docker-compose restart web-x86-dev
}

build_images() {
    echo -e "\033[34m🔨 Building Docker images...\033[0m"
    docker-compose build
}

show_logs() {
    echo -e "\033[36m📋 Showing logs from development container...\033[0m"
    docker-compose logs -f web-x86-dev
}

open_shell() {
    echo -e "\033[35m🐚 Opening shell in development container...\033[0m"
    docker-compose exec web-x86-dev /bin/bash
}

clean_env() {
    echo -e "\033[31m🧹 Cleaning up Docker environment...\033[0m"
    docker-compose down -v
    docker system prune -f
    echo -e "\033[32m✅ Cleanup complete!\033[0m"
}

start_prod() {
    echo -e "\033[32m🏭 Starting production build...\033[0m"
    docker-compose --profile production up --build web-x86-prod
}

start_tools() {
    echo -e "\033[34m🛠️ Starting development tools container...\033[0m"
    docker-compose --profile tools up -d dev-tools
    docker-compose exec dev-tools /bin/bash
}

# Main command dispatcher
case $COMMAND in
    start)
        start_dev
        ;;
    stop)
        stop_dev
        ;;
    restart)
        restart_dev
        ;;
    build)
        build_images
        ;;
    logs)
        show_logs
        ;;
    shell)
        open_shell
        ;;
    clean)
        clean_env
        ;;
    prod)
        start_prod
        ;;
    tools)
        start_tools
        ;;
    help)
        show_help
        ;;
    *)
        echo -e "\033[31m❌ Unknown command: $COMMAND\033[0m"
        show_help
        exit 1
        ;;
esac

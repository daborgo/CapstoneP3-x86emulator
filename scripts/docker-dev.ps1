# Docker Development Script for Web x86 Emulator
# This script provides easy commands for Docker development

param(
    [Parameter(Position=0)]
    [string]$Command = "help"
)

function Show-Help {
    Write-Host "Web x86 Emulator - Docker Development Commands" -ForegroundColor Green
    Write-Host ""
    Write-Host "Available commands:" -ForegroundColor Yellow
    Write-Host "  start     - Start development environment with hot reload"
    Write-Host "  stop      - Stop all containers"
    Write-Host "  restart   - Restart development environment"
    Write-Host "  build     - Build Docker images"
    Write-Host "  logs      - Show logs from development container"
    Write-Host "  shell     - Open shell in development container"
    Write-Host "  clean     - Clean up containers and images"
    Write-Host "  prod      - Start production build"
    Write-Host "  tools     - Start development tools container"
    Write-Host "  help      - Show this help message"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\scripts\docker-dev.ps1 start"
    Write-Host "  .\scripts\docker-dev.ps1 logs"
    Write-Host "  .\scripts\docker-dev.ps1 shell"
}

function Start-DevEnvironment {
    Write-Host "Starting Web x86 Emulator Development Environment..." -ForegroundColor Green
    docker-compose up --build web-x86-dev
}

function Stop-DevEnvironment {
    Write-Host "Stopping development environment..." -ForegroundColor Yellow
    docker-compose down
}

function Restart-DevEnvironment {
    Write-Host "Restarting development environment..." -ForegroundColor Yellow
    docker-compose restart web-x86-dev
}

function Build-Images {
    Write-Host "Building Docker images..." -ForegroundColor Blue
    docker-compose build
}

function Show-Logs {
    Write-Host "Showing logs from development container..." -ForegroundColor Cyan
    docker-compose logs -f web-x86-dev
}

function Open-Shell {
    Write-Host "Opening shell in development container..." -ForegroundColor Magenta
    docker-compose exec web-x86-dev /bin/bash
}

function Clean-Environment {
    Write-Host "Cleaning up Docker environment..." -ForegroundColor Red
    docker-compose down -v
    docker system prune -f
    Write-Host "✅ Cleanup complete!" -ForegroundColor Green
}

function Start-Production {
    Write-Host "Starting production build..." -ForegroundColor Green
    docker-compose --profile production up --build web-x86-prod
}

function Start-Tools {
    Write-Host "Starting development tools container..." -ForegroundColor Blue
    docker-compose --profile tools up -d dev-tools
    docker-compose exec dev-tools /bin/bash
}

# Main command dispatcher
switch ($Command.ToLower()) {
    "start" { Start-DevEnvironment }
    "stop" { Stop-DevEnvironment }
    "restart" { Restart-DevEnvironment }
    "build" { Build-Images }
    "logs" { Show-Logs }
    "shell" { Open-Shell }
    "clean" { Clean-Environment }
    "prod" { Start-Production }
    "tools" { Start-Tools }
    "help" { Show-Help }
    default { 
        Write-Host "❌ Unknown command: $Command" -ForegroundColor Red
        Show-Help 
    }
}

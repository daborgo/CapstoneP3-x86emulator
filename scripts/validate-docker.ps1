# Docker Environment Validation Script
# This script validates that the Docker setup is working correctly

Write-Host " Validating Docker Environment for Web x86 Emulator..." -ForegroundColor Green

# Check if Docker is running
Write-Host "`n1. Checking Docker daemon..." -ForegroundColor Yellow
try {
    $dockerVersion = docker --version
    Write-Host " Docker is running: $dockerVersion" -ForegroundColor Green
} catch {
    Write-Host " Docker is not running or not installed" -ForegroundColor Red
    Write-Host "Please install Docker Desktop and ensure it's running" -ForegroundColor Red
    exit 1
}

# Check if Docker Compose is available
Write-Host "`n2. Checking Docker Compose..." -ForegroundColor Yellow
try {
    $composeVersion = docker-compose --version
    Write-Host " Docker Compose is available: $composeVersion" -ForegroundColor Green
} catch {
    Write-Host " Docker Compose is not available" -ForegroundColor Red
    Write-Host "Please install Docker Compose" -ForegroundColor Red
    exit 1
}

# Check if required files exist
Write-Host "`n3. Checking required files..." -ForegroundColor Yellow
$requiredFiles = @(
    "Dockerfile",
    "docker-compose.yml",
    ".dockerignore",
    "scripts/docker-dev.ps1",
    "scripts/docker-dev.sh"
)

$allFilesExist = $true
foreach ($file in $requiredFiles) {
    if (Test-Path $file) {
        Write-Host " $file exists" -ForegroundColor Green
    } else {
        Write-Host " $file is missing" -ForegroundColor Red
        $allFilesExist = $false
    }
}

if (-not $allFilesExist) {
    Write-Host "`n Some required files are missing. Please check the setup." -ForegroundColor Red
    exit 1
}

# Test Docker build
Write-Host "`n4. Testing Docker build..." -ForegroundColor Yellow
try {
    Write-Host "Building Docker image (this may take a few minutes)..." -ForegroundColor Cyan
    docker-compose build web-x86-dev
    Write-Host " Docker build successful" -ForegroundColor Green
} catch {
    Write-Host " Docker build failed" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}

# Test container startup
Write-Host "`n5. Testing container startup..." -ForegroundColor Yellow
try {
    Write-Host "Starting container in background..." -ForegroundColor Cyan
    docker-compose up -d web-x86-dev
    
    # Wait a moment for container to start
    Start-Sleep -Seconds 10
    
    # Check if container is running
    $containerStatus = docker-compose ps web-x86-dev
    if ($containerStatus -match "Up") {
        Write-Host " Container started successfully" -ForegroundColor Green
    } else {
        Write-Host " Container failed to start" -ForegroundColor Red
        Write-Host "Container status: $containerStatus" -ForegroundColor Red
    }
} catch {
    Write-Host " Container startup failed" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Red
} finally {
    # Clean up
    Write-Host "`n6. Cleaning up test containers..." -ForegroundColor Yellow
    docker-compose down
    Write-Host " Cleanup complete" -ForegroundColor Green
}

Write-Host "`n Docker environment validation complete!" -ForegroundColor Green
Write-Host "`nNext steps:" -ForegroundColor Cyan
Write-Host "1. Run: .\scripts\docker-dev.ps1 start" -ForegroundColor White
Write-Host "2. Open: http://localhost:5173" -ForegroundColor White
Write-Host "3. Start developing!" -ForegroundColor White

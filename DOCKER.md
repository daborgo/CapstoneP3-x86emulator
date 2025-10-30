# Docker Development Environment

This document describes how to use Docker for the Web x86 Emulator project development.

## Overview

The Docker setup provides a reproducible development environment with:
- **Node.js 20** for frontend development
- **Rust 1.90** with wasm-pack for WASM compilation
- **Hot reload** for both frontend and Rust code
- **Volume mounting** for seamless development

## Quick Start

### Prerequisites
- Docker Desktop installed and running
- Git (for cloning the repository)

### Development Commands

#### Using PowerShell (Windows)
```powershell
# Start development environment
.\scripts\docker-dev.ps1 start

# View logs
.\scripts\docker-dev.ps1 logs

# Open shell in container
.\scripts\docker-dev.ps1 shell

# Stop environment
.\scripts\docker-dev.ps1 stop
```

#### Using Bash (Linux/macOS)
```bash
# Make script executable (first time only)
chmod +x scripts/docker-dev.sh

# Start development environment
./scripts/docker-dev.sh start

# View logs
./scripts/docker-dev.sh logs

# Open shell in container
./scripts/docker-dev.sh shell

# Stop environment
./scripts/docker-dev.sh stop
```

#### Using Docker Compose Directly
```bash
# Start development environment
docker-compose up --build web-x86-dev

# Start in background
docker-compose up -d web-x86-dev

# Stop environment
docker-compose down

# View logs
docker-compose logs -f web-x86-dev
```

## Available Services

### Development Services

| Service | Port | Description |
|---------|------|-------------|
| `web-x86-dev` | 5173 | Main development server with hot reload |
| `web-x86-prod` | 3000 | Production build (profile: production) |
| `dev-tools` | - | Development tools container (profile: tools) |

### Optional Services (Profiles)

| Service | Port | Profile | Description |
|---------|------|---------|-------------|
| `postgres` | 5432 | database | PostgreSQL database |
| `redis` | 6379 | cache | Redis cache |

## Development Workflow

### 1. Start Development Environment
```bash
# Using helper script
./scripts/docker-dev.sh start

# Or using docker-compose directly
docker-compose up --build web-x86-dev
```

### 2. Access the Application
- Frontend: http://localhost:5173
- The application will automatically reload when you make changes

### 3. Development Features
- **Hot Reload**: Changes to React components reload automatically
- **WASM Rebuild**: Changes to Rust code trigger automatic WASM rebuild
- **Volume Mounting**: Your local files are mounted into the container
- **Debug Logging**: Rust debug logs are enabled

### 4. Working with Rust Code
```bash
# Open shell in container
./scripts/docker-dev.sh shell

# Or use docker-compose
docker-compose exec web-x86-dev /bin/bash

# Inside container, you can run:
cd core
cargo check
cargo test
wasm-pack build --target web --out-dir ../frontend/src/wasm/pkg
```

### 5. Working with Frontend Code
```bash
# Open shell in container
./scripts/docker-dev.sh shell

# Inside container:
cd frontend
npm run dev
npm run build
npm run lint
```

## File Structure

```
project/
├── Dockerfile              # Multi-stage Docker build
├── docker-compose.yml      # Development services
├── .dockerignore           # Docker ignore patterns
├── scripts/
│   ├── docker-dev.sh       # Bash development script
│   └── docker-dev.ps1      # PowerShell development script
├── frontend/               # React frontend (mounted)
├── core/                   # Rust WASM core (mounted)
└── DOCKER.md              # This file
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NODE_ENV` | development | Node.js environment |
| `RUST_LOG` | debug | Rust logging level |
| `CARGO_TARGET_DIR` | /app/core/target | Rust build directory |

## Troubleshooting

### Container Won't Start
```bash
# Check if ports are available
netstat -tulpn | grep :5173

# Clean up and restart
./scripts/docker-dev.sh clean
./scripts/docker-dev.sh start
```

### WASM Build Issues
```bash
# Open shell and rebuild manually
./scripts/docker-dev.sh shell
cd core
wasm-pack build --target web --out-dir ../frontend/src/wasm/pkg
```

### Permission Issues (Linux/macOS)
```bash
# Fix file permissions
sudo chown -R $USER:$USER .
chmod +x scripts/docker-dev.sh
```

### Windows PowerShell Execution Policy
```powershell
# Allow script execution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## Production Build

### Build Production Image
```bash
docker-compose --profile production up --build web-x86-prod
```

### Run Production Container
```bash
docker run -p 3000:3000 web-x86-emulator-prod
```

## Development Tools

### Access Development Tools Container
```bash
# Start tools container
./scripts/docker-dev.sh tools

# Or with docker-compose
docker-compose --profile tools up -d dev-tools
docker-compose exec dev-tools /bin/bash
```

### Available Tools in Container
- `cargo` - Rust package manager
- `wasm-pack` - WASM packaging tool
- `npm` - Node.js package manager
- `git` - Version control
- `curl` - HTTP client
- `build-essential` - Compilation tools

## Performance Tips

1. **Exclude node_modules**: The `node_modules` directory is excluded from volume mounting for better performance
2. **Exclude target**: The Rust `target` directory is excluded from volume mounting
3. **Use .dockerignore**: Large files and directories are excluded from the build context
4. **Multi-stage builds**: Only necessary files are copied to the final image

## Security Considerations

- The development environment runs with user privileges
- No root access required for development
- Volume mounts are read-write for development convenience
- Production builds use minimal base images

## Contributing

When adding new dependencies or changing the build process:

1. Update the `Dockerfile` if new system packages are needed
2. Update `docker-compose.yml` if new services or environment variables are needed
3. Update this documentation if new commands or workflows are added
4. Test the changes with `./scripts/docker-dev.sh clean && ./scripts/docker-dev.sh start`

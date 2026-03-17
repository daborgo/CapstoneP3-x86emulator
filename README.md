# Web x86 (32-bit Subset) Emulator

Educational 32-bit x86 subset emulator for the web: registers, stack, calling conventions, debugger, visualizations, and sandboxed virtual I/O.

## Quick Start

### Predeploy: Build WASM Module

If you made any changes to WASM module files or instruction files, first rebuild the WASM module:

```bash
# Move into core folder
cd core

# Build WASM module
wasm-pack build --target web --out-dir ../frontend/src/wasm/pkg --dev --out-name web_x86_core
```

### Option 1: Docker (Recommended)
```bash
# Validate Docker setup
./scripts/validate-docker.sh

# Start development environment
./scripts/docker-dev.sh start

# Access the application at http://localhost:5173
```

### Option 2: Local Development
```bash
# requires: Node 18+, Rust + wasm-pack, pnpm (or npm/yarn)
pnpm i
# build all workspace packages
pnpm -w build
# start the frontend dev server (pnpm workspace)
pnpm -w --filter frontend dev

# If you don't use pnpm, from the `frontend` folder you can use npm/yarn:
cd frontend
npm install
npm run dev
```

## Development Environment

This project supports both Docker and local development:

- **Docker**: Reproducible environment with Node.js 20 + Rust 1.90 + wasm-pack
- **Local**: Requires Node.js 18+, Rust, and wasm-pack installed locally

See [DOCKER.md](DOCKER.md) for detailed Docker setup instructions.
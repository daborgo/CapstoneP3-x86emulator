# Multi-stage Dockerfile for Web x86 Emulator Development Environment
# Supports React frontend + Rust toolchain

# Stage 1: Base image with Rust toolchain
FROM rust:1.90-slim as rust-base

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*


# Install Rust and set PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Stage 2: Node.js development environment
FROM node:20-slim as node-base

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    python3 \
    make \
    && rm -rf /var/lib/apt/lists/*

# Stage 3: Combined development environment
FROM node-base as development

# Copy Rust toolchain from rust-base
# COPY --from=rust-base /root/.rustup /root/.rustup
# COPY --from=rust-base /root/.cargo /root/.cargo


# Install Rust and set up environment variables
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
ENV RUSTUP_HOME="/root/.rustup"
ENV CARGO_HOME="/root/.cargo"

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install pnpm for better workspace management
RUN npm install -g pnpm@latest

# Create app directory
WORKDIR /app


# Copy package files
COPY package*.json ./
COPY frontend/package*.json ./frontend/
COPY core ./core

# Install root dependencies
RUN npm install

# Install frontend dependencies
WORKDIR /app/frontend
RUN npm install

# Install Rust dependencies
WORKDIR /app/core
RUN cargo fetch

# Go back to app root
WORKDIR /app

# Copy source code
COPY . .

# Build WASM module
RUN cd core && wasm-pack build --target web --out-dir ../frontend/src/wasm/pkg

# Expose ports
EXPOSE 5173 3000 8080

# Create development script
RUN echo '#!/bin/bash\n\
set -e\n\
echo "Starting Web x86 Emulator Development Environment"\n\
echo "Building WASM module..."\n\
cd core && wasm-pack build --target web --out-dir ../frontend/src/wasm/pkg\n\
echo "Starting frontend development server..."\n\
cd ../frontend && npm run dev\n\
' > /app/dev.sh && chmod +x /app/dev.sh

# Default command
CMD ["/app/dev.sh"]

# Production stage
FROM node:20-alpine as production

# Install system dependencies
RUN apk add --no-cache \
    curl \
    git \
    build-essential \
    python3 \
    make


# Install Rust and set PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./
COPY frontend/package*.json ./frontend/
COPY core/Cargo.toml ./core/

# Install dependencies
RUN npm install
WORKDIR /app/frontend
RUN npm install

# Copy source code
COPY . .

# Build WASM module
WORKDIR /app/core
RUN wasm-pack build --target web --release --out-dir ../frontend/src/wasm/pkg

# Build frontend
WORKDIR /app/frontend
RUN npm run build

# Serve static files
RUN npm install -g serve
EXPOSE 3000
CMD ["serve", "-s", "dist", "-l", "3000"]

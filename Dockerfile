# ==============================================================================
# Atena Studio - Multi-Stage Container Image
# https://github.com/atena-ai/atena
# ==============================================================================

# ------------------------------------------------------------------------------
# Stage 1: Frontend Builder (Nuxt SPA Static Generation)
# ------------------------------------------------------------------------------
FROM node:22-bookworm-slim AS frontend-builder

WORKDIR /app

# Enable pnpm via Corepack
RUN corepack enable && corepack prepare pnpm@latest --activate

# Copy package manifests for layer caching
COPY frontend/package.json frontend/pnpm-lock.yaml frontend/pnpm-workspace.yaml* ./frontend/

WORKDIR /app/frontend

# Install frontend dependencies without running postinstall prematurely
RUN pnpm install --frozen-lockfile --ignore-scripts

# Copy frontend source files
COPY frontend/ ./

# Prepare Nuxt metadata with full source context and generate pre-rendered SPA static files
RUN pnpm run postinstall && pnpm run generate

# ------------------------------------------------------------------------------
# Stage 2: Backend Builder (Rust + Tauri v2 Linux Compilation)
# ------------------------------------------------------------------------------
FROM rust:1-bookworm AS backend-builder

WORKDIR /app

# Install Tauri v2 Linux build prerequisites and packaging tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    curl \
    ca-certificates \
    tar \
    gzip \
    && rm -rf /var/lib/apt/lists/*

# Copy static frontend distribution required by tauri-build
COPY --from=frontend-builder /app/frontend/.output/public ./frontend/.output/public

# Copy src-tauri sources
COPY src-tauri/ ./src-tauri/

# Copy scripts and provision Linux sidecar binaries
COPY scripts/ ./scripts/
RUN bash ./scripts/setup-sidecars.sh

# Compile production release binary for the server/desktop app
RUN cargo build --release --manifest-path src-tauri/Cargo.toml --bin app

# ------------------------------------------------------------------------------
# Stage 3: Runtime Container (Ubuntu 24.04 matching llama.cpp glibc/libstdc++)
# ------------------------------------------------------------------------------
FROM ubuntu:24.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive

# Install minimal runtime shared libraries
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    libgtk-3-0t64 \
    libwebkit2gtk-4.1-0 \
    libayatana-appindicator3-1 \
    librsvg2-2 \
    libgomp1 \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Create dedicated non-root application user and group (removing default Ubuntu 24.04 'ubuntu' user if present)
RUN if id -u ubuntu >/dev/null 2>&1; then userdel -r ubuntu 2>/dev/null || true; fi && \
    groupadd -g 1000 atena && \
    useradd -u 1000 -g atena -m -s /bin/bash atena

WORKDIR /app

# Copy compiled Atena server binary
COPY --from=backend-builder /app/src-tauri/target/release/app /usr/local/bin/atena

# Copy sidecar binaries and companion libraries
COPY --from=backend-builder /app/src-tauri/binaries/ /usr/local/bin/

# Ensure generic symlinks and register dynamic libraries
RUN for f in /usr/local/bin/*-linux-*; do \
      [ -f "$f" ] || continue; \
      base=$(basename "$f" | sed -E 's/-(x86_64|aarch64)-unknown-linux-gnu//'); \
      [ ! -f "/usr/local/bin/$base" ] && ln -sf "$f" "/usr/local/bin/$base"; \
    done && \
    chmod +x /usr/local/bin/* && \
    ldconfig /usr/local/bin || true

# Copy static frontend SPA distribution
COPY --from=frontend-builder /app/frontend/.output/public /app/public

# Initialize data directory and set permissions
RUN mkdir -p /home/atena/.atena/models \
             /home/atena/.atena/brain \
             /home/atena/.atena/skills \
             /home/atena/.atena/plugins && \
    chown -R atena:atena /app /home/atena

USER atena

# Default environment configuration
ENV ATENA_SERVER=true \
    ATENA_HOST=0.0.0.0 \
    ATENA_PORT=7860 \
    ATENA_STATIC_DIR=/app/public \
    HOME=/home/atena \
    LD_LIBRARY_PATH=/usr/local/bin:/usr/local/lib:${LD_LIBRARY_PATH}

# Data persistence volume
VOLUME ["/home/atena/.atena"]

# Expose web server and websocket port
EXPOSE 7860

# Healthcheck against Axum /health endpoint
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD curl -f http://localhost:7860/health || exit 1

# Entrypoint running headless web server
ENTRYPOINT ["/usr/local/bin/atena"]
CMD ["--server", "--host", "0.0.0.0", "--port", "7860"]

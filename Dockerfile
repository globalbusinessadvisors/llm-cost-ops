# Multi-stage Dockerfile for LLM Cost Ops
# Production-ready, optimized for size and security
# Version: 1.0.0

# ============================================================================
# Stage 1: Build Environment
# ============================================================================
FROM rust:1.91-slim-bullseye AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    libsqlite3-dev \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY migrations_postgres ./migrations_postgres

# Build the actual application
RUN cargo build --release --locked

# Verify the binary exists and is executable
RUN test -f /app/target/release/llm-cost-ops && \
    chmod +x /app/target/release/llm-cost-ops

# ============================================================================
# Stage 2: Runtime Environment
# ============================================================================
FROM debian:bullseye-slim AS runtime

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libsqlite3-0 \
    libssl1.1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r llmcostops && \
    useradd -r -g llmcostops -s /bin/bash -m llmcostops

# Create application directories
RUN mkdir -p /app/data /app/logs /app/config /app/migrations /app/migrations_postgres && \
    chown -R llmcostops:llmcostops /app

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=llmcostops:llmcostops /app/target/release/llm-cost-ops /app/llm-cost-ops

# Copy migrations
COPY --chown=llmcostops:llmcostops migrations /app/migrations
COPY --chown=llmcostops:llmcostops migrations_postgres /app/migrations_postgres

# Copy configuration template
COPY --chown=llmcostops:llmcostops config.toml.template /app/config/config.toml.template

# Copy health check script
COPY --chown=llmcostops:llmcostops docker/healthcheck.sh /app/healthcheck.sh
RUN chmod +x /app/healthcheck.sh

# Switch to non-root user
USER llmcostops

# Expose ports
EXPOSE 8080 9090

# Environment variables (can be overridden)
ENV RUST_LOG=info \
    DATABASE_URL=sqlite:///app/data/llm-cost-ops.db \
    CONFIG_PATH=/app/config/config.toml \
    LOG_LEVEL=info \
    PORT=8080 \
    METRICS_PORT=9090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/healthcheck.sh"]

# Volume for persistent data
VOLUME ["/app/data", "/app/logs", "/app/config"]

# Start the application
CMD ["/app/llm-cost-ops"]

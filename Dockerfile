# Multi-stage Dockerfile for LLM Cost Ops

# Stage 1: Build
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release && \
    rm -rf src/

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release --bin cost-ops

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser -u 1000 appuser

# Copy binary from builder
COPY --from=builder /app/target/release/cost-ops /app/cost-ops

# Create necessary directories
RUN mkdir -p /app/cache /app/logs /tmp && \
    chown -R appuser:appuser /app /tmp

# Switch to non-root user
USER appuser

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/cost-ops", "health"] || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the application
ENTRYPOINT ["/app/cost-ops"]
CMD ["serve"]

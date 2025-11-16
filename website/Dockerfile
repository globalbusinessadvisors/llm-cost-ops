# Multi-stage Dockerfile for Docusaurus Documentation
# Stage 1: Build the documentation
FROM node:20-alpine AS builder

# Set working directory
WORKDIR /app

# Install dependencies first (better caching)
COPY package*.json ./
RUN npm ci --only=production && npm cache clean --force

# Copy source files
COPY . .

# Build the documentation site
RUN npm run build

# Stage 2: Production image with Nginx
FROM nginx:1.27-alpine AS production

# Install security updates
RUN apk update && \
    apk upgrade && \
    apk add --no-cache \
    curl \
    ca-certificates && \
    rm -rf /var/cache/apk/*

# Copy built documentation from builder stage
COPY --from=builder /app/build /usr/share/nginx/html

# Copy custom Nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf
COPY nginx-default.conf /etc/nginx/conf.d/default.conf

# Create non-root user for Nginx
RUN addgroup -g 101 -S nginx-runner && \
    adduser -S -D -H -u 101 -h /var/cache/nginx -s /sbin/nologin -G nginx-runner -g nginx-runner nginx-runner && \
    chown -R nginx-runner:nginx-runner /usr/share/nginx/html && \
    chown -R nginx-runner:nginx-runner /var/cache/nginx && \
    chown -R nginx-runner:nginx-runner /var/log/nginx && \
    touch /var/run/nginx.pid && \
    chown -R nginx-runner:nginx-runner /var/run/nginx.pid

# Switch to non-root user
USER nginx-runner

# Expose port 8080 (non-privileged port)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ || exit 1

# Start Nginx
CMD ["nginx", "-g", "daemon off;"]

# Labels for metadata
LABEL maintainer="LLM Cost Ops <support@llm-cost-ops.dev>"
LABEL description="Documentation for LLM Cost Ops - Enterprise-grade cost tracking for LLM applications"
LABEL org.opencontainers.image.source="https://github.com/llm-cost-ops/llm-cost-ops"
LABEL org.opencontainers.image.documentation="https://docs.llm-cost-ops.dev"
LABEL org.opencontainers.image.vendor="LLM Cost Ops"
LABEL org.opencontainers.image.licenses="Apache-2.0"

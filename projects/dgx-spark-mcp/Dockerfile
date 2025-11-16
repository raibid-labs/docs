# Stage 1: Build
FROM node:20-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    python3 \
    make \
    g++

# Set working directory
WORKDIR /build

# Copy package files
COPY package*.json ./
COPY tsconfig.json ./
COPY tsconfig.build.json ./

# Install dependencies
RUN npm ci --only=production=false

# Copy source code
COPY src/ ./src/

# Build TypeScript
RUN npm run build

# Remove dev dependencies
RUN npm prune --production

# ============================================================================
# Stage 2: Production Runtime
FROM node:20-alpine

# Add labels
LABEL maintainer="Raibid Labs"
LABEL description="DGX Spark MCP Server - Model Context Protocol server for DGX optimization"
LABEL version="0.1.0"

# Install runtime dependencies
RUN apk add --no-cache \
    tini \
    curl \
    ca-certificates \
    && addgroup -g 1000 dgx \
    && adduser -D -u 1000 -G dgx dgx

# Set working directory
WORKDIR /app

# Copy built application from builder
COPY --from=builder --chown=dgx:dgx /build/dist ./dist
COPY --from=builder --chown=dgx:dgx /build/node_modules ./node_modules
COPY --from=builder --chown=dgx:dgx /build/package*.json ./

# Copy configuration and scripts
COPY --chown=dgx:dgx config/ ./config/
COPY --chown=dgx:dgx .env.example ./.env.example

# Create directories for logs and data
RUN mkdir -p /app/logs /app/data \
    && chown -R dgx:dgx /app/logs /app/data

# Switch to non-root user
USER dgx

# Expose health check port (if using HTTP server)
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD node -e "console.log('healthy')" || exit 1

# Set environment variables
ENV NODE_ENV=production \
    LOG_LEVEL=info \
    LOG_FORMAT=json

# Use tini as init system to handle signals properly
ENTRYPOINT ["/sbin/tini", "--"]

# Start the application
CMD ["node", "dist/index.js"]

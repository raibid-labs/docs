# Multi-stage Dockerfile for raibid-server
# Optimized for fast builds with cargo-chef and minimal runtime image
# Target: ARM64 (NVIDIA DGX Spark) and x86_64
# Base: Debian Bookworm (stable)

# Stage 1: Prepare cargo-chef
FROM docker.io/library/rust:1.82-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# Stage 2: Generate recipe (dependency manifest)
FROM chef AS planner
# Copy the entire workspace to generate a complete recipe
COPY . .
# Generate recipe.json with all dependencies
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (cached layer)
FROM chef AS dependencies
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies only - this layer is cached until dependencies change
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Build the application
FROM chef AS builder
# Copy cached dependencies from previous stage
COPY --from=dependencies /app/target target
COPY --from=dependencies /usr/local/cargo /usr/local/cargo

# Install additional build dependencies if needed
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the full source tree
COPY . .

# Build the server binary
# Use workspace-aware build to handle crate dependencies
RUN cargo build --release --bin raibid-server

# Verify binary was built
RUN ls -lah /app/target/release/raibid-server && \
    file /app/target/release/raibid-server

# Stage 5: Runtime image (minimal)
FROM docker.io/library/debian:bookworm-slim AS runtime

# Install runtime dependencies only
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r server --gid=1000 \
    && useradd -r -g server --uid=1000 --home-dir=/home/server --create-home server \
    && mkdir -p /home/server/.cache \
    && chown -R server:server /home/server

# Copy server binary from builder
COPY --from=builder /app/target/release/raibid-server /usr/local/bin/raibid-server
RUN chmod +x /usr/local/bin/raibid-server

# Copy healthcheck script
COPY --chown=server:server healthcheck.sh /usr/local/bin/healthcheck.sh
RUN chmod +x /usr/local/bin/healthcheck.sh

# Set working directory
WORKDIR /home/server
RUN chown server:server /home/server

# Switch to non-root user
USER server

# Environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

# Expose HTTP and metrics ports
EXPOSE 8080 8081

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/healthcheck.sh"]

# OCI image labels
LABEL org.opencontainers.image.title="raibid-server"
LABEL org.opencontainers.image.description="raibid-ci API server for job dispatch and orchestration"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.vendor="Raibid Labs"
LABEL org.opencontainers.image.source="https://github.com/raibid-labs/raibid-ci"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
LABEL raibid.component="server"
LABEL raibid.arch="multi"

# Run the server
CMD ["/usr/local/bin/raibid-server"]

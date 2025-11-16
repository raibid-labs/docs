# Multi-stage Dockerfile for raibid-ci agent
# Target: ARM64 (NVIDIA DGX Spark)
# Base: Debian Bookworm (stable)
# Purpose: Ephemeral CI agent for Rust builds

# Stage 1: Base system dependencies
FROM docker.io/library/rust:1.82-bookworm AS base

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Version control
    git \
    git-lfs \
    # SSH and certificates
    openssh-client \
    ca-certificates \
    # Build essentials
    build-essential \
    pkg-config \
    # SSL/TLS libraries
    libssl-dev \
    # Compression
    zlib1g-dev \
    # Docker CLI for image builds
    curl \
    gnupg \
    lsb-release \
    # Process utilities
    procps \
    # Cleanup tools
    && rm -rf /var/lib/apt/lists/*

# Install Docker CLI (official repository)
RUN install -m 0755 -d /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg \
    && chmod a+r /etc/apt/keyrings/docker.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get update \
    && apt-get install -y --no-install-recommends docker-ce-cli \
    && rm -rf /var/lib/apt/lists/*

# Stage 2: Prepare cargo-chef
FROM base AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# Stage 3: Generate recipe (dependency manifest)
FROM chef AS planner
# Copy the entire workspace to generate a complete recipe
COPY . .
# Generate recipe.json with all dependencies
RUN cargo chef prepare --recipe-path recipe.json

# Stage 4: Build dependencies (cached layer)
FROM chef AS dependencies
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies only - this layer is cached until dependencies change
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 5: Build agent binary
FROM chef AS agent-builder
# Copy cached dependencies from previous stage
COPY --from=dependencies /app/target target
COPY --from=dependencies /usr/local/cargo /usr/local/cargo

# Copy the full source tree
COPY . .

# Build the agent binary
# Use workspace-aware build to handle crate dependencies
RUN cargo build --release --bin raibid-agent

# Verify binary was built
RUN ls -lah /app/target/release/raibid-agent && \
    file /app/target/release/raibid-agent

# Stage 6: Build CI tools
FROM base AS tools-builder

# Install Rust tools for CI workflows
RUN cargo install --locked \
    # Advanced test runner with better reporting
    cargo-nextest@0.9.72 \
    # Security auditing
    cargo-audit@0.20.0 \
    # License and dependency checking
    cargo-deny@0.14.24 \
    && rm -rf /usr/local/cargo/registry

# Stage 7: Final runtime image
FROM base AS runtime

# Copy installed cargo binaries from tools-builder
COPY --from=tools-builder /usr/local/cargo/bin/cargo-nextest /usr/local/cargo/bin/
COPY --from=tools-builder /usr/local/cargo/bin/cargo-audit /usr/local/cargo/bin/
COPY --from=tools-builder /usr/local/cargo/bin/cargo-deny /usr/local/cargo/bin/

# Copy agent binary from agent-builder
COPY --from=agent-builder /app/target/release/raibid-agent /usr/local/bin/raibid-agent
RUN chmod +x /usr/local/bin/raibid-agent

# Create non-root user for agent execution
RUN groupadd -r agent --gid=1000 \
    && useradd -r -g agent --uid=1000 --home-dir=/home/agent --create-home agent \
    && mkdir -p /home/agent/.cargo /home/agent/.cache \
    && chown -R agent:agent /home/agent

# Configure Cargo for caching optimization
RUN mkdir -p /home/agent/.cargo && \
    echo '[net]' > /home/agent/.cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> /home/agent/.cargo/config.toml && \
    echo '[build]' >> /home/agent/.cargo/config.toml && \
    echo 'incremental = true' >> /home/agent/.cargo/config.toml && \
    chown -R agent:agent /home/agent/.cargo

# Set up workspace directory
WORKDIR /workspace
RUN chown agent:agent /workspace

# Copy health check script
COPY --chown=agent:agent healthcheck.sh /usr/local/bin/healthcheck.sh
RUN chmod +x /usr/local/bin/healthcheck.sh

# Switch to non-root user
USER agent

# Environment variables for runtime configuration
ENV RUST_BACKTRACE=1
ENV CARGO_HOME=/home/agent/.cargo
ENV CARGO_TARGET_DIR=/workspace/target
ENV CARGO_INCREMENTAL=1
ENV RUST_LOG=info

# Health check configuration
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/healthcheck.sh"]

# Metadata labels
LABEL org.opencontainers.image.title="raibid-ci-agent"
LABEL org.opencontainers.image.description="Ephemeral CI agent for Rust builds on NVIDIA DGX Spark"
LABEL org.opencontainers.image.version="0.1.0"
LABEL org.opencontainers.image.vendor="Raibid Labs"
LABEL org.opencontainers.image.source="https://github.com/raibid-labs/raibid-ci"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
LABEL raibid.agent.type="rust"
LABEL raibid.agent.arch="arm64"

# Run the agent
CMD ["/usr/local/bin/raibid-agent"]

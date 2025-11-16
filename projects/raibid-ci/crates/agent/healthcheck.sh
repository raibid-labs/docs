#!/usr/bin/env bash
# Health check script for raibid-ci agent container
# Verifies that all required tools are available and functional

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Track overall health status
HEALTHY=true

# Function to check command availability
check_command() {
    local cmd="$1"
    local description="$2"

    if command -v "$cmd" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $description: $(command -v "$cmd")"
        return 0
    else
        echo -e "${RED}✗${NC} $description: NOT FOUND"
        HEALTHY=false
        return 1
    fi
}

# Function to check command execution
check_execution() {
    local cmd="$1"
    local description="$2"

    if eval "$cmd" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $description"
        return 0
    else
        echo -e "${RED}✗${NC} $description: FAILED"
        HEALTHY=false
        return 1
    fi
}

echo "=== raibid-ci Agent Health Check ==="
echo ""

# Check core system tools
echo "Core System Tools:"
check_command "git" "Git"
check_command "ssh" "SSH client"
check_command "docker" "Docker CLI"

echo ""

# Check Rust toolchain
echo "Rust Toolchain:"
check_command "rustc" "Rust compiler"
check_command "cargo" "Cargo"
check_command "rustfmt" "Rustfmt"
check_command "clippy-driver" "Clippy"

echo ""

# Check Cargo tools
echo "Cargo Tools:"
check_command "cargo-nextest" "Nextest"
check_command "cargo-audit" "Audit"
check_command "cargo-deny" "Deny"

echo ""

# Check Rust toolchain version
echo "Version Information:"
if rustc --version &> /dev/null; then
    echo -e "${GREEN}✓${NC} Rust: $(rustc --version)"
else
    echo -e "${RED}✗${NC} Rust version check failed"
    HEALTHY=false
fi

if cargo --version &> /dev/null; then
    echo -e "${GREEN}✓${NC} Cargo: $(cargo --version)"
else
    echo -e "${RED}✗${NC} Cargo version check failed"
    HEALTHY=false
fi

if docker --version &> /dev/null; then
    echo -e "${GREEN}✓${NC} Docker: $(docker --version)"
else
    echo -e "${RED}✗${NC} Docker version check failed"
    HEALTHY=false
fi

echo ""

# Check Git functionality
echo "Git Configuration:"
if git config --global user.email &> /dev/null || [ -n "${GIT_AUTHOR_EMAIL:-}" ]; then
    echo -e "${GREEN}✓${NC} Git email configured"
else
    echo -e "${RED}!${NC} Git email not configured (will be set by agent)"
fi

if git config --global user.name &> /dev/null || [ -n "${GIT_AUTHOR_NAME:-}" ]; then
    echo -e "${GREEN}✓${NC} Git name configured"
else
    echo -e "${RED}!${NC} Git name not configured (will be set by agent)"
fi

echo ""

# Check filesystem permissions
echo "Filesystem Permissions:"
if [ -w /workspace ]; then
    echo -e "${GREEN}✓${NC} /workspace is writable"
else
    echo -e "${RED}✗${NC} /workspace is not writable"
    HEALTHY=false
fi

if [ -w "$CARGO_HOME" ]; then
    echo -e "${GREEN}✓${NC} \$CARGO_HOME is writable"
else
    echo -e "${RED}✗${NC} \$CARGO_HOME is not writable"
    HEALTHY=false
fi

echo ""

# Check environment variables
echo "Environment Variables:"
ENV_VARS=("RUST_BACKTRACE" "CARGO_HOME" "CARGO_TARGET_DIR")
for var in "${ENV_VARS[@]}"; do
    if [ -n "${!var:-}" ]; then
        echo -e "${GREEN}✓${NC} $var=${!var}"
    else
        echo -e "${RED}!${NC} $var not set"
    fi
done

echo ""
echo "==================================="

# Final status
if [ "$HEALTHY" = true ]; then
    echo -e "${GREEN}✓ Health check PASSED${NC}"
    exit 0
else
    echo -e "${RED}✗ Health check FAILED${NC}"
    exit 1
fi

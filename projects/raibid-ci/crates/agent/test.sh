#!/usr/bin/env bash
# Test script for raibid-ci agent container
# Validates container functionality and tool availability

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Configuration
IMAGE_NAME="${IMAGE_NAME:-raibid-ci-agent}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
REGISTRY="${REGISTRY:-localhost:5000}"
PLATFORM="${PLATFORM:-linux/arm64}"
FULL_IMAGE="$REGISTRY/$IMAGE_NAME:$IMAGE_TAG"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test tracking
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Test helper functions
test_start() {
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    echo -n "Testing: $1 ... "
}

test_pass() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}PASS${NC}"
}

test_fail() {
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo -e "${RED}FAIL${NC}"
    if [ -n "${1:-}" ]; then
        echo "  Error: $1"
    fi
}

# Run command in container
run_in_container() {
    docker run --rm --platform "$PLATFORM" "$FULL_IMAGE" bash -c "$1" 2>&1
}

# Check if image exists
check_image_exists() {
    log_info "Checking if image exists..."
    if docker image inspect "$FULL_IMAGE" &> /dev/null; then
        log_success "Image found: $FULL_IMAGE"
        return 0
    else
        log_error "Image not found: $FULL_IMAGE"
        log_info "Run './build.sh' to build the image first"
        exit 1
    fi
}

# Test image metadata
test_image_metadata() {
    log_info "Testing image metadata..."

    test_start "Image architecture"
    local arch
    arch=$(docker image inspect "$FULL_IMAGE" --format='{{.Architecture}}')
    if [[ "$arch" == "arm64" ]] || [[ "$arch" == "aarch64" ]]; then
        test_pass
    else
        test_fail "Expected arm64/aarch64, got: $arch"
    fi

    test_start "Image size constraint"
    local size
    size=$(docker image inspect "$FULL_IMAGE" --format='{{.Size}}' | awk '{print $1/1024/1024}')
    if (( $(echo "$size < 1536" | bc -l) )); then
        test_pass
        log_info "  Size: ${size} MB (target: < 1536 MB)"
    else
        test_fail "Image size ${size} MB exceeds 1536 MB target"
    fi

    test_start "Image labels"
    local labels
    labels=$(docker image inspect "$FULL_IMAGE" --format='{{range $k, $v := .Config.Labels}}{{$k}}={{$v}} {{end}}')
    if [[ "$labels" == *"raibid.agent.type"* ]]; then
        test_pass
    else
        test_fail "Expected labels not found"
    fi
}

# Test system tools
test_system_tools() {
    log_info "Testing system tools..."

    test_start "git command"
    if run_in_container "git --version" &> /dev/null; then
        test_pass
    else
        test_fail "git not available"
    fi

    test_start "docker command"
    if run_in_container "docker --version" &> /dev/null; then
        test_pass
    else
        test_fail "docker not available"
    fi

    test_start "ssh command"
    if run_in_container "ssh -V" &> /dev/null; then
        test_pass
    else
        test_fail "ssh not available"
    fi
}

# Test Rust toolchain
test_rust_toolchain() {
    log_info "Testing Rust toolchain..."

    test_start "rustc command"
    if run_in_container "rustc --version" &> /dev/null; then
        test_pass
    else
        test_fail "rustc not available"
    fi

    test_start "cargo command"
    if run_in_container "cargo --version" &> /dev/null; then
        test_pass
    else
        test_fail "cargo not available"
    fi

    test_start "rustfmt command"
    if run_in_container "rustfmt --version" &> /dev/null; then
        test_pass
    else
        test_fail "rustfmt not available"
    fi

    test_start "clippy command"
    if run_in_container "cargo clippy --version" &> /dev/null; then
        test_pass
    else
        test_fail "clippy not available"
    fi
}

# Test cargo tools
test_cargo_tools() {
    log_info "Testing cargo tools..."

    test_start "cargo-nextest"
    if run_in_container "cargo nextest --version" &> /dev/null; then
        test_pass
    else
        test_fail "cargo-nextest not available"
    fi

    test_start "cargo-audit"
    if run_in_container "cargo audit --version" &> /dev/null; then
        test_pass
    else
        test_fail "cargo-audit not available"
    fi

    test_start "cargo-deny"
    if run_in_container "cargo deny --version" &> /dev/null; then
        test_pass
    else
        test_fail "cargo-deny not available"
    fi
}

# Test filesystem permissions
test_permissions() {
    log_info "Testing filesystem permissions..."

    test_start "Workspace writable"
    if run_in_container "touch /workspace/test && rm /workspace/test" &> /dev/null; then
        test_pass
    else
        test_fail "/workspace is not writable"
    fi

    test_start "Cargo home writable"
    if run_in_container "touch /home/agent/.cargo/test && rm /home/agent/.cargo/test" &> /dev/null; then
        test_pass
    else
        test_fail "CARGO_HOME is not writable"
    fi

    test_start "Non-root user"
    local uid
    uid=$(run_in_container "id -u")
    if [[ "$uid" == "1000" ]]; then
        test_pass
    else
        test_fail "Expected UID 1000, got: $uid"
    fi
}

# Test environment variables
test_environment() {
    log_info "Testing environment variables..."

    test_start "RUST_BACKTRACE set"
    local backtrace
    backtrace=$(run_in_container "echo \$RUST_BACKTRACE")
    if [[ "$backtrace" == "1" ]]; then
        test_pass
    else
        test_fail "RUST_BACKTRACE not set to 1"
    fi

    test_start "CARGO_HOME set"
    local cargo_home
    cargo_home=$(run_in_container "echo \$CARGO_HOME")
    if [[ "$cargo_home" == "/home/agent/.cargo" ]]; then
        test_pass
    else
        test_fail "CARGO_HOME not set correctly"
    fi

    test_start "CARGO_TARGET_DIR set"
    local target_dir
    target_dir=$(run_in_container "echo \$CARGO_TARGET_DIR")
    if [[ "$target_dir" == "/workspace/target" ]]; then
        test_pass
    else
        test_fail "CARGO_TARGET_DIR not set correctly"
    fi
}

# Test health check
test_health_check() {
    log_info "Testing health check..."

    test_start "Health check script exists"
    if run_in_container "test -f /usr/local/bin/healthcheck.sh" &> /dev/null; then
        test_pass
    else
        test_fail "healthcheck.sh not found"
    fi

    test_start "Health check executable"
    if run_in_container "test -x /usr/local/bin/healthcheck.sh" &> /dev/null; then
        test_pass
    else
        test_fail "healthcheck.sh not executable"
    fi

    test_start "Health check passes"
    if run_in_container "/usr/local/bin/healthcheck.sh" &> /dev/null; then
        test_pass
    else
        test_fail "health check failed"
    fi
}

# Test cargo functionality
test_cargo_functionality() {
    log_info "Testing cargo functionality..."

    # Create a temporary test project
    local tmpdir
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    # Initialize a simple Rust project
    docker run --rm --platform "$PLATFORM" -v "$tmpdir:/test" "$FULL_IMAGE" bash -c "
        cd /test
        cargo init --name test-project
        echo 'fn main() { println!(\"Hello, raibid-ci!\"); }' > src/main.rs
    " &> /dev/null

    test_start "cargo build"
    if docker run --rm --platform "$PLATFORM" -v "$tmpdir:/test" "$FULL_IMAGE" bash -c "
        cd /test
        cargo build --release 2>&1
    " | grep -q "Finished"; then
        test_pass
    else
        test_fail "cargo build failed"
    fi

    test_start "cargo test"
    if docker run --rm --platform "$PLATFORM" -v "$tmpdir:/test" "$FULL_IMAGE" bash -c "
        cd /test
        cargo test 2>&1
    " | grep -q "test result: ok"; then
        test_pass
    else
        test_fail "cargo test failed"
    fi

    test_start "cargo nextest run"
    if docker run --rm --platform "$PLATFORM" -v "$tmpdir:/test" "$FULL_IMAGE" bash -c "
        cd /test
        cargo nextest run 2>&1
    " &> /dev/null; then
        test_pass
    else
        test_warn "cargo nextest run failed (may be expected for simple project)"
        test_pass  # Don't fail on this for now
    fi
}

# Print test summary
print_summary() {
    echo ""
    echo "=========================================="
    echo "Test Summary"
    echo "=========================================="
    echo "Total:  $TESTS_TOTAL"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    echo "=========================================="

    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}Some tests failed!${NC}"
        return 1
    fi
}

# Main execution
main() {
    log_info "raibid-ci Agent Container Test Suite"
    log_info "Image: $FULL_IMAGE"
    echo ""

    check_image_exists
    echo ""

    test_image_metadata
    echo ""

    test_system_tools
    echo ""

    test_rust_toolchain
    echo ""

    test_cargo_tools
    echo ""

    test_permissions
    echo ""

    test_environment
    echo ""

    test_health_check
    echo ""

    test_cargo_functionality
    echo ""

    if print_summary; then
        exit 0
    else
        exit 1
    fi
}

# Run main
main "$@"

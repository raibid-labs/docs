#!/usr/bin/env bash
# Build Scarab for all supported platforms
# Useful for testing cross-compilation before release

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Build counters
BUILDS_TOTAL=0
BUILDS_SUCCESS=0
BUILDS_FAILED=0
BUILDS_SKIPPED=0

# Logging
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_fail() {
    echo -e "${RED}[✗]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

# Check if target is installed
is_target_installed() {
    local target="$1"
    rustup target list --installed | grep -q "^${target}$"
}

# Install target if not present
install_target() {
    local target="$1"

    log_info "Installing target: $target"
    if rustup target add "$target"; then
        log_success "Installed $target"
        return 0
    else
        log_fail "Failed to install $target"
        return 1
    fi
}

# Build for a specific target
build_target() {
    local target="$1"
    local name="$2"

    ((BUILDS_TOTAL++))

    log_info "Building for $name ($target)..."

    # Check if target is installed
    if ! is_target_installed "$target"; then
        log_warn "Target $target not installed"
        read -p "Install $target? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if ! install_target "$target"; then
                ((BUILDS_SKIPPED++))
                return 1
            fi
        else
            log_warn "Skipping $name"
            ((BUILDS_SKIPPED++))
            return 1
        fi
    fi

    # Build
    local start_time
    start_time=$(date +%s)

    if cargo build --release --target "$target" --quiet 2>&1 | tee "/tmp/scarab-build-${target}.log"; then
        local end_time
        end_time=$(date +%s)
        local duration=$((end_time - start_time))

        log_success "Built $name in ${duration}s"
        ((BUILDS_SUCCESS++))

        # Show binary sizes
        if [[ -f "target/$target/release/scarab-daemon" ]]; then
            local daemon_size
            daemon_size=$(du -h "target/$target/release/scarab-daemon" | cut -f1)
            log_info "  scarab-daemon: $daemon_size"
        fi
        if [[ -f "target/$target/release/scarab-client" ]]; then
            local client_size
            client_size=$(du -h "target/$target/release/scarab-client" | cut -f1)
            log_info "  scarab-client: $client_size"
        fi

        return 0
    else
        log_fail "Build failed for $name"
        log_info "See log: /tmp/scarab-build-${target}.log"
        ((BUILDS_FAILED++))
        return 1
    fi
}

# Main script
main() {
    cd "$PROJECT_ROOT"

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}Scarab Cross-Platform Build${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo

    # Determine which targets to build based on host OS
    local current_os
    current_os=$(uname -s)

    log_info "Detected host OS: $current_os"
    echo

    # Define targets based on OS
    case "$current_os" in
        Linux)
            log_info "Building Linux targets..."
            build_target "x86_64-unknown-linux-gnu" "Linux (x86_64 glibc)"
            build_target "x86_64-unknown-linux-musl" "Linux (x86_64 musl)"

            # ARM64 requires cross-compilation setup
            if command -v aarch64-linux-gnu-gcc &>/dev/null; then
                build_target "aarch64-unknown-linux-gnu" "Linux (ARM64)"
            else
                log_warn "ARM64 cross-compiler not found, skipping aarch64-unknown-linux-gnu"
                log_info "Install: sudo apt-get install gcc-aarch64-linux-gnu"
                ((BUILDS_SKIPPED++))
            fi
            ;;

        Darwin)
            log_info "Building macOS targets..."

            # Detect current architecture
            local arch
            arch=$(uname -m)

            if [[ "$arch" == "arm64" ]]; then
                # On Apple Silicon, build both
                build_target "aarch64-apple-darwin" "macOS (Apple Silicon)"
                build_target "x86_64-apple-darwin" "macOS (Intel)"
            else
                # On Intel, build both
                build_target "x86_64-apple-darwin" "macOS (Intel)"
                build_target "aarch64-apple-darwin" "macOS (Apple Silicon)"
            fi
            ;;

        MINGW* | MSYS* | CYGWIN*)
            log_info "Building Windows targets..."
            build_target "x86_64-pc-windows-msvc" "Windows (x86_64 MSVC)"

            # GNU target
            if is_target_installed "x86_64-pc-windows-gnu"; then
                build_target "x86_64-pc-windows-gnu" "Windows (x86_64 GNU)"
            else
                log_warn "x86_64-pc-windows-gnu not installed, skipping"
                ((BUILDS_SKIPPED++))
            fi
            ;;

        *)
            log_fail "Unsupported OS: $current_os"
            exit 1
            ;;
    esac

    # Try cross-OS targets if explicitly requested
    if [[ "${BUILD_ALL_PLATFORMS:-}" == "1" ]]; then
        echo
        log_info "Building all platforms (BUILD_ALL_PLATFORMS=1)..."

        case "$current_os" in
            Linux)
                log_warn "Cross-building for macOS/Windows from Linux requires additional setup"
                log_info "Consider using GitHub Actions for full cross-platform builds"
                ;;
            Darwin)
                log_warn "Cross-building for Linux/Windows from macOS requires additional setup"
                log_info "Consider using GitHub Actions for full cross-platform builds"
                ;;
            *)
                log_warn "Cross-building from $current_os not recommended"
                ;;
        esac
    fi

    # Summary
    echo
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}Build Summary${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}Successful:${NC} $BUILDS_SUCCESS"
    echo -e "${RED}Failed:${NC}     $BUILDS_FAILED"
    echo -e "${YELLOW}Skipped:${NC}    $BUILDS_SKIPPED"
    echo -e "Total:      $BUILDS_TOTAL"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if [[ $BUILDS_FAILED -eq 0 ]]; then
        echo -e "${GREEN}✓ All builds completed successfully!${NC}"
        echo
        log_info "Build artifacts:"
        find target -type f \( -name "scarab-daemon" -o -name "scarab-client" \) -path "*/release/*" | while read -r file; do
            local size
            size=$(du -h "$file" | cut -f1)
            echo "  $file ($size)"
        done
        return 0
    else
        echo -e "${RED}✗ Some builds failed.${NC}"
        echo
        log_info "Failed build logs:"
        ls -1 /tmp/scarab-build-*.log 2>/dev/null || true
        return 1
    fi
}

# Usage information
usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Build Scarab for all supported platforms on the current OS.

OPTIONS:
    -h, --help              Show this help message
    -a, --all               Attempt to build for ALL platforms (requires setup)

ENVIRONMENT VARIABLES:
    BUILD_ALL_PLATFORMS=1   Same as --all flag

EXAMPLES:
    # Build for native platforms only
    ./scripts/build-all-targets.sh

    # Attempt all platforms (may fail without cross-compilation setup)
    BUILD_ALL_PLATFORMS=1 ./scripts/build-all-targets.sh

NOTES:
    - Builds are release builds (--release flag)
    - Missing targets will prompt for installation
    - Cross-compilation may require additional tools:
        - Linux ARM64: gcc-aarch64-linux-gnu
        - Linux musl: musl-tools
        - Windows from Linux: mingw-w64
    - For complete cross-platform testing, use GitHub Actions

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        -a|--all)
            export BUILD_ALL_PLATFORMS=1
            shift
            ;;
        *)
            log_fail "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Run main
main "$@"

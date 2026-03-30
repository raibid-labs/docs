#!/bin/bash
#
# Build Scarab for all supported platforms
# This script handles cross-compilation for macOS, Linux, and Windows

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="$PROJECT_ROOT/target"
RELEASE_DIR="$PROJECT_ROOT/releases"

# Build profiles
PROFILE="${BUILD_PROFILE:-release}"
CARGO_FLAGS=""

# Print colored message
print_status() {
    echo -e "${BLUE}[BUILD]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check for required tools
check_dependencies() {
    print_status "Checking dependencies..."

    if ! command -v rustup &> /dev/null; then
        print_error "rustup is not installed. Please install from https://rustup.rs/"
        exit 1
    fi

    if ! command -v cargo &> /dev/null; then
        print_error "cargo is not installed."
        exit 1
    fi
}

# Install cross-compilation targets
install_targets() {
    print_status "Installing compilation targets..."

    # macOS targets
    rustup target add aarch64-apple-darwin || true
    rustup target add x86_64-apple-darwin || true

    # Linux targets
    rustup target add x86_64-unknown-linux-gnu || true
    rustup target add x86_64-unknown-linux-musl || true
    rustup target add aarch64-unknown-linux-gnu || true

    # Windows targets
    rustup target add x86_64-pc-windows-msvc || true
    rustup target add x86_64-pc-windows-gnu || true
}

# Build for a specific target
build_target() {
    local target=$1
    local features=$2

    print_status "Building for $target..."

    # Set environment variables for cross-compilation
    case "$target" in
        *-windows-gnu)
            export CC=x86_64-w64-mingw32-gcc
            export CXX=x86_64-w64-mingw32-g++
            export AR=x86_64-w64-mingw32-ar
            ;;
        *-linux-musl)
            export CC=musl-gcc
            ;;
    esac

    # Build command
    if [ -n "$features" ]; then
        cargo build --target "$target" --profile "$PROFILE" --features "$features" $CARGO_FLAGS
    else
        cargo build --target "$target" --profile "$PROFILE" $CARGO_FLAGS
    fi

    if [ $? -eq 0 ]; then
        print_success "Built $target successfully"

        # Strip binaries for release builds
        if [ "$PROFILE" = "release" ]; then
            strip_binary "$target"
        fi

        # Package the binary
        package_binary "$target"
    else
        print_error "Failed to build $target"
        return 1
    fi
}

# Strip debug symbols from binary
strip_binary() {
    local target=$1
    local binary_dir="$TARGET_DIR/$target/$PROFILE"

    print_status "Stripping binaries for $target..."

    case "$target" in
        *-apple-darwin)
            find "$binary_dir" -name "scarab-*" -type f -perm +111 -exec strip -x {} \; 2>/dev/null || true
            ;;
        *-linux-*)
            find "$binary_dir" -name "scarab-*" -type f -executable -exec strip --strip-all {} \; 2>/dev/null || true
            ;;
        *-windows-*)
            # Windows binaries are already optimized
            ;;
    esac
}

# Package binary for distribution
package_binary() {
    local target=$1
    local binary_dir="$TARGET_DIR/$target/$PROFILE"
    local package_dir="$RELEASE_DIR/$target"

    mkdir -p "$package_dir"

    print_status "Packaging binaries for $target..."

    # Copy binaries
    case "$target" in
        *-windows-*)
            cp "$binary_dir"/scarab-*.exe "$package_dir"/ 2>/dev/null || true
            ;;
        *)
            cp "$binary_dir"/scarab-* "$package_dir"/ 2>/dev/null || true
            ;;
    esac

    # Create archive
    local archive_name="scarab-$(git describe --tags --always 2>/dev/null || echo "dev")-$target"

    cd "$RELEASE_DIR"
    case "$target" in
        *-windows-*)
            if command -v zip &> /dev/null; then
                zip -r "$archive_name.zip" "$target"
                print_success "Created $archive_name.zip"
            fi
            ;;
        *)
            tar czf "$archive_name.tar.gz" "$target"
            print_success "Created $archive_name.tar.gz"
            ;;
    esac
    cd - > /dev/null
}

# Build all targets
build_all() {
    local failed_targets=()

    # Detect host platform
    local host_os="$(uname -s)"
    local host_arch="$(uname -m)"

    print_status "Host platform: $host_os $host_arch"

    # Build based on host platform
    case "$host_os" in
        Darwin)
            print_status "Building macOS targets..."
            build_target "aarch64-apple-darwin" "" || failed_targets+=("aarch64-apple-darwin")
            build_target "x86_64-apple-darwin" "" || failed_targets+=("x86_64-apple-darwin")
            ;;
        Linux)
            print_status "Building Linux targets..."
            build_target "x86_64-unknown-linux-gnu" "" || failed_targets+=("x86_64-unknown-linux-gnu")

            # Only build musl if available
            if command -v musl-gcc &> /dev/null; then
                build_target "x86_64-unknown-linux-musl" "" || failed_targets+=("x86_64-unknown-linux-musl")
            else
                print_warning "musl-gcc not found, skipping musl builds"
            fi

            # Cross-compile for Windows if mingw is available
            if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
                build_target "x86_64-pc-windows-gnu" "" || failed_targets+=("x86_64-pc-windows-gnu")
            else
                print_warning "mingw-w64 not found, skipping Windows cross-compilation"
            fi
            ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            print_status "Building Windows targets..."
            build_target "x86_64-pc-windows-msvc" "" || failed_targets+=("x86_64-pc-windows-msvc")
            ;;
        *)
            print_error "Unsupported host platform: $host_os"
            exit 1
            ;;
    esac

    # Print summary
    echo
    print_status "Build Summary:"
    echo "=================="

    if [ ${#failed_targets[@]} -eq 0 ]; then
        print_success "All targets built successfully!"
    else
        print_warning "The following targets failed to build:"
        for target in "${failed_targets[@]}"; do
            echo "  - $target"
        done
    fi

    # Show binary sizes
    echo
    print_status "Binary sizes:"
    du -sh "$RELEASE_DIR"/*/*.tar.gz 2>/dev/null || true
    du -sh "$RELEASE_DIR"/*/*.zip 2>/dev/null || true
}

# Clean build artifacts
clean() {
    print_status "Cleaning build artifacts..."
    cargo clean
    rm -rf "$RELEASE_DIR"
    print_success "Cleaned build artifacts"
}

# Main function
main() {
    case "${1:-}" in
        clean)
            clean
            ;;
        install-targets)
            check_dependencies
            install_targets
            ;;
        *)
            check_dependencies
            install_targets
            build_all
            ;;
    esac
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --features)
            CARGO_FLAGS="--features $2"
            shift 2
            ;;
        --verbose)
            CARGO_FLAGS="$CARGO_FLAGS --verbose"
            shift
            ;;
        *)
            break
            ;;
    esac
done

main "$@"
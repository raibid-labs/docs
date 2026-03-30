#!/usr/bin/env bash
#
# Local Release Testing Script
#
# This script simulates the GitHub release workflow locally.
# It builds release binaries, creates archives, and verifies archive contents.
#
# Usage:
#   ./scripts/test-release-locally.sh [--version VERSION]
#
# Options:
#   --version VERSION   Specify version string (default: current git tag or "local-test")
#   --output-dir DIR    Output directory for archives (default: ./dist)
#   --clean             Clean output directory before building

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/dist"
VERSION=""
CLEAN=0

# Platform detection
PLATFORM="$(uname -s)"
ARCH="$(uname -m)"

case "$PLATFORM" in
    Linux*)
        OS_NAME="linux"
        ;;
    Darwin*)
        OS_NAME="macos"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        OS_NAME="windows"
        ;;
    *)
        echo -e "${RED}Unsupported platform: $PLATFORM${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_NAME="x64"
        ;;
    aarch64|arm64)
        ARCH_NAME="arm64"
        ;;
    *)
        echo -e "${YELLOW}Warning: Unknown architecture $ARCH, using as-is${NC}"
        ARCH_NAME="$ARCH"
        ;;
esac

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --clean)
            CLEAN=1
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --version VERSION   Specify version string (default: git tag or 'local-test')"
            echo "  --output-dir DIR    Output directory for archives (default: ./dist)"
            echo "  --clean             Clean output directory before building"
            echo "  -h, --help          Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Determine version if not specified
if [[ -z "$VERSION" ]]; then
    if git describe --tags --exact-match 2>/dev/null; then
        VERSION=$(git describe --tags --exact-match)
        echo -e "${BLUE}[INFO]${NC} Using git tag: $VERSION"
    else
        VERSION="local-test-$(date +%Y%m%d)"
        echo -e "${YELLOW}[WARNING]${NC} No git tag found, using: $VERSION"
    fi
fi

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Clean output directory
clean_output() {
    if [[ $CLEAN -eq 1 ]]; then
        log_info "Cleaning output directory: $OUTPUT_DIR"
        rm -rf "$OUTPUT_DIR"
    fi

    mkdir -p "$OUTPUT_DIR"
}

# Build release binaries
build_release() {
    log_info "Building release binaries..."
    cd "$PROJECT_ROOT"

    cargo build --release --workspace

    if [[ $? -eq 0 ]]; then
        log_success "Release build completed"
        return 0
    else
        log_error "Release build failed"
        return 1
    fi
}

# Find release directory
find_release_dir() {
    local project_target="$PROJECT_ROOT/target/release"
    local cargo_home="${CARGO_HOME:-$HOME/.cargo}"
    local cargo_target="$cargo_home/target/release"

    if [[ -d "$project_target" ]]; then
        echo "$project_target"
    elif [[ -d "$cargo_target" ]]; then
        echo "$cargo_target"
    else
        log_error "No release directory found"
        exit 1
    fi
}

# Create archive for current platform
create_archive() {
    log_info "Creating release archive for $OS_NAME-$ARCH_NAME..."

    local release_dir=$(find_release_dir)
    local archive_name="scarab-${VERSION}-${OS_NAME}-${ARCH_NAME}"
    local archive_dir="$OUTPUT_DIR/$archive_name"

    # Create temporary directory for archive contents
    mkdir -p "$archive_dir/bin"

    # Copy binaries
    log_info "Copying binaries..."
    local binaries=(
        "scarab-daemon"
        "scarab-client"
        "scarab-plugin-compiler"
    )

    for binary in "${binaries[@]}"; do
        local binary_path="$release_dir/$binary"
        if [[ "$OS_NAME" == "windows" ]]; then
            binary_path="${binary_path}.exe"
        fi

        if [[ -f "$binary_path" ]]; then
            cp "$binary_path" "$archive_dir/bin/"
            log_success "Copied: $binary"
        else
            log_error "Binary not found: $binary_path"
            return 1
        fi
    done

    # Copy additional files
    log_info "Copying documentation..."
    if [[ -f "$PROJECT_ROOT/README.md" ]]; then
        cp "$PROJECT_ROOT/README.md" "$archive_dir/"
    fi

    if [[ -f "$PROJECT_ROOT/LICENSE" ]] || [[ -f "$PROJECT_ROOT/LICENSE-MIT" ]]; then
        cp "$PROJECT_ROOT"/LICENSE* "$archive_dir/" 2>/dev/null || true
    fi

    if [[ -f "$PROJECT_ROOT/CHANGELOG.md" ]]; then
        cp "$PROJECT_ROOT/CHANGELOG.md" "$archive_dir/"
    fi

    # Create examples directory if it exists
    if [[ -d "$PROJECT_ROOT/examples" ]]; then
        log_info "Copying examples..."
        cp -r "$PROJECT_ROOT/examples" "$archive_dir/"
    fi

    # Create INSTALL.md
    cat > "$archive_dir/INSTALL.md" <<EOF
# Scarab Terminal Emulator - Installation Guide

Version: $VERSION
Platform: $OS_NAME-$ARCH_NAME
Built: $(date)

## Installation

### 1. Extract Archive

Extract this archive to your desired installation location:

\`\`\`bash
tar -xzf $archive_name.tar.gz
cd $archive_name
\`\`\`

### 2. Add to PATH

Add the bin directory to your PATH:

\`\`\`bash
export PATH="\$(pwd)/bin:\$PATH"
\`\`\`

Or copy binaries to a directory already in your PATH:

\`\`\`bash
cp bin/* /usr/local/bin/
\`\`\`

### 3. Verify Installation

\`\`\`bash
scarab-daemon --help
scarab-client --help
scarab-plugin-compiler --help
\`\`\`

## Usage

### Start Daemon

\`\`\`bash
scarab-daemon
\`\`\`

### Start Client (in a new terminal)

\`\`\`bash
scarab-client
\`\`\`

### Compile Plugins

\`\`\`bash
scarab-plugin-compiler examples/plugin-template/src/lib.fsx -o my-plugin.fzb
\`\`\`

## Configuration

Configuration files are located at:
- Linux/macOS: ~/.config/scarab/
- Windows: %APPDATA%/scarab/

## Documentation

See README.md for more information.

## Support

- GitHub: https://github.com/YOUR_ORG/scarab
- Issues: https://github.com/YOUR_ORG/scarab/issues
EOF

    # Create archive
    cd "$OUTPUT_DIR"

    case "$OS_NAME" in
        windows)
            log_info "Creating ZIP archive..."
            if command -v zip &> /dev/null; then
                zip -r "${archive_name}.zip" "$archive_name"
                log_success "Created: ${archive_name}.zip"
            else
                log_error "zip command not found"
                return 1
            fi
            ;;
        *)
            log_info "Creating tar.gz archive..."
            tar -czf "${archive_name}.tar.gz" "$archive_name"
            log_success "Created: ${archive_name}.tar.gz"

            # Create checksum
            if command -v sha256sum &> /dev/null; then
                sha256sum "${archive_name}.tar.gz" > "${archive_name}.tar.gz.sha256"
                log_success "Created checksum: ${archive_name}.tar.gz.sha256"
            elif command -v shasum &> /dev/null; then
                shasum -a 256 "${archive_name}.tar.gz" > "${archive_name}.tar.gz.sha256"
                log_success "Created checksum: ${archive_name}.tar.gz.sha256"
            fi
            ;;
    esac

    # Clean up temporary directory
    rm -rf "$archive_dir"

    return 0
}

# Verify archive contents
verify_archive() {
    log_info "Verifying archive contents..."

    cd "$OUTPUT_DIR"

    local archive_name="scarab-${VERSION}-${OS_NAME}-${ARCH_NAME}"
    local archive_file=""

    if [[ "$OS_NAME" == "windows" ]]; then
        archive_file="${archive_name}.zip"
    else
        archive_file="${archive_name}.tar.gz"
    fi

    if [[ ! -f "$archive_file" ]]; then
        log_error "Archive not found: $archive_file"
        return 1
    fi

    # Extract to temporary location for verification
    local temp_dir=$(mktemp -d)
    log_info "Extracting to temporary directory for verification..."

    case "$OS_NAME" in
        windows)
            if ! unzip -q "$archive_file" -d "$temp_dir"; then
                log_error "Failed to extract archive"
                rm -rf "$temp_dir"
                return 1
            fi
            ;;
        *)
            if ! tar -xzf "$archive_file" -C "$temp_dir"; then
                log_error "Failed to extract archive"
                rm -rf "$temp_dir"
                return 1
            fi
            ;;
    esac

    # Verify expected files
    local expected_files=(
        "$archive_name/bin/scarab-daemon"
        "$archive_name/bin/scarab-client"
        "$archive_name/bin/scarab-plugin-compiler"
        "$archive_name/README.md"
        "$archive_name/INSTALL.md"
    )

    local all_found=1
    for file in "${expected_files[@]}"; do
        if [[ -f "$temp_dir/$file" ]] || [[ -f "$temp_dir/${file}.exe" ]]; then
            log_success "Found: $file"
        else
            log_error "Missing: $file"
            all_found=0
        fi
    done

    # Verify binaries are executable
    if [[ "$OS_NAME" != "windows" ]]; then
        for binary in scarab-daemon scarab-client scarab-plugin-compiler; do
            local binary_path="$temp_dir/$archive_name/bin/$binary"
            if [[ -x "$binary_path" ]]; then
                log_success "Executable: $binary"
            else
                log_error "Not executable: $binary"
                all_found=0
            fi
        done
    fi

    # Clean up
    rm -rf "$temp_dir"

    if [[ $all_found -eq 1 ]]; then
        log_success "Archive verification passed"
        return 0
    else
        log_error "Archive verification failed"
        return 1
    fi
}

# Generate release notes
generate_release_notes() {
    log_info "Generating release notes..."

    local archive_name="scarab-${VERSION}-${OS_NAME}-${ARCH_NAME}"
    local release_notes="$OUTPUT_DIR/RELEASE-NOTES-${VERSION}.md"

    cat > "$release_notes" <<EOF
# Scarab Terminal Emulator ${VERSION}

Release Date: $(date +%Y-%m-%d)
Platform: $OS_NAME-$ARCH_NAME

## Downloads

### $OS_NAME ($ARCH_NAME)

EOF

    # Add download links (placeholders for actual releases)
    if [[ "$OS_NAME" == "windows" ]]; then
        echo "- [scarab-${VERSION}-${OS_NAME}-${ARCH_NAME}.zip]()" >> "$release_notes"
    else
        echo "- [scarab-${VERSION}-${OS_NAME}-${ARCH_NAME}.tar.gz]()" >> "$release_notes"
        echo "- [scarab-${VERSION}-${OS_NAME}-${ARCH_NAME}.tar.gz.sha256]()" >> "$release_notes"
    fi

    cat >> "$release_notes" <<EOF

## Installation

See INSTALL.md in the archive for detailed installation instructions.

## What's New

- High-performance split-process terminal emulator
- Fusabi plugin system for extensibility
- GPU-accelerated rendering with Bevy
- Zero-copy IPC via shared memory
- Multi-session support

## System Requirements

- $OS_NAME ($ARCH_NAME)
- OpenGL 3.3 or Vulkan support (for GPU rendering)
- 100MB disk space (minimum)

## Checksums

EOF

    # Add checksums if available
    if [[ -f "$OUTPUT_DIR/${archive_name}.tar.gz.sha256" ]]; then
        cat "$OUTPUT_DIR/${archive_name}.tar.gz.sha256" >> "$release_notes"
    fi

    log_success "Release notes: $release_notes"
}

# Print summary
print_summary() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Release Build Summary${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo -e "Version:  ${BLUE}$VERSION${NC}"
    echo -e "Platform: ${BLUE}$OS_NAME-$ARCH_NAME${NC}"
    echo -e "Output:   ${BLUE}$OUTPUT_DIR${NC}"
    echo ""
    echo "Created files:"

    cd "$OUTPUT_DIR"
    for file in scarab-${VERSION}-*; do
        if [[ -f "$file" ]]; then
            local size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
            local size_mb=$(echo "scale=2; $size / 1024 / 1024" | bc)
            echo -e "  ${GREEN}✓${NC} $file (${size_mb} MB)"
        fi
    done

    echo ""
}

# Main execution
main() {
    log_info "Local Release Testing for Scarab"
    log_info "Platform: $OS_NAME-$ARCH_NAME"
    log_info "Version: $VERSION"

    # Clean output directory
    clean_output

    # Build release
    if ! build_release; then
        log_error "Build failed"
        exit 1
    fi

    # Create archive
    if ! create_archive; then
        log_error "Archive creation failed"
        exit 1
    fi

    # Verify archive
    if ! verify_archive; then
        log_error "Archive verification failed"
        exit 1
    fi

    # Generate release notes
    generate_release_notes

    # Print summary
    print_summary

    log_success "Local release testing complete!"
}

# Run main function
main

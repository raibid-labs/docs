#!/usr/bin/env bash
# Scarab Terminal Installer
# Universal installation script for Linux and macOS

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Configuration
REPO="raibid-labs/scarab"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/scarab"

# Logging functions
log_info() {
    echo -e "${BLUE}${BOLD}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}${BOLD}[✓]${NC} $1"
}

log_error() {
    echo -e "${RED}${BOLD}[✗]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}${BOLD}[⚠]${NC} $1"
}

# Detect platform
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="darwin"
            ;;
        *)
            log_error "Unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    # Determine target triple
    if [ "$OS" = "linux" ]; then
        if command -v ldd &>/dev/null && ldd --version 2>&1 | grep -q musl; then
            TARGET="${ARCH}-unknown-linux-musl"
        else
            TARGET="${ARCH}-unknown-linux-gnu"
        fi
    else
        TARGET="${ARCH}-apple-${OS}"
    fi

    log_info "Detected platform: $OS ($ARCH) - $TARGET"
}

# Check dependencies
check_dependencies() {
    local missing=()

    for cmd in curl tar; do
        if ! command -v $cmd &>/dev/null; then
            missing+=("$cmd")
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing[*]}"
        log_info "Please install them and try again:"

        if [ "$OS" = "linux" ]; then
            log_info "  Ubuntu/Debian: sudo apt-get install ${missing[*]}"
            log_info "  Fedora/RHEL: sudo dnf install ${missing[*]}"
            log_info "  Arch: sudo pacman -S ${missing[*]}"
        else
            log_info "  macOS: brew install ${missing[*]}"
        fi

        exit 1
    fi
}

# Get latest release version
get_latest_version() {
    log_info "Fetching latest release..."

    VERSION=$(curl -sSf "https://api.github.com/repos/$REPO/releases/latest" \
        | grep '"tag_name":' \
        | sed -E 's/.*"([^"]+)".*/\1/')

    if [ -z "$VERSION" ]; then
        log_error "Failed to fetch latest version"
        exit 1
    fi

    log_info "Latest version: $VERSION"
}

# Download and extract binaries
download_binaries() {
    local archive_name="scarab-${VERSION}-${TARGET}.tar.gz"
    local download_url="https://github.com/$REPO/releases/download/$VERSION/$archive_name"
    local temp_dir=$(mktemp -d)

    log_info "Downloading from $download_url..."

    if ! curl -sSfL "$download_url" -o "$temp_dir/$archive_name"; then
        log_error "Failed to download release archive"
        log_info "URL: $download_url"
        rm -rf "$temp_dir"
        exit 1
    fi

    log_info "Extracting binaries..."
    tar -xzf "$temp_dir/$archive_name" -C "$temp_dir"

    # Install binaries
    mkdir -p "$INSTALL_DIR"

    log_info "Installing to $INSTALL_DIR..."

    for binary in scarab-daemon scarab-client scarab-plugin-compiler; do
        if [ -f "$temp_dir/$binary" ]; then
            cp "$temp_dir/$binary" "$INSTALL_DIR/"
            chmod +x "$INSTALL_DIR/$binary"
            log_success "Installed $binary"
        fi
    done

    # Create convenience symlink
    if [ -f "$INSTALL_DIR/scarab-client" ]; then
        ln -sf "$INSTALL_DIR/scarab-client" "$INSTALL_DIR/scarab"
        log_success "Created scarab symlink"
    fi

    rm -rf "$temp_dir"
}

# Setup configuration
setup_config() {
    mkdir -p "$CONFIG_DIR"

    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        log_info "Creating default configuration..."

        cat > "$CONFIG_DIR/config.toml" <<'EOF'
# Scarab Terminal Configuration

[terminal]
shell = "$SHELL"
font_size = 14.0
font_family = "monospace"

[theme]
name = "default"

[gpu]
vsync = true

[plugins]
enabled = true
EOF

        log_success "Created $CONFIG_DIR/config.toml"
    else
        log_info "Configuration already exists at $CONFIG_DIR/config.toml"
    fi
}

# Check PATH
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        log_warn "$INSTALL_DIR is not in your PATH"
        log_info "Add it to your shell profile:"
        echo ""
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
        log_info "Then restart your shell or run: source ~/.bashrc (or ~/.zshrc)"
    fi
}

# Print post-install message
print_success() {
    echo ""
    log_success "Scarab Terminal installed successfully!"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "  ${BOLD}Quick Start:${NC}"
    echo ""
    echo "  1. Start the daemon:"
    echo "     ${BLUE}scarab-daemon &${NC}"
    echo ""
    echo "  2. Launch the terminal:"
    echo "     ${BLUE}scarab${NC}"
    echo ""
    echo "  ${BOLD}Configuration:${NC}"
    echo "     $CONFIG_DIR/config.toml"
    echo ""
    echo "  ${BOLD}Documentation:${NC}"
    echo "     https://github.com/$REPO"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# Main installation
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  ${BOLD}Scarab Terminal Installer${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    detect_platform
    check_dependencies
    get_latest_version
    download_binaries
    setup_config
    check_path
    print_success
}

# Handle Ctrl+C
trap 'log_error "Installation cancelled"; exit 1' INT

# Run installer
main "$@"

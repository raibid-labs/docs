#!/bin/bash
set -e

# Build release artifacts for raibid-cli
# Usage: ./scripts/build-release.sh [VERSION]

VERSION=${1:-0.1.0}
PROJECT_NAME="raibid-cli"
RELEASE_DIR="release"

echo "Building release artifacts for $PROJECT_NAME version $VERSION"
echo "========================================================"

# Clean previous release directory
if [ -d "$RELEASE_DIR" ]; then
    echo "Cleaning previous release directory..."
    rm -rf "$RELEASE_DIR"
fi

# Create release directory structure
echo "Creating release directory structure..."
mkdir -p "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}"
mkdir -p "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/examples"
mkdir -p "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/docs"

# Build for x86_64 (native)
echo ""
echo "Building for x86_64-unknown-linux-gnu..."
cargo build --release
echo "✓ x86_64 build complete"

# Copy x86_64 binary
cp target/release/$PROJECT_NAME "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/"
echo "✓ Copied x86_64 binary"

# Build for ARM64 (DGX Spark)
echo ""
echo "Checking for ARM64 toolchain..."
if rustup target list | grep -q "aarch64-unknown-linux-gnu (installed)"; then
    echo "Building for aarch64-unknown-linux-gnu (ARM64)..."
    cargo build --release --target aarch64-unknown-linux-gnu
    echo "✓ ARM64 build complete"

    # Create ARM64-specific directory
    mkdir -p "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}-aarch64"
    cp target/aarch64-unknown-linux-gnu/release/$PROJECT_NAME "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}-aarch64/"
    echo "✓ Copied ARM64 binary"
else
    echo "⚠ ARM64 toolchain not installed. Skipping ARM64 build."
    echo "  To build for ARM64, run: rustup target add aarch64-unknown-linux-gnu"
fi

# Copy documentation
echo ""
echo "Copying documentation..."
cp README.md "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/"
cp docs/USER_GUIDE.md "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/docs/"
cp docs/raibid-cli.1 "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/docs/"
echo "✓ Documentation copied"

# Copy licenses
echo "Copying licenses..."
if [ -f LICENSE-MIT ]; then
    cp LICENSE-MIT "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/"
fi
if [ -f LICENSE-APACHE ]; then
    cp LICENSE-APACHE "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/"
fi
if [ -f LICENSE ]; then
    cp LICENSE "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/"
fi
echo "✓ Licenses copied"

# Copy example configurations
echo "Copying example configurations..."
if [ -d examples ]; then
    cp examples/*.yaml "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/examples/" 2>/dev/null || true
    cp examples/*.toml "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/examples/" 2>/dev/null || true
fi
echo "✓ Example configurations copied"

# Create installation script
echo ""
echo "Creating installation script..."
cat > "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/install.sh" << 'EOF'
#!/bin/bash
set -e

echo "Installing raibid-cli..."

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
    MAN_DIR="/usr/local/share/man/man1"
    DOC_DIR="/usr/local/share/doc/raibid-cli"
else
    INSTALL_DIR="$HOME/.local/bin"
    MAN_DIR="$HOME/.local/share/man/man1"
    DOC_DIR="$HOME/.local/share/doc/raibid-cli"
fi

# Create directories
mkdir -p "$INSTALL_DIR"
mkdir -p "$MAN_DIR"
mkdir -p "$DOC_DIR"

# Install binary
echo "Installing binary to $INSTALL_DIR..."
cp raibid-cli "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/raibid-cli"

# Install man page
echo "Installing man page to $MAN_DIR..."
cp docs/raibid-cli.1 "$MAN_DIR/"
if command -v mandb &> /dev/null; then
    mandb -q 2>/dev/null || true
fi

# Install documentation
echo "Installing documentation to $DOC_DIR..."
cp README.md "$DOC_DIR/"
cp docs/USER_GUIDE.md "$DOC_DIR/"

# Create config directory
CONFIG_DIR="$HOME/.config/raibid"
if [ ! -d "$CONFIG_DIR" ]; then
    echo "Creating config directory at $CONFIG_DIR..."
    mkdir -p "$CONFIG_DIR"
fi

echo ""
echo "✓ Installation complete!"
echo ""
echo "Binary installed to: $INSTALL_DIR/raibid-cli"
echo "Man page installed to: $MAN_DIR/raibid-cli.1"
echo "Documentation installed to: $DOC_DIR"
echo ""
echo "To get started:"
echo "  1. Run 'raibid-cli config init' to create configuration"
echo "  2. Run 'raibid-cli --help' to see available commands"
echo "  3. Run 'man raibid-cli' to view the manual"
echo "  4. Run 'raibid-cli tui' to launch the TUI dashboard"
echo ""

# Check if binary is in PATH
if ! command -v raibid-cli &> /dev/null; then
    echo "⚠ Warning: $INSTALL_DIR is not in your PATH"
    echo "  Add it to your shell profile:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
fi
EOF
chmod +x "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/install.sh"
echo "✓ Installation script created"

# Create README for release
echo ""
echo "Creating release README..."
cat > "$RELEASE_DIR/${PROJECT_NAME}-${VERSION}/INSTALL.md" << EOF
# ${PROJECT_NAME} v${VERSION}

## Installation

### Quick Install

\`\`\`bash
# Run the installation script
./install.sh
\`\`\`

This will install raibid-cli to your system. If you run as root (with sudo),
it will install to /usr/local/bin. Otherwise, it installs to ~/.local/bin.

### Manual Installation

\`\`\`bash
# Copy binary to a directory in your PATH
cp raibid-cli /usr/local/bin/
# or
cp raibid-cli ~/.local/bin/

# Make it executable
chmod +x /usr/local/bin/raibid-cli

# Install man page (optional)
sudo cp docs/raibid-cli.1 /usr/local/share/man/man1/
sudo mandb
\`\`\`

## Getting Started

1. **Initialize configuration:**
   \`\`\`bash
   raibid-cli config init
   \`\`\`

2. **View help:**
   \`\`\`bash
   raibid-cli --help
   \`\`\`

3. **Launch TUI dashboard:**
   \`\`\`bash
   raibid-cli tui
   \`\`\`

## Documentation

- **README.md** - Project overview and quick start
- **docs/USER_GUIDE.md** - Comprehensive user guide
- **docs/raibid-cli.1** - Man page (view with: man ./docs/raibid-cli.1)
- **examples/** - Example configuration files

## Support

- GitHub: https://github.com/raibid-labs/raibid-cli
- Issues: https://github.com/raibid-labs/raibid-cli/issues

## Version

${VERSION}
EOF
echo "✓ Release README created"

# Create tarballs
echo ""
echo "Creating release tarballs..."

cd "$RELEASE_DIR"

# x86_64 tarball
echo "Creating x86_64 tarball..."
tar czf "${PROJECT_NAME}-${VERSION}-x86_64-linux.tar.gz" "${PROJECT_NAME}-${VERSION}/"
echo "✓ Created ${PROJECT_NAME}-${VERSION}-x86_64-linux.tar.gz"

# ARM64 tarball (if built)
if [ -d "${PROJECT_NAME}-${VERSION}-aarch64" ]; then
    echo "Creating aarch64 tarball..."
    # Copy docs to ARM64 dir
    cp -r "${PROJECT_NAME}-${VERSION}/docs" "${PROJECT_NAME}-${VERSION}-aarch64/"
    cp "${PROJECT_NAME}-${VERSION}/README.md" "${PROJECT_NAME}-${VERSION}-aarch64/"
    cp "${PROJECT_NAME}-${VERSION}/INSTALL.md" "${PROJECT_NAME}-${VERSION}-aarch64/"
    cp "${PROJECT_NAME}-${VERSION}/install.sh" "${PROJECT_NAME}-${VERSION}-aarch64/"
    if [ -d "${PROJECT_NAME}-${VERSION}/examples" ]; then
        cp -r "${PROJECT_NAME}-${VERSION}/examples" "${PROJECT_NAME}-${VERSION}-aarch64/"
    fi
    if [ -f "${PROJECT_NAME}-${VERSION}/LICENSE-MIT" ]; then
        cp "${PROJECT_NAME}-${VERSION}/LICENSE-MIT" "${PROJECT_NAME}-${VERSION}-aarch64/"
    fi
    if [ -f "${PROJECT_NAME}-${VERSION}/LICENSE-APACHE" ]; then
        cp "${PROJECT_NAME}-${VERSION}/LICENSE-APACHE" "${PROJECT_NAME}-${VERSION}-aarch64/"
    fi

    tar czf "${PROJECT_NAME}-${VERSION}-aarch64-linux.tar.gz" "${PROJECT_NAME}-${VERSION}-aarch64/"
    echo "✓ Created ${PROJECT_NAME}-${VERSION}-aarch64-linux.tar.gz"
fi

cd ..

# Generate checksums
echo ""
echo "Generating checksums..."
cd "$RELEASE_DIR"
sha256sum *.tar.gz > SHA256SUMS
echo "✓ Checksums generated"
cd ..

# Display summary
echo ""
echo "========================================================"
echo "✓ Release build complete!"
echo ""
echo "Release artifacts created in: $RELEASE_DIR/"
echo ""
echo "Files:"
ls -lh "$RELEASE_DIR"/*.tar.gz
echo ""
echo "Checksums:"
cat "$RELEASE_DIR/SHA256SUMS"
echo ""
echo "To test installation:"
echo "  1. Extract tarball: tar xzf release/${PROJECT_NAME}-${VERSION}-x86_64-linux.tar.gz"
echo "  2. Run installer: cd ${PROJECT_NAME}-${VERSION} && ./install.sh"
echo "  3. Test binary: raibid-cli --version"
echo ""
echo "To create a GitHub release:"
echo "  1. Create a new tag: git tag v${VERSION}"
echo "  2. Push tag: git push origin v${VERSION}"
echo "  3. Upload tarballs and SHA256SUMS from release/ directory"
echo ""

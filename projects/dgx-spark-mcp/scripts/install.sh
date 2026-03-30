#!/bin/bash
set -euo pipefail

# DGX Spark MCP Server - Installation Script
# This script installs the DGX Spark MCP Server as a systemd service

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INSTALL_DIR="/opt/dgx-spark-mcp"
SERVICE_FILE="dgx-spark-mcp.service"
USER="dgx"
GROUP="dgx"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Check if Node.js is installed
check_nodejs() {
    if ! command -v node &> /dev/null; then
        log_error "Node.js is not installed. Please install Node.js 18+ first."
        exit 1
    fi

    local node_version=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
    if [[ $node_version -lt 18 ]]; then
        log_error "Node.js version 18+ is required. Current version: $(node -v)"
        exit 1
    fi

    log_info "Node.js version: $(node -v)"
}

# Check if npm is installed
check_npm() {
    if ! command -v npm &> /dev/null; then
        log_error "npm is not installed. Please install npm first."
        exit 1
    fi

    log_info "npm version: $(npm -v)"
}

# Create system user and group
create_user() {
    if ! id -u "$USER" &>/dev/null; then
        log_info "Creating user: $USER"
        useradd --system --no-create-home --shell /bin/false "$USER"
    else
        log_info "User $USER already exists"
    fi
}

# Create installation directory
create_install_dir() {
    log_info "Creating installation directory: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
    chown "$USER:$GROUP" "$INSTALL_DIR"
}

# Build the project
build_project() {
    log_info "Building project..."
    cd "$PROJECT_ROOT"

    if [[ ! -d "node_modules" ]]; then
        log_info "Installing dependencies..."
        npm ci --only=production
    fi

    if [[ ! -d "dist" ]]; then
        log_info "Building TypeScript..."
        npm run build
    fi
}

# Copy files to installation directory
install_files() {
    log_info "Installing files to $INSTALL_DIR..."

    # Copy build artifacts
    cp -r "$PROJECT_ROOT/dist" "$INSTALL_DIR/"
    cp -r "$PROJECT_ROOT/node_modules" "$INSTALL_DIR/"
    cp "$PROJECT_ROOT/package.json" "$INSTALL_DIR/"
    cp "$PROJECT_ROOT/package-lock.json" "$INSTALL_DIR/"

    # Copy configuration
    if [[ -d "$PROJECT_ROOT/config" ]]; then
        cp -r "$PROJECT_ROOT/config" "$INSTALL_DIR/"
    fi

    # Copy .env.example if .env doesn't exist
    if [[ ! -f "$INSTALL_DIR/.env" ]] && [[ -f "$PROJECT_ROOT/.env.example" ]]; then
        log_info "Creating .env from .env.example"
        cp "$PROJECT_ROOT/.env.example" "$INSTALL_DIR/.env"
    fi

    # Create log and data directories
    mkdir -p "$INSTALL_DIR/logs"
    mkdir -p "$INSTALL_DIR/data"

    # Set permissions
    chown -R "$USER:$GROUP" "$INSTALL_DIR"
    chmod 755 "$INSTALL_DIR"
    chmod -R 755 "$INSTALL_DIR/dist"
    chmod 600 "$INSTALL_DIR/.env" 2>/dev/null || true
}

# Install systemd service
install_service() {
    log_info "Installing systemd service..."

    # Copy service file
    cp "$PROJECT_ROOT/deploy/$SERVICE_FILE" "/etc/systemd/system/$SERVICE_FILE"

    # Reload systemd
    systemctl daemon-reload

    log_info "Systemd service installed"
}

# Enable and start service
enable_service() {
    log_info "Enabling service..."
    systemctl enable "$SERVICE_FILE"

    log_info "Starting service..."
    systemctl start "$SERVICE_FILE"

    # Wait a moment for service to start
    sleep 2

    # Check service status
    if systemctl is-active --quiet "$SERVICE_FILE"; then
        log_info "Service started successfully!"
        systemctl status "$SERVICE_FILE" --no-pager
    else
        log_error "Service failed to start. Check logs with: journalctl -u $SERVICE_FILE"
        exit 1
    fi
}

# Print post-installation instructions
print_instructions() {
    echo ""
    log_info "Installation complete!"
    echo ""
    echo "Useful commands:"
    echo "  - View status:  sudo systemctl status $SERVICE_FILE"
    echo "  - View logs:    sudo journalctl -u $SERVICE_FILE -f"
    echo "  - Stop service: sudo systemctl stop $SERVICE_FILE"
    echo "  - Restart:      sudo systemctl restart $SERVICE_FILE"
    echo ""
    echo "Configuration file: $INSTALL_DIR/.env"
    echo "Log files: $INSTALL_DIR/logs/"
    echo ""
}

# Main installation function
main() {
    log_info "Starting DGX Spark MCP Server installation..."

    check_root
    check_nodejs
    check_npm
    create_user
    create_install_dir
    build_project
    install_files
    install_service
    enable_service
    print_instructions
}

# Run main function
main "$@"

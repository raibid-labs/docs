#!/bin/bash
set -euo pipefail

# DGX Spark MCP Server - Update Script
# This script updates the DGX Spark MCP Server to the latest version

INSTALL_DIR="/opt/dgx-spark-mcp"
BACKUP_DIR="/opt/dgx-spark-mcp-backups"
SERVICE_FILE="dgx-spark-mcp.service"
USER="dgx"
GROUP="dgx"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Check if service is installed
check_installation() {
    if [[ ! -d "$INSTALL_DIR" ]]; then
        log_error "DGX Spark MCP Server is not installed at $INSTALL_DIR"
        exit 1
    fi

    if [[ ! -f "/etc/systemd/system/$SERVICE_FILE" ]]; then
        log_error "Systemd service not found. Please run install.sh first."
        exit 1
    fi
}

# Create backup
create_backup() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_path="$BACKUP_DIR/backup_$timestamp"

    log_info "Creating backup at $backup_path..."
    mkdir -p "$BACKUP_DIR"

    # Copy current installation
    cp -r "$INSTALL_DIR" "$backup_path"

    # Store current version if available
    if [[ -f "$INSTALL_DIR/package.json" ]]; then
        local current_version=$(grep -oP '(?<="version": ")[^"]*' "$INSTALL_DIR/package.json" || echo "unknown")
        echo "$current_version" > "$backup_path/.version"
        log_info "Current version: $current_version"
    fi

    # Clean old backups (keep last 5)
    local backup_count=$(ls -1 "$BACKUP_DIR" | wc -l)
    if [[ $backup_count -gt 5 ]]; then
        log_info "Cleaning old backups (keeping last 5)..."
        ls -1t "$BACKUP_DIR" | tail -n +6 | xargs -I {} rm -rf "$BACKUP_DIR/{}"
    fi

    echo "$backup_path"
}

# Stop service
stop_service() {
    log_info "Stopping service..."
    systemctl stop "$SERVICE_FILE" || true
}

# Pull latest changes
pull_updates() {
    log_info "Pulling latest changes..."

    # If this is a git repository, pull latest
    if [[ -d ".git" ]]; then
        git pull origin main
    else
        log_warn "Not a git repository. Skipping git pull."
    fi
}

# Install dependencies
install_dependencies() {
    log_info "Installing dependencies..."
    npm ci --only=production
}

# Build project
build_project() {
    log_info "Building project..."
    npm run build
}

# Update installation
update_installation() {
    log_info "Updating installation at $INSTALL_DIR..."

    # Copy updated files
    cp -r dist/* "$INSTALL_DIR/dist/"
    cp -r node_modules/* "$INSTALL_DIR/node_modules/"
    cp package.json "$INSTALL_DIR/"
    cp package-lock.json "$INSTALL_DIR/"

    # Update configuration files (preserve existing .env)
    if [[ -d "config" ]]; then
        cp -r config/* "$INSTALL_DIR/config/"
    fi

    # Set permissions
    chown -R "$USER:$GROUP" "$INSTALL_DIR"
}

# Update systemd service
update_service() {
    log_info "Updating systemd service..."

    if [[ -f "deploy/$SERVICE_FILE" ]]; then
        cp "deploy/$SERVICE_FILE" "/etc/systemd/system/$SERVICE_FILE"
        systemctl daemon-reload
    fi
}

# Start service
start_service() {
    log_info "Starting service..."
    systemctl start "$SERVICE_FILE"

    # Wait a moment for service to start
    sleep 2

    # Check service status
    if systemctl is-active --quiet "$SERVICE_FILE"; then
        log_info "Service started successfully!"
    else
        log_error "Service failed to start. Check logs with: journalctl -u $SERVICE_FILE"
        log_warn "You can rollback using: ./scripts/rollback.sh"
        exit 1
    fi
}

# Verify update
verify_update() {
    log_info "Verifying update..."

    # Check if service is running
    if systemctl is-active --quiet "$SERVICE_FILE"; then
        log_info "Service is running"
    else
        log_error "Service is not running!"
        return 1
    fi

    # Check logs for errors
    local error_count=$(journalctl -u "$SERVICE_FILE" --since "1 minute ago" | grep -i error | wc -l)
    if [[ $error_count -gt 0 ]]; then
        log_warn "Found $error_count errors in logs. Please review: journalctl -u $SERVICE_FILE"
    fi

    log_info "Update verification complete"
}

# Print update summary
print_summary() {
    local new_version=$(grep -oP '(?<="version": ")[^"]*' "$INSTALL_DIR/package.json" || echo "unknown")

    echo ""
    log_info "Update complete!"
    echo ""
    echo "New version: $new_version"
    echo ""
    echo "Useful commands:"
    echo "  - View status:  sudo systemctl status $SERVICE_FILE"
    echo "  - View logs:    sudo journalctl -u $SERVICE_FILE -f"
    echo "  - Rollback:     sudo ./scripts/rollback.sh"
    echo ""
}

# Main update function
main() {
    log_info "Starting DGX Spark MCP Server update..."

    check_root
    check_installation

    # Create backup and store path
    BACKUP_PATH=$(create_backup)
    log_info "Backup created: $BACKUP_PATH"

    stop_service
    pull_updates
    install_dependencies
    build_project
    update_installation
    update_service
    start_service

    if verify_update; then
        print_summary
    else
        log_error "Update verification failed!"
        log_warn "Consider rolling back: sudo ./scripts/rollback.sh"
        exit 1
    fi
}

# Run main function
main "$@"

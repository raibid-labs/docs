#!/bin/bash
set -euo pipefail

# DGX Spark MCP Server - Rollback Script
# This script rolls back the DGX Spark MCP Server to a previous backup

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

# Check if backups exist
check_backups() {
    if [[ ! -d "$BACKUP_DIR" ]] || [[ -z "$(ls -A "$BACKUP_DIR" 2>/dev/null)" ]]; then
        log_error "No backups found in $BACKUP_DIR"
        exit 1
    fi
}

# List available backups
list_backups() {
    echo ""
    log_info "Available backups:"
    echo ""

    local index=1
    for backup in $(ls -1t "$BACKUP_DIR"); do
        local backup_path="$BACKUP_DIR/$backup"
        local timestamp=$(echo "$backup" | sed 's/backup_//')
        local version="unknown"

        if [[ -f "$backup_path/.version" ]]; then
            version=$(cat "$backup_path/.version")
        fi

        echo "  $index) $timestamp (version: $version)"
        ((index++))
    done

    echo ""
}

# Select backup
select_backup() {
    local backup_count=$(ls -1 "$BACKUP_DIR" | wc -l)

    if [[ $# -eq 1 ]]; then
        # Backup number provided as argument
        local selection=$1
    else
        # Interactive selection
        read -p "Select backup number to restore (1-$backup_count, or 'latest'): " selection
    fi

    if [[ "$selection" == "latest" ]] || [[ "$selection" == "1" ]]; then
        # Get most recent backup
        SELECTED_BACKUP=$(ls -1t "$BACKUP_DIR" | head -n 1)
    elif [[ "$selection" =~ ^[0-9]+$ ]] && [[ $selection -ge 1 ]] && [[ $selection -le $backup_count ]]; then
        # Get backup by index
        SELECTED_BACKUP=$(ls -1t "$BACKUP_DIR" | sed -n "${selection}p")
    else
        log_error "Invalid selection"
        exit 1
    fi

    echo "$SELECTED_BACKUP"
}

# Confirm rollback
confirm_rollback() {
    local backup_name=$1
    local backup_path="$BACKUP_DIR/$backup_name"
    local version="unknown"

    if [[ -f "$backup_path/.version" ]]; then
        version=$(cat "$backup_path/.version")
    fi

    echo ""
    log_warn "You are about to rollback to:"
    echo "  Backup: $backup_name"
    echo "  Version: $version"
    echo ""

    if [[ "${AUTO_CONFIRM:-false}" != "true" ]]; then
        read -p "Continue with rollback? (yes/no): " confirm
        if [[ "$confirm" != "yes" ]]; then
            log_info "Rollback cancelled"
            exit 0
        fi
    fi
}

# Stop service
stop_service() {
    log_info "Stopping service..."
    systemctl stop "$SERVICE_FILE" || true
}

# Backup current state (before rollback)
backup_current() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local pre_rollback_backup="$BACKUP_DIR/pre_rollback_$timestamp"

    log_info "Creating pre-rollback backup..."
    cp -r "$INSTALL_DIR" "$pre_rollback_backup"
}

# Restore backup
restore_backup() {
    local backup_name=$1
    local backup_path="$BACKUP_DIR/$backup_name"

    log_info "Restoring backup: $backup_name..."

    # Remove current installation (except .env)
    if [[ -f "$INSTALL_DIR/.env" ]]; then
        cp "$INSTALL_DIR/.env" "/tmp/dgx-spark-mcp.env.backup"
    fi

    rm -rf "$INSTALL_DIR"/*

    # Restore from backup
    cp -r "$backup_path"/* "$INSTALL_DIR/"

    # Restore .env if it existed
    if [[ -f "/tmp/dgx-spark-mcp.env.backup" ]]; then
        cp "/tmp/dgx-spark-mcp.env.backup" "$INSTALL_DIR/.env"
        rm "/tmp/dgx-spark-mcp.env.backup"
    fi

    # Set permissions
    chown -R "$USER:$GROUP" "$INSTALL_DIR"
    chmod 755 "$INSTALL_DIR"
    chmod -R 755 "$INSTALL_DIR/dist"
    chmod 600 "$INSTALL_DIR/.env" 2>/dev/null || true
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
        exit 1
    fi
}

# Verify rollback
verify_rollback() {
    log_info "Verifying rollback..."

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

    log_info "Rollback verification complete"
}

# Print rollback summary
print_summary() {
    local version=$(grep -oP '(?<="version": ")[^"]*' "$INSTALL_DIR/package.json" || echo "unknown")

    echo ""
    log_info "Rollback complete!"
    echo ""
    echo "Restored version: $version"
    echo ""
    echo "Useful commands:"
    echo "  - View status:  sudo systemctl status $SERVICE_FILE"
    echo "  - View logs:    sudo journalctl -u $SERVICE_FILE -f"
    echo ""
}

# Main rollback function
main() {
    log_info "Starting DGX Spark MCP Server rollback..."

    check_root
    check_backups

    # List and select backup
    list_backups
    local backup_name=$(select_backup "$@")

    # Confirm rollback
    confirm_rollback "$backup_name"

    # Perform rollback
    stop_service
    backup_current
    restore_backup "$backup_name"
    start_service

    if verify_rollback; then
        print_summary
    else
        log_error "Rollback verification failed!"
        exit 1
    fi
}

# Run main function
main "$@"

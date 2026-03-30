#!/usr/bin/env bash
# Bump version across all workspace crates
# Usage: ./scripts/bump-version.sh X.Y.Z

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Version validation regex (semantic versioning)
VERSION_REGEX='^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+(\.[0-9]+)?)?$'

# Function to print colored messages
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to update version in Cargo.toml
update_crate_version() {
    local crate_toml="$1"
    local new_version="$2"

    if [[ ! -f "$crate_toml" ]]; then
        log_error "File not found: $crate_toml"
        return 1
    fi

    # Update version using sed
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$crate_toml"
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$crate_toml"
    fi

    log_info "Updated $(basename $(dirname "$crate_toml")) to $new_version"
}

# Function to update inter-crate dependencies
update_crate_dependency() {
    local crate_toml="$1"
    local dep_name="$2"
    local new_version="$3"

    if [[ ! -f "$crate_toml" ]]; then
        return 0
    fi

    # Update dependency version
    if grep -q "^$dep_name = {.*version = \".*\"" "$crate_toml"; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/^$dep_name = {.*version = \"[^\"]*\"/\0\"$new_version\"/" "$crate_toml"
        else
            sed -i "s/^$dep_name = {.*version = \"[^\"]*\"/\0\"$new_version\"/" "$crate_toml"
        fi
    fi
}

# Main script
main() {
    if [[ $# -ne 1 ]]; then
        log_error "Usage: $0 X.Y.Z"
        log_info "Example: $0 0.1.0"
        log_info "Example: $0 0.2.0-alpha.1"
        exit 1
    fi

    local new_version="$1"

    # Validate version format
    if ! [[ "$new_version" =~ $VERSION_REGEX ]]; then
        log_error "Invalid version format: $new_version"
        log_info "Version must follow semantic versioning: X.Y.Z or X.Y.Z-prerelease"
        exit 1
    fi

    log_info "Bumping version to $new_version"

    # Change to project root
    cd "$PROJECT_ROOT"

    # Check if git working directory is clean
    if [[ -n "$(git status --porcelain)" ]]; then
        log_warn "Git working directory is not clean"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Aborting version bump"
            exit 1
        fi
    fi

    # List of all workspace crates
    crates=(
        "crates/scarab-client"
        "crates/scarab-daemon"
        "crates/scarab-protocol"
        "crates/scarab-plugin-api"
        "crates/scarab-plugin-compiler"
        "crates/scarab-config"
        "crates/scarab-platform"
        "crates/scarab-nav"
        "crates/scarab-session"
    )

    # Update version in each crate
    log_info "Updating crate versions..."
    for crate in "${crates[@]}"; do
        if [[ -f "$crate/Cargo.toml" ]]; then
            update_crate_version "$crate/Cargo.toml" "$new_version"
        else
            log_warn "Crate not found: $crate"
        fi
    done

    # Update inter-crate dependencies
    log_info "Updating inter-crate dependencies..."
    local dep_names=(
        "scarab-protocol"
        "scarab-plugin-api"
        "scarab-config"
        "scarab-platform"
        "scarab-nav"
        "scarab-session"
    )

    for crate in "${crates[@]}"; do
        for dep in "${dep_names[@]}"; do
            update_crate_dependency "$crate/Cargo.toml" "$dep" "$new_version"
        done
    done

    # Update Cargo.lock
    log_info "Updating Cargo.lock..."
    cargo update --workspace

    # Verify the changes
    log_info "Verifying workspace..."
    if cargo check --workspace --quiet; then
        log_info "✓ Workspace check passed"
    else
        log_error "✗ Workspace check failed"
        log_error "Please review the changes and fix any issues"
        exit 1
    fi

    # Summary
    echo
    log_info "Version bump complete!"
    log_info "Changed files:"
    git status --short

    echo
    log_info "Next steps:"
    echo "  1. Review the changes: git diff"
    echo "  2. Update CHANGELOG.md"
    echo "  3. Commit: git commit -am 'chore: Bump version to $new_version'"
    echo "  4. Tag: git tag -a v$new_version -m 'Release v$new_version'"
    echo "  5. Push: git push origin main v$new_version"
}

main "$@"

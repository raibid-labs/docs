#!/bin/bash
# Release helper script for Scarab Navigation Protocol
# This script helps prepare a new release

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_error() {
    echo -e "${RED}Error: $1${NC}" >&2
}

print_success() {
    echo -e "${GREEN}$1${NC}"
}

print_warning() {
    echo -e "${YELLOW}$1${NC}"
}

# Check if version is provided
if [ -z "$1" ]; then
    print_error "Version number required"
    echo "Usage: $0 <version> [protocol|client|both]"
    echo "Example: $0 0.2.0 both"
    exit 1
fi

VERSION=$1
CRATE=${2:-both}

# Validate version format
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-z]+\.[0-9]+)?$ ]]; then
    print_error "Invalid version format: $VERSION"
    echo "Expected format: X.Y.Z or X.Y.Z-prerelease.N"
    exit 1
fi

# Check git status
if [[ -n $(git status -s) ]]; then
    print_warning "Working directory is not clean. Uncommitted changes detected:"
    git status -s
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Function to update Cargo.toml version
update_version() {
    local cargo_toml=$1
    local current_version=$(grep "^version" "$cargo_toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')

    echo "Updating $cargo_toml from $current_version to $VERSION"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" "$cargo_toml"
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$VERSION\"/" "$cargo_toml"
    fi
}

# Update versions based on selection
case $CRATE in
    protocol)
        print_success "Updating scarab-nav-protocol to v$VERSION"
        update_version "crates/scarab-nav-protocol/Cargo.toml"
        TAG="scarab-nav-protocol-v$VERSION"
        ;;
    client)
        print_success "Updating scarab-nav-client to v$VERSION"
        update_version "crates/scarab-nav-client/Cargo.toml"

        # Ask if protocol dependency should be updated
        read -p "Update scarab-nav-protocol dependency version? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            read -p "Enter protocol version: " PROTOCOL_VERSION
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/scarab-nav-protocol = { version = \".*\"/scarab-nav-protocol = { version = \"$PROTOCOL_VERSION\"/" "crates/scarab-nav-client/Cargo.toml"
            else
                sed -i "s/scarab-nav-protocol = { version = \".*\"/scarab-nav-protocol = { version = \"$PROTOCOL_VERSION\"/" "crates/scarab-nav-client/Cargo.toml"
            fi
        fi
        TAG="scarab-nav-client-v$VERSION"
        ;;
    both)
        print_success "Updating both crates to v$VERSION"
        update_version "crates/scarab-nav-protocol/Cargo.toml"
        update_version "crates/scarab-nav-client/Cargo.toml"

        # Update client dependency on protocol
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/scarab-nav-protocol = { version = \".*\"/scarab-nav-protocol = { version = \"$VERSION\"/" "crates/scarab-nav-client/Cargo.toml"
        else
            sed -i "s/scarab-nav-protocol = { version = \".*\"/scarab-nav-protocol = { version = \"$VERSION\"/" "crates/scarab-nav-client/Cargo.toml"
        fi
        TAG="v$VERSION"
        ;;
    *)
        print_error "Invalid crate selection: $CRATE"
        echo "Must be one of: protocol, client, both"
        exit 1
        ;;
esac

# Show changes
echo ""
print_warning "Changed files:"
git diff --name-only

echo ""
print_warning "Version changes:"
git diff crates/*/Cargo.toml

# Verify build
echo ""
print_success "Running tests..."
if cargo test --all-features --workspace; then
    print_success "Tests passed!"
else
    print_error "Tests failed! Fix tests before releasing."
    exit 1
fi

# Run clippy
echo ""
print_success "Running clippy..."
if cargo clippy --all-features --workspace -- -D warnings; then
    print_success "Clippy passed!"
else
    print_error "Clippy found issues! Fix them before releasing."
    exit 1
fi

# Check formatting
echo ""
print_success "Checking formatting..."
if cargo fmt --all -- --check; then
    print_success "Formatting is correct!"
else
    print_error "Code needs formatting! Run 'cargo fmt --all'"
    exit 1
fi

# Dry run publish
echo ""
print_success "Testing package..."
if [[ $CRATE == "protocol" || $CRATE == "both" ]]; then
    if (cd crates/scarab-nav-protocol && cargo publish --dry-run --allow-dirty); then
        print_success "Protocol package is ready!"
    else
        print_error "Protocol package failed verification!"
        exit 1
    fi
fi

# Summary
echo ""
print_success "==================================="
print_success "Release preparation complete!"
print_success "==================================="
echo ""
echo "Next steps:"
echo "1. Review the changes above"
echo "2. Commit the version bump:"
echo "   git add ."
echo "   git commit -m 'chore: bump version to $VERSION'"
echo "3. Push to main:"
echo "   git push origin main"
echo "4. Create and push the release tag:"
echo "   git tag $TAG"
echo "   git push origin $TAG"
echo ""
echo "The GitHub Actions workflow will then:"
echo "- Run all tests"
echo "- Publish to crates.io"
echo "- Create a GitHub Release"

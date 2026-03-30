#!/usr/bin/env bash
# Update Homebrew formula with release checksums

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }

# Check arguments
if [ $# -eq 0 ]; then
    log_error "Usage: $0 <version>"
    echo "Example: $0 v0.1.0-alpha.7"
    exit 1
fi

VERSION="$1"
REPO="raibid-labs/scarab"
FORMULA_FILE="packaging/homebrew/scarab.rb"

# Remove 'v' prefix if present
VERSION_NUM="${VERSION#v}"

log_info "Updating Homebrew formula for version $VERSION..."

# Construct download URLs
ARM64_URL="https://github.com/$REPO/releases/download/$VERSION/scarab-$VERSION-aarch64-apple-darwin.tar.gz"
X64_URL="https://github.com/$REPO/releases/download/$VERSION/scarab-$VERSION-x86_64-apple-darwin.tar.gz"

# Check if releases exist
log_info "Checking if release assets exist..."

if ! curl -sSfI "$ARM64_URL" > /dev/null 2>&1; then
    log_error "ARM64 release not found at: $ARM64_URL"
    exit 1
fi

if ! curl -sSfI "$X64_URL" > /dev/null 2>&1; then
    log_error "x86_64 release not found at: $X64_URL"
    exit 1
fi

log_success "Release assets found"

# Calculate SHA256 checksums
log_info "Calculating SHA256 for ARM64 (this may take a moment)..."
ARM64_SHA=$(curl -sSL "$ARM64_URL" | shasum -a 256 | cut -d' ' -f1)
log_success "ARM64 SHA256: $ARM64_SHA"

log_info "Calculating SHA256 for x86_64 (this may take a moment)..."
X64_SHA=$(curl -sSL "$X64_URL" | shasum -a 256 | cut -d' ' -f1)
log_success "x86_64 SHA256: $X64_SHA"

# Update formula file
log_info "Updating formula file..."

# Create backup
cp "$FORMULA_FILE" "$FORMULA_FILE.bak"

# Update version
sed -i.tmp "s/version \".*\"/version \"$VERSION_NUM\"/" "$FORMULA_FILE"

# Update ARM64 SHA
sed -i.tmp "/aarch64-apple-darwin/,/sha256/ s/sha256 \".*\"/sha256 \"$ARM64_SHA\"/" "$FORMULA_FILE"

# Update x86_64 SHA
sed -i.tmp "/x86_64-apple-darwin/,/sha256/ s/sha256 \".*\"/sha256 \"$X64_SHA\"/" "$FORMULA_FILE"

# Clean up temp files
rm -f "$FORMULA_FILE.tmp"

log_success "Formula updated successfully!"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Version:      $VERSION_NUM"
echo "  ARM64 SHA:    $ARM64_SHA"
echo "  x86_64 SHA:   $X64_SHA"
echo "  Formula:      $FORMULA_FILE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

log_info "Review changes:"
echo ""
git diff "$FORMULA_FILE"
echo ""

log_info "Next steps:"
echo "  1. Review the changes above"
echo "  2. Test the formula: brew audit --strict $FORMULA_FILE"
echo "  3. Commit: git add $FORMULA_FILE && git commit -m 'chore: Update Homebrew formula to $VERSION'"
echo "  4. Push to homebrew-scarab tap repository"
echo ""

# Restore backup if something went wrong
if ! grep -q "$ARM64_SHA" "$FORMULA_FILE" || ! grep -q "$X64_SHA" "$FORMULA_FILE"; then
    log_error "Formula update failed! Restoring backup..."
    mv "$FORMULA_FILE.bak" "$FORMULA_FILE"
    exit 1
fi

# Keep backup for safety
log_info "Backup saved at: $FORMULA_FILE.bak"

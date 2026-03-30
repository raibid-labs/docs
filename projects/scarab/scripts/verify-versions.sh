#!/bin/bash
# Script to verify version consistency across the Scarab workspace

set -e

echo "=== Scarab Version Verification ==="
echo ""

# Check VERSION file
echo "1. VERSION file:"
cat VERSION
echo ""

# Check workspace version in root Cargo.toml
echo "2. Workspace version in Cargo.toml:"
grep "^version = " Cargo.toml | head -1
echo ""

# Check all crate versions
echo "3. All crate versions:"
cargo metadata --format-version=1 --no-deps 2>/dev/null | \
  jq -r '.packages[] | select(.name | startswith("scarab-")) | "\(.name): \(.version)"' | \
  sort
echo ""

# Check for hardcoded versions
echo "4. Checking for hardcoded versions in workspace crates..."
hardcoded=$(grep -r "^version = \"" crates/*/Cargo.toml 2>/dev/null || true)
if [ -z "$hardcoded" ]; then
    echo "   ✓ All crates use workspace version inheritance"
else
    echo "   ✗ Found hardcoded versions:"
    echo "$hardcoded"
    exit 1
fi
echo ""

# Check workspace inheritance
echo "5. Checking workspace version inheritance..."
count=$(grep -r "^version.workspace = true" crates/*/Cargo.toml 2>/dev/null | wc -l)
expected=10  # Number of workspace member crates
if [ "$count" -eq "$expected" ]; then
    echo "   ✓ All $expected crates inherit workspace version"
else
    echo "   ✗ Only $count/$expected crates inherit workspace version"
    exit 1
fi
echo ""

echo "=== All version checks passed! ==="

#!/bin/bash
# Documentation validation script
# Checks that all required documentation files exist and have valid structure

set -e

DOCS_DIR="docs/versions/vNEXT"
REQUIRED_FILES=("INTRO.md" "INSTALL.md" "USAGE.md" "API.md" "CHANGELOG.md")
EXIT_CODE=0

echo "📚 Checking documentation structure..."

# Check if docs directory exists
if [ ! -d "$DOCS_DIR" ]; then
    echo "❌ Error: $DOCS_DIR directory not found"
    exit 1
fi

# Check required files
for file in "${REQUIRED_FILES[@]}"; do
    filepath="$DOCS_DIR/$file"
    if [ ! -f "$filepath" ]; then
        echo "❌ Missing required file: $filepath"
        EXIT_CODE=1
    else
        # Check if file is not empty
        if [ ! -s "$filepath" ]; then
            echo "⚠️  Warning: $filepath is empty"
            EXIT_CODE=1
        else
            echo "✅ Found: $filepath"
        fi
    fi
done

# Check for STRUCTURE.md
if [ ! -f "docs/STRUCTURE.md" ]; then
    echo "❌ Missing docs/STRUCTURE.md"
    EXIT_CODE=1
else
    echo "✅ Found: docs/STRUCTURE.md"
fi

# Check for broken internal links (basic check)
echo ""
echo "🔗 Checking for obviously broken internal links..."

find docs -name "*.md" -type f | while read -r file; do
    # Look for markdown links like [text](path.md)
    broken_links=$(grep -oP '\[.*?\]\(\K[^)]+(?=\))' "$file" 2>/dev/null | grep -v '^http' | grep -v '^#' || true)

    for link in $broken_links; do
        # Resolve relative path
        dir=$(dirname "$file")
        target="$dir/$link"

        if [ ! -f "$target" ]; then
            echo "⚠️  Potential broken link in $file: $link"
            # Don't fail on this, just warn
        fi
    done
done

# Summary
echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Documentation structure validation passed!"
else
    echo "❌ Documentation structure validation failed!"
fi

exit $EXIT_CODE

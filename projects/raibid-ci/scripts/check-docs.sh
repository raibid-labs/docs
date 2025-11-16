#!/usr/bin/env bash
set -euo pipefail

# Simple documentation validation script
DOCS_DIR="${DOCS_DIR:-./docs}"
ERRORS=0

echo "Checking documentation..."

# Find markdown files
MARKDOWN_FILES=$(find "$DOCS_DIR" -type f -name "*.md")
FILE_COUNT=$(echo "$MARKDOWN_FILES" | wc -l)
echo "Found $FILE_COUNT markdown files"

# Check for broken internal links
for file in $MARKDOWN_FILES; do
    LINKS=$(grep -oP '\]\(([^)]+)\)' "$file" | tr -d '][)(') || true
    for link in $LINKS; do
        # Skip external links
        if [[ "$link" =~ ^https?:// ]]; then
            continue
        fi
        # Skip anchors
        if [[ "$link" =~ ^# ]]; then
            continue
        fi
        # Check if file exists
        file_dir=$(dirname "$file")
        link_path="${link%%#*}"
        if [[ "$link_path" =~ ^/ ]]; then
            target=".$link_path"
        else
            target="$file_dir/$link_path"
        fi
        if [ ! -f "$target" ] && [ ! -d "$target" ]; then
            echo "ERROR: Broken link in $file: $link"
            ERRORS=$((ERRORS + 1))
        fi
    done
done

if [ $ERRORS -eq 0 ]; then
    echo "✓ Documentation validation passed!"
    exit 0
else
    echo "✗ Found $ERRORS broken links"
    exit 1
fi

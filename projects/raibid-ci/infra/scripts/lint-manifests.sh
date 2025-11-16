#!/usr/bin/env bash
# Lint Kubernetes manifests for best practices and style

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Linting Kubernetes manifests..."
echo "================================"
echo

# Check if yamllint is installed
if ! command -v yamllint &> /dev/null; then
    echo -e "${YELLOW}Warning: yamllint not installed. Install with: pip install yamllint${NC}"
    echo "Skipping YAML linting..."
else
    echo "Running yamllint..."
    if yamllint -c "$INFRA_DIR/.yamllint" "$INFRA_DIR" 2>&1; then
        echo -e "${GREEN}YAML linting passed!${NC}"
    else
        echo -e "${RED}YAML linting failed. See errors above.${NC}"
        exit 1
    fi
fi

echo
echo "================================"
echo "Checking manifest best practices..."
echo

# Best practices checks
ISSUES_FOUND=0

# Check for missing labels
echo "Checking for required labels..."
while IFS= read -r -d '' file; do
    # Skip Helm values files
    if [[ "$file" == *"values"*.yaml ]]; then
        continue
    fi

    # Check for app.kubernetes.io/name label
    if ! grep -q "app.kubernetes.io/name:" "$file" 2>/dev/null; then
        echo -e "${YELLOW}Warning${NC}: Missing app.kubernetes.io/name label in $file"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi

    # Check for app.kubernetes.io/part-of label
    if ! grep -q "app.kubernetes.io/part-of:" "$file" 2>/dev/null; then
        echo -e "${YELLOW}Warning${NC}: Missing app.kubernetes.io/part-of label in $file"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
done < <(find "$INFRA_DIR" -type f \( -name "*.yaml" -o -name "*.yml" \) -print0)

# Check for resource limits
echo
echo "Checking for resource limits..."
while IFS= read -r -d '' file; do
    # Skip non-workload files
    if ! grep -q "kind: \(Deployment\|StatefulSet\|DaemonSet\|Job\|CronJob\)" "$file" 2>/dev/null; then
        continue
    fi

    # Skip Helm values files
    if [[ "$file" == *"values"*.yaml ]]; then
        continue
    fi

    # Check for resource limits
    if ! grep -q "limits:" "$file" 2>/dev/null; then
        echo -e "${YELLOW}Warning${NC}: Missing resource limits in $file"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi

    # Check for resource requests
    if ! grep -q "requests:" "$file" 2>/dev/null; then
        echo -e "${YELLOW}Warning${NC}: Missing resource requests in $file"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
done < <(find "$INFRA_DIR" -type f \( -name "*.yaml" -o -name "*.yml" \) -print0)

echo
echo "================================"
echo "Linting Summary"
echo "================================"

if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}All checks passed! No issues found.${NC}"
    exit 0
else
    echo -e "${YELLOW}Found $ISSUES_FOUND potential issues (warnings only).${NC}"
    echo "These are recommendations, not failures."
    exit 0  # Exit 0 for warnings
fi

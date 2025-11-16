#!/usr/bin/env bash
# Validate Kubernetes manifests for syntax and schema correctness

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
TOTAL_FILES=0
VALID_FILES=0
INVALID_FILES=0

echo "Validating Kubernetes manifests..."
echo "=================================="
echo

# Function to validate YAML syntax
validate_yaml_syntax() {
    local file=$1
    if python3 -c "import yaml; yaml.safe_load(open('$file'))" 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

# Function to validate Kubernetes schema
validate_k8s_schema() {
    local file=$1

    # Skip Helm values files
    if [[ "$file" == *"values"*.yaml ]]; then
        echo -e "${YELLOW}SKIP${NC} (Helm values): $file"
        return 0
    fi

    # Skip k3s configuration files (not K8s manifests)
    if [[ "$file" == */k3s/config.yaml ]] || [[ "$file" == */k3s/rootless-config.yaml ]]; then
        echo -e "${YELLOW}SKIP${NC} (k3s config): $file"
        return 0
    fi

    # Skip Taskfile (not a K8s manifest)
    if [[ "$file" == */Taskfile.yml ]]; then
        echo -e "${YELLOW}SKIP${NC} (Taskfile): $file"
        return 0
    fi

    # Check if kubeval is available
    if command -v kubeval &> /dev/null; then
        if kubeval --strict "$file" &> /dev/null; then
            echo -e "${GREEN}PASS${NC}: $file"
            return 0
        else
            echo -e "${RED}FAIL${NC}: $file"
            kubeval --strict "$file" 2>&1 | sed 's/^/  /'
            return 1
        fi
    else
        # Fallback to kubectl dry-run if kubeval not available
        if kubectl apply --dry-run=client -f "$file" &> /dev/null; then
            echo -e "${GREEN}PASS${NC}: $file"
            return 0
        else
            echo -e "${RED}FAIL${NC}: $file"
            kubectl apply --dry-run=client -f "$file" 2>&1 | sed 's/^/  /'
            return 1
        fi
    fi
}

# Find all YAML files
while IFS= read -r -d '' file; do
    # Skip hidden files and directories
    if [[ "$file" == *"/.git/"* ]] || [[ "$file" == */node_modules/* ]]; then
        continue
    fi

    TOTAL_FILES=$((TOTAL_FILES + 1))

    # Validate YAML syntax first
    if ! validate_yaml_syntax "$file"; then
        echo -e "${RED}FAIL${NC} (Invalid YAML): $file"
        INVALID_FILES=$((INVALID_FILES + 1))
        continue
    fi

    # Validate Kubernetes schema
    if validate_k8s_schema "$file"; then
        VALID_FILES=$((VALID_FILES + 1))
    else
        INVALID_FILES=$((INVALID_FILES + 1))
    fi

done < <(find "$INFRA_DIR" -type f \( -name "*.yaml" -o -name "*.yml" \) -print0)

echo
echo "=================================="
echo "Validation Summary"
echo "=================================="
echo "Total files:   $TOTAL_FILES"
echo -e "Valid files:   ${GREEN}$VALID_FILES${NC}"
echo -e "Invalid files: ${RED}$INVALID_FILES${NC}"
echo

if [ $INVALID_FILES -eq 0 ]; then
    echo -e "${GREEN}All manifests are valid!${NC}"
    exit 0
else
    echo -e "${RED}Some manifests are invalid. Please fix the errors above.${NC}"
    exit 1
fi

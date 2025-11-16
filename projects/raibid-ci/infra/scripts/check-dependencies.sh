#!/usr/bin/env bash
# Check infrastructure component dependencies

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "Checking Infrastructure Dependencies"
echo "===================================="
echo

# Component dependency map
declare -A DEPENDENCIES=(
    ["k3s"]=""
    ["redis"]="k3s"
    ["gitea"]="k3s"
    ["keda"]="k3s redis"
    ["flux"]="k3s gitea"
)

# Deployment order
DEPLOYMENT_ORDER=("k3s" "redis" "gitea" "keda" "flux")

echo -e "${BLUE}Dependency Graph:${NC}"
echo "  k3s (base layer)"
echo "  ├── redis"
echo "  ├── gitea"
echo "  │   └── flux"
echo "  └── keda (requires redis)"
echo

# Function to check if component is installed
is_component_installed() {
    local component=$1

    case "$component" in
        k3s)
            systemctl is-active --quiet k3s 2>/dev/null || \
            systemctl is-active --quiet k3s-rootless 2>/dev/null
            ;;
        redis)
            kubectl get namespace raibid-redis &>/dev/null && \
            kubectl get pods -n raibid-redis -l app.kubernetes.io/name=redis --field-selector=status.phase=Running &>/dev/null
            ;;
        gitea)
            kubectl get namespace raibid-gitea &>/dev/null && \
            kubectl get pods -n raibid-gitea -l app.kubernetes.io/name=gitea --field-selector=status.phase=Running &>/dev/null
            ;;
        keda)
            kubectl get namespace keda &>/dev/null && \
            kubectl get pods -n keda -l app=keda-operator --field-selector=status.phase=Running &>/dev/null
            ;;
        flux)
            kubectl get namespace flux-system &>/dev/null && \
            kubectl get pods -n flux-system -l app=source-controller --field-selector=status.phase=Running &>/dev/null
            ;;
        *)
            return 1
            ;;
    esac
}

# Function to check dependencies for a component
check_dependencies() {
    local component=$1
    local deps="${DEPENDENCIES[$component]}"
    local all_satisfied=true

    if [ -z "$deps" ]; then
        echo -e "${GREEN}✓${NC} $component: No dependencies"
        return 0
    fi

    echo -e "${BLUE}?${NC} $component dependencies:"
    for dep in $deps; do
        if is_component_installed "$dep"; then
            echo -e "  ${GREEN}✓${NC} $dep (installed)"
        else
            echo -e "  ${RED}✗${NC} $dep (missing)"
            all_satisfied=false
        fi
    done

    if [ "$all_satisfied" = true ]; then
        return 0
    else
        return 1
    fi
}

# Check all components
echo -e "${BLUE}Checking installed components...${NC}"
echo

ALL_DEPS_SATISFIED=true

for component in "${DEPLOYMENT_ORDER[@]}"; do
    if ! check_dependencies "$component"; then
        ALL_DEPS_SATISFIED=false
    fi
    echo
done

echo "===================================="
echo "Deployment Order"
echo "===================================="
echo
echo "Components should be deployed in this order:"
for i in "${!DEPLOYMENT_ORDER[@]}"; do
    component="${DEPLOYMENT_ORDER[$i]}"
    echo "  $((i + 1)). $component"
done
echo

echo "===================================="
echo "Summary"
echo "===================================="
echo

if [ "$ALL_DEPS_SATISFIED" = true ]; then
    echo -e "${GREEN}✓ All dependencies satisfied!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Some dependencies are missing.${NC}"
    echo "Install missing components in the order shown above."
    exit 1
fi

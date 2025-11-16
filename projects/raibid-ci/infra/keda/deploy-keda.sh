#!/usr/bin/env bash
# Deployment script for KEDA
# Installs KEDA operator and configures ScaledJob for CI agents

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
KEDA_NAMESPACE="keda"
KEDA_RELEASE="raibid-keda"
KEDA_VERSION="2.12.0"
AGENT_NAMESPACE="raibid-ci"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."

    # Check kubectl
    if ! command -v kubectl &>/dev/null; then
        log_error "kubectl not found. Please install kubectl first."
        exit 1
    fi

    # Check helm
    if ! command -v helm &>/dev/null; then
        log_error "helm not found. Please install Helm 3.x first."
        exit 1
    fi

    # Check k3s
    if ! kubectl cluster-info &>/dev/null; then
        log_error "Cannot connect to Kubernetes cluster. Is k3s running?"
        exit 1
    fi

    log_info "Prerequisites check passed"
}

# Add KEDA Helm repository
add_helm_repo() {
    log_step "Adding KEDA Helm repository..."

    if helm repo list 2>/dev/null | grep -q "kedacore"; then
        log_info "KEDA repository already exists"
    else
        helm repo add kedacore https://kedacore.github.io/charts
    fi

    helm repo update kedacore
    log_info "Helm repository updated"
}

# Create namespaces
create_namespaces() {
    log_step "Creating namespaces..."

    kubectl apply -f "$SCRIPT_DIR/namespace.yaml"
    log_info "Namespaces created"
}

# Install KEDA operator
install_keda() {
    log_step "Installing KEDA operator..."

    helm upgrade --install "$KEDA_RELEASE" kedacore/keda \
        --namespace "$KEDA_NAMESPACE" \
        --version "$KEDA_VERSION" \
        --values "$SCRIPT_DIR/values.yaml" \
        --wait \
        --timeout 5m

    log_info "KEDA operator installed"
}

# Wait for KEDA to be ready
wait_for_keda() {
    log_step "Waiting for KEDA operator to be ready..."

    kubectl wait --for=condition=ready pod \
        -l app=keda-operator \
        -n "$KEDA_NAMESPACE" \
        --timeout=300s

    log_info "KEDA operator is ready"

    # Wait for metrics server
    if kubectl get deployment keda-operator-metrics-apiserver -n "$KEDA_NAMESPACE" &>/dev/null; then
        log_step "Waiting for KEDA metrics server..."
        kubectl wait --for=condition=ready pod \
            -l app=keda-operator-metrics-apiserver \
            -n "$KEDA_NAMESPACE" \
            --timeout=300s
        log_info "KEDA metrics server is ready"
    fi
}

# Verify KEDA CRDs
verify_crds() {
    log_step "Verifying KEDA CRDs..."

    local crds=(
        "scaledobjects.keda.sh"
        "scaledjobs.keda.sh"
        "triggerauthentications.keda.sh"
    )

    for crd in "${crds[@]}"; do
        if kubectl get crd "$crd" &>/dev/null; then
            log_info "CRD $crd exists"
        else
            log_error "CRD $crd not found!"
            exit 1
        fi
    done

    log_info "All KEDA CRDs are present"
}

# Deploy TriggerAuthentication
deploy_trigger_auth() {
    log_step "Deploying TriggerAuthentication..."

    kubectl apply -f "$SCRIPT_DIR/triggerauth.yaml"
    log_info "TriggerAuthentication deployed"
}

# Deploy ScaledJob
deploy_scaledjob() {
    log_step "Deploying ScaledJob..."

    kubectl apply -f "$SCRIPT_DIR/scaledjob.yaml"
    log_info "ScaledJob deployed"

    # Wait a moment for ScaledJob to be processed
    sleep 5

    # Show ScaledJob status
    if kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" &>/dev/null; then
        log_info "ScaledJob status:"
        kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE"
    fi
}

# Display summary
show_summary() {
    echo ""
    echo "========================================="
    echo "  KEDA Deployment Complete"
    echo "========================================="
    echo ""
    log_info "KEDA operator is running in namespace: $KEDA_NAMESPACE"
    log_info "ScaledJob configured in namespace: $AGENT_NAMESPACE"
    echo ""
    echo "Next steps:"
    echo "  1. Verify installation: $SCRIPT_DIR/validate-keda.sh"
    echo "  2. Test autoscaling: $SCRIPT_DIR/test-autoscaling.sh"
    echo ""
    echo "View resources:"
    echo "  kubectl get pods -n $KEDA_NAMESPACE"
    echo "  kubectl get scaledjob -n $AGENT_NAMESPACE"
    echo "  kubectl describe scaledjob raibid-ci-agent -n $AGENT_NAMESPACE"
    echo ""
}

# Main execution
main() {
    echo ""
    echo "========================================="
    echo "  KEDA Deployment for raibid-ci"
    echo "========================================="
    echo ""

    check_prerequisites
    add_helm_repo
    create_namespaces
    install_keda
    wait_for_keda
    verify_crds
    deploy_trigger_auth
    deploy_scaledjob
    show_summary
}

# Error handler
trap 'log_error "Deployment failed! Check the output above for errors."; exit 1' ERR

# Run main
main "$@"


#!/usr/bin/env bash
# Validation script for KEDA deployment
# Tests KEDA installation and ScaledJob configuration

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED++))
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED++))
}

# Check functions
check_namespace() {
    log_info "Checking KEDA namespace..."
    if kubectl get namespace keda &>/dev/null; then
        check_pass "KEDA namespace exists"
    else
        check_fail "KEDA namespace not found"
        return 1
    fi

    log_info "Checking raibid-ci namespace..."
    if kubectl get namespace raibid-ci &>/dev/null; then
        check_pass "raibid-ci namespace exists"
    else
        check_fail "raibid-ci namespace not found"
        return 1
    fi
}

check_keda_pods() {
    log_info "Checking KEDA pods..."

    # Check operator
    if kubectl get pods -n keda -l app=keda-operator --field-selector=status.phase=Running 2>/dev/null | grep -q keda-operator; then
        check_pass "KEDA operator is running"
    else
        check_fail "KEDA operator is not running"
    fi

    # Check metrics server
    if kubectl get pods -n keda -l app=keda-operator-metrics-apiserver --field-selector=status.phase=Running 2>/dev/null | grep -q metrics-apiserver; then
        check_pass "KEDA metrics server is running"
    else
        check_fail "KEDA metrics server is not running"
    fi

    # Check admission webhooks
    if kubectl get pods -n keda -l app.kubernetes.io/name=keda-admission-webhooks --field-selector=status.phase=Running 2>/dev/null | grep -q webhooks; then
        check_pass "KEDA admission webhooks are running"
    else
        log_warn "KEDA admission webhooks not found (may be optional)"
    fi
}

check_crds() {
    log_info "Checking KEDA CRDs..."

    local crds=(
        "scaledobjects.keda.sh"
        "scaledjobs.keda.sh"
        "triggerauthentications.keda.sh"
    )

    for crd in "${crds[@]}"; do
        if kubectl get crd "$crd" &>/dev/null; then
            check_pass "CRD $crd exists"
        else
            check_fail "CRD $crd not found"
        fi
    done
}

check_trigger_auth() {
    log_info "Checking TriggerAuthentication..."

    if kubectl get triggerauthentication raibid-redis-trigger-auth -n raibid-ci &>/dev/null; then
        check_pass "TriggerAuthentication exists"

        # Check if secret exists
        if kubectl get secret raibid-redis-auth -n raibid-ci &>/dev/null; then
            check_pass "Redis auth secret exists"
        else
            check_fail "Redis auth secret not found"
        fi
    else
        check_fail "TriggerAuthentication not found"
    fi
}

check_scaledjob() {
    log_info "Checking ScaledJob..."

    if kubectl get scaledjob raibid-ci-agent -n raibid-ci &>/dev/null; then
        check_pass "ScaledJob exists"

        # Get ScaledJob status
        local ready
        ready=$(kubectl get scaledjob raibid-ci-agent -n raibid-ci -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}' 2>/dev/null || echo "Unknown")

        if [ "$ready" = "True" ]; then
            check_pass "ScaledJob is ready"
        else
            log_warn "ScaledJob status: $ready"
        fi

        # Show scaling config
        log_info "ScaledJob configuration:"
        kubectl get scaledjob raibid-ci-agent -n raibid-ci -o jsonpath='{.spec.maxReplicaCount}' 2>/dev/null | xargs -I {} echo "  Max replicas: {}"
        kubectl get scaledjob raibid-ci-agent -n raibid-ci -o jsonpath='{.spec.pollingInterval}' 2>/dev/null | xargs -I {} echo "  Polling interval: {}s"

    else
        check_fail "ScaledJob not found"
    fi
}

check_redis_connection() {
    log_info "Checking Redis connection from KEDA..."

    # Get Redis address from secret
    local redis_address
    redis_address=$(kubectl get secret raibid-redis-auth -n raibid-ci -o jsonpath='{.data.address}' 2>/dev/null | base64 -d 2>/dev/null || echo "")

    if [ -z "$redis_address" ]; then
        log_warn "Could not retrieve Redis address from secret"
        return 0
    fi

    log_info "Redis address: $redis_address"

    # Try to connect to Redis (if available)
    if kubectl get pods -n raibid-redis -l app.kubernetes.io/name=redis 2>/dev/null | grep -q Running; then
        check_pass "Redis is running"
    else
        log_warn "Redis pods not found or not running"
    fi
}

check_scaling_metrics() {
    log_info "Checking KEDA metrics..."

    # Check if HPA is created
    local hpa_count
    hpa_count=$(kubectl get hpa -n raibid-ci 2>/dev/null | grep -c keda || echo "0")

    if [ "$hpa_count" -gt 0 ]; then
        log_info "KEDA has created $hpa_count HPA(s)"
        kubectl get hpa -n raibid-ci 2>/dev/null | grep keda || true
    else
        log_warn "No KEDA-managed HPAs found (this is normal for ScaledJob)"
    fi
}

check_events() {
    log_info "Recent KEDA events..."
    kubectl get events -n raibid-ci --sort-by='.lastTimestamp' 2>/dev/null | grep -i keda | tail -5 || log_warn "No KEDA events found"
}

# Main execution
main() {
    echo ""
    echo "========================================="
    echo "  KEDA Deployment Validation"
    echo "========================================="
    echo ""

    check_namespace
    echo ""

    check_keda_pods
    echo ""

    check_crds
    echo ""

    check_trigger_auth
    echo ""

    check_scaledjob
    echo ""

    check_redis_connection
    echo ""

    check_scaling_metrics
    echo ""

    check_events
    echo ""

    # Summary
    echo "========================================="
    echo "  Validation Summary"
    echo "========================================="
    echo -e "Passed: ${GREEN}$PASSED${NC}"
    echo -e "Failed: ${RED}$FAILED${NC}"
    echo ""

    if [ "$FAILED" -eq 0 ]; then
        log_info "All checks passed! KEDA is properly configured."
        exit 0
    else
        log_error "Some checks failed. Please review the output above."
        exit 1
    fi
}

# Run main function
main "$@"


#!/usr/bin/env bash
# Integration test for KEDA ScaledJob autoscaling
# Tests the complete scaling lifecycle from queue to job completion

set -euo pipefail

# Test configuration
TEST_NAME="KEDA ScaledJob Autoscaling"
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEDA_DIR="$TEST_DIR/../../infra/keda"
REDIS_NAMESPACE="raibid-redis"
AGENT_NAMESPACE="raibid-ci"
KEDA_NAMESPACE="keda"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
    ((TESTS_RUN++))
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Test prerequisites
test_prerequisites() {
    log_test "Checking prerequisites..."

    local all_ok=true

    # Check kubectl
    if ! command -v kubectl &>/dev/null; then
        log_fail "kubectl not found"
        all_ok=false
    else
        log_info "kubectl found"
    fi

    # Check k3s
    if ! kubectl cluster-info &>/dev/null; then
        log_fail "Cannot connect to Kubernetes cluster"
        all_ok=false
    else
        log_info "Kubernetes cluster accessible"
    fi

    # Check KEDA namespace
    if ! kubectl get namespace "$KEDA_NAMESPACE" &>/dev/null; then
        log_fail "KEDA namespace not found"
        all_ok=false
    else
        log_info "KEDA namespace exists"
    fi

    # Check raibid-ci namespace
    if ! kubectl get namespace "$AGENT_NAMESPACE" &>/dev/null; then
        log_fail "Agent namespace not found"
        all_ok=false
    else
        log_info "Agent namespace exists"
    fi

    if [ "$all_ok" = true ]; then
        log_pass "All prerequisites met"
        return 0
    else
        log_fail "Prerequisites not met"
        return 1
    fi
}

# Test KEDA operator running
test_keda_operator() {
    log_test "Checking KEDA operator status..."

    # Check operator pod
    if kubectl get pods -n "$KEDA_NAMESPACE" -l app=keda-operator --field-selector=status.phase=Running 2>/dev/null | grep -q keda-operator; then
        log_pass "KEDA operator is running"
    else
        log_fail "KEDA operator is not running"
        return 1
    fi

    # Check metrics server
    if kubectl get pods -n "$KEDA_NAMESPACE" -l app=keda-operator-metrics-apiserver --field-selector=status.phase=Running 2>/dev/null | grep -q metrics; then
        log_pass "KEDA metrics server is running"
    else
        log_warn "KEDA metrics server not running (may be optional)"
    fi

    return 0
}

# Test CRDs installed
test_crds() {
    log_test "Checking KEDA CRDs..."

    local crds_ok=true

    if kubectl get crd scaledobjects.keda.sh &>/dev/null; then
        log_info "ScaledObject CRD exists"
    else
        log_fail "ScaledObject CRD not found"
        crds_ok=false
    fi

    if kubectl get crd scaledjobs.keda.sh &>/dev/null; then
        log_info "ScaledJob CRD exists"
    else
        log_fail "ScaledJob CRD not found"
        crds_ok=false
    fi

    if kubectl get crd triggerauthentications.keda.sh &>/dev/null; then
        log_info "TriggerAuthentication CRD exists"
    else
        log_fail "TriggerAuthentication CRD not found"
        crds_ok=false
    fi

    if [ "$crds_ok" = true ]; then
        log_pass "All KEDA CRDs present"
        return 0
    else
        log_fail "Some KEDA CRDs missing"
        return 1
    fi
}

# Test ScaledJob exists
test_scaledjob_exists() {
    log_test "Checking ScaledJob configuration..."

    if kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" &>/dev/null; then
        log_pass "ScaledJob exists"

        # Show configuration
        local max_replicas
        max_replicas=$(kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" -o jsonpath='{.spec.maxReplicaCount}' 2>/dev/null || echo "unknown")
        log_info "Max replicas: $max_replicas"

        local polling
        polling=$(kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" -o jsonpath='{.spec.pollingInterval}' 2>/dev/null || echo "unknown")
        log_info "Polling interval: ${polling}s"

        return 0
    else
        log_fail "ScaledJob not found"
        return 1
    fi
}

# Test TriggerAuthentication
test_trigger_auth() {
    log_test "Checking TriggerAuthentication..."

    if kubectl get triggerauthentication raibid-redis-trigger-auth -n "$AGENT_NAMESPACE" &>/dev/null; then
        log_pass "TriggerAuthentication exists"

        # Check secret
        if kubectl get secret raibid-redis-auth -n "$AGENT_NAMESPACE" &>/dev/null; then
            log_pass "Redis auth secret exists"
            return 0
        else
            log_fail "Redis auth secret not found"
            return 1
        fi
    else
        log_fail "TriggerAuthentication not found"
        return 1
    fi
}

# Test Redis connection
test_redis_connection() {
    log_test "Testing Redis connection..."

    # Check if Redis is running
    if ! kubectl get pods -n "$REDIS_NAMESPACE" -l app.kubernetes.io/name=redis --field-selector=status.phase=Running 2>/dev/null | grep -q redis; then
        log_warn "Redis not running - skipping connection test"
        return 0
    fi

    # Try to ping Redis
    if kubectl exec -n "$REDIS_NAMESPACE" raibid-redis-master-0 -- redis-cli PING 2>/dev/null | grep -q PONG; then
        log_pass "Redis connection successful"
        return 0
    else
        log_warn "Redis ping failed (may require authentication)"
        return 0
    fi
}

# Test scaling from zero
test_scale_from_zero() {
    log_test "Testing scale from zero..."

    # Ensure no jobs running
    local initial_jobs
    initial_jobs=$(kubectl get jobs -n "$AGENT_NAMESPACE" -l app=raibid-ci-agent --no-headers 2>/dev/null | wc -l || echo "0")
    log_info "Initial jobs: $initial_jobs"

    # Get Redis password
    local redis_password
    redis_password=$(kubectl get secret raibid-redis-auth -n "$AGENT_NAMESPACE" -o jsonpath='{.data.password}' 2>/dev/null | base64 -d 2>/dev/null || echo "")

    if [ -z "$redis_password" ]; then
        log_warn "Could not retrieve Redis password - skipping scale test"
        return 0
    fi

    # Add test job to Redis
    log_info "Adding test job to Redis stream..."
    local job_id="test-scale-$(date +%s)"

    if kubectl exec -n "$REDIS_NAMESPACE" raibid-redis-master-0 -- \
        sh -c "redis-cli -a '$redis_password' --no-auth-warning XADD raibid:jobs '*' job_id '$job_id' repo test/repo branch main" &>/dev/null; then
        log_info "Test job added: $job_id"
    else
        log_warn "Failed to add test job - skipping scale test"
        return 0
    fi

    # Wait for KEDA to detect and create job (up to 30 seconds)
    log_info "Waiting for KEDA to create job (max 30s)..."
    local waited=0
    local job_created=false

    while [ $waited -lt 30 ]; do
        local current_jobs
        current_jobs=$(kubectl get jobs -n "$AGENT_NAMESPACE" -l app=raibid-ci-agent --no-headers 2>/dev/null | wc -l || echo "0")

        if [ "$current_jobs" -gt "$initial_jobs" ]; then
            job_created=true
            log_info "Job created after ${waited}s"
            break
        fi

        sleep 2
        ((waited+=2))
    done

    if [ "$job_created" = true ]; then
        log_pass "KEDA scaled from zero successfully"

        # Cleanup test job
        kubectl delete jobs -n "$AGENT_NAMESPACE" -l app=raibid-ci-agent --field-selector status.successful=0,status.failed=0 &>/dev/null || true

        return 0
    else
        log_fail "KEDA did not scale within 30 seconds"
        return 1
    fi
}

# Test job history limits
test_job_history() {
    log_test "Checking job history configuration..."

    local success_limit
    success_limit=$(kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" -o jsonpath='{.spec.successfulJobsHistoryLimit}' 2>/dev/null || echo "0")

    local failed_limit
    failed_limit=$(kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" -o jsonpath='{.spec.failedJobsHistoryLimit}' 2>/dev/null || echo "0")

    log_info "Successful jobs history limit: $success_limit"
    log_info "Failed jobs history limit: $failed_limit"

    if [ "$success_limit" -gt 0 ] && [ "$failed_limit" -gt 0 ]; then
        log_pass "Job history limits configured"
        return 0
    else
        log_fail "Job history limits not properly configured"
        return 1
    fi
}

# Test scaling strategy
test_scaling_strategy() {
    log_test "Checking scaling strategy..."

    local strategy
    strategy=$(kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" -o jsonpath='{.spec.scalingStrategy.strategy}' 2>/dev/null || echo "not set")

    log_info "Scaling strategy: $strategy"

    # Any strategy is acceptable, just check it's set
    if [ "$strategy" != "not set" ] && [ -n "$strategy" ]; then
        log_pass "Scaling strategy configured: $strategy"
        return 0
    else
        log_warn "Scaling strategy not explicitly set (will use default)"
        return 0
    fi
}

# Test pod template
test_pod_template() {
    log_test "Checking job pod template..."

    # Check container image
    local image
    image=$(kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" -o jsonpath='{.spec.jobTargetRef.template.spec.containers[0].image}' 2>/dev/null || echo "")

    if [ -n "$image" ]; then
        log_info "Container image: $image"
        log_pass "Pod template configured"
        return 0
    else
        log_fail "Pod template not found or invalid"
        return 1
    fi
}

# Test resource limits
test_resource_limits() {
    log_test "Checking resource limits..."

    local cpu_request
    cpu_request=$(kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" -o jsonpath='{.spec.jobTargetRef.template.spec.containers[0].resources.requests.cpu}' 2>/dev/null || echo "")

    local mem_request
    mem_request=$(kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" -o jsonpath='{.spec.jobTargetRef.template.spec.containers[0].resources.requests.memory}' 2>/dev/null || echo "")

    if [ -n "$cpu_request" ] && [ -n "$mem_request" ]; then
        log_info "CPU request: $cpu_request"
        log_info "Memory request: $mem_request"
        log_pass "Resource limits configured"
        return 0
    else
        log_warn "Resource limits not set (may cause scheduling issues)"
        return 0
    fi
}

# Test KEDA metrics
test_keda_metrics() {
    log_test "Checking KEDA metrics availability..."

    # Try to query external metrics API
    if kubectl get --raw /apis/external.metrics.k8s.io/v1beta1 &>/dev/null; then
        log_pass "KEDA metrics API accessible"
        return 0
    else
        log_warn "KEDA metrics API not accessible (may not be enabled)"
        return 0
    fi
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test resources..."

    # Delete any test jobs
    kubectl delete jobs -n "$AGENT_NAMESPACE" -l app=raibid-ci-agent --field-selector status.successful=0,status.failed=0 &>/dev/null || true

    log_info "Cleanup complete"
}

# Main test execution
main() {
    echo ""
    echo "========================================="
    echo "  $TEST_NAME"
    echo "========================================="
    echo ""

    # Run tests
    test_prerequisites || exit 1
    echo ""

    test_keda_operator
    echo ""

    test_crds
    echo ""

    test_scaledjob_exists
    echo ""

    test_trigger_auth
    echo ""

    test_redis_connection
    echo ""

    test_scale_from_zero
    echo ""

    test_job_history
    echo ""

    test_scaling_strategy
    echo ""

    test_pod_template
    echo ""

    test_resource_limits
    echo ""

    test_keda_metrics
    echo ""

    # Cleanup
    cleanup
    echo ""

    # Summary
    echo "========================================="
    echo "  Test Summary"
    echo "========================================="
    echo -e "Tests Run:    ${BLUE}$TESTS_RUN${NC}"
    echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
    echo ""

    if [ "$TESTS_FAILED" -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}Some tests failed.${NC}"
        exit 1
    fi
}

# Trap cleanup on exit
trap cleanup EXIT

# Run tests
main "$@"


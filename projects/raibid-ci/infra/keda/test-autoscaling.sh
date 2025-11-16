#!/usr/bin/env bash
# Test script for KEDA autoscaling behavior
# Demonstrates adding jobs to Redis and watching agents scale

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
REDIS_NAMESPACE="raibid-redis"
REDIS_SERVICE="raibid-redis-master"
REDIS_STREAM="raibid:jobs"
REDIS_CONSUMER_GROUP="raibid-workers"
AGENT_NAMESPACE="raibid-ci"
JOBS_TO_CREATE=${1:-5}

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."

    if ! kubectl get namespace "$REDIS_NAMESPACE" &>/dev/null; then
        echo "Error: Redis namespace not found. Please deploy Redis first."
        exit 1
    fi

    if ! kubectl get namespace "$AGENT_NAMESPACE" &>/dev/null; then
        echo "Error: raibid-ci namespace not found."
        exit 1
    fi

    if ! kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" &>/dev/null; then
        echo "Error: ScaledJob not found. Please deploy KEDA first."
        exit 1
    fi

    log_info "Prerequisites check passed"
}

# Get Redis password
get_redis_password() {
    log_step "Retrieving Redis password..."

    local password
    password=$(kubectl get secret raibid-redis-auth -n "$AGENT_NAMESPACE" -o jsonpath='{.data.password}' 2>/dev/null | base64 -d 2>/dev/null || echo "")

    if [ -z "$password" ]; then
        password=$(kubectl get secret raibid-redis -n "$REDIS_NAMESPACE" -o jsonpath='{.data.redis-password}' 2>/dev/null | base64 -d 2>/dev/null || echo "")
    fi

    if [ -z "$password" ]; then
        log_warn "Could not retrieve Redis password, trying without auth..."
        echo ""
    else
        log_info "Redis password retrieved"
        echo "$password"
    fi
}

# Create consumer group
create_consumer_group() {
    local password=$1

    log_step "Creating Redis consumer group..."

    local cmd="redis-cli"
    if [ -n "$password" ]; then
        cmd="redis-cli -a '$password' --no-auth-warning"
    fi

    # Try to create consumer group (ignore error if exists)
    kubectl exec -n "$REDIS_NAMESPACE" "$REDIS_SERVICE-0" -- \
        sh -c "$cmd XGROUP CREATE '$REDIS_STREAM' '$REDIS_CONSUMER_GROUP' 0 MKSTREAM" \
        2>&1 | grep -v "BUSYGROUP" || log_info "Consumer group already exists"
}

# Check initial state
check_initial_state() {
    log_step "Checking initial state..."

    local job_count
    job_count=$(kubectl get jobs -n "$AGENT_NAMESPACE" -l app=raibid-ci-agent --no-headers 2>/dev/null | wc -l || echo "0")

    log_info "Current agent jobs: $job_count"

    if [ "$job_count" -gt 0 ]; then
        log_warn "There are $job_count existing jobs. This test will add more jobs."
    fi
}

# Add test jobs to Redis
add_test_jobs() {
    local password=$1
    local count=$2

    log_step "Adding $count test jobs to Redis stream..."

    local cmd="redis-cli"
    if [ -n "$password" ]; then
        cmd="redis-cli -a '$password' --no-auth-warning"
    fi

    for i in $(seq 1 "$count"); do
        local job_id="test-$(date +%s)-$i"
        local repo="raibid-labs/test-repo"
        local branch="main"
        local commit="abc123def456"

        kubectl exec -n "$REDIS_NAMESPACE" "$REDIS_SERVICE-0" -- \
            sh -c "$cmd XADD '$REDIS_STREAM' '*' \
                job_id '$job_id' \
                repo '$repo' \
                branch '$branch' \
                commit '$commit' \
                timestamp '$(date -u +%Y-%m-%dT%H:%M:%SZ)'" \
            >/dev/null

        echo -n "."
    done

    echo ""
    log_info "$count jobs added to stream"
}

# Check stream length
check_stream() {
    local password=$1

    log_step "Checking Redis stream status..."

    local cmd="redis-cli"
    if [ -n "$password" ]; then
        cmd="redis-cli -a '$password' --no-auth-warning"
    fi

    local stream_len
    stream_len=$(kubectl exec -n "$REDIS_NAMESPACE" "$REDIS_SERVICE-0" -- \
        sh -c "$cmd XLEN '$REDIS_STREAM'" 2>/dev/null || echo "0")

    log_info "Stream length: $stream_len"

    # Show pending entries
    log_info "Pending entries in consumer group:"
    kubectl exec -n "$REDIS_NAMESPACE" "$REDIS_SERVICE-0" -- \
        sh -c "$cmd XPENDING '$REDIS_STREAM' '$REDIS_CONSUMER_GROUP'" 2>/dev/null || log_warn "Could not check pending entries"
}

# Watch scaling
watch_scaling() {
    log_step "Watching KEDA autoscaling (press Ctrl+C to stop)..."
    echo ""
    log_info "Monitor ScaledJob status:"
    echo ""

    # Watch jobs in a loop
    local max_iterations=60  # Watch for up to 5 minutes (60 * 5 seconds)
    local iteration=0

    while [ $iteration -lt $max_iterations ]; do
        clear
        echo "========================================="
        echo "  KEDA Autoscaling Test - Iteration $iteration"
        echo "========================================="
        echo ""

        # ScaledJob status
        echo "ScaledJob Status:"
        kubectl get scaledjob raibid-ci-agent -n "$AGENT_NAMESPACE" 2>/dev/null || echo "ScaledJob not found"
        echo ""

        # Jobs
        echo "Active Jobs:"
        kubectl get jobs -n "$AGENT_NAMESPACE" -l app=raibid-ci-agent --sort-by=.metadata.creationTimestamp 2>/dev/null || echo "No jobs found"
        echo ""

        # Pods
        echo "Agent Pods:"
        kubectl get pods -n "$AGENT_NAMESPACE" -l app=raibid-ci-agent --sort-by=.metadata.creationTimestamp 2>/dev/null || echo "No pods found"
        echo ""

        # Recent events
        echo "Recent Events:"
        kubectl get events -n "$AGENT_NAMESPACE" --sort-by='.lastTimestamp' 2>/dev/null | grep -i "keda\|scaledjob\|job" | tail -5 || echo "No events"
        echo ""

        echo "Press Ctrl+C to stop watching..."
        sleep 5
        ((iteration++))
    done
}

# Cleanup test jobs
cleanup() {
    log_step "Cleaning up test jobs..."

    # Delete completed/failed jobs
    kubectl delete jobs -n "$AGENT_NAMESPACE" -l app=raibid-ci-agent --field-selector status.successful=1 2>/dev/null || true
    kubectl delete jobs -n "$AGENT_NAMESPACE" -l app=raibid-ci-agent --field-selector status.failed=1 2>/dev/null || true

    log_info "Cleanup complete"
}

# Main execution
main() {
    echo ""
    echo "========================================="
    echo "  KEDA Autoscaling Test"
    echo "========================================="
    echo ""
    echo "This script will:"
    echo "  1. Add $JOBS_TO_CREATE test jobs to Redis"
    echo "  2. Watch KEDA scale up agent pods"
    echo "  3. Monitor job execution"
    echo ""
    echo "Press Enter to continue or Ctrl+C to cancel..."
    read -r

    check_prerequisites

    local redis_password
    redis_password=$(get_redis_password)

    create_consumer_group "$redis_password"

    check_initial_state

    check_stream "$redis_password"

    add_test_jobs "$redis_password" "$JOBS_TO_CREATE"

    check_stream "$redis_password"

    echo ""
    log_info "Jobs added! KEDA should detect them within 10 seconds..."
    echo ""
    log_info "Waiting 15 seconds for KEDA to react..."
    sleep 15

    watch_scaling

    echo ""
    log_info "Test complete!"
    echo ""
    echo "To cleanup test jobs, run:"
    echo "  kubectl delete jobs -n $AGENT_NAMESPACE -l app=raibid-ci-agent"
}

# Handle Ctrl+C gracefully
trap 'echo ""; log_info "Test interrupted by user"; exit 0' INT

# Run main
main "$@"


#!/usr/bin/env bash
# Validation script for k3s installation on DGX Spark
# Tests k3s cluster health, configuration, and readiness

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test
run_test() {
    local test_name=$1
    local test_command=$2

    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -n "Testing: ${test_name}... "

    if eval "$test_command" &>/dev/null; then
        echo -e "${GREEN}PASS${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}FAIL${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Function to run a test with output
run_test_with_output() {
    local test_name=$1
    local test_command=$2
    local expected_output=$3

    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -n "Testing: ${test_name}... "

    local output
    output=$(eval "$test_command" 2>&1)

    if echo "$output" | grep -q "$expected_output"; then
        echo -e "${GREEN}PASS${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}FAIL${NC}"
        echo "  Expected: $expected_output"
        echo "  Got: $output"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

echo "=================================="
echo "k3s Installation Validation"
echo "=================================="
echo

# Test 1: Check if k3s is installed
run_test "k3s binary exists" "test -f /usr/local/bin/k3s"

# Test 2: Check if k3s service is running (standard or rootless)
run_test "k3s service is active" \
    "systemctl is-active k3s || systemctl is-active --user k3s-rootless"

# Test 3: kubectl command works
run_test "kubectl command available" "command -v kubectl"

# Test 4: kubectl can communicate with cluster
run_test "kubectl cluster communication" "kubectl cluster-info"

# Test 5: Check node is Ready
run_test_with_output "Node is Ready" \
    "kubectl get nodes --no-headers" \
    "Ready"

# Test 6: Check node has correct labels
run_test_with_output "Node has raibid-ci label" \
    "kubectl get nodes --show-labels" \
    "raibid-ci=true"

# Test 7: Check required namespaces exist
echo
echo "Checking namespaces..."
for ns in kube-system raibid-ci raibid-infrastructure raibid-monitoring; do
    run_test "Namespace $ns exists" "kubectl get namespace $ns"
done

# Test 8: Check system pods are running
echo
echo "Checking system pods..."
run_test_with_output "CoreDNS is running" \
    "kubectl get pods -n kube-system -l k8s-app=kube-dns --no-headers" \
    "Running"

run_test_with_output "Metrics server is running" \
    "kubectl get pods -n kube-system -l k8s-app=metrics-server --no-headers" \
    "Running"

# Test 9: Check storage class exists
run_test "Local storage class exists" \
    "kubectl get storageclass local-path"

# Test 10: Test storage provisioning
echo
echo "Testing storage provisioning..."
TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo -n "Testing: PVC creation and binding... "

# Create test PVC
cat <<EOF | kubectl apply -f - &>/dev/null
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: k3s-validation-test-pvc
  namespace: default
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: local-path
  resources:
    requests:
      storage: 1Gi
EOF

# Wait for PVC to be bound (max 30 seconds)
if kubectl wait --for=jsonpath='{.status.phase}'=Bound pvc/k3s-validation-test-pvc -n default --timeout=30s &>/dev/null; then
    echo -e "${GREEN}PASS${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}FAIL${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# Cleanup test PVC
kubectl delete pvc k3s-validation-test-pvc -n default &>/dev/null || true

# Test 11: Check networking
echo
echo "Checking networking..."
run_test "CNI plugins exist" "test -d /var/lib/rancher/k3s/data/current/bin"

# Test 12: Check DNS resolution
TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo -n "Testing: DNS resolution... "

# Create test pod for DNS testing
cat <<EOF | kubectl apply -f - &>/dev/null
apiVersion: v1
kind: Pod
metadata:
  name: k3s-dns-test
  namespace: default
spec:
  containers:
  - name: test
    image: busybox:1.36
    command: ['sh', '-c', 'nslookup kubernetes.default && sleep 10']
  restartPolicy: Never
EOF

# Wait for pod to complete (max 30 seconds)
if kubectl wait --for=condition=Ready pod/k3s-dns-test -n default --timeout=30s &>/dev/null; then
    echo -e "${GREEN}PASS${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}FAIL${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# Cleanup DNS test pod
kubectl delete pod k3s-dns-test -n default &>/dev/null || true

# Test 13: Check kubeconfig permissions
run_test "kubeconfig is readable" \
    "test -r ~/.kube/config || test -r /etc/rancher/k3s/k3s.yaml"

# Test 14: Check cluster resource reservations
echo
echo "Checking resource configuration..."
run_test_with_output "Max pods configuration" \
    "kubectl get nodes -o jsonpath='{.items[0].status.capacity.pods}'" \
    "110"

# Test 15: Check secrets encryption
if grep -q "secrets-encryption: true" /etc/rancher/k3s/config.yaml 2>/dev/null || \
   grep -q "secrets-encryption: true" ~/.config/k3s/config.yaml 2>/dev/null; then
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "Testing: Secrets encryption enabled... ${GREEN}PASS${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
fi

# Test 16: Check k3s version (should be ARM64)
echo
echo "Checking platform..."
run_test_with_output "k3s is ARM64 binary" \
    "file /usr/local/bin/k3s" \
    "ARM aarch64"

# Summary
echo
echo "=================================="
echo "Validation Summary"
echo "=================================="
echo "Total tests:   $TOTAL_TESTS"
echo -e "Passed tests:  ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed tests:  ${RED}$FAILED_TESTS${NC}"
echo

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}All validation tests passed!${NC}"
    echo "k3s cluster is ready for use."
    exit 0
else
    echo -e "${RED}Some validation tests failed.${NC}"
    echo "Please check the errors above and fix the issues."
    exit 1
fi

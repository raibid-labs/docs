#!/bin/bash
set -e
NS="raibid-redis"
POD=$(kubectl get pods -n "$NS" -o name | head -1)
kubectl exec -n "$NS" "$POD" -- redis-cli PING
kubectl exec -n "$NS" "$POD" -- redis-cli XLEN raibid:jobs
echo "✓ Connection test passed"

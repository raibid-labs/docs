#!/bin/bash
set -e
NS="raibid-redis"
kubectl get ns "$NS" && echo "✓ Namespace OK"
POD=$(kubectl get pods -n "$NS" -l app.kubernetes.io/component=master -o name | head -1)
kubectl exec -n "$NS" "$POD" -- redis-cli PING && echo "✓ Connection OK"
echo "✓ Validation passed"

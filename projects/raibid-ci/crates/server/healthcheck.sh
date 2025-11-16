#!/bin/sh
# Healthcheck script for raibid-server
# Checks if the server is responding on the health endpoint

set -e

# Default values
HOST="${SERVER_HOST:-0.0.0.0}"
PORT="${SERVER_PORT:-8080}"
HEALTH_PATH="${HEALTH_PATH:-/health}"

# Use wget (available in debian-slim) to check health endpoint
# -q: quiet mode
# -O-: output to stdout
# --timeout: connection timeout in seconds
# Exit code 0 if HTTP 200, non-zero otherwise

wget -q -O- --timeout=5 "http://${HOST}:${PORT}${HEALTH_PATH}" > /dev/null 2>&1

exit $?

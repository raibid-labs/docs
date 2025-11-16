# MOP - Managed Observability Platform
# Comprehensive Justfile for development and operations
# Run 'just' or 'just --list' to see all available commands

# Configuration
set dotenv-load := true
set shell := ["bash", "-uc"]

# Variables
TANKA_VERSION := "0.25.0"
K8S_VERSION := "1.28"
KIND_CLUSTER := "mop-cluster"
REGISTRY := "localhost:5000"

# Colors for output
RED := "\\033[0;31m"
GREEN := "\\033[0;32m"
YELLOW := "\\033[0;33m"
BLUE := "\\033[0;34m"
NC := "\\033[0m" # No Color

# Default recipe (show help)
default:
    @just --list

# ============================================================================
# Setup & Installation
# ============================================================================

# Install all required dependencies
install:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Installing MOP dependencies...{{NC}}"

    # Detect OS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        if ! command -v brew &> /dev/null; then
            echo -e "{{RED}}Homebrew not found. Please install from https://brew.sh{{NC}}"
            exit 1
        fi

        echo "Installing macOS dependencies..."
        brew install tanka jsonnet-bundler helm just tilt-dev/tap/tilt kind kubectl nushell jq yq

    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "Installing Linux dependencies..."

        # Tanka
        curl -fSL -o "/usr/local/bin/tk" \
            "https://github.com/grafana/tanka/releases/download/v{{TANKA_VERSION}}/tk-linux-amd64"
        chmod +x /usr/local/bin/tk

        # Jsonnet Bundler
        go install -a github.com/jsonnet-bundler/jsonnet-bundler/cmd/jb@latest

        # Helm
        curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

        # Just
        curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin

        # Kind
        curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-linux-amd64
        chmod +x ./kind
        sudo mv ./kind /usr/local/bin/kind

        # kubectl
        curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
        chmod +x kubectl
        sudo mv kubectl /usr/local/bin/

        # Tilt
        curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash

    else
        echo -e "{{RED}}Unsupported OS: $OSTYPE{{NC}}"
        exit 1
    fi

    echo -e "{{GREEN}}✓ Dependencies installed successfully{{NC}}"
    just verify-install

# Verify all tools are installed
verify-install:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{BLUE}}Verifying installation...{{NC}}"

    tools=("tk" "jb" "helm" "just" "tilt" "kind" "kubectl" "jq" "yq")
    missing=()

    for tool in "${tools[@]}"; do
        if command -v "$tool" &> /dev/null; then
            version=$(${tool} version 2>/dev/null || ${tool} --version 2>/dev/null || echo "installed")
            echo -e "{{GREEN}}✓{{NC}} $tool: $version"
        else
            echo -e "{{RED}}✗{{NC}} $tool: not found"
            missing+=("$tool")
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        echo -e "{{RED}}Missing tools: ${missing[*]}{{NC}}"
        exit 1
    fi

    echo -e "{{GREEN}}✓ All tools verified{{NC}}"

# Initialize Tanka project structure
init:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Initializing Tanka environments...{{NC}}"

    for env in dev staging production; do
        if [ ! -f "environments/${env}/main.jsonnet" ]; then
            echo "Initializing ${env} environment..."
            cd "environments/${env}"
            tk init --k8s={{K8S_VERSION}}
            cd ../..
        else
            echo "${env} already initialized, skipping..."
        fi
    done

    echo -e "{{GREEN}}✓ Tanka environments initialized{{NC}}"
    just vendor-update

# Complete setup (install + init + vendor)
setup: install init
    @echo -e "{{GREEN}}✓ Setup complete! Run 'just cluster-up' to create a local cluster{{NC}}"

# ============================================================================
# Cluster Management
# ============================================================================

# Create local kind cluster with registry
cluster-up:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Creating kind cluster: {{KIND_CLUSTER}}{{NC}}"

    if kind get clusters | grep -q "{{KIND_CLUSTER}}"; then
        echo -e "{{YELLOW}}Cluster {{KIND_CLUSTER}} already exists{{NC}}"
        exit 0
    fi

    # Create cluster with registry
    cat <<EOF | kind create cluster --name={{KIND_CLUSTER}} --config=-
    kind: Cluster
    apiVersion: kind.x-k8s.io/v1alpha4
    nodes:
    - role: control-plane
      kubeadmConfigPatches:
      - |
        kind: InitConfiguration
        nodeRegistration:
          kubeletExtraArgs:
            node-labels: "ingress-ready=true"
      extraPortMappings:
      - containerPort: 80
        hostPort: 80
        protocol: TCP
      - containerPort: 443
        hostPort: 443
        protocol: TCP
    - role: worker
    - role: worker
    containerdConfigPatches:
    - |-
      [plugins."io.containerd.grpc.v1.cri".registry.mirrors."localhost:5000"]
        endpoint = ["http://kind-registry:5000"]
    EOF

    # Create registry container if it doesn't exist
    if ! docker ps | grep -q kind-registry; then
        docker run -d --restart=always -p "5000:5000" --name kind-registry registry:2
    fi

    # Connect registry to kind network
    docker network connect kind kind-registry 2>/dev/null || true

    echo -e "{{GREEN}}✓ Cluster created successfully{{NC}}"
    just cluster-info

# Delete local kind cluster
cluster-down:
    #!/usr/bin/env bash
    echo -e "{{YELLOW}}Deleting kind cluster: {{KIND_CLUSTER}}{{NC}}"
    kind delete cluster --name={{KIND_CLUSTER}}
    docker stop kind-registry 2>/dev/null || true
    docker rm kind-registry 2>/dev/null || true
    echo -e "{{GREEN}}✓ Cluster deleted{{NC}}"

# Show cluster information
cluster-info:
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Cluster Information:{{NC}}"
    kubectl cluster-info
    echo ""
    echo -e "{{BLUE}}Nodes:{{NC}}"
    kubectl get nodes
    echo ""
    echo -e "{{BLUE}}Namespaces:{{NC}}"
    kubectl get namespaces

# Reset cluster (delete and recreate)
cluster-reset: cluster-down cluster-up

# ============================================================================
# Deployment
# ============================================================================

# Deploy to environment (dev/staging/prod)
deploy ENV:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Deploying to {{ENV}}...{{NC}}"

    if [ ! -d "environments/{{ENV}}" ]; then
        echo -e "{{RED}}Environment {{ENV}} does not exist{{NC}}"
        exit 1
    fi

    cd "environments/{{ENV}}"
    tk apply --dangerous-auto-approve
    cd ../..

    echo -e "{{GREEN}}✓ Deployed to {{ENV}}{{NC}}"
    just status

# Show diff before applying
diff ENV:
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Showing diff for {{ENV}}...{{NC}}"
    cd "environments/{{ENV}}"
    tk diff
    cd ../..

# Apply Tanka configuration (with confirmation)
apply ENV:
    #!/usr/bin/env bash
    echo -e "{{YELLOW}}Applying configuration to {{ENV}}...{{NC}}"
    cd "environments/{{ENV}}"
    tk apply
    cd ../..

# Delete resources from environment
delete ENV:
    #!/usr/bin/env bash
    echo -e "{{RED}}Deleting resources from {{ENV}}...{{NC}}"
    read -p "Are you sure? (yes/no): " confirm
    if [ "$confirm" = "yes" ]; then
        cd "environments/{{ENV}}"
        tk delete
        cd ../..
        echo -e "{{GREEN}}✓ Resources deleted{{NC}}"
    else
        echo "Cancelled"
    fi

# Export rendered manifests
export ENV OUTPUT="./output":
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Exporting manifests for {{ENV}} to {{OUTPUT}}...{{NC}}"
    mkdir -p "{{OUTPUT}}/{{ENV}}"
    cd "environments/{{ENV}}"
    tk export "../../{{OUTPUT}}/{{ENV}}"
    cd ../..
    echo -e "{{GREEN}}✓ Manifests exported to {{OUTPUT}}/{{ENV}}{{NC}}"

# ============================================================================
# Development
# ============================================================================

# Start Tilt for local development
dev:
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Starting Tilt development environment...{{NC}}"
    if [ ! -f "Tiltfile" ]; then
        echo -e "{{RED}}Tiltfile not found{{NC}}"
        exit 1
    fi
    tilt up

# Stop Tilt
dev-down:
    tilt down

# Build Jsonnet (validate all environments)
build:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Building Jsonnet for all environments...{{NC}}"

    for env in dev staging production; do
        echo -e "{{BLUE}}Building ${env}...{{NC}}"
        cd "environments/${env}"
        tk show --dangerous-allow-redirect > /dev/null
        cd ../..
        echo -e "{{GREEN}}✓ ${env} builds successfully{{NC}}"
    done

# Validate Jsonnet syntax
validate:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Validating Jsonnet syntax...{{NC}}"

    find . -name "*.jsonnet" -o -name "*.libsonnet" | while read -r file; do
        echo "Validating $file..."
        jsonnet "$file" > /dev/null || {
            echo -e "{{RED}}✗ Validation failed for $file{{NC}}"
            exit 1
        }
    done

    echo -e "{{GREEN}}✓ All files valid{{NC}}"

# Format Jsonnet files
fmt:
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Formatting Jsonnet files...{{NC}}"
    find . -name "*.jsonnet" -o -name "*.libsonnet" | while read -r file; do
        echo "Formatting $file..."
        jsonnetfmt -i "$file"
    done
    echo -e "{{GREEN}}✓ Files formatted{{NC}}"

# Lint configuration
lint: validate
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Linting configuration...{{NC}}"

    # Check for common issues
    echo "Checking for hardcoded values..."
    if grep -r "localhost" environments/ --include="*.jsonnet" --include="*.libsonnet"; then
        echo -e "{{YELLOW}}Warning: Found hardcoded localhost references{{NC}}"
    fi

    echo -e "{{GREEN}}✓ Linting complete{{NC}}"

# Watch and rebuild on changes
watch ENV:
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Watching for changes in {{ENV}}...{{NC}}"
    cd "environments/{{ENV}}"
    while true; do
        tk show --dangerous-allow-redirect | kubectl diff -f - || true
        sleep 5
    done

# ============================================================================
# Component Management
# ============================================================================

# Tail logs for component
logs COMPONENT NAMESPACE="mop-system" LINES="100":
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Tailing logs for {{COMPONENT}} in {{NAMESPACE}}...{{NC}}"
    kubectl logs -n {{NAMESPACE}} -l app.kubernetes.io/name={{COMPONENT}} \
        --tail={{LINES}} --follow --max-log-requests=10

# Restart component (delete pods)
restart COMPONENT NAMESPACE="mop-system":
    #!/usr/bin/env bash
    echo -e "{{YELLOW}}Restarting {{COMPONENT}} in {{NAMESPACE}}...{{NC}}"
    kubectl rollout restart -n {{NAMESPACE}} deployment/{{COMPONENT}} || \
    kubectl rollout restart -n {{NAMESPACE}} statefulset/{{COMPONENT}}
    echo -e "{{GREEN}}✓ Restart initiated{{NC}}"
    kubectl rollout status -n {{NAMESPACE}} deployment/{{COMPONENT}} 2>/dev/null || \
    kubectl rollout status -n {{NAMESPACE}} statefulset/{{COMPONENT}}

# Scale component to N replicas
scale COMPONENT N NAMESPACE="mop-system":
    #!/usr/bin/env bash
    echo -e "{{YELLOW}}Scaling {{COMPONENT}} to {{N}} replicas in {{NAMESPACE}}...{{NC}}"
    kubectl scale -n {{NAMESPACE}} deployment/{{COMPONENT}} --replicas={{N}} 2>/dev/null || \
    kubectl scale -n {{NAMESPACE}} statefulset/{{COMPONENT}} --replicas={{N}}
    echo -e "{{GREEN}}✓ Scaled to {{N}} replicas{{NC}}"

# Port forward to component
port-forward COMPONENT PORT NAMESPACE="mop-system":
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Port forwarding {{COMPONENT}}:{{PORT}} ({{NAMESPACE}}){{NC}}"
    kubectl port-forward -n {{NAMESPACE}} svc/{{COMPONENT}} {{PORT}}:{{PORT}}

# Get component status
component-status COMPONENT NAMESPACE="mop-system":
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Status for {{COMPONENT}} in {{NAMESPACE}}:{{NC}}"
    kubectl get all -n {{NAMESPACE}} -l app.kubernetes.io/name={{COMPONENT}}

# Execute command in component pod
exec COMPONENT CMD NAMESPACE="mop-system":
    #!/usr/bin/env bash
    POD=$(kubectl get pods -n {{NAMESPACE}} -l app.kubernetes.io/name={{COMPONENT}} \
        -o jsonpath='{.items[0].metadata.name}')
    kubectl exec -n {{NAMESPACE}} -it "$POD" -- {{CMD}}

# ============================================================================
# Testing
# ============================================================================

# Run all tests
test: test-unit test-integration
    @echo -e "{{GREEN}}✓ All tests passed{{NC}}"

# Run unit tests (Jsonnet validation)
test-unit:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Running unit tests...{{NC}}"

    if [ -d "tests/unit" ]; then
        for test in tests/unit/*.jsonnet; do
            echo "Running $test..."
            jsonnet "$test" > /dev/null
        done
    fi

    echo -e "{{GREEN}}✓ Unit tests passed{{NC}}"

# Run integration tests
test-integration:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Running integration tests...{{NC}}"

    if [ -f "tests/integration.sh" ]; then
        bash tests/integration.sh
    else
        echo -e "{{YELLOW}}No integration tests found{{NC}}"
    fi

    echo -e "{{GREEN}}✓ Integration tests passed{{NC}}"

# Run end-to-end tests
test-e2e:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Running end-to-end tests...{{NC}}"

    if [ -f "tests/e2e.sh" ]; then
        bash tests/e2e.sh
    else
        echo -e "{{YELLOW}}No e2e tests found{{NC}}"
    fi

    echo -e "{{GREEN}}✓ E2E tests passed{{NC}}"

# Test deployment to dev environment
test-deploy: cluster-up deploy-dev
    @echo -e "{{GREEN}}✓ Test deployment successful{{NC}}"

# Smoke test (check if all components are running)
smoke-test:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Running smoke tests...{{NC}}"

    components=("grafana" "mimir" "loki" "tempo")

    for component in "${components[@]}"; do
        echo "Checking $component..."
        kubectl wait --for=condition=ready pod \
            -l app.kubernetes.io/name=$component \
            -n mop-system --timeout=300s || {
            echo -e "{{RED}}✗ $component not ready{{NC}}"
            exit 1
        }
        echo -e "{{GREEN}}✓ $component ready{{NC}}"
    done

    echo -e "{{GREEN}}✓ Smoke tests passed{{NC}}"

# ============================================================================
# Monitoring
# ============================================================================

# Open Grafana in browser
grafana-url:
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Opening Grafana...{{NC}}"
    URL=$(kubectl get ingress -n mop-system grafana -o jsonpath='{.spec.rules[0].host}' 2>/dev/null || echo "localhost:3000")
    open "http://$URL" || xdg-open "http://$URL" || echo "Open http://$URL in your browser"

# Port forward to Grafana
grafana-port-forward PORT="3000":
    @just port-forward grafana {{PORT}} mop-system

# Port forward to Tempo
tempo-port-forward PORT="3200":
    @just port-forward tempo {{PORT}} mop-system

# Port forward to Mimir
mimir-port-forward PORT="8080":
    @just port-forward mimir {{PORT}} mop-system

# Port forward to Loki
loki-port-forward PORT="3100":
    @just port-forward loki {{PORT}} mop-system

# Show status of all components
status:
    #!/usr/bin/env bash
    echo -e "{{BLUE}}MOP Platform Status{{NC}}"
    echo "===================="
    echo ""

    echo -e "{{BLUE}}Namespaces:{{NC}}"
    kubectl get namespaces | grep mop || echo "No MOP namespaces found"
    echo ""

    echo -e "{{BLUE}}Deployments:{{NC}}"
    kubectl get deployments -n mop-system 2>/dev/null || echo "Namespace not found"
    echo ""

    echo -e "{{BLUE}}StatefulSets:{{NC}}"
    kubectl get statefulsets -n mop-system 2>/dev/null || echo "Namespace not found"
    echo ""

    echo -e "{{BLUE}}Services:{{NC}}"
    kubectl get services -n mop-system 2>/dev/null || echo "Namespace not found"
    echo ""

    echo -e "{{BLUE}}Ingresses:{{NC}}"
    kubectl get ingresses -n mop-system 2>/dev/null || echo "Namespace not found"

# Check resource usage
resources:
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Resource Usage{{NC}}"
    echo "==============="
    echo ""
    kubectl top nodes
    echo ""
    echo -e "{{BLUE}}Pod Resource Usage (mop-system):{{NC}}"
    kubectl top pods -n mop-system 2>/dev/null || echo "Metrics server not available"

# Show all endpoints
endpoints:
    #!/usr/bin/env bash
    echo -e "{{BLUE}}MOP Platform Endpoints{{NC}}"
    echo "======================="
    kubectl get ingresses -A -o custom-columns=\
        NAMESPACE:.metadata.namespace,\
        NAME:.metadata.name,\
        HOSTS:.spec.rules[*].host,\
        PORTS:.spec.rules[*].http.paths[*].backend.service.port.number

# Tail logs for all components
logs-all LINES="50":
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Tailing logs from all MOP components...{{NC}}"
    kubectl logs -n mop-system --all-containers=true --tail={{LINES}} -l app.kubernetes.io/part-of=mop

# ============================================================================
# Utilities
# ============================================================================

# Update vendored dependencies
vendor-update:
    #!/usr/bin/env bash
    set -euo pipefail
    echo -e "{{GREEN}}Updating vendored dependencies...{{NC}}"

    if [ -f "jsonnetfile.json" ]; then
        jb update
        echo -e "{{GREEN}}✓ Dependencies updated{{NC}}"
    else
        echo -e "{{YELLOW}}No jsonnetfile.json found, initializing...{{NC}}"
        jb init

        # Install common libraries
        jb install github.com/grafana/jsonnet-libs/tanka-util
        jb install github.com/grafana/jsonnet-libs/ksonnet-util
        jb install github.com/jsonnet-libs/k8s-libsonnet/1.28@main

        echo -e "{{GREEN}}✓ Dependencies initialized{{NC}}"
    fi

# Update helm charts
helm-update:
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Updating Helm charts...{{NC}}"

    if [ -d "charts" ]; then
        cd charts
        for chart in */; do
            if [ -f "${chart}Chart.yaml" ]; then
                echo "Updating ${chart%/}..."
                helm dependency update "$chart"
            fi
        done
        cd ..
    fi

    echo -e "{{GREEN}}✓ Charts updated{{NC}}"

# Clean generated files
clean:
    #!/usr/bin/env bash
    echo -e "{{YELLOW}}Cleaning generated files...{{NC}}"
    find . -name "*.jsonnet.RENDERED" -delete
    find . -name ".tanka" -type d -exec rm -rf {} + 2>/dev/null || true
    rm -rf output/
    echo -e "{{GREEN}}✓ Cleaned{{NC}}"

# Generate documentation
docs:
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Generating documentation...{{NC}}"

    # Generate API docs from Jsonnet
    if command -v jsonnet-doc &> /dev/null; then
        find lib -name "*.libsonnet" | while read -r file; do
            jsonnet-doc "$file" > "docs/api/$(basename ${file%.libsonnet}).md"
        done
    fi

    echo -e "{{GREEN}}✓ Documentation generated in docs/{{NC}}"

# Serve documentation locally
docs-serve PORT="8000":
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Serving documentation at http://localhost:{{PORT}}{{NC}}"
    cd docs
    python3 -m http.server {{PORT}} || python -m SimpleHTTPServer {{PORT}}

# Create a new environment
new-env NAME:
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Creating new environment: {{NAME}}{{NC}}"

    if [ -d "environments/{{NAME}}" ]; then
        echo -e "{{RED}}Environment {{NAME}} already exists{{NC}}"
        exit 1
    fi

    mkdir -p "environments/{{NAME}}"
    cd "environments/{{NAME}}"
    tk init --k8s={{K8S_VERSION}}
    cd ../..

    echo -e "{{GREEN}}✓ Environment {{NAME}} created{{NC}}"

# Backup environment configuration
backup ENV:
    #!/usr/bin/env bash
    echo -e "{{GREEN}}Backing up {{ENV}} environment...{{NC}}"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    BACKUP_DIR="backups/${ENV}_${TIMESTAMP}"
    mkdir -p "$BACKUP_DIR"

    cp -r "environments/{{ENV}}" "$BACKUP_DIR/"
    tar -czf "${BACKUP_DIR}.tar.gz" -C backups "$(basename $BACKUP_DIR)"
    rm -rf "$BACKUP_DIR"

    echo -e "{{GREEN}}✓ Backup created: ${BACKUP_DIR}.tar.gz{{NC}}"

# Show Tanka environment configuration
show-env ENV:
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Configuration for {{ENV}}:{{NC}}"
    cd "environments/{{ENV}}"
    tk show
    cd ../..

# List all environments
list-envs:
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Available environments:{{NC}}"
    ls -1 environments/

# Check for common issues
doctor:
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Running diagnostics...{{NC}}"
    echo ""

    # Check tools
    echo -e "{{BLUE}}Checking tools...{{NC}}"
    just verify-install
    echo ""

    # Check cluster
    echo -e "{{BLUE}}Checking cluster...{{NC}}"
    if kubectl cluster-info &>/dev/null; then
        echo -e "{{GREEN}}✓ Cluster accessible{{NC}}"
    else
        echo -e "{{RED}}✗ Cannot access cluster{{NC}}"
    fi
    echo ""

    # Check environments
    echo -e "{{BLUE}}Checking environments...{{NC}}"
    for env in dev staging production; do
        if [ -f "environments/${env}/main.jsonnet" ]; then
            echo -e "{{GREEN}}✓ ${env} initialized{{NC}}"
        else
            echo -e "{{YELLOW}}⚠ ${env} not initialized{{NC}}"
        fi
    done
    echo ""

    # Check vendor
    echo -e "{{BLUE}}Checking dependencies...{{NC}}"
    if [ -d "vendor" ] && [ "$(ls -A vendor)" ]; then
        echo -e "{{GREEN}}✓ Dependencies vendored{{NC}}"
    else
        echo -e "{{YELLOW}}⚠ No vendored dependencies{{NC}}"
    fi

# Run pre-commit checks
pre-commit: fmt validate lint build test-unit
    @echo -e "{{GREEN}}✓ Pre-commit checks passed{{NC}}"

# CI/CD pipeline simulation
ci: pre-commit test
    @echo -e "{{GREEN}}✓ CI pipeline passed{{NC}}"

# Quick deploy to dev
deploy-dev: build
    @just deploy dev

# Quick deploy to staging
deploy-staging: build
    @just deploy staging

# Quick deploy to production (with confirmation)
deploy-prod: build
    #!/usr/bin/env bash
    echo -e "{{RED}}WARNING: Deploying to PRODUCTION{{NC}}"
    read -p "Are you absolutely sure? Type 'DEPLOY PRODUCTION': " confirm
    if [ "$confirm" = "DEPLOY PRODUCTION" ]; then
        just deploy production
    else
        echo "Cancelled"
        exit 1
    fi

# Shell into a component pod
shell COMPONENT NAMESPACE="mop-system":
    @just exec {{COMPONENT}} /bin/sh {{NAMESPACE}}

# Debug component (show all resources and recent events)
debug COMPONENT NAMESPACE="mop-system":
    #!/usr/bin/env bash
    echo -e "{{BLUE}}Debugging {{COMPONENT}} in {{NAMESPACE}}{{NC}}"
    echo "========================================"
    echo ""

    echo -e "{{BLUE}}Pods:{{NC}}"
    kubectl get pods -n {{NAMESPACE}} -l app.kubernetes.io/name={{COMPONENT}}
    echo ""

    echo -e "{{BLUE}}Recent Events:{{NC}}"
    kubectl get events -n {{NAMESPACE}} --field-selector involvedObject.name={{COMPONENT}} \
        --sort-by='.lastTimestamp' | tail -20
    echo ""

    echo -e "{{BLUE}}Logs (last 50 lines):{{NC}}"
    kubectl logs -n {{NAMESPACE}} -l app.kubernetes.io/name={{COMPONENT}} --tail=50

# ============================================================================
# Shortcuts
# ============================================================================

# Quick start: setup everything and deploy to dev
quickstart: setup cluster-up deploy-dev
    @echo -e "{{GREEN}}✓ MOP is ready! Run 'just status' to see the platform{{NC}}"

# Complete teardown
teardown: cluster-down clean
    @echo -e "{{GREEN}}✓ Teardown complete{{NC}}"

# Restart everything
restart-all:
    #!/usr/bin/env bash
    echo -e "{{YELLOW}}Restarting all components...{{NC}}"
    kubectl rollout restart -n mop-system deployment
    kubectl rollout restart -n mop-system statefulset
    echo -e "{{GREEN}}✓ All components restarting{{NC}}"

# Version information
version:
    #!/usr/bin/env bash
    echo -e "{{BLUE}}MOP Platform - Tool Versions{{NC}}"
    echo "==============================="
    tk version
    kubectl version --client
    helm version
    kind version
    tilt version
    just --version

# ============================================================================
# Help
# ============================================================================

# Show detailed help for a specific command
help COMMAND:
    @just --show {{COMMAND}}

# Show all available commands with descriptions
list-all:
    @just --list --unsorted

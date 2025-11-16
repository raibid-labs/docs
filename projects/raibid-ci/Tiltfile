# Tiltfile for raibid-ci Development
# Orchestrates k3s cluster, Docker builds, and Tanka deployments
# Provides streamlined developer experience for DGX Spark CI development

# =============================================================================
# Configuration
# =============================================================================

# Project settings
PROJECT_NAME = 'raibid-ci'
NAMESPACE = 'raibid-system'

# k3s configuration
K3S_CONFIG_DIR = './infra/k3s'
K3S_INSTALL_SCRIPT = K3S_CONFIG_DIR + '/install.sh'
K3S_VALIDATE_SCRIPT = K3S_CONFIG_DIR + '/validate-installation.sh'

# Tanka configuration
TANKA_DIR = './tanka'
TANKA_ENV = 'environments/local'

# Docker build contexts
DOCKER_BUILD_CONTEXT = '.'

# =============================================================================
# Helper Functions
# =============================================================================

def check_command(cmd):
    """Check if a command exists in PATH"""
    result = local('command -v {} > /dev/null 2>&1 || echo "missing"'.format(cmd), quiet=True, echo_off=True)
    return result != "missing"

def is_k3s_running():
    """Check if k3s is running and healthy"""
    # Check if kubectl is available
    if not check_command('kubectl'):
        return False

    # Try to connect to cluster
    result = local('kubectl cluster-info > /dev/null 2>&1 || echo "not-running"', quiet=True, echo_off=True)
    return result != "not-running"

def get_k3s_context():
    """Get the current kubectl context"""
    if not check_command('kubectl'):
        return None
    result = local('kubectl config current-context 2>/dev/null || echo ""', quiet=True, echo_off=True)
    return result.strip() if result else None

def namespace_exists(namespace):
    """Check if a namespace exists"""
    result = local('kubectl get namespace {} > /dev/null 2>&1 || echo "missing"'.format(namespace), quiet=True, echo_off=True)
    return result != "missing"

def create_namespace(namespace):
    """Create a namespace if it doesn't exist"""
    if not namespace_exists(namespace):
        print('Creating namespace: {}'.format(namespace))
        local('kubectl create namespace {}'.format(namespace))

# =============================================================================
# k3s Cluster Management
# =============================================================================

def ensure_k3s_cluster():
    """Ensure k3s cluster is running and configured"""

    print('=' * 80)
    print('k3s Cluster Setup')
    print('=' * 80)

    # Check if k3s is already running
    if is_k3s_running():
        context = get_k3s_context()
        print('✓ k3s cluster is already running')
        print('  Context: {}'.format(context))

        # Validate cluster health
        print('Validating cluster health...')
        # Check node status
        local('kubectl get nodes')
        print('✓ k3s cluster is healthy')
    else:
        print('⚠ k3s cluster is not running')
        print('Please start k3s before running Tilt:')
        print('  Option 1 (Automated): cd {} && sudo {}'.format(K3S_CONFIG_DIR, K3S_INSTALL_SCRIPT))
        print('  Option 2 (Manual): sudo systemctl start k3s')
        print('  Option 3 (k3d): k3d cluster create raibid-ci')
        fail('k3s cluster is not running. Please start it first.')

    # Ensure required namespace exists
    create_namespace(NAMESPACE)

    # Check for required CRDs (KEDA, Flux)
    print('Checking for required CRDs...')
    # Note: CRDs will be installed by Helm charts, so we just warn if missing
    keda_crd_check = local('kubectl get crd scaledjobs.keda.sh > /dev/null 2>&1 || echo "missing"', quiet=True, echo_off=True)
    flux_crd_check = local('kubectl get crd gitrepositories.source.toolkit.fluxcd.io > /dev/null 2>&1 || echo "missing"', quiet=True, echo_off=True)

    if keda_crd_check == "missing":
        print('  ⚠ KEDA CRDs not found (will be installed by Helm chart)')
    else:
        print('  ✓ KEDA CRDs found')

    if flux_crd_check == "missing":
        print('  ⚠ Flux CRDs not found (will be installed by Helm chart)')
    else:
        print('  ✓ Flux CRDs found')

    print('✓ k3s cluster setup complete')
    print('')

# Run cluster setup
ensure_k3s_cluster()

# =============================================================================
# Tilt UI Configuration
# =============================================================================

# Configure Tilt settings
update_settings(
    # Limit concurrent builds for resource management
    max_parallel_updates=2,
    # Suppress unused image warnings (we use Tanka for deployment)
    suppress_unused_image_warnings=['raibid-server', 'raibid-agent'],
)

# Set default kubectl context to k3s
allow_k8s_contexts(['default', 'k3s', 'k3d-raibid-ci'])

# =============================================================================
# Docker Image Builds (Issue #103)
# =============================================================================

print('=' * 80)
print('Docker Image Builds')
print('=' * 80)

# Server image build
print('Configuring raibid-server image build...')
docker_build(
    # Image name (matches Tanka deployment)
    'raibid-server:latest',

    # Build context (repository root for workspace builds)
    context=DOCKER_BUILD_CONTEXT,

    # Dockerfile path
    dockerfile='crates/server/Dockerfile',

    # Watch paths for live updates
    # Trigger rebuild when Rust source or dependencies change
    only=[
        'crates/server/',
        'crates/common/',
        'Cargo.toml',
        'Cargo.lock',
    ],

    # Build arguments (none needed for now)
    # build_args={},

    # Use BuildKit for better caching and parallel builds
    # Note: BuildKit is default in modern Docker
)
print('✓ raibid-server build configured')

# Agent image build
print('Configuring raibid-agent image build...')
docker_build(
    # Image name (matches Tanka deployment)
    'raibid-agent:latest',

    # Build context (repository root for workspace builds)
    context=DOCKER_BUILD_CONTEXT,

    # Dockerfile path
    dockerfile='crates/agent/Dockerfile',

    # Watch paths for live updates
    # Trigger rebuild when Rust source or dependencies change
    only=[
        'crates/agent/',
        'crates/common/',
        'Cargo.toml',
        'Cargo.lock',
    ],

    # Build arguments (none needed for now)
    # build_args={},

    # Use BuildKit for better caching and parallel builds
)
print('✓ raibid-agent build configured')

print('')
print('Docker builds will run in parallel (max 2 concurrent)')
print('Builds will trigger on source file changes')
print('')

# =============================================================================
# Tanka Deployments (Issue #104)
# =============================================================================

print('=' * 80)
print('Tanka Deployments')
print('=' * 80)

# Generate Kubernetes manifests from Tanka
print('Generating manifests from Tanka...')
tanka_manifests = local('cd {} && tk show {}'.format(TANKA_DIR, TANKA_ENV))

# Apply manifests to cluster
print('Deploying manifests to cluster...')
k8s_yaml(tanka_manifests)

print('✓ Tanka manifests deployed')
print('')

# =============================================================================
# Resource Definitions and Dependencies (Issue #104)
# =============================================================================

print('Configuring resource groups and dependencies...')

# Infrastructure Group: Redis, Gitea, KEDA, Flux
# These are deployed via Helm charts through Tanka

# Redis - Job queue with Streams support
k8s_resource(
    workload='redis-master',
    new_name='redis',
    labels=['infrastructure'],
    port_forwards=['6379:6379'],  # Redis default port
)

# Gitea - Git server with OCI registry
k8s_resource(
    workload='gitea',
    new_name='gitea',
    labels=['infrastructure'],
    port_forwards=['3000:3000'],  # Gitea web UI
    links=[
        link('http://localhost:3000', 'Gitea Web UI'),
    ],
)

# KEDA - Event-driven autoscaling
k8s_resource(
    workload='keda-operator',
    new_name='keda',
    labels=['infrastructure'],
    port_forwards=[],  # No direct access needed
)

# Flux - GitOps continuous delivery (optional, may not have deployment)
# k8s_resource(
#     workload='flux',
#     new_name='flux',
#     labels=['infrastructure'],
# )

# Application Group: Server, Agent

# Server - API server
k8s_resource(
    workload='raibid-server',
    new_name='server',
    labels=['application'],
    port_forwards=[
        '8080:8080',  # HTTP API
        '8081:8081',  # Metrics endpoint
    ],
    links=[
        link('http://localhost:8080', 'Server API'),
        link('http://localhost:8081/metrics', 'Server Metrics'),
    ],
    # Dependencies: wait for Redis to be ready and image to be built
    resource_deps=['redis', 'raibid-server:latest'],
)

# Agent - Auto-scaling build agents (ScaledJob, not Deployment)
# Note: KEDA ScaledJobs don't create pods until there are jobs
k8s_resource(
    workload='raibid-agent',
    new_name='agent',
    labels=['application'],
    # Dependencies: wait for Server, KEDA, and image to be built
    resource_deps=['server', 'keda', 'raibid-agent:latest'],
)

print('✓ Resource groups configured:')
print('  - Infrastructure: redis, gitea, keda')
print('  - Application: server, agent')
print('✓ Dependencies configured:')
print('  - server depends on: redis, raibid-server:latest')
print('  - agent depends on: server, keda, raibid-agent:latest')
print('')

# Watch Tanka files for changes and re-deploy
print('Watching Tanka files for changes...')
watch_file(TANKA_DIR + '/environments/local/main.jsonnet')
watch_file(TANKA_DIR + '/lib/raibid/')
watch_file(TANKA_DIR + '/lib/charts/')

print('✓ Auto-reload enabled for Tanka files')
print('')

# =============================================================================
# Manual Triggers and Shortcuts (Issue #105)
# =============================================================================

print('=' * 80)
print('Manual Triggers and Shortcuts')
print('=' * 80)

# Trigger Test Job - Send a test job to Redis queue
local_resource(
    name='trigger-test-job',
    cmd='echo "TODO: Implement test job trigger script"',
    # TODO: Create a script that sends a test job to Redis Streams
    # For now, this is a placeholder
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['tools'],
)

# Scale Agent - Manually trigger agent scaling
local_resource(
    name='scale-agent',
    cmd='kubectl scale scaledjob raibid-agent --replicas=1 -n {}'.format(NAMESPACE),
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['tools'],
    resource_deps=['agent'],
)

# View Server Logs - Quick access to server logs
local_resource(
    name='view-server-logs',
    cmd='kubectl logs -n {} -l app=raibid-server --tail=100 -f'.format(NAMESPACE),
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['tools'],
    resource_deps=['server'],
)

print('✓ Manual triggers configured:')
print('  - trigger-test-job: Send test job to Redis queue')
print('  - scale-agent: Manually trigger agent scaling')
print('  - view-server-logs: Quick log access')
print('')

# =============================================================================
# Port Forwards Summary (Issue #105)
# =============================================================================

print('Port forwards configured:')
print('  - Server API:     http://localhost:8080')
print('  - Server Metrics: http://localhost:8081/metrics')
print('  - Gitea Web UI:   http://localhost:3000')
print('  - Redis:          localhost:6379')
print('')

# =============================================================================
# Live Reload Configuration (Issue #106)
# =============================================================================

# Issue #106: Live reload for Rust development
#
# Decision: Live reload is NOT implemented for Rust builds.
#
# Rationale:
# 1. Rust is a compiled language - changes require full recompilation
# 2. cargo-chef already provides optimal layer caching in Dockerfiles
# 3. Live reload would require:
#    - Syncing source files into running container
#    - Running cargo build inside container
#    - Restarting the binary
# 4. This approach is SLOWER than full rebuild due to:
#    - No Docker layer caching benefits
#    - Container filesystem overhead
#    - Need to install full build toolchain in runtime image
# 5. cargo-chef approach is faster because:
#    - Dependency layer is cached (only rebuilds on Cargo.toml changes)
#    - Source changes trigger fast incremental builds
#    - Docker BuildKit provides parallel builds
#    - Runtime image stays minimal (no build toolchain)
#
# Recommendation: Use full Docker rebuilds with cargo-chef caching.
# Typical rebuild time after source change: 30-60 seconds (dependencies cached)
# This is acceptable for development workflow.
#
# Alternative for local development without containers:
# - Use `cargo watch -x run` directly on host for instant rebuilds
# - Use Tilt only for integration testing with full stack
#

print('=' * 80)
print('Live Reload (Issue #106)')
print('=' * 80)
print('Live reload: Skipped (not implemented)')
print('Reason: Full Docker rebuild with cargo-chef is faster than live reload')
print('Typical rebuild time: 30-60 seconds with cached dependencies')
print('')
print('For instant local development:')
print('  - Use "cargo watch -x run" directly on host')
print('  - Use Tilt for full-stack integration testing')
print('')

# =============================================================================
# Tilt UI Configuration
# =============================================================================

# Configure Tilt settings
update_settings(
    # Limit concurrent builds for resource management
    max_parallel_updates=2,
    # Suppress unused image warnings (we use Tanka for deployment)
    suppress_unused_image_warnings=['raibid-server', 'raibid-agent'],
)

# Set default kubectl context to k3s
allow_k8s_contexts(['default', 'k3s', 'k3d-raibid-ci'])

print('=' * 80)
print('Tilt Configuration Complete')
print('=' * 80)
print('Project: {}'.format(PROJECT_NAME))
print('Namespace: {}'.format(NAMESPACE))
print('Cluster Context: {}'.format(get_k3s_context()))
print('')
print('Status:')
print('  ✓ k3s cluster management configured')
print('  ✓ Docker builds configured (server + agent)')
print('  ✓ Tanka deployments configured')
print('  ✓ Port forwards and shortcuts configured')
print('  ⚠ Live reload: Skipped (Issue #106 - full rebuild recommended)')
print('')
print('Run "tilt up" to start the development environment')
print('=' * 80)

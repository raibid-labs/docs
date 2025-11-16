# DGX Music Tiltfile
# Development environment orchestration for Kubernetes deployment
# Note: MVP uses systemd deployment, but this sets up k8s infrastructure for Phase 2+

# =============================================================================
# Configuration
# =============================================================================

PROJECT_NAME = 'dgx-music'
NAMESPACE = 'dgx-music-system'

# Local registry for development (k3d auto-creates this)
REGISTRY_HOST = 'localhost:5000'

# Docker build context
DOCKER_BUILD_CONTEXT = '.'

# =============================================================================
# Helper Functions
# =============================================================================

def check_command(cmd):
    """Check if a command exists in PATH"""
    result = local('command -v {} > /dev/null 2>&1 || echo "missing"'.format(cmd), quiet=True, echo_off=True)
    return str(result).strip() != "missing"

def namespace_exists(namespace):
    """Check if a namespace exists"""
    result = local('kubectl get namespace {} > /dev/null 2>&1 || echo "missing"'.format(namespace), quiet=True, echo_off=True)
    return str(result).strip() != "missing"

def create_namespace(namespace):
    """Create a namespace if it doesn't exist"""
    if not namespace_exists(namespace):
        print('Creating namespace: {}'.format(namespace))
        local('kubectl create namespace {}'.format(namespace))

# =============================================================================
# Prerequisites Check
# =============================================================================

print('🎵 DGX Music Development Environment')
print('=' * 60)

# Check for required tools
required_tools = ['kubectl', 'docker']
missing_tools = []

for tool in required_tools:
    if not check_command(tool):
        missing_tools.append(tool)
        print('❌ {} not found'.format(tool))
    else:
        print('✅ {} found'.format(tool))

if missing_tools:
    fail('Missing required tools: {}. Please install them first.'.format(', '.join(missing_tools)))

# Check for optional tools
if check_command('k3d'):
    print('✅ k3d found (recommended for local development)')
else:
    print('⚠️  k3d not found (optional, but recommended)')

# =============================================================================
# Namespace Setup
# =============================================================================

print('\n📦 Setting up namespace...')
create_namespace(NAMESPACE)

# =============================================================================
# Docker Build (Future)
# =============================================================================

# MVP uses systemd deployment, but this prepares for Phase 2 k8s deployment

# Generation service Docker build
#docker_build(
#    ref='{}/dgx-music-generation'.format(REGISTRY_HOST),
#    context=DOCKER_BUILD_CONTEXT,
#    dockerfile='./docker/Dockerfile.generation',
#    live_update=[
#        sync('./services/generation', '/app/services/generation'),
#        run('pip install -r requirements.txt', trigger='requirements.txt'),
#    ]
#)

# =============================================================================
# Kubernetes Resources (Future)
# =============================================================================

# MVP: Comment out k8s deployment for now
# Will be enabled in Phase 2

#k8s_yaml([
#    './k8s/base/namespace.yaml',
#    './k8s/base/configmap.yaml',
#    './k8s/base/generation-deployment.yaml',
#    './k8s/base/generation-service.yaml',
#])

# =============================================================================
# Local Resource: Python Development Server
# =============================================================================

# For MVP development: Run the FastAPI server locally
local_resource(
    name='dgx-music-api',
    cmd='source venv/bin/activate && uvicorn services.generation.api:app --reload --port 8000',
    serve_cmd='source venv/bin/activate && uvicorn services.generation.api:app --reload --port 8000',
    serve_dir='.',
    readiness_probe=probe(
        http_get=http_get_action(port=8000, path='/health'),
        initial_delay_secs=5,
        timeout_secs=5,
    ),
    links=[
        link('http://localhost:8000/docs', 'API Docs'),
        link('http://localhost:8000/health', 'Health'),
    ],
    labels=['api'],
)

# =============================================================================
# Local Resource: Model Validation
# =============================================================================

local_resource(
    name='validate-gpu',
    cmd='just validate-gpu',
    auto_init=True,
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['setup'],
)

# =============================================================================
# Local Resource: Database Management
# =============================================================================

local_resource(
    name='database',
    cmd='just db-init',
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['database'],
)

# =============================================================================
# Development Workflow Info
# =============================================================================

print('\n' + '=' * 60)
print('🚀 DGX Music Development Environment Ready!')
print('=' * 60)
print('\nMVP Development (Current):')
print('  • API Server: http://localhost:8000')
print('  • API Docs: http://localhost:8000/docs')
print('  • Run: just serve')
print('  • Test: just test')
print('\nPhase 2 (Kubernetes):')
print('  • Uncomment k8s resources in Tiltfile')
print('  • Deploy with: tilt up')
print('\nUseful Commands:')
print('  • just validate-gpu  - Check DGX Spark GPU')
print('  • just test-model    - Test model inference')
print('  • just generate "prompt" - Generate music')
print('=' * 60)

# =============================================================================
# Development Guidelines
# =============================================================================

# For contributors: See docs/DEVELOPMENT.md for setup instructions
# For orchestrator pattern: See ORCHESTRATOR.md

# Note: This Tiltfile is designed for future k8s deployment
# MVP uses systemd deployment on DGX Spark (see scripts/bash/deploy-dgx.sh)

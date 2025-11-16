#!/usr/bin/env bash
# Build script for raibid-ci agent container image
# Optimized for ARM64 with BuildKit caching

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Configuration
IMAGE_NAME="${IMAGE_NAME:-raibid-ci-agent}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
REGISTRY="${REGISTRY:-localhost:5000}"
PLATFORM="${PLATFORM:-linux/arm64}"
CACHE_FROM="${CACHE_FROM:-}"
CACHE_TO="${CACHE_TO:-}"
BUILD_ARGS="${BUILD_ARGS:-}"
PUSH="${PUSH:-false}"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Check for required tools
check_requirements() {
    log_info "Checking requirements..."

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    # Check Docker BuildKit support
    if ! docker buildx version &> /dev/null; then
        log_error "Docker BuildKit (buildx) is not available"
        exit 1
    fi

    log_success "All requirements satisfied"
}

# Display build configuration
show_config() {
    log_info "Build Configuration:"
    echo "  Image Name:    $IMAGE_NAME"
    echo "  Image Tag:     $IMAGE_TAG"
    echo "  Registry:      $REGISTRY"
    echo "  Platform:      $PLATFORM"
    echo "  Push:          $PUSH"
    if [ -n "$CACHE_FROM" ]; then
        echo "  Cache From:    $CACHE_FROM"
    fi
    if [ -n "$CACHE_TO" ]; then
        echo "  Cache To:      $CACHE_TO"
    fi
    echo ""
}

# Build the Docker image
build_image() {
    log_info "Building image: $REGISTRY/$IMAGE_NAME:$IMAGE_TAG"

    # Prepare build arguments
    local build_cmd=(
        docker buildx build
        --platform "$PLATFORM"
        --tag "$REGISTRY/$IMAGE_NAME:$IMAGE_TAG"
        --file Dockerfile
    )

    # Add cache configuration
    if [ -n "$CACHE_FROM" ]; then
        build_cmd+=(--cache-from "type=registry,ref=$CACHE_FROM")
    fi

    if [ -n "$CACHE_TO" ]; then
        build_cmd+=(--cache-to "type=registry,ref=$CACHE_TO,mode=max")
    fi

    # Add local cache as fallback
    build_cmd+=(--cache-from "type=local,src=/tmp/buildkit-cache")
    build_cmd+=(--cache-to "type=local,dest=/tmp/buildkit-cache,mode=max")

    # Add build arguments
    if [ -n "$BUILD_ARGS" ]; then
        # shellcheck disable=SC2086
        build_cmd+=($BUILD_ARGS)
    fi

    # Add push flag if requested
    if [ "$PUSH" = "true" ]; then
        build_cmd+=(--push)
    else
        build_cmd+=(--load)
    fi

    # Add progress output
    build_cmd+=(--progress=plain)

    # Build context
    build_cmd+=(.)

    # Execute build
    log_info "Executing: ${build_cmd[*]}"
    "${build_cmd[@]}"

    log_success "Image built successfully"
}

# Verify the built image
verify_image() {
    if [ "$PUSH" = "true" ]; then
        log_info "Image pushed to registry, skipping local verification"
        return 0
    fi

    log_info "Verifying image..."

    # Check if image exists
    if ! docker image inspect "$REGISTRY/$IMAGE_NAME:$IMAGE_TAG" &> /dev/null; then
        log_error "Image not found after build"
        exit 1
    fi

    # Get image size
    local size
    size=$(docker image inspect "$REGISTRY/$IMAGE_NAME:$IMAGE_TAG" --format='{{.Size}}' | awk '{print $1/1024/1024}')
    log_info "Image size: ${size} MB"

    # Check size constraint (target < 1.5GB = 1536 MB)
    if (( $(echo "$size > 1536" | bc -l) )); then
        log_warn "Image size exceeds target of 1536 MB (1.5 GB)"
    else
        log_success "Image size within target constraint"
    fi

    # Run health check
    log_info "Running health check..."
    if docker run --rm --platform "$PLATFORM" "$REGISTRY/$IMAGE_NAME:$IMAGE_TAG" /usr/local/bin/healthcheck.sh; then
        log_success "Health check passed"
    else
        log_error "Health check failed"
        exit 1
    fi
}

# Display usage information
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Build raibid-ci agent container image

OPTIONS:
    -n, --name NAME         Image name (default: raibid-ci-agent)
    -t, --tag TAG           Image tag (default: latest)
    -r, --registry URL      Registry URL (default: localhost:5000)
    -p, --platform PLATFORM Target platform (default: linux/arm64)
    --cache-from REF        Cache source reference
    --cache-to REF          Cache destination reference
    --build-arg ARG         Additional build arguments
    --push                  Push to registry after build
    -h, --help              Show this help message

ENVIRONMENT VARIABLES:
    IMAGE_NAME              Override image name
    IMAGE_TAG               Override image tag
    REGISTRY                Override registry URL
    PLATFORM                Override target platform
    CACHE_FROM              Override cache source
    CACHE_TO                Override cache destination
    BUILD_ARGS              Additional build arguments
    PUSH                    Set to 'true' to push

EXAMPLES:
    # Basic build
    $0

    # Build and push to Gitea registry
    $0 --registry gitea.local:3000/raibid --push

    # Build with remote cache
    $0 --cache-from gitea.local:3000/raibid/cache:agent \\
       --cache-to gitea.local:3000/raibid/cache:agent

    # Build for AMD64 (testing)
    $0 --platform linux/amd64

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -n|--name)
                IMAGE_NAME="$2"
                shift 2
                ;;
            -t|--tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            -r|--registry)
                REGISTRY="$2"
                shift 2
                ;;
            -p|--platform)
                PLATFORM="$2"
                shift 2
                ;;
            --cache-from)
                CACHE_FROM="$2"
                shift 2
                ;;
            --cache-to)
                CACHE_TO="$2"
                shift 2
                ;;
            --build-arg)
                BUILD_ARGS="$BUILD_ARGS --build-arg $2"
                shift 2
                ;;
            --push)
                PUSH="true"
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Main execution
main() {
    log_info "raibid-ci Agent Container Build Script"
    echo ""

    parse_args "$@"
    check_requirements
    show_config
    build_image
    verify_image

    echo ""
    log_success "Build completed successfully!"
    echo ""
    log_info "Image: $REGISTRY/$IMAGE_NAME:$IMAGE_TAG"
    log_info "Platform: $PLATFORM"

    if [ "$PUSH" = "true" ]; then
        log_info "Status: Pushed to registry"
    else
        log_info "Status: Available locally"
        log_info "To push: docker push $REGISTRY/$IMAGE_NAME:$IMAGE_TAG"
    fi
}

# Run main function
main "$@"

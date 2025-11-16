#!/usr/bin/env bash
# k3s Installation Script for DGX Spark
# Installs and configures k3s with ARM64 optimizations

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
K3S_VERSION="${K3S_VERSION:-v1.28.4+k3s1}"  # Latest stable as of writing
K3S_INSTALL_DIR="${K3S_INSTALL_DIR:-/usr/local/bin}"
ROOTLESS_MODE="${ROOTLESS_MODE:-false}"
USER="${USER:-raibid-agent}"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running on ARM64
check_architecture() {
    log_info "Checking system architecture..."

    local arch
    arch=$(uname -m)

    if [[ "$arch" != "aarch64" && "$arch" != "arm64" ]]; then
        log_error "This script is designed for ARM64 architecture (DGX Spark)"
        log_error "Detected architecture: $arch"
        exit 1
    fi

    log_success "ARM64 architecture confirmed"
}

# Check system requirements
check_requirements() {
    log_info "Checking system requirements..."

    # Check available memory
    local mem_total
    mem_total=$(free -g | awk '/^Mem:/{print $2}')

    if [ "$mem_total" -lt 4 ]; then
        log_warning "System has less than 4GB RAM. Recommended: 4GB+"
    else
        log_success "Memory check passed: ${mem_total}GB"
    fi

    # Check available disk space
    local disk_avail
    disk_avail=$(df -BG /var/lib | awk 'NR==2 {print $4}' | sed 's/G//')

    if [ "$disk_avail" -lt 20 ]; then
        log_warning "Less than 20GB available in /var/lib. Recommended: 20GB+"
    else
        log_success "Disk space check passed: ${disk_avail}GB available"
    fi

    # Check if k3s is already installed
    if command -v k3s &> /dev/null; then
        log_warning "k3s is already installed. This script will reconfigure it."
        read -p "Continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Installation cancelled."
            exit 0
        fi
    fi
}

# Download k3s binary with checksum verification
download_k3s() {
    log_info "Downloading k3s ${K3S_VERSION} for ARM64..."

    local download_url="https://github.com/k3s-io/k3s/releases/download/${K3S_VERSION}/k3s-arm64"
    local checksum_url="https://github.com/k3s-io/k3s/releases/download/${K3S_VERSION}/sha256sum-arm64.txt"
    local temp_dir
    temp_dir=$(mktemp -d)

    # Download binary
    if ! curl -sfL "$download_url" -o "${temp_dir}/k3s"; then
        log_error "Failed to download k3s binary"
        rm -rf "$temp_dir"
        exit 1
    fi

    # Download checksum
    if ! curl -sfL "$checksum_url" -o "${temp_dir}/sha256sum.txt"; then
        log_error "Failed to download checksum"
        rm -rf "$temp_dir"
        exit 1
    fi

    # Verify checksum
    log_info "Verifying checksum..."
    local expected_checksum
    expected_checksum=$(grep "k3s-arm64" "${temp_dir}/sha256sum.txt" | awk '{print $1}')
    local actual_checksum
    actual_checksum=$(sha256sum "${temp_dir}/k3s" | awk '{print $1}')

    if [[ "$expected_checksum" != "$actual_checksum" ]]; then
        log_error "Checksum verification failed!"
        log_error "Expected: $expected_checksum"
        log_error "Got:      $actual_checksum"
        rm -rf "$temp_dir"
        exit 1
    fi

    log_success "Checksum verified"

    # Install binary
    log_info "Installing k3s binary to ${K3S_INSTALL_DIR}..."
    sudo install -o root -g root -m 0755 "${temp_dir}/k3s" "${K3S_INSTALL_DIR}/k3s"

    # Create symlinks
    sudo ln -sf "${K3S_INSTALL_DIR}/k3s" "${K3S_INSTALL_DIR}/kubectl"
    sudo ln -sf "${K3S_INSTALL_DIR}/k3s" "${K3S_INSTALL_DIR}/crictl"
    sudo ln -sf "${K3S_INSTALL_DIR}/k3s" "${K3S_INSTALL_DIR}/ctr"

    rm -rf "$temp_dir"
    log_success "k3s binary installed"
}

# Install k3s using official installer
install_k3s_standard() {
    log_info "Installing k3s in standard mode..."

    # Prepare configuration directory
    sudo mkdir -p /etc/rancher/k3s

    # Copy configuration files
    log_info "Copying configuration files..."
    sudo cp "${SCRIPT_DIR}/config.yaml" /etc/rancher/k3s/config.yaml

    # Copy registries configuration (template)
    sudo cp "${SCRIPT_DIR}/registries.yaml" /etc/rancher/k3s/registries.yaml

    # Install using official installer
    log_info "Running k3s installer..."
    curl -sfL https://get.k3s.io | sh -s - \
        --config=/etc/rancher/k3s/config.yaml

    # Wait for k3s to be ready
    log_info "Waiting for k3s to be ready..."
    local max_wait=120
    local waited=0

    while ! sudo k3s kubectl get nodes &>/dev/null; do
        if [ $waited -ge $max_wait ]; then
            log_error "k3s failed to start within ${max_wait} seconds"
            exit 1
        fi
        sleep 2
        waited=$((waited + 2))
    done

    log_success "k3s is ready"

    # Setup kubeconfig for current user
    setup_kubeconfig
}

# Install k3s in rootless mode
install_k3s_rootless() {
    log_info "Installing k3s in rootless mode for user: ${USER}..."

    # Check rootless prerequisites
    check_rootless_prerequisites

    # Prepare configuration directory
    mkdir -p ~/.config/k3s

    # Copy configuration files
    log_info "Copying rootless configuration..."
    cp "${SCRIPT_DIR}/rootless-config.yaml" ~/.config/k3s/config.yaml

    # Install using official installer in rootless mode
    log_info "Running k3s rootless installer..."
    curl -sfL https://get.k3s.io | sh -s - \
        --rootless \
        --config=~/.config/k3s/config.yaml

    # Wait for k3s to be ready
    log_info "Waiting for k3s to be ready..."
    local max_wait=120
    local waited=0

    while ! k3s kubectl get nodes &>/dev/null; do
        if [ $waited -ge $max_wait ]; then
            log_error "k3s failed to start within ${max_wait} seconds"
            exit 1
        fi
        sleep 2
        waited=$((waited + 2))
    done

    log_success "k3s rootless is ready"

    # Setup kubeconfig
    mkdir -p ~/.kube
    cp ~/.kube/k3s.yaml ~/.kube/config 2>/dev/null || true
}

# Check rootless prerequisites
check_rootless_prerequisites() {
    log_info "Checking rootless prerequisites..."

    # Check for required tools
    local required_tools=("slirp4netns" "fuse-overlayfs" "rootlesskit")
    local missing_tools=()

    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done

    if [ ${#missing_tools[@]} -gt 0 ]; then
        log_warning "Missing required tools for rootless mode: ${missing_tools[*]}"
        log_info "Installing missing tools..."

        # Install rootless tools
        sudo apt-get update
        sudo apt-get install -y slirp4netns fuse-overlayfs uidmap

        log_success "Rootless prerequisites installed"
    else
        log_success "Rootless prerequisites satisfied"
    fi

    # Check user namespaces
    if ! grep -q "^${USER}:" /etc/subuid; then
        log_warning "User subordinate UID mapping not found"
        log_info "Configuring subordinate UID/GID mapping..."

        sudo usermod --add-subuids 100000-165535 "$USER"
        sudo usermod --add-subgids 100000-165535 "$USER"

        log_success "Subordinate UID/GID mapping configured"
    fi
}

# Setup kubeconfig for current user
setup_kubeconfig() {
    log_info "Setting up kubeconfig..."

    mkdir -p ~/.kube

    if [ "$ROOTLESS_MODE" = "true" ]; then
        # Rootless kubeconfig is already in ~/.kube/config
        log_success "Kubeconfig configured for rootless mode"
    else
        # Copy from k3s default location
        sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
        sudo chown "$(id -u):$(id -g)" ~/.kube/config
        chmod 600 ~/.kube/config

        log_success "Kubeconfig copied to ~/.kube/config"
    fi

    # Test kubectl access
    if kubectl cluster-info &>/dev/null; then
        log_success "kubectl configured successfully"
    else
        log_warning "kubectl configuration may have issues"
    fi
}

# Apply namespace manifests
apply_namespaces() {
    log_info "Creating namespaces..."

    kubectl apply -f "${SCRIPT_DIR}/namespaces.yaml"

    log_success "Namespaces created"
}

# Apply storage configuration
apply_storage() {
    log_info "Configuring storage..."

    # Apply storage class (k3s comes with local-path, this ensures correct config)
    kubectl apply -f "${SCRIPT_DIR}/storageclass.yaml"

    log_success "Storage configured"
}

# Apply resource quotas
apply_resource_quotas() {
    log_info "Applying resource quotas and limits..."

    kubectl apply -f "${SCRIPT_DIR}/resource-quotas.yaml"

    log_success "Resource quotas applied"
}

# Apply CoreDNS customization
apply_coredns_custom() {
    log_info "Applying CoreDNS customization..."

    # Apply custom CoreDNS config
    kubectl apply -f "${SCRIPT_DIR}/coredns-custom.yaml"

    # Restart CoreDNS to pick up changes
    kubectl rollout restart deployment/coredns -n kube-system

    log_success "CoreDNS customization applied"
}

# Display cluster info
display_cluster_info() {
    echo
    echo "=================================="
    echo "k3s Installation Complete"
    echo "=================================="
    echo

    # Get node info
    echo "Cluster Information:"
    kubectl cluster-info
    echo

    echo "Node Status:"
    kubectl get nodes -o wide
    echo

    echo "Namespaces:"
    kubectl get namespaces
    echo

    echo "Storage Classes:"
    kubectl get storageclass
    echo

    log_success "k3s is ready for use!"
    echo
    echo "Next steps:"
    echo "  1. Deploy infrastructure services (Gitea, Redis)"
    echo "  2. Deploy KEDA for autoscaling"
    echo "  3. Configure Flux for GitOps"
    echo
    echo "Run 'kubectl get all -A' to see all resources"
}

# Main installation flow
main() {
    echo "=================================="
    echo "k3s Installation for DGX Spark"
    echo "=================================="
    echo

    # Check if rootless mode is requested
    if [[ "${1:-}" == "--rootless" ]]; then
        ROOTLESS_MODE=true
        log_info "Rootless mode enabled"
    fi

    # Pre-flight checks
    check_architecture
    check_requirements

    # Install k3s
    if [ "$ROOTLESS_MODE" = "true" ]; then
        install_k3s_rootless
    else
        # Standard mode requires download for checksum verification
        download_k3s
        install_k3s_standard
    fi

    # Wait a bit for cluster to stabilize
    sleep 5

    # Apply manifests
    apply_namespaces
    apply_storage
    apply_resource_quotas
    apply_coredns_custom

    # Display results
    display_cluster_info
}

# Run main function
main "$@"

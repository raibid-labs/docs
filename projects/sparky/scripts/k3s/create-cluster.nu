#!/usr/bin/env nu
# Create k3d cluster for Sparky local development

def main [] {
    print "🚀 Creating k3d cluster for Sparky..."

    # Check if k3d is installed
    if (which k3d | is-empty) {
        print "❌ k3d not found. Installing..."
        install_k3d
    }

    # Check if cluster already exists
    let existing = (k3d cluster list | grep sparky-local)
    if not ($existing | is-empty) {
        print "⚠️  Cluster 'sparky-local' already exists"
        let response = (input "Delete and recreate? (y/N): ")
        if $response == "y" {
            print "🗑️  Deleting existing cluster..."
            k3d cluster delete sparky-local
        } else {
            print "✅ Using existing cluster"
            return
        }
    }

    # Create cluster with GPU support and port mappings
    print "🏗️  Creating cluster with GPU support..."
    k3d cluster create sparky-local `
        --agents 2 `
        --port "8080:80@loadbalancer" `
        --port "8443:443@loadbalancer" `
        --gpus all `
        --volume "/tmp/k3d-sparky:/tmp/k3d@all" `
        --k3s-arg "--disable=traefik@server:0"

    # Wait for cluster to be ready
    print "⏳ Waiting for cluster to be ready..."
    sleep 5sec

    # Create namespace
    print "📦 Creating namespace 'sparky'..."
    kubectl create namespace sparky

    # Label namespace for GPU access
    kubectl label namespace sparky gpu=enabled

    # Verify cluster
    print "✅ Cluster created successfully!"
    print "\n📊 Cluster info:"
    kubectl cluster-info

    print "\n🔍 Nodes:"
    kubectl get nodes

    print "\n✨ Ready to deploy Sparky!"
    print "   Run: just deploy-ollama"
    print "   Then: just deploy-local"
}

def install_k3d [] {
    print "📥 Installing k3d..."

    match $nu.os-info.name {
        "linux" => {
            wget -q -O - https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash
        },
        "macos" => {
            brew install k3d
        },
        _ => {
            print "❌ Unsupported OS. Please install k3d manually."
            exit 1
        }
    }
}

main

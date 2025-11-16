# Raibid-CI kubectl Utility Module
# Helper functions for interacting with k3s/Kubernetes clusters
#
# Usage:
#   use kubectl.nu *
#   kubectl-check-cluster

# Check if kubectl is available
export def kubectl-check [] {
    if (which kubectl | is-empty) {
        error make {msg: "kubectl not found. Please install kubectl."}
    }
}

# Check cluster connectivity
export def kubectl-check-cluster [] {
    kubectl-check

    print "Checking Kubernetes cluster connectivity..."

    try {
        let result = (kubectl cluster-info | complete)
        if $result.exit_code == 0 {
            print $"(ansi green)✓(ansi reset) Connected to cluster"
            return true
        } else {
            print $"(ansi red)✗(ansi reset) Cannot connect to cluster"
            print $result.stderr
            return false
        }
    } catch {
        print $"(ansi red)✗(ansi reset) Error connecting to cluster"
        return false
    }
}

# Get cluster nodes
export def kubectl-get-nodes [] {
    kubectl-check

    kubectl get nodes -o json |
        from json |
        get items |
        select metadata.name status.nodeInfo.osImage status.conditions |
        each {|node|
            {
                name: $node.metadata.name
                os: $node.status.nodeInfo.osImage
                ready: ($node.status.conditions | where type == "Ready" | first | get status)
            }
        }
}

# Get pods in namespace
export def kubectl-get-pods [
    namespace: string = "default"  # Namespace to query
] {
    kubectl-check

    kubectl get pods -n $namespace -o json |
        from json |
        get items |
        select metadata.name status.phase spec.containers |
        each {|pod|
            {
                name: $pod.metadata.name
                phase: $pod.status.phase
                containers: ($pod.spec.containers | length)
            }
        }
}

# Get services in namespace
export def kubectl-get-services [
    namespace: string = "default"  # Namespace to query
] {
    kubectl-check

    kubectl get services -n $namespace -o json |
        from json |
        get items |
        select metadata.name spec.type spec.ports spec.clusterIP |
        each {|svc|
            {
                name: $svc.metadata.name
                type: $svc.spec.type
                cluster_ip: $svc.spec.clusterIP
                ports: ($svc.spec.ports | each {|p| $"($p.port):($p.targetPort)/($p.protocol)"} | str join ", ")
            }
        }
}

# Check if namespace exists
export def kubectl-namespace-exists [
    namespace: string  # Namespace to check
] {
    kubectl-check

    let result = (kubectl get namespace $namespace -o json | complete)
    $result.exit_code == 0
}

# Create namespace if it doesn't exist
export def kubectl-ensure-namespace [
    namespace: string  # Namespace to create
] {
    kubectl-check

    if not (kubectl-namespace-exists $namespace) {
        print $"Creating namespace: ($namespace)"
        kubectl create namespace $namespace
        print $"(ansi green)✓(ansi reset) Namespace created: ($namespace)"
    } else {
        print $"(ansi blue)ℹ(ansi reset) Namespace already exists: ($namespace)"
    }
}

# Get pod logs
export def kubectl-logs [
    pod: string          # Pod name
    namespace: string = "default"  # Namespace
    --follow (-f)        # Follow log output
    --tail: int = 100    # Number of lines to show
] {
    kubectl-check

    mut args = ["logs", $pod, "-n", $namespace, "--tail", ($tail | into string)]

    if $follow {
        $args = ($args | append "--follow")
    }

    kubectl ...$args
}

# Port forward to a service or pod
export def kubectl-port-forward [
    resource: string     # Resource to forward (pod/name or service/name)
    local_port: int      # Local port
    remote_port: int     # Remote port
    namespace: string = "default"  # Namespace
] {
    kubectl-check

    print $"Port forwarding ($resource) ($local_port):($remote_port) in namespace ($namespace)"
    print "Press Ctrl+C to stop"

    kubectl port-forward -n $namespace $resource $"($local_port):($remote_port)"
}

# Delete resources by label
export def kubectl-delete-by-label [
    resource_type: string  # Resource type (pod, service, deployment, etc.)
    label: string          # Label selector (e.g., "app=myapp")
    namespace: string = "default"  # Namespace
] {
    kubectl-check

    print $"Deleting ($resource_type) with label ($label) in namespace ($namespace)"
    kubectl delete $resource_type -n $namespace -l $label
}

# Apply manifest from file
export def kubectl-apply [
    manifest: string     # Path to manifest file or directory
    namespace: string = "default"  # Namespace
] {
    kubectl-check

    if not ($manifest | path exists) {
        error make {msg: $"Manifest not found: ($manifest)"}
    }

    print $"Applying manifest: ($manifest)"
    kubectl apply -f $manifest -n $namespace
}

# Wait for deployment to be ready
export def kubectl-wait-deployment [
    deployment: string   # Deployment name
    namespace: string = "default"  # Namespace
    timeout: string = "300s"  # Timeout
] {
    kubectl-check

    print $"Waiting for deployment ($deployment) to be ready..."
    kubectl wait --for=condition=available --timeout=$timeout deployment/$deployment -n $namespace
}

# Get resource usage (requires metrics-server)
export def kubectl-top-pods [
    namespace: string = "default"  # Namespace
] {
    kubectl-check

    try {
        kubectl top pods -n $namespace
    } catch {
        print $"(ansi yellow)⚠(ansi reset) Metrics not available. Is metrics-server installed?"
    }
}

# Execute command in pod
export def kubectl-exec [
    pod: string          # Pod name
    command: list<string>  # Command to execute
    namespace: string = "default"  # Namespace
    --interactive (-i)   # Interactive mode
] {
    kubectl-check

    mut args = ["exec", $pod, "-n", $namespace]

    if $interactive {
        $args = ($args | append ["-it"])
    }

    $args = ($args | append "--")
    $args = ($args | append $command)

    kubectl ...$args
}

# Describe resource
export def kubectl-describe [
    resource: string     # Resource (type/name or just name)
    namespace: string = "default"  # Namespace
] {
    kubectl-check

    kubectl describe -n $namespace $resource
}

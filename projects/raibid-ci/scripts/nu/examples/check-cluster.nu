#!/usr/bin/env nu
# Raibid-CI Cluster Health Check Example
# Demonstrates using kubectl module to check cluster health

# Source environment and config
source ../env.nu
source ../config.nu

# Import kubectl module
use ../modules/kubectl.nu *

def main [] {
    print "=== Raibid-CI Cluster Health Check ==="
    print ""

    # Check cluster connectivity
    if not (kubectl-check-cluster) {
        log-error "Cannot connect to cluster. Is k3s running?"
        exit 1
    }

    print ""

    # Get nodes
    log-info "Cluster Nodes:"
    let nodes = (kubectl-get-nodes)
    $nodes | table

    print ""

    # Get pods in key namespaces
    let namespaces = ["default", "kube-system", "raibid-ci"]

    for ns in $namespaces {
        if (kubectl-namespace-exists $ns) {
            log-info $"Pods in ($ns) namespace:"
            let pods = (kubectl-get-pods $ns)

            if ($pods | length) > 0 {
                $pods | table
            } else {
                print "  (no pods)"
            }

            print ""
        }
    }

    # Get services in raibid-ci namespace
    if (kubectl-namespace-exists "raibid-ci") {
        log-info "Services in raibid-ci namespace:"
        let services = (kubectl-get-services "raibid-ci")

        if ($services | length) > 0 {
            $services | table
        } else {
            print "  (no services)"
        }
    }

    print ""
    log-success "Health check complete!"
}

main

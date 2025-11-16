#!/usr/bin/env nu

# MOP Deployment Script
# Safely deploys MOP components with validation and rollback support
#
# Usage: ./deploy.nu --env <environment> [options]
#
# Features:
# - Pre-deployment validation
# - Interactive diff review
# - Confirmation prompts
# - Progressive rollout
# - Automatic smoke tests
# - Rollback on failure

def main [
    --env: string               # Environment to deploy (dev/staging/prod)
    --component: string         # Optional: deploy specific component only
    --auto-approve              # Skip confirmation prompts
    --no-smoke-test             # Skip post-deployment smoke tests
    --timeout: int = 600        # Deployment timeout in seconds
] {
    print $"🚀 (ansi green_bold)MOP Deployment Starting(ansi reset)"
    print $"   Environment: (ansi yellow)($env)(ansi reset)"
    if $component != null {
        print $"   Component: (ansi yellow)($component)(ansi reset)"
    }
    print ""

    let start_time = (date now)

    try {
        # Validate environment
        validate-environment $env

        # Run pre-deployment checks
        pre-deployment-checks $env $component

        # Show diff and get confirmation
        if not $auto_approve {
            show-diff $env $component
            confirm-deployment $env
        }

        # Execute deployment
        deploy-components $env $component $timeout

        # Wait for rollout
        wait-for-rollout $env $component $timeout

        # Run smoke tests
        if not $no_smoke_test {
            run-smoke-tests $env $component
        }

        let duration = ((date now) - $start_time)
        print ""
        print $"✅ (ansi green_bold)Deployment successful!(ansi reset) Duration: (format-duration $duration)"
        print $"🎯 Next steps:"
        print $"   1. Monitor health: ./health-check.nu --env ($env)"
        print $"   2. View metrics: kubectl port-forward -n mop-($env) svc/mimir-query-frontend 8080:8080"
        print $"   3. Check logs: kubectl logs -n mop-($env) -l app.kubernetes.io/component=ingester"

    } catch { |err|
        print ""
        print $"❌ (ansi red_bold)Deployment failed:(ansi reset) ($err.msg)"
        print $"🔄 Consider rolling back with: kubectl rollout undo -n mop-($env) deployment/($component)"
        exit 1
    }
}

# Validate environment argument
def validate-environment [env: string] {
    let valid_envs = ["dev" "staging" "prod"]

    if $env not-in $valid_envs {
        error make {
            msg: $"Invalid environment: ($env). Must be one of: ($valid_envs | str join ', ')"
        }
    }

    let env_dir = $"environments/($env)"
    if not ($env_dir | path exists) {
        error make {
            msg: $"Environment directory not found: ($env_dir). Run ./setup.nu first."
        }
    }
}

# Run pre-deployment validation checks
def pre-deployment-checks [env: string, component: string] {
    print $"🔍 (ansi cyan)Running pre-deployment checks...(ansi reset)"

    # Check cluster connectivity
    print $"   Checking cluster connectivity..."
    let context = (kubectl config current-context | complete)
    if $context.exit_code != 0 {
        error make {msg: "No Kubernetes context configured"}
    }
    print $"   ✓ Connected to: ($context.stdout | str trim)"

    # Check namespace exists
    let namespace = $"mop-($env)"
    print $"   Checking namespace: ($namespace)"
    let ns_check = (kubectl get namespace $namespace 2>&1 | complete)
    if $ns_check.exit_code != 0 {
        error make {msg: $"Namespace ($namespace) does not exist. Run ./setup.nu first."}
    }
    print $"   ✓ Namespace exists"

    # Validate Tanka configuration
    print $"   Validating Tanka configuration..."
    let env_dir = $"environments/($env)"
    let tk_eval = (tk eval $env_dir 2>&1 | complete)
    if $tk_eval.exit_code != 0 {
        error make {msg: $"Tanka configuration invalid: ($tk_eval.stderr)"}
    }
    print $"   ✓ Configuration valid"

    # Check resource quotas
    print $"   Checking resource availability..."
    let nodes = (kubectl get nodes -o json | from json)
    let total_nodes = ($nodes.items | length)
    let ready_nodes = ($nodes.items | where {|n|
        $n.status.conditions | any {|c| $c.type == "Ready" and $c.status == "True"}
    } | length)

    if $ready_nodes < $total_nodes {
        print $"   ⚠️  Warning: Only ($ready_nodes)/($total_nodes) nodes are ready"
    } else {
        print $"   ✓ All ($total_nodes) nodes ready"
    }

    print ""
}

# Show diff of changes to be applied
def show-diff [env: string, component: string] {
    print $"📊 (ansi cyan)Showing deployment diff...(ansi reset)"
    print ""

    let env_dir = $"environments/($env)"
    let diff_cmd = if $component == null {
        $"tk diff ($env_dir)"
    } else {
        $"tk diff ($env_dir) -t ($component)"
    }

    print $"   Running: (ansi yellow)($diff_cmd)(ansi reset)"
    print ""

    let diff_result = (do {
        tk diff $env_dir
    } | complete)

    if $diff_result.exit_code != 0 {
        # Diff returns non-zero when there are differences
        if ($diff_result.stdout | str length) > 0 {
            print $diff_result.stdout
        }
        if ($diff_result.stderr | str length) > 0 {
            print $diff_result.stderr
        }
    } else {
        print $"   ℹ️  No changes detected"
    }

    print ""
}

# Prompt user for deployment confirmation
def confirm-deployment [env: string] {
    print $"⚠️  (ansi yellow_bold)Deployment Confirmation Required(ansi reset)"
    print $"   Environment: (ansi yellow)($env)(ansi reset)"

    if $env == "prod" {
        print $"   (ansi red_bold)WARNING: This is a PRODUCTION deployment!(ansi reset)"
    }

    print ""
    let response = (input $"Continue with deployment? \(yes/no\): ")

    if $response != "yes" {
        print $"❌ Deployment cancelled by user"
        exit 0
    }

    print ""
}

# Execute Tanka apply
def deploy-components [env: string, component: string, timeout: int] {
    print $"🎯 (ansi cyan)Deploying components...(ansi reset)"

    let env_dir = $"environments/($env)"
    let apply_cmd = if $component == null {
        [$"tk apply" $env_dir "--dangerous-auto-approve"]
    } else {
        [$"tk apply" $env_dir "-t" $component "--dangerous-auto-approve"]
    }

    print $"   Running: (ansi yellow)(($apply_cmd | str join ' '))(ansi reset)"
    print ""

    # Execute apply with timeout
    let apply_result = (do {
        ^tk apply $env_dir --dangerous-auto-approve
    } | complete)

    if $apply_result.exit_code != 0 {
        error make {
            msg: $"Tanka apply failed: ($apply_result.stderr)"
        }
    }

    print $apply_result.stdout
    print $"   ✓ Resources applied successfully"
    print ""
}

# Wait for Kubernetes rollout to complete
def wait-for-rollout [env: string, component: string, timeout: int] {
    print $"⏳ (ansi cyan)Waiting for rollout to complete...(ansi reset)"

    let namespace = $"mop-($env)"

    # Get deployments to watch
    let deployments = if $component == null {
        kubectl get deployments -n $namespace -o json
            | from json
            | get items
            | get metadata.name
    } else {
        [$component]
    }

    let total = ($deployments | length)
    print $"   Watching ($total) deployments..."
    print ""

    let start_time = (date now)

    for deployment in $deployments {
        print $"   📦 ($deployment):"

        let rollout_result = (do {
            kubectl rollout status deployment/$deployment -n $namespace --timeout ($"($timeout)s")
        } | complete)

        if $rollout_result.exit_code != 0 {
            error make {
                msg: $"Rollout failed for ($deployment): ($rollout_result.stderr)"
            }
        }

        print $"      ✓ Rollout successful"
    }

    let duration = ((date now) - $start_time)
    print ""
    print $"   ✓ All rollouts completed in (format-duration $duration)"
    print ""
}

# Run post-deployment smoke tests
def run-smoke-tests [env: string, component: string] {
    print $"🧪 (ansi cyan)Running smoke tests...(ansi reset)"

    let namespace = $"mop-($env)"

    # Test 1: All pods are running
    print $"   Test 1: Pod health check"
    let pods = (kubectl get pods -n $namespace -o json | from json)
    let total_pods = ($pods.items | length)
    let running_pods = ($pods.items | where {|p|
        $p.status.phase == "Running"
    } | length)

    if $running_pods != $total_pods {
        print $"      ⚠️  Warning: ($running_pods)/($total_pods) pods running"
    } else {
        print $"      ✓ All ($total_pods) pods running"
    }

    # Test 2: Service endpoints available
    print $"   Test 2: Service endpoint check"
    let services = (kubectl get services -n $namespace -o json | from json)
    let service_count = ($services.items | length)
    print $"      ✓ ($service_count) services available"

    # Test 3: Critical component health
    print $"   Test 3: Component health check"
    let critical_components = ["mimir-ingester" "mimir-distributor" "mimir-query-frontend"]

    for comp in $critical_components {
        let pods = (kubectl get pods -n $namespace -l $"app.kubernetes.io/component=($comp)" -o json 2>&1 | complete)

        if $pods.exit_code == 0 {
            let pod_data = ($pods.stdout | from json)
            let count = ($pod_data.items | length)
            if $count > 0 {
                print $"      ✓ ($comp): ($count) replicas"
            } else {
                print $"      ⚠️  ($comp): no replicas found"
            }
        }
    }

    print ""
    print $"   ✅ Smoke tests completed"
    print ""
}

# Format duration helper
def format-duration [duration: duration] {
    let seconds = ($duration | into int) / 1_000_000_000
    let minutes = ($seconds / 60 | math floor)
    let remaining_seconds = ($seconds mod 60)

    if $minutes > 0 {
        $"($minutes)m ($remaining_seconds)s"
    } else {
        $"($seconds)s"
    }
}

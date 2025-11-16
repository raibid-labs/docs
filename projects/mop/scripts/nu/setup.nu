#!/usr/bin/env nu

# MOP Setup Script
# Initializes the MOP environment with all prerequisites
#
# Usage: ./setup.nu [--env dev|staging|prod] [--skip-vendor]
#
# Features:
# - Validates all required tools
# - Checks Kubernetes cluster connectivity
# - Initializes Tanka environments
# - Vendors dependencies
# - Creates namespaces
# - Installs CRDs

def main [
    --env: string = "dev"           # Environment to setup (dev/staging/prod)
    --skip-vendor                    # Skip vendoring dependencies
    --force                          # Force reinstall CRDs
] {
    print $"🚀 (ansi green_bold)Starting MOP setup for ($env) environment...(ansi reset)"
    print ""

    # Run all setup steps
    let start_time = (date now)

    try {
        check-prerequisites
        validate-cluster
        init-tanka $env

        if not $skip_vendor {
            vendor-dependencies
        }

        create-namespaces $env
        install-crds $force

        let duration = ((date now) - $start_time)
        print ""
        print $"✅ (ansi green_bold)Setup complete!(ansi reset) Duration: ($duration)"
        print $"🎯 Next steps:"
        print $"   1. Review configuration: tk show environments/($env)"
        print $"   2. Deploy components: ./deploy.nu --env ($env)"
        print $"   3. Check health: ./health-check.nu --env ($env)"

    } catch { |err|
        print $"❌ (ansi red_bold)Setup failed:(ansi reset) ($err.msg)"
        exit 1
    }
}

# Check if all required tools are installed
def check-prerequisites [] {
    print $"📋 (ansi cyan)Checking prerequisites...(ansi reset)"

    let required_tools = [
        {name: "kubectl", version_cmd: "kubectl version --client --short"}
        {name: "tanka", version_cmd: "tk --version"}
        {name: "helm", version_cmd: "helm version --short"}
        {name: "jq", version_cmd: "jq --version"}
        {name: "jsonnet", version_cmd: "jsonnet --version"}
        {name: "jsonnet-bundler", version_cmd: "jb --version"}
    ]

    let results = $required_tools | each { |tool|
        let installed = (which $tool.name | length) > 0

        if $installed {
            let version = (do -i {
                ^($tool.version_cmd | split row " " | first)
                    ...(($tool.version_cmd | split row " " | skip 1))
            } | complete | get stdout | str trim)

            {
                name: $tool.name
                installed: true
                version: $version
                status: "✓"
            }
        } else {
            {
                name: $tool.name
                installed: false
                version: "not found"
                status: "✗"
            }
        }
    }

    # Display results table
    print ($results | table -e)

    # Check if any tools are missing
    let missing = ($results | where installed == false)
    if ($missing | length) > 0 {
        print ""
        print $"❌ (ansi red)Missing required tools:(ansi reset)"
        $missing | each { |t| print $"   - ($t.name)" }
        print ""
        print $"📖 Install instructions:"
        print $"   kubectl: https://kubernetes.io/docs/tasks/tools/"
        print $"   tanka:   brew install tanka"
        print $"   helm:    brew install helm"
        print $"   jq:      brew install jq"
        print $"   jsonnet: brew install jsonnet"
        print $"   jb:      brew install jsonnet-bundler"
        error make {msg: "Missing required tools"}
    }

    print $"   ✓ All prerequisites satisfied"
    print ""
}

# Validate Kubernetes cluster connectivity
def validate-cluster [] {
    print $"🔌 (ansi cyan)Validating Kubernetes cluster connectivity...(ansi reset)"

    # Get current context
    let context = (kubectl config current-context | complete)
    if $context.exit_code != 0 {
        error make {msg: "No Kubernetes context configured"}
    }

    let context_name = ($context.stdout | str trim)
    print $"   Current context: (ansi yellow)($context_name)(ansi reset)"

    # Test cluster connectivity
    let nodes = (kubectl get nodes --no-headers 2>&1 | complete)
    if $nodes.exit_code != 0 {
        error make {msg: $"Cannot connect to cluster: ($nodes.stderr)"}
    }

    let node_count = ($nodes.stdout | lines | length)
    print $"   ✓ Connected to cluster with ($node_count) nodes"

    # Check cluster version
    let version = (kubectl version -o json | from json)
    print $"   Kubernetes version: ($version.serverVersion.gitVersion)"
    print ""
}

# Initialize Tanka environments
def init-tanka [env: string] {
    print $"🎯 (ansi cyan)Initializing Tanka for ($env) environment...(ansi reset)"

    let env_dir = $"environments/($env)"

    # Check if environment exists
    if not ($env_dir | path exists) {
        print $"   Creating environment directory: ($env_dir)"
        mkdir $env_dir
    }

    # Validate tanka.yml exists
    if not ("tanka.yml" | path exists) {
        print $"   Creating tanka.yml"
        {
            apiVersion: "tanka.dev/v1alpha1"
            kind: "Environment"
            metadata: {
                name: "mop"
            }
            spec: {
                apiServer: "https://kubernetes.default.svc"
                namespace: $"mop-($env)"
                resourceDefaults: {}
                expectVersions: {}
            }
        } | to yaml | save -f "tanka.yml"
    }

    # Initialize main.jsonnet if not exists
    let main_file = $"($env_dir)/main.jsonnet"
    if not ($main_file | path exists) {
        print $"   Creating ($main_file)"
        $"local mop = import 'mop/main.libsonnet';\n
{
  _config:: {
    namespace: 'mop-($env)',
    environment: '($env)',
  },

  mop: mop.new($._config),
}" | save -f $main_file
    }

    # Validate environment
    let validation = (tk eval $env_dir 2>&1 | complete)
    if $validation.exit_code != 0 {
        print $"   ⚠️  Warning: Environment validation failed"
        print $"   ($validation.stderr)"
    } else {
        print $"   ✓ Environment validated successfully"
    }

    print ""
}

# Vendor Jsonnet dependencies
def vendor-dependencies [] {
    print $"📦 (ansi cyan)Vendoring dependencies...(ansi reset)"

    # Check if jsonnetfile.json exists
    if not ("jsonnetfile.json" | path exists) {
        print $"   Creating jsonnetfile.json"
        {
            version: 1
            dependencies: [
                {
                    source: {
                        git: {
                            remote: "https://github.com/grafana/mimir"
                            subdir: "operations/mimir"
                        }
                    }
                    version: "main"
                }
                {
                    source: {
                        git: {
                            remote: "https://github.com/grafana/tempo"
                            subdir: "operations/jsonnet"
                        }
                    }
                    version: "main"
                }
                {
                    source: {
                        git: {
                            remote: "https://github.com/grafana/loki"
                            subdir: "production"
                        }
                    }
                    version: "main"
                }
            ]
            legacyImports: true
        } | to json | save -f "jsonnetfile.json"
    }

    # Run jb install
    print $"   Running jsonnet-bundler install..."
    let jb_result = (jb install 2>&1 | complete)
    if $jb_result.exit_code != 0 {
        error make {msg: $"Failed to vendor dependencies: ($jb_result.stderr)"}
    }

    print $"   ✓ Dependencies vendored to ./vendor"
    print ""
}

# Create Kubernetes namespaces
def create-namespaces [env: string] {
    print $"📁 (ansi cyan)Creating namespaces...(ansi reset)"

    let namespaces = [
        $"mop-($env)"
        $"mop-($env)-monitoring"
    ]

    for ns in $namespaces {
        let exists = (kubectl get namespace $ns 2>&1 | complete)
        if $exists.exit_code != 0 {
            print $"   Creating namespace: ($ns)"
            kubectl create namespace $ns
        } else {
            print $"   ✓ Namespace exists: ($ns)"
        }
    }

    print ""
}

# Install Custom Resource Definitions
def install-crds [force: bool] {
    print $"🔧 (ansi cyan)Installing CRDs...(ansi reset)"

    let crds = [
        {
            name: "prometheus-operator"
            url: "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_servicemonitors.yaml"
        }
        {
            name: "grafana-agent"
            url: "https://raw.githubusercontent.com/grafana/agent/main/operations/agent-static-operator/crds/monitoring.grafana.com_grafanaagents.yaml"
        }
    ]

    for crd in $crds {
        print $"   Installing CRD: ($crd.name)"

        if $force {
            kubectl delete crd $crd.name 2>&1 | ignore
        }

        let result = (http get $crd.url | kubectl apply -f - 2>&1 | complete)
        if $result.exit_code == 0 {
            print $"   ✓ ($crd.name) installed"
        } else {
            print $"   ⚠️  ($crd.name) installation failed: ($result.stderr)"
        }
    }

    print ""
}

# Helper to format duration
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

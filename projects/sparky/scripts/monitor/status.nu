#!/usr/bin/env nu
# Display Sparky system status

def main [
    --env: string = "local"  # Environment: local, production
] {
    print "📊 Sparky System Status"
    print $"🌍 Environment: ($env)"
    print ""

    # Check if kubectl is available
    if (which kubectl | is-empty) {
        print "❌ kubectl not found. Cannot check k8s status."
        return
    }

    # Get namespace
    let namespace = "sparky"

    # Check if namespace exists
    let ns_exists = (kubectl get namespace $namespace 2>/dev/null | complete)
    if $ns_exists.exit_code != 0 {
        print $"❌ Namespace '($namespace)' does not exist"
        print "   Run: just deploy-local (or just deploy-production)"
        return
    }

    # Get pods
    print "🎯 Pods:"
    kubectl get pods -n $namespace -o wide
    print ""

    # Get services
    print "🔌 Services:"
    kubectl get services -n $namespace
    print ""

    # Get deployments
    print "📦 Deployments:"
    kubectl get deployments -n $namespace
    print ""

    # Check Ollama specifically
    print "🧠 Ollama Status:"
    let ollama_pod = (
        kubectl get pods -n $namespace -l app=ollama -o json
        | from json
        | get items.0?
    )

    if ($ollama_pod | is-empty) {
        print "   ❌ Ollama not deployed"
        print "      Run: just deploy-ollama"
    } else {
        let status = $ollama_pod.status.phase
        print $"   Status: ($status)"

        if $status == "Running" {
            # Test Ollama API
            try {
                let response = (
                    http get "http://ollama.sparky.svc.cluster.local:11434/api/tags"
                    | from json
                )
                let models = $response.models
                print $"   ✅ Ollama is healthy"
                print $"   Models loaded: ($models | length)"
                for model in $models {
                    print $"     - ($model.name)"
                }
            } catch {
                print "   ⚠️  Ollama API not responding"
            }
        }
    }

    print ""

    # Check recent collections
    print "📊 Recent Collections:"
    let raw_dir = "data/raw"
    if ($raw_dir | path exists) {
        let recent = (ls $raw_dir | sort-by modified -r | first 5)
        for file in $recent {
            print $"   ($file.name) - ($file.modified | date humanize)"
        }
    } else {
        print "   No collections yet"
    }

    print ""

    # Check output
    print "📝 Recent Output:"
    let output_dir = "output/daily"
    if ($output_dir | path exists) {
        let recent = (ls $output_dir | sort-by modified -r | first 5)
        for file in $recent {
            print $"   ($file.name) - ($file.modified | date humanize)"
        }
    } else {
        print "   No output yet"
    }

    print ""
    print "✨ Use 'just logs-follow' to view service logs"
}

main

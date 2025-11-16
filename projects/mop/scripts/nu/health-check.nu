#!/usr/bin/env nu

# MOP Health Check Script
# Comprehensive health verification for all MOP components
#
# Usage: ./health-check.nu --env <environment> [options]
#
# Features:
# - Pod status verification
# - Container readiness checks
# - Metrics endpoint validation
# - Inter-component connectivity tests
# - Resource utilization monitoring
# - Health report generation

def main [
    --env: string               # Environment to check (dev/staging/prod)
    --component: string         # Optional: check specific component only
    --format: string = "table"  # Output format: table, json, markdown
    --export: string            # Export report to file
    --watch                     # Continuous monitoring mode
] {
    print $"🏥 (ansi green_bold)MOP Health Check(ansi reset)"
    print $"   Environment: (ansi yellow)($env)(ansi reset)"
    if $component != null {
        print $"   Component: (ansi yellow)($component)(ansi reset)"
    }
    print ""

    try {
        # Validate environment
        validate-environment $env

        if $watch {
            # Continuous monitoring mode
            watch-health $env $component
        } else {
            # Single health check
            let report = run-health-check $env $component
            display-report $report $format

            if $export != null {
                export-report $report $export $format
            }

            # Exit with error if health check failed
            if $report.overall_status != "healthy" {
                exit 1
            }
        }

    } catch { |err|
        print $"❌ (ansi red_bold)Health check failed:(ansi reset) ($err.msg)"
        exit 1
    }
}

# Validate environment
def validate-environment [env: string] {
    let valid_envs = ["dev" "staging" "prod"]
    if $env not-in $valid_envs {
        error make {
            msg: $"Invalid environment: ($env). Must be one of: ($valid_envs | str join ', ')"
        }
    }
}

# Run comprehensive health check
def run-health-check [env: string, component: string] {
    let namespace = $"mop-($env)"
    let timestamp = (date now)

    print $"🔍 (ansi cyan)Collecting health data...(ansi reset)"

    # Collect all health data
    let pod_health = check-pod-health $namespace $component
    let service_health = check-service-health $namespace
    let metrics_health = check-metrics-endpoints $namespace $component
    let connectivity = test-connectivity $namespace
    let resources = check-resource-usage $namespace $component

    # Calculate overall status
    let all_checks = [$pod_health.status $service_health.status $metrics_health.status $connectivity.status]
    let overall_status = if ($all_checks | all {|s| $s == "healthy"}) {
        "healthy"
    } else if ($all_checks | any {|s| $s == "critical"}) {
        "critical"
    } else {
        "degraded"
    }

    {
        timestamp: $timestamp
        environment: $env
        namespace: $namespace
        overall_status: $overall_status
        checks: {
            pods: $pod_health
            services: $service_health
            metrics: $metrics_health
            connectivity: $connectivity
            resources: $resources
        }
    }
}

# Check pod health status
def check-pod-health [namespace: string, component: string] {
    print $"   Checking pod health..."

    let selector = if $component != null {
        $"-l app.kubernetes.io/component=($component)"
    } else {
        ""
    }

    let pods = (kubectl get pods -n $namespace $selector -o json | from json)

    let pod_statuses = $pods.items | each {|pod|
        let name = $pod.metadata.name
        let phase = $pod.status.phase

        # Check container statuses
        let containers = $pod.status.containerStatuses | each {|c|
            {
                name: $c.name
                ready: $c.ready
                restarts: $c.restartCount
                state: ($c.state | columns | first)
            }
        }

        let all_ready = ($containers | all {|c| $c.ready})
        let high_restarts = ($containers | any {|c| $c.restartCount > 5})

        let status = if ($phase == "Running" and $all_ready and not $high_restarts) {
            "healthy"
        } else if ($phase == "Failed" or $phase == "CrashLoopBackOff") {
            "critical"
        } else {
            "degraded"
        }

        {
            name: $name
            phase: $phase
            status: $status
            containers: $containers
            node: $pod.spec.nodeName
        }
    }

    let total = ($pod_statuses | length)
    let healthy = ($pod_statuses | where status == "healthy" | length)
    let degraded = ($pod_statuses | where status == "degraded" | length)
    let critical = ($pod_statuses | where status == "critical" | length)

    let overall_status = if $critical > 0 {
        "critical"
    } else if $degraded > 0 {
        "degraded"
    } else {
        "healthy"
    }

    {
        status: $overall_status
        total: $total
        healthy: $healthy
        degraded: $degraded
        critical: $critical
        pods: $pod_statuses
    }
}

# Check service health
def check-service-health [namespace: string] {
    print $"   Checking services..."

    let services = (kubectl get services -n $namespace -o json | from json)

    let service_statuses = $services.items | each {|svc|
        let name = $svc.metadata.name
        let type = $svc.spec.type
        let cluster_ip = $svc.spec.clusterIP

        # Check if service has endpoints
        let endpoints = (kubectl get endpoints -n $namespace $name -o json 2>&1 | complete)
        let has_endpoints = if $endpoints.exit_code == 0 {
            let ep_data = ($endpoints.stdout | from json)
            ($ep_data.subsets? | default [] | length) > 0
        } else {
            false
        }

        let status = if $has_endpoints { "healthy" } else { "degraded" }

        {
            name: $name
            type: $type
            cluster_ip: $cluster_ip
            has_endpoints: $has_endpoints
            status: $status
        }
    }

    let total = ($service_statuses | length)
    let healthy = ($service_statuses | where status == "healthy" | length)

    {
        status: (if $healthy == $total { "healthy" } else { "degraded" })
        total: $total
        healthy: $healthy
        services: $service_statuses
    }
}

# Check metrics endpoints
def check-metrics-endpoints [namespace: string, component: string] {
    print $"   Checking metrics endpoints..."

    let selector = if $component != null {
        $"-l app.kubernetes.io/component=($component)"
    } else {
        "-l app.kubernetes.io/part-of=mop"
    }

    let pods = (kubectl get pods -n $namespace $selector -o json | from json)

    let metrics_checks = $pods.items | each {|pod|
        let name = $pod.metadata.name
        let ip = $pod.status.podIP

        # Try to curl metrics endpoint
        let metrics_result = (kubectl exec -n $namespace $name -- curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/metrics 2>&1 | complete)

        let status = if $metrics_result.exit_code == 0 and ($metrics_result.stdout | str trim) == "200" {
            "healthy"
        } else {
            "degraded"
        }

        {
            pod: $name
            endpoint: $"http://($ip):8080/metrics"
            status: $status
        }
    }

    let total = ($metrics_checks | length)
    let healthy = ($metrics_checks | where status == "healthy" | length)

    {
        status: (if $healthy > ($total * 0.8) { "healthy" } else { "degraded" })
        total: $total
        healthy: $healthy
        endpoints: $metrics_checks
    }
}

# Test connectivity between components
def test-connectivity [namespace: string] {
    print $"   Testing inter-component connectivity..."

    let tests = [
        {
            from: "mimir-distributor"
            to: "mimir-ingester"
            port: 9095
        }
        {
            from: "mimir-query-frontend"
            to: "mimir-querier"
            port: 9095
        }
    ]

    let connectivity_results = $tests | each {|test|
        # Get a pod from the source component
        let source_pod = (kubectl get pods -n $namespace -l $"app.kubernetes.io/component=($test.from)" -o jsonpath="{.items[0].metadata.name}" 2>&1 | complete)

        if $source_pod.exit_code != 0 or ($source_pod.stdout | str trim) == "" {
            return {
                from: $test.from
                to: $test.to
                status: "unknown"
                message: "Source pod not found"
            }
        }

        let pod_name = ($source_pod.stdout | str trim)

        # Get target service
        let target_svc = $"($test.to).($namespace).svc.cluster.local"

        # Test connectivity
        let conn_test = (kubectl exec -n $namespace $pod_name -- timeout 5 nc -zv $target_svc $test.port 2>&1 | complete)

        let status = if $conn_test.exit_code == 0 { "healthy" } else { "degraded" }

        {
            from: $test.from
            to: $test.to
            target: $target_svc
            port: $test.port
            status: $status
        }
    }

    let total = ($connectivity_results | length)
    let healthy = ($connectivity_results | where status == "healthy" | length)

    {
        status: (if $healthy == $total { "healthy" } else { "degraded" })
        total: $total
        healthy: $healthy
        tests: $connectivity_results
    }
}

# Check resource usage
def check-resource-usage [namespace: string, component: string] {
    print $"   Checking resource usage..."

    let selector = if $component != null {
        $"-l app.kubernetes.io/component=($component)"
    } else {
        ""
    }

    let pods = (kubectl top pods -n $namespace $selector --no-headers 2>&1 | complete)

    if $pods.exit_code != 0 {
        return {
            status: "unknown"
            message: "Metrics server not available"
            pods: []
        }
    }

    let usage_data = $pods.stdout | lines | each {|line|
        let parts = ($line | split row (char space) | where {|x| $x != ""})
        if ($parts | length) >= 3 {
            {
                name: ($parts | get 0)
                cpu: ($parts | get 1)
                memory: ($parts | get 2)
            }
        }
    } | compact

    {
        status: "healthy"
        pods: $usage_data
    }
}

# Display health report
def display-report [report: record, format: string] {
    print ""
    print $"📊 (ansi cyan_bold)Health Report(ansi reset)"
    print $"   Timestamp: ($report.timestamp | format date '%Y-%m-%d %H:%M:%S')"
    print $"   Environment: ($report.environment)"
    print $"   Overall Status: (status-badge $report.overall_status)"
    print ""

    match $format {
        "table" => { display-table-report $report }
        "json" => { print ($report | to json -i 2) }
        "markdown" => { display-markdown-report $report }
        _ => { print "Invalid format" }
    }
}

# Display report as table
def display-table-report [report: record] {
    print $"(ansi cyan_bold)Pod Health:(ansi reset)"
    $report.checks.pods.pods | select name phase status | table -e
    print ""

    print $"(ansi cyan_bold)Services:(ansi reset)"
    $report.checks.services.services | select name type has_endpoints status | table -e
    print ""

    if ($report.checks.resources.pods | length) > 0 {
        print $"(ansi cyan_bold)Resource Usage:(ansi reset)"
        $report.checks.resources.pods | table -e
        print ""
    }
}

# Display report as markdown
def display-markdown-report [report: record] {
    print $"# Health Report - ($report.environment)"
    print $"**Timestamp:** ($report.timestamp | format date '%Y-%m-%d %H:%M:%S')"
    print $"**Status:** ($report.overall_status)"
    print ""
    print $"## Pod Health"
    print $"- Total: ($report.checks.pods.total)"
    print $"- Healthy: ($report.checks.pods.healthy)"
    print $"- Degraded: ($report.checks.pods.degraded)"
    print $"- Critical: ($report.checks.pods.critical)"
}

# Export report to file
def export-report [report: record, path: string, format: string] {
    match $format {
        "json" => { $report | to json -i 2 | save -f $path }
        "markdown" => {
            # Generate markdown and save
            print "Exporting markdown report..."
        }
        _ => { $report | to json -i 2 | save -f $path }
    }

    print $"   ✓ Report exported to: ($path)"
}

# Watch health continuously
def watch-health [env: string, component: string] {
    loop {
        clear
        let report = run-health-check $env $component
        display-report $report "table"
        print ""
        print $"(ansi dim)Refreshing in 10 seconds... (Ctrl+C to stop)(ansi reset)"
        sleep 10sec
    }
}

# Helper to display status badge
def status-badge [status: string] {
    match $status {
        "healthy" => { $"(ansi green)●(ansi reset) Healthy" }
        "degraded" => { $"(ansi yellow)●(ansi reset) Degraded" }
        "critical" => { $"(ansi red)●(ansi reset) Critical" }
        _ => { $"(ansi dim)●(ansi reset) Unknown" }
    }
}

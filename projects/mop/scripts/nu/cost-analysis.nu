#!/usr/bin/env nu

# MOP Cost Analysis Script
# Analyzes costs and provides optimization recommendations
#
# Usage: ./cost-analysis.nu --env <environment> [options]
#
# Features:
# - Query metrics from Mimir
# - Calculate trace volume by service
# - Estimate storage costs
# - Generate cost reports
# - Compare to baselines
# - Provide optimization recommendations

def main [
    --env: string               # Environment to analyze (dev/staging/prod)
    --period: string = "7d"     # Analysis period (1h, 1d, 7d, 30d)
    --format: string = "table"  # Output format: table, json, csv
    --export: string            # Export report to file
    --baseline: string          # Compare to baseline file
    --mimir-url: string = "http://localhost:8080" # Mimir query endpoint
] {
    print $"💰 (ansi green_bold)MOP Cost Analysis(ansi reset)"
    print $"   Environment: (ansi yellow)($env)(ansi reset)"
    print $"   Period: (ansi yellow)($period)(ansi reset)"
    print ""

    try {
        # Setup port forwarding to Mimir if needed
        setup-port-forward $env $mimir_url

        # Collect cost data
        let cost_data = collect-cost-data $env $period $mimir_url

        # Calculate costs
        let analysis = analyze-costs $cost_data $period

        # Compare to baseline if provided
        let comparison = if $baseline != null {
            compare-to-baseline $analysis $baseline
        } else {
            null
        }

        # Display report
        display-cost-report $analysis $comparison $format

        # Generate recommendations
        let recommendations = generate-recommendations $analysis
        display-recommendations $recommendations

        # Export if requested
        if $export != null {
            export-analysis $analysis $recommendations $export $format
        }

    } catch { |err|
        print $"❌ (ansi red_bold)Cost analysis failed:(ansi reset) ($err.msg)"
        exit 1
    }
}

# Setup port forwarding to Mimir query frontend
def setup-port-forward [env: string, url: string] {
    # Check if we need to setup port forward
    if ($url | str contains "localhost") {
        print $"🔌 (ansi cyan)Setting up port forward to Mimir...(ansi reset)"

        let namespace = $"mop-($env)"

        # Check if port forward is already active
        let existing = (ps | where name =~ "port-forward" and command =~ "mimir-query-frontend")

        if ($existing | length) == 0 {
            print $"   Starting port forward on :8080..."

            # Start port forward in background
            kubectl port-forward -n $namespace svc/mimir-query-frontend 8080:8080 2>&1 | ignore &

            # Wait for port to be ready
            sleep 2sec
            print $"   ✓ Port forward active"
        } else {
            print $"   ✓ Port forward already active"
        }
        print ""
    }
}

# Collect cost-related metrics from Mimir
def collect-cost-data [env: string, period: string, mimir_url: string] {
    print $"📊 (ansi cyan)Collecting metrics data...(ansi reset)"

    let queries = [
        {
            name: "trace_volume_by_service"
            query: 'sum by (service_name) (rate(traces_received_total[1h]))'
            description: "Trace ingestion rate per service"
        }
        {
            name: "storage_usage"
            query: 'sum(mimir_ingester_memory_series)'
            description: "Active time series in memory"
        }
        {
            name: "samples_ingested"
            query: 'sum(rate(mimir_distributor_samples_in_total[1h]))'
            description: "Sample ingestion rate"
        }
        {
            name: "query_rate"
            query: 'sum(rate(mimir_request_duration_seconds_count{route=~"/api/v1/query.*"}[1h]))'
            description: "Query request rate"
        }
        {
            name: "ingester_instances"
            query: 'count(up{job="mimir-ingester"})'
            description: "Number of ingester instances"
        }
        {
            name: "storage_blocks"
            query: 'sum(cortex_bucket_store_blocks_loaded)'
            description: "Number of storage blocks"
        }
    ]

    let results = $queries | each {|q|
        print $"   Querying: ($q.name)..."

        let result = query-mimir $mimir_url $q.query $period

        {
            name: $q.name
            description: $q.description
            data: $result
        }
    }

    print $"   ✓ Collected ($results | length) metrics"
    print ""

    $results
}

# Query Mimir Prometheus API
def query-mimir [base_url: string, query: string, period: string] {
    let url = $"($base_url)/prometheus/api/v1/query"

    let response = (http get $"($url)?query=($query)" 2>&1 | complete)

    if $response.exit_code != 0 {
        return {status: "error", error: $response.stderr}
    }

    try {
        let data = ($response.stdout | from json)

        if $data.status == "success" {
            $data.data.result
        } else {
            {status: "error", error: $data.error}
        }
    } catch {
        {status: "error", error: "Failed to parse response"}
    }
}

# Analyze costs from collected data
def analyze-costs [cost_data: list, period: string] {
    print $"🔍 (ansi cyan)Analyzing costs...(ansi reset)"

    # Extract metrics
    let trace_volume = ($cost_data | where name == "trace_volume_by_service" | get data | first)
    let storage = ($cost_data | where name == "storage_usage" | get data | first)
    let samples = ($cost_data | where name == "samples_ingested" | get data | first)
    let queries = ($cost_data | where name == "query_rate" | get data | first)
    let ingesters = ($cost_data | where name == "ingester_instances" | get data | first)

    # Calculate costs (example pricing)
    let pricing = {
        storage_per_gb: 0.023  # $0.023 per GB-month (S3 standard)
        compute_per_hour: 0.05  # $0.05 per vCPU hour
        samples_per_million: 0.001  # $0.001 per million samples
    }

    # Estimate storage cost
    let series_count = if ($storage | length) > 0 {
        ($storage | get 0.value.1 | into float)
    } else {
        0
    }
    let avg_series_size = 1000  # bytes
    let estimated_storage_gb = ($series_count * $avg_series_size / 1024 / 1024 / 1024)
    let storage_cost_monthly = ($estimated_storage_gb * $pricing.storage_per_gb)

    # Estimate compute cost
    let ingester_count = if ($ingesters | length) > 0 {
        ($ingesters | get 0.value.1 | into float)
    } else {
        0
    }
    let compute_cost_monthly = ($ingester_count * $pricing.compute_per_hour * 24 * 30)

    # Estimate ingestion cost
    let sample_rate = if ($samples | length) > 0 {
        ($samples | get 0.value.1 | into float)
    } else {
        0
    }
    let samples_per_month = ($sample_rate * 3600 * 24 * 30)
    let ingestion_cost_monthly = ($samples_per_month / 1_000_000 * $pricing.samples_per_million)

    # Calculate total
    let total_cost_monthly = $storage_cost_monthly + $compute_cost_monthly + $ingestion_cost_monthly

    # Cost breakdown by service
    let service_costs = if ($trace_volume | length) > 0 {
        $trace_volume | each {|item|
            let service = ($item.metric.service_name? | default "unknown")
            let rate = ($item.value.1 | into float)
            let percentage = if $sample_rate > 0 { ($rate / $sample_rate * 100) } else { 0 }
            let estimated_cost = ($total_cost_monthly * $percentage / 100)

            {
                service: $service
                trace_rate: $rate
                percentage: $percentage
                estimated_monthly_cost: $estimated_cost
            }
        }
    } else {
        []
    }

    print $"   ✓ Analysis complete"
    print ""

    {
        period: $period
        timestamp: (date now)
        metrics: {
            active_series: $series_count
            sample_rate: $sample_rate
            ingester_count: $ingester_count
            estimated_storage_gb: $estimated_storage_gb
        }
        costs: {
            storage: $storage_cost_monthly
            compute: $compute_cost_monthly
            ingestion: $ingestion_cost_monthly
            total: $total_cost_monthly
        }
        by_service: $service_costs
        pricing: $pricing
    }
}

# Compare analysis to baseline
def compare-to-baseline [analysis: record, baseline_path: string] {
    print $"📈 (ansi cyan)Comparing to baseline...(ansi reset)"

    if not ($baseline_path | path exists) {
        print $"   ⚠️  Baseline file not found: ($baseline_path)"
        return null
    }

    let baseline = (open $baseline_path | from json)

    let comparison = {
        storage_change: (($analysis.costs.storage - $baseline.costs.storage) / $baseline.costs.storage * 100)
        compute_change: (($analysis.costs.compute - $baseline.costs.compute) / $baseline.costs.compute * 100)
        total_change: (($analysis.costs.total - $baseline.costs.total) / $baseline.costs.total * 100)
    }

    print $"   Storage: (format-change $comparison.storage_change)"
    print $"   Compute: (format-change $comparison.compute_change)"
    print $"   Total: (format-change $comparison.total_change)"
    print ""

    $comparison
}

# Display cost report
def display-cost-report [analysis: record, comparison: record, format: string] {
    print $"💵 (ansi cyan_bold)Cost Report(ansi reset)"
    print $"   Period: ($analysis.period)"
    print $"   Timestamp: ($analysis.timestamp | format date '%Y-%m-%d %H:%M:%S')"
    print ""

    match $format {
        "table" => { display-table-report $analysis }
        "json" => { print ($analysis | to json -i 2) }
        "csv" => { display-csv-report $analysis }
        _ => { print "Invalid format" }
    }
}

# Display cost report as table
def display-table-report [analysis: record] {
    print $"(ansi yellow_bold)Overall Costs (Monthly Estimates):(ansi reset)"

    [
        {category: "Storage" cost: ($analysis.costs.storage | format-currency)}
        {category: "Compute" cost: ($analysis.costs.compute | format-currency)}
        {category: "Ingestion" cost: ($analysis.costs.ingestion | format-currency)}
        {category: "TOTAL" cost: ($analysis.costs.total | format-currency)}
    ] | table -e

    print ""
    print $"(ansi yellow_bold)Cost by Service:(ansi reset)"

    if ($analysis.by_service | length) > 0 {
        $analysis.by_service
            | each {|s|
                {
                    service: $s.service
                    percentage: ($"($s.percentage | math round -p 2)%")
                    monthly_cost: (format-currency $s.estimated_monthly_cost)
                }
            }
            | table -e
    } else {
        print "   No service-level data available"
    }
    print ""
}

# Display cost report as CSV
def display-csv-report [analysis: record] {
    print "category,monthly_cost"
    print $"storage,($analysis.costs.storage)"
    print $"compute,($analysis.costs.compute)"
    print $"ingestion,($analysis.costs.ingestion)"
    print $"total,($analysis.costs.total)"
}

# Generate optimization recommendations
def generate-recommendations [analysis: record] {
    print $"💡 (ansi cyan)Generating recommendations...(ansi reset)"

    mut recommendations = []

    # Check storage usage
    if $analysis.metrics.estimated_storage_gb > 100 {
        $recommendations = ($recommendations | append {
            priority: "high"
            category: "storage"
            message: $"High storage usage detected \(($analysis.metrics.estimated_storage_gb | math round -p 2)GB\). Consider implementing data retention policies."
            potential_savings: ($analysis.costs.storage * 0.3)
        })
    }

    # Check ingester count
    if $analysis.metrics.ingester_count > 10 {
        $recommendations = ($recommendations | append {
            priority: "medium"
            category: "compute"
            message: $"Consider optimizing ingester count \(currently ($analysis.metrics.ingester_count)\) based on load patterns."
            potential_savings: ($analysis.costs.compute * 0.2)
        })
    }

    # Check service distribution
    let top_services = ($analysis.by_service | sort-by estimated_monthly_cost -r | first 3)
    if ($top_services | length) > 0 {
        let top_service = ($top_services | first)
        if $top_service.percentage > 50 {
            $recommendations = ($recommendations | append {
                priority: "medium"
                category: "optimization"
                message: $"Service '($top_service.service)' accounts for ($top_service.percentage | math round -p 2)% of costs. Consider optimizing trace sampling rate."
                potential_savings: ($top_service.estimated_monthly_cost * 0.3)
            })
        }
    }

    # General recommendations
    $recommendations = ($recommendations | append {
        priority: "low"
        category: "monitoring"
        message: "Enable adaptive sampling to automatically adjust trace rates based on traffic patterns."
        potential_savings: ($analysis.costs.total * 0.15)
    })

    $recommendations = ($recommendations | append {
        priority: "low"
        category: "storage"
        message: "Implement tiered storage strategy with hot/warm/cold data lifecycle."
        potential_savings: ($analysis.costs.storage * 0.4)
    })

    print $"   ✓ Generated ($recommendations | length) recommendations"
    print ""

    $recommendations
}

# Display recommendations
def display-recommendations [recommendations: list] {
    print $"🎯 (ansi cyan_bold)Optimization Recommendations:(ansi reset)"
    print ""

    let sorted = ($recommendations | sort-by priority)

    for rec in $sorted {
        let priority_badge = match $rec.priority {
            "high" => { $"(ansi red_bold)HIGH(ansi reset)" }
            "medium" => { $"(ansi yellow_bold)MEDIUM(ansi reset)" }
            "low" => { $"(ansi cyan_bold)LOW(ansi reset)" }
            _ => { $rec.priority }
        }

        print $"[($priority_badge)] ($rec.category | str upcase)"
        print $"   ($rec.message)"
        print $"   Potential monthly savings: (format-currency $rec.potential_savings)"
        print ""
    }
}

# Export analysis to file
def export-analysis [analysis: record, recommendations: list, path: string, format: string] {
    let export_data = {
        analysis: $analysis
        recommendations: $recommendations
        generated_at: (date now)
    }

    match $format {
        "json" => { $export_data | to json -i 2 | save -f $path }
        "csv" => {
            # Export costs as CSV
            $analysis.by_service | to csv | save -f $path
        }
        _ => { $export_data | to json -i 2 | save -f $path }
    }

    print $"   ✓ Analysis exported to: ($path)"
}

# Helper to format currency
def format-currency [amount: float] {
    $"$($amount | math round -p 2)"
}

# Helper to format change percentage
def format-change [change: float] {
    let formatted = ($change | math round -p 2)
    if $change > 0 {
        $"(ansi red)+($formatted)%(ansi reset)"
    } else {
        $"(ansi green)($formatted)%(ansi reset)"
    }
}

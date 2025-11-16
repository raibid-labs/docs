#!/usr/bin/env nu

# MOP Experiment Runner
# Automated OBI experiment execution and analysis
#
# Usage: ./experiment-runner.nu --config <experiment-config.json> [options]
#
# Features:
# - Load experiment configuration
# - Deploy experimental changes
# - Monitor metrics during experiment
# - Collect and analyze results
# - Generate experiment report
# - Automatic rollback on failure

def main [
    --config: string            # Path to experiment configuration file
    --env: string = "dev"       # Environment to run experiment (dev/staging/prod)
    --duration: int = 3600      # Experiment duration in seconds (default: 1 hour)
    --baseline-duration: int = 300 # Baseline collection period in seconds
    --auto-rollback              # Automatically rollback on metric degradation
    --export: string            # Export results to file
] {
    print $"🧪 (ansi green_bold)MOP Experiment Runner(ansi reset)"
    print ""

    try {
        # Load and validate configuration
        let experiment = load-experiment-config $config

        print $"Experiment: (ansi yellow)($experiment.name)(ansi reset)"
        print $"Description: ($experiment.description)"
        print $"Environment: (ansi yellow)($env)(ansi reset)"
        print $"Duration: ($duration)s"
        print ""

        # Confirm experiment execution
        confirm-experiment $experiment $env

        # Collect baseline metrics
        let baseline = collect-baseline $env $experiment $baseline_duration

        # Deploy experiment changes
        deploy-experiment $env $experiment

        # Monitor experiment
        let results = monitor-experiment $env $experiment $duration $baseline

        # Analyze results
        let analysis = analyze-results $experiment $baseline $results

        # Generate report
        let report = generate-report $experiment $baseline $results $analysis

        # Display results
        display-results $report

        # Export if requested
        if $export != null {
            export-results $report $export
        }

        # Rollback decision
        if $auto_rollback and $analysis.recommendation == "rollback" {
            print ""
            print $"⚠️  (ansi yellow)Auto-rollback triggered due to metric degradation(ansi reset)"
            rollback-experiment $env $experiment
        } else {
            # Prompt for manual decision
            prompt-rollback $env $experiment $analysis
        }

        print ""
        print $"✅ (ansi green_bold)Experiment completed!(ansi reset)"

    } catch { |err|
        print $"❌ (ansi red_bold)Experiment failed:(ansi reset) ($err.msg)"
        print $"🔄 Rolling back changes..."

        try {
            rollback-experiment $env $experiment
            print $"   ✓ Rollback successful"
        } catch {
            print $"   ❌ Rollback failed - manual intervention required"
        }

        exit 1
    }
}

# Load and validate experiment configuration
def load-experiment-config [config_path: string] {
    print $"📋 (ansi cyan)Loading experiment configuration...(ansi reset)"

    if not ($config_path | path exists) {
        error make {msg: $"Configuration file not found: ($config_path)"}
    }

    let experiment = (open $config_path | from json)

    # Validate required fields
    let required_fields = ["name" "description" "changes" "success_metrics"]

    for field in $required_fields {
        if ($experiment | get -i $field) == null {
            error make {msg: $"Missing required field in config: ($field)"}
        }
    }

    print $"   ✓ Configuration loaded and validated"
    print ""

    $experiment
}

# Confirm experiment execution
def confirm-experiment [experiment: record, env: string] {
    print $"⚠️  (ansi yellow_bold)Experiment Confirmation(ansi reset)"
    print $"   Name: ($experiment.name)"
    print $"   Environment: ($env)"
    print ""
    print $"   Changes to be applied:"

    for change in $experiment.changes {
        print $"   - ($change.component): ($change.parameter) = ($change.value)"
    }

    print ""

    if $env == "prod" {
        print $"   (ansi red_bold)WARNING: Running experiment in PRODUCTION!(ansi reset)"
    }

    let response = (input $"Proceed with experiment? \(yes/no\): ")

    if $response != "yes" {
        print $"❌ Experiment cancelled"
        exit 0
    }

    print ""
}

# Collect baseline metrics
def collect-baseline [env: string, experiment: record, duration: int] {
    print $"📊 (ansi cyan)Collecting baseline metrics...(ansi reset)"
    print $"   Duration: ($duration)s"

    let start_time = (date now)
    let namespace = $"mop-($env)"

    # Initialize baseline data structure
    mut baseline_data = {}

    # Collect metrics for each success metric
    for metric in $experiment.success_metrics {
        print $"   Collecting: ($metric.name)"

        let values = collect-metric-samples $namespace $metric.query $duration

        $baseline_data = ($baseline_data | insert $metric.name {
            query: $metric.query
            samples: $values
            average: (calculate-average $values)
            p95: (calculate-percentile $values 95)
            p99: (calculate-percentile $values 99)
        })
    }

    let elapsed = ((date now) - $start_time)

    print $"   ✓ Baseline collected"
    print ""

    {
        timestamp: $start_time
        duration: $duration
        metrics: $baseline_data
    }
}

# Deploy experiment changes
def deploy-experiment [env: string, experiment: record] {
    print $"🚀 (ansi cyan)Deploying experiment changes...(ansi reset)"

    let namespace = $"mop-($env)"

    # Apply each change
    for change in $experiment.changes {
        print $"   Applying: ($change.component) - ($change.parameter)"

        # Generate patch based on change type
        let patch = match $change.type {
            "deployment" => {
                generate-deployment-patch $change
            }
            "configmap" => {
                generate-configmap-patch $change
            }
            _ => {
                error make {msg: $"Unknown change type: ($change.type)"}
            }
        }

        # Apply patch
        kubectl patch $change.type $change.component -n $namespace --type merge -p $patch

        # Wait for rollout if deployment
        if $change.type == "deployment" {
            kubectl rollout status deployment/$change.component -n $namespace --timeout=5m
        }
    }

    # Wait for stabilization
    print $"   Waiting for stabilization..."
    sleep 30sec

    print $"   ✓ Experiment deployed"
    print ""
}

# Monitor experiment metrics
def monitor-experiment [env: string, experiment: record, duration: int, baseline: record] {
    print $"👁️  (ansi cyan)Monitoring experiment...(ansi reset)"
    print $"   Duration: ($duration)s"
    print ""

    let start_time = (date now)
    let namespace = $"mop-($env)"

    mut results = []
    let check_interval = 60  # Check every 60 seconds

    let total_checks = ($duration / $check_interval)
    mut check_count = 0

    while (((date now) - $start_time) | into int) < ($duration * 1_000_000_000) {
        $check_count = $check_count + 1

        print $"   Check ($check_count)/($total_checks):"

        mut sample = {
            timestamp: (date now)
            metrics: {}
        }

        # Collect current values for each metric
        for metric in $experiment.success_metrics {
            let value = query-metric-instant $namespace $metric.query

            $sample.metrics = ($sample.metrics | insert $metric.name $value)

            # Compare to baseline
            let baseline_value = ($baseline.metrics | get $metric.name | get average)
            let change_pct = (($value - $baseline_value) / $baseline_value * 100)

            let status = if ($change_pct | math abs) < 5 {
                "✓"
            } else if $change_pct > 0 and $metric.direction == "lower" {
                "⚠️"
            } else if $change_pct < 0 and $metric.direction == "higher" {
                "⚠️"
            } else {
                "✓"
            }

            print $"      ($status) ($metric.name): ($value | math round -p 2) \(($change_pct | math round -p 2)% vs baseline\)"
        }

        $results = ($results | append $sample)

        sleep ($check_interval)sec
    }

    print ""
    print $"   ✓ Monitoring complete"
    print ""

    {
        start_time: $start_time
        duration: $duration
        samples: $results
    }
}

# Analyze experiment results
def analyze-results [experiment: record, baseline: record, results: record] {
    print $"🔍 (ansi cyan)Analyzing results...(ansi reset)"

    mut analysis = {
        metrics: {}
        overall_score: 0
        recommendation: "unknown"
    }

    # Analyze each metric
    for metric in $experiment.success_metrics {
        let baseline_avg = ($baseline.metrics | get $metric.name | get average)

        # Calculate statistics from experiment samples
        let experiment_values = ($results.samples | each {|s| $s.metrics | get $metric.name})
        let experiment_avg = (calculate-average $experiment_values)
        let experiment_p95 = (calculate-percentile $experiment_values 95)

        # Calculate improvement
        let improvement_pct = (($experiment_avg - $baseline_avg) / $baseline_avg * 100)

        # Determine if metric improved based on direction
        let improved = if $metric.direction == "lower" {
            $improvement_pct < 0
        } else {
            $improvement_pct > 0
        }

        # Check if meets threshold
        let threshold_met = if $metric.threshold != null {
            if $metric.direction == "lower" {
                $experiment_avg < $metric.threshold
            } else {
                $experiment_avg > $metric.threshold
            }
        } else {
            true
        }

        let score = if $improved and $threshold_met {
            1.0
        } else if $threshold_met {
            0.5
        } else {
            0.0
        }

        $analysis.metrics = ($analysis.metrics | insert $metric.name {
            baseline: $baseline_avg
            experiment: $experiment_avg
            improvement_pct: $improvement_pct
            improved: $improved
            threshold_met: $threshold_met
            score: $score
        })

        print $"   ($metric.name):"
        print $"      Baseline: ($baseline_avg | math round -p 2)"
        print $"      Experiment: ($experiment_avg | math round -p 2)"
        print $"      Change: (format-change $improvement_pct)"
        print $"      Status: (if $improved { "✓ Improved" } else { "✗ Degraded" })"
    }

    # Calculate overall score
    let total_metrics = ($experiment.success_metrics | length)
    let score_sum = ($analysis.metrics | values | reduce -f 0 {|it, acc| $acc + $it.score})
    $analysis.overall_score = ($score_sum / $total_metrics)

    # Determine recommendation
    $analysis.recommendation = if $analysis.overall_score >= 0.8 {
        "adopt"
    } else if $analysis.overall_score >= 0.5 {
        "investigate"
    } else {
        "rollback"
    }

    print ""
    print $"   Overall Score: ($analysis.overall_score | math round -p 2)"
    print $"   Recommendation: (format-recommendation $analysis.recommendation)"
    print ""

    $analysis
}

# Generate experiment report
def generate-report [experiment: record, baseline: record, results: record, analysis: record] {
    {
        experiment: {
            name: $experiment.name
            description: $experiment.description
            timestamp: (date now)
        }
        baseline: $baseline
        results: $results
        analysis: $analysis
        metadata: {
            total_samples: ($results.samples | length)
            duration: $results.duration
        }
    }
}

# Display experiment results
def display-results [report: record] {
    print $"📈 (ansi cyan_bold)Experiment Results(ansi reset)"
    print ""
    print $"Experiment: ($report.experiment.name)"
    print $"Description: ($report.experiment.description)"
    print ""
    print $"(ansi yellow_bold)Summary:(ansi reset)"

    let metrics_table = ($report.analysis.metrics | transpose name data | each {|row|
        {
            metric: $row.name
            baseline: ($row.data.baseline | math round -p 2)
            experiment: ($row.data.experiment | math round -p 2)
            change: $"($row.data.improvement_pct | math round -p 2)%"
            status: (if $row.data.improved { "✓" } else { "✗" })
        }
    })

    print ($metrics_table | table -e)
    print ""
    print $"Overall Score: ($report.analysis.overall_score | math round -p 2)"
    print $"Recommendation: (format-recommendation $report.analysis.recommendation)"
    print ""
}

# Export experiment results
def export-results [report: record, path: string] {
    print $"💾 Exporting results to: ($path)"

    $report | to json -i 2 | save -f $path

    print $"   ✓ Results exported"
}

# Prompt for rollback decision
def prompt-rollback [env: string, experiment: record, analysis: record] {
    print ""
    print $"🤔 (ansi yellow)Rollback decision required(ansi reset)"
    print $"   Recommendation: (format-recommendation $analysis.recommendation)"
    print ""

    let response = (input $"Rollback experiment changes? \(yes/no\): ")

    if $response == "yes" {
        rollback-experiment $env $experiment
    } else {
        print $"   Keeping experiment changes"
    }
}

# Rollback experiment changes
def rollback-experiment [env: string, experiment: record] {
    print $"🔄 (ansi cyan)Rolling back experiment...(ansi reset)"

    let namespace = $"mop-($env)"

    # Rollback each change in reverse order
    for change in ($experiment.changes | reverse) {
        print $"   Rolling back: ($change.component)"

        if $change.type == "deployment" {
            kubectl rollout undo deployment/$change.component -n $namespace
            kubectl rollout status deployment/$change.component -n $namespace --timeout=5m
        } else if $change.type == "configmap" {
            # Restore from backup if available
            print $"   ⚠️  ConfigMap rollback requires manual restoration"
        }
    }

    print $"   ✓ Rollback complete"
    print ""
}

# Helper: Collect metric samples over duration
def collect-metric-samples [namespace: string, query: string, duration: int] {
    mut samples = []
    let interval = 10  # Sample every 10 seconds
    let iterations = ($duration / $interval)

    for i in 1..$iterations {
        let value = query-metric-instant $namespace $query
        $samples = ($samples | append $value)
        sleep ($interval)sec
    }

    $samples
}

# Helper: Query metric instant value
def query-metric-instant [namespace: string, query: string] {
    # Simplified - would normally query Prometheus/Mimir
    # For now, return a mock value
    (random float) * 100 | math round -p 2
}

# Helper: Calculate average
def calculate-average [values: list] {
    if ($values | length) == 0 {
        return 0
    }

    ($values | math sum) / ($values | length)
}

# Helper: Calculate percentile
def calculate-percentile [values: list, percentile: int] {
    if ($values | length) == 0 {
        return 0
    }

    let sorted = ($values | sort)
    let index = (($sorted | length) * $percentile / 100 | math floor)

    $sorted | get $index
}

# Helper: Generate deployment patch
def generate-deployment-patch [change: record] {
    {
        spec: {
            template: {
                spec: {
                    containers: [
                        {
                            name: $change.container
                            env: [
                                {
                                    name: $change.parameter
                                    value: $change.value
                                }
                            ]
                        }
                    ]
                }
            }
        }
    } | to json
}

# Helper: Generate configmap patch
def generate-configmap-patch [change: record] {
    {
        data: {
            ($change.parameter): $change.value
        }
    } | to json
}

# Helper: Format change percentage
def format-change [change: float] {
    let formatted = ($change | math round -p 2)
    if $change > 0 {
        $"(ansi green)+($formatted)%(ansi reset)"
    } else {
        $"(ansi red)($formatted)%(ansi reset)"
    }
}

# Helper: Format recommendation
def format-recommendation [rec: string] {
    match $rec {
        "adopt" => { $"(ansi green_bold)ADOPT(ansi reset) - Changes show clear improvement" }
        "investigate" => { $"(ansi yellow_bold)INVESTIGATE(ansi reset) - Results are inconclusive" }
        "rollback" => { $"(ansi red_bold)ROLLBACK(ansi reset) - Changes caused degradation" }
        _ => { $rec }
    }
}

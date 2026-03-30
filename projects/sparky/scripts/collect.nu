#!/usr/bin/env nu
# Sparky data collection script

def main [
    --date: string,        # Date to collect (YYYY-MM-DD)
    --mode: string = "daily"  # Collection mode: daily, weekly, monthly
] {
    let collection_date = if ($date | is-empty) {
        date now | format date "%Y-%m-%d"
    } else {
        $date
    }

    print $"🚀 Sparky Data Collection - ($collection_date)"
    print $"📊 Mode: ($mode)"
    print ""

    # Create output directory
    let output_dir = $"data/raw"
    mkdir $output_dir

    # Check if GitHub CLI is available
    if (which gh | is-empty) {
        print "❌ GitHub CLI (gh) not found. Please install it first."
        print "   brew install gh     # macOS"
        print "   apt install gh      # Ubuntu/Debian"
        exit 1
    }

    # Collect repository list
    print "📦 Discovering repositories..."
    let repos = (
        gh repo list raibid-labs --limit 100 --json name,updatedAt,pushedAt
        | from json
    )

    let repo_count = ($repos | length)
    print $"   Found ($repo_count) repositories"

    # Filter active repos based on mode
    let cutoff_date = match $mode {
        "daily" => (date now | date to-record | get year month day | $"($in.year)-($in.month)-($in.day)T00:00:00Z"),
        "weekly" => (date now | date to-record | get year month day | $"($in.year)-($in.month)-($in.day)T00:00:00Z"),
        "monthly" => (date now | date to-record | get year month | $"($in.year)-($in.month)-01T00:00:00Z"),
        _ => (date now | format date "%Y-%m-%dT%H:%M:%SZ")
    }

    let active_repos = ($repos | where pushedAt >= $cutoff_date)
    let active_count = ($active_repos | length)
    print $"   ($active_count) active in collection period"

    # Save repository list
    $repos | to json | save -f $"($output_dir)/($collection_date)-repos.json"

    # Collect commits
    print ""
    print "📝 Collecting commits..."
    let commits = (collect_commits $active_repos $mode)
    let commit_count = ($commits | length)
    print $"   Collected ($commit_count) commits"
    $commits | to json | save -f $"($output_dir)/($collection_date)-commits.json"

    # Collect pull requests
    print ""
    print "🔀 Collecting pull requests..."
    let prs = (collect_prs $active_repos $mode)
    let pr_count = ($prs | length)
    print $"   Collected ($pr_count) merged PRs"
    $prs | to json | save -f $"($output_dir)/($collection_date)-prs.json"

    # Collect issues
    print ""
    print "📋 Collecting issues..."
    let issues = (collect_issues $active_repos $mode)
    let issue_count = ($issues | length)
    print $"   Collected ($issue_count) issue activities"
    $issues | to json | save -f $"($output_dir)/($collection_date)-issues.json"

    # Calculate statistics
    print ""
    print "📊 Calculating statistics..."
    let top_contributors = (
        $commits
        | get author
        | uniq --count
        | sort-by count --reverse
        | first 5
    )

    let active_repos_list = ($commits | get repo | uniq)

    let summary = {
        date: $collection_date,
        mode: $mode,
        commits: $commit_count,
        pull_requests: $pr_count,
        issues: $issue_count,
        active_repositories: ($active_repos_list | length),
        contributors: ($commits | get author | uniq | length),
        top_contributors: $top_contributors,
        active_repos: $active_repos_list
    }

    $summary | to json | save -f $"($output_dir)/($collection_date)-summary.json"

    # Display summary
    print ""
    print "✅ Collection Complete!"
    print ""
    print "Summary:"
    print $summary
    print ""
    print "Data files created:"
    print $"  - ($output_dir)/($collection_date)-repos.json"
    print $"  - ($output_dir)/($collection_date)-commits.json"
    print $"  - ($output_dir)/($collection_date)-prs.json"
    print $"  - ($output_dir)/($collection_date)-issues.json"
    print $"  - ($output_dir)/($collection_date)-summary.json"
    print ""
    print "💰 Cost: $0 (GitHub CLI is free!)"
}

def collect_commits [repos: list, mode: string] {
    let since = match $mode {
        "daily" => "24 hours ago",
        "weekly" => "7 days ago",
        "monthly" => "30 days ago",
        _ => "24 hours ago"
    }

    mut all_commits = []

    for repo in $repos {
        try {
            let commits = (
                gh api $"repos/raibid-labs/($repo.name)/commits" `
                    --jq $'.[].commit | select(.author.date >= "($since)") | {
                        repo: "($repo.name)",
                        sha: .tree.sha,
                        author: .author.name,
                        email: .author.email,
                        date: .author.date,
                        message: .message
                    }'
                | from json
            )
            $all_commits = ($all_commits | append $commits)
        } catch {
            # Skip repos with no commits
        }
    }

    $all_commits
}

def collect_prs [repos: list, mode: string] {
    mut all_prs = []

    for repo in $repos {
        try {
            let prs = (
                gh pr list --repo $"raibid-labs/($repo.name)" `
                    --state merged `
                    --limit 50 `
                    --json number,title,author,mergedAt,url
                | from json
                | insert repo $repo.name
            )
            $all_prs = ($all_prs | append $prs)
        } catch {
            # Skip repos with no PRs
        }
    }

    $all_prs
}

def collect_issues [repos: list, mode: string] {
    mut all_issues = []

    for repo in $repos {
        try {
            let issues = (
                gh issue list --repo $"raibid-labs/($repo.name)" `
                    --limit 50 `
                    --json number,title,author,createdAt,updatedAt,state,url
                | from json
                | insert repo $repo.name
            )
            $all_issues = ($all_issues | append $issues)
        } catch {
            # Skip repos with no issues
        }
    }

    $all_issues
}

main

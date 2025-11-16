#!/usr/bin/env nu

# DGX Music Orchestrator
# Monitors GitHub issues and spawns development agents

def main [] {
    print "🎵 DGX Music Orchestrator Starting..."

    # Check prerequisites
    check_prerequisites

    # Main orchestration loop
    loop {
        print $"(date now | format date '%Y-%m-%d %H:%M:%S') - Checking for work..."

        # Get open issues
        let issues = get_open_issues

        # Process each issue
        for issue in $issues {
            process_issue $issue
        }

        # Wait before next poll
        sleep 60sec
    }
}

def check_prerequisites [] {
    # Check gh CLI
    if (which gh | is-empty) {
        print "❌ gh CLI not found. Install with: brew install gh"
        exit 1
    }

    # Check Claude Code
    if (which claude | is-empty) {
        print "⚠️  Claude Code CLI not found. Orchestrator may not work."
    }

    print "✅ Prerequisites checked"
}

def get_open_issues [] {
    # Get open issues from GitHub
    let issues_json = (
        gh issue list
            --repo raibid-labs/dgx-music
            --state open
            --json number,title,labels,body,comments
    )

    $issues_json | from json
}

def process_issue [issue: record] {
    let issue_number = $issue.number
    let title = $issue.title
    let labels = ($issue.labels | each { |l| $l.name })

    print $"📋 Issue #($issue_number): ($title)"

    # Check if issue is ready for work
    if not (is_ready_for_work $issue) {
        print "   ⏸️  Not ready (waiting for answers or draft)"
        return
    }

    # Check if agent already spawned
    if (has_spawn_trigger $issue) {
        print "   ✅ Agent already spawned"
        return
    }

    # Spawn agent for this issue
    spawn_agent $issue
}

def is_ready_for_work [issue: record] -> bool {
    let labels = ($issue.labels | each { |l| $l.name })

    # Skip draft issues
    if ("draft" in $labels or "status:draft" in $labels) {
        return false
    }

    # Skip if waiting for answers
    if ("waiting:answers" in $labels) {
        # Check if answers provided in recent comments
        return (check_for_answers $issue)
    }

    true
}

def check_for_answers [issue: record] -> bool {
    let comments = $issue.comments

    # Check last comment for numbered answers
    if ($comments | length) == 0 {
        return false
    }

    let last_comment = ($comments | last)
    let body = $last_comment.body

    # Check for numbered list format
    ($body | str contains "1. ")
}

def has_spawn_trigger [issue: record] -> bool {
    let comments = $issue.comments

    # Check for spawn trigger comment
    $comments | any { |c|
        $c.body | str contains "🤖 SPAWN_TRIGGER"
    }
}

def spawn_agent [issue: record] {
    let issue_number = $issue.number
    let title = $issue.title

    print $"🚀 Spawning agent for issue #($issue_number)..."

    # Post spawn trigger comment
    gh issue comment $issue_number
        --repo raibid-labs/dgx-music
        --body "🤖 SPAWN_TRIGGER: orchestrator-auto-spawn"

    # TODO: Actually spawn Claude Code agent via Task tool
    # For now, just log the action
    print $"✅ Trigger posted for issue #($issue_number)"
}

# Run the orchestrator
main

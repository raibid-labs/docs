#!/usr/bin/env nu
# Raibid-CI Multi-Agent Development Orchestrator Launcher
#
# This script launches the Claude Code orchestrator agent that coordinates
# multi-agent parallel development for the raibid-ci project.
#
# Usage: nu scripts/launch-orchestrator.nu
#
# Prerequisites:
# - Claude Code CLI installed and authenticated
# - GitHub CLI (gh) installed and authenticated
# - Repository properly configured (see docs/SETUP_COMPLETE.md)

def main [] {
    print "🚀 Raibid-CI Multi-Agent Orchestrator Launcher\n"

    # Check prerequisites
    print "📋 Checking prerequisites..."
    check-prerequisites

    # Show current status
    print "\n📊 Current Status:"
    show-project-status

    # Confirm launch
    print "\n⚠️  About to launch orchestrator agent that will:"
    print "   1. Monitor GitHub issues every 5 minutes"
    print "   2. Detect answered clarifying questions"
    print "   3. Spawn development agents for available work"
    print "   4. Track progress and manage dependencies"
    print "   5. Post status updates and progress reports"

    let confirm = (input "\n❓ Launch orchestrator? [y/N]: ")

    if $confirm != "y" and $confirm != "Y" {
        print "❌ Cancelled."
        exit 0
    }

    print "\n🎯 Launching orchestrator agent...\n"
    launch-orchestrator
}

# Check that required tools are available
def check-prerequisites [] {
    # Check for gh CLI
    try {
        gh --version | complete
        print "  ✓ GitHub CLI (gh) available"
    } catch {
        print "  ✗ GitHub CLI (gh) not found"
        print "    Install: https://cli.github.com/"
        exit 1
    }

    # Check gh authentication
    try {
        gh auth status | complete
        print "  ✓ GitHub CLI authenticated"
    } catch {
        print "  ✗ GitHub CLI not authenticated"
        print "    Run: gh auth login"
        exit 1
    }

    # Check repository
    try {
        let repo = (gh repo view --json nameWithOwner | from json | get nameWithOwner)
        print $"  ✓ Repository: ($repo)"
    } catch {
        print "  ✗ Not in a GitHub repository"
        exit 1
    }

    # Check for documentation
    if not ("docs/ORCHESTRATOR_AGENT.md" | path exists) {
        print "  ✗ Missing docs/ORCHESTRATOR_AGENT.md"
        exit 1
    }
    print "  ✓ Orchestrator documentation found"

    if not ("docs/CLARIFYING_QUESTIONS.md" | path exists) {
        print "  ✗ Missing docs/CLARIFYING_QUESTIONS.md"
        exit 1
    }
    print "  ✓ Clarifying questions document found"
}

# Show current project status
def show-project-status [] {
    # Count open issues
    let issues = (gh issue list --state open --json number,title | from json)
    let issue_count = ($issues | length)
    print $"  📌 Open issues: ($issue_count)"

    # Count issues with unanswered questions
    let paused_count = ($issues | where {|issue|
        let has_pause_label = (gh issue view $issue.number --json labels |
                               from json |
                               get labels |
                               any {|label| $label.name == "status:paused"})
        $has_pause_label
    } | length)

    if $paused_count > 0 {
        print $"  ⏸️  Issues with unanswered questions: ($paused_count)"
    }

    # Show WS-01 issues
    let ws01_issues = ($issues | where {|issue|
        $issue.title | str starts-with "CLI-"
    })

    if ($ws01_issues | length) > 0 {
        print "\n  🎯 WS-01: CLI/TUI Application Issues:"
        for issue in $ws01_issues {
            print $"     #($issue.number): ($issue.title)"
        }
    }
}

# Launch the orchestrator agent
def launch-orchestrator [] {
    print "═══════════════════════════════════════════════════════════════"
    print "🤖 ORCHESTRATOR AGENT INSTRUCTIONS"
    print "═══════════════════════════════════════════════════════════════\n"

    print "You are the Orchestrator Agent for raibid-ci. Your instructions are in:"
    print "  📄 docs/ORCHESTRATOR_AGENT.md"
    print "  📄 docs/CLARIFYING_QUESTIONS.md"
    print "  📄 docs/ORCHESTRATION.md\n"

    print "🎯 YOUR IMMEDIATE TASKS:\n"
    print "1. Monitor GitHub issues every 5 minutes"
    print "   - Check for unanswered clarifying questions"
    print "   - Detect when questions are answered"
    print "   - Track agent states and dependencies\n"

    print "2. Review WS-01 issues (CLI-001 through CLI-008)"
    print "   - All have clarifying questions posted"
    print "   - None should be started until questions answered"
    print "   - Post acknowledgment on each issue\n"

    print "3. When questions are answered:"
    print "   - Spawn development agents using Claude Code's Task tool"
    print "   - Example:"
    print "     Task(\"CLI Developer for CLI-001\","
    print "          \"Complete CLI-001. Check issue #1 for questions.\","
    print "          \"rust-pro\")\n"

    print "4. Track and report:"
    print "   - Agent states (AVAILABLE, ASSIGNED, PAUSED, ACTIVE, etc.)"
    print "   - Question answer turnaround time"
    print "   - Work completion progress"
    print "   - Any blockers or issues\n"

    print "📊 SUCCESS METRICS:"
    print "   - Agent utilization >70%"
    print "   - Question turnaround <4 hours"
    print "   - Issue completion: 2-3 per day (team of 4-6 agents)"
    print "   - PR cycle time <24 hours"
    print "   - Zero untested code\n"

    print "🔄 MONITORING LOOP (every 5 minutes):"
    print "   gh issue list --state open --json number,title,body,comments,updatedAt"
    print "   - Check for new answers"
    print "   - Resume paused agents"
    print "   - Spawn new agents for available work"
    print "   - Post status updates\n"

    print "═══════════════════════════════════════════════════════════════"
    print "🎼 Begin orchestration! Keep the development flowing smoothly."
    print "═══════════════════════════════════════════════════════════════\n"

    print "💡 Tip: You can monitor progress with:"
    print "   gh issue list"
    print "   gh pr list"
    print "   gh run list\n"

    print "⚠️  IMPORTANT: This script provides instructions. To actually spawn"
    print "   the orchestrator agent in Claude Code, you need to use the Task tool:"
    print "\n   Task(\"Orchestrator\","
    print "        \"Follow instructions in docs/ORCHESTRATOR_AGENT.md. Monitor issues\","
    print "        \"tdd-orchestrator\")\n"

    print "✅ Setup complete. Orchestrator instructions displayed above."
    print "   Ready to begin multi-agent development!\n"
}

# Run main
main

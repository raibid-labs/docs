#!/usr/bin/env nu
# Agent Spawner - Automated Claude Code agent spawning from GitHub issues
#
# This script monitors GitHub issues for ORCHESTRATOR-SPAWN-AGENT comments
# and provides the data needed to spawn agents via Claude Code Task tool

export def main [
    --once              # Run once and exit (default: continuous)
    --interval: int = 30 # Polling interval in seconds
    --issue: int        # Spawn agent for specific issue
    --dry-run           # Show what would be spawned without spawning
] {
    print $"(ansi blue)🤖 Agent Spawner(ansi reset)"
    print $"(ansi blue)===============(ansi reset)"
    print $"Timestamp: (date now | format date '%Y-%m-%d %H:%M:%S')"
    print ""

    # Initialize state
    let state_file = "/tmp/raibid_agent_spawner_state.json"
    init_state $state_file

    if $dry_run {
        print $"(ansi yellow)⚠ DRY RUN MODE - No agents will actually spawn(ansi reset)"
    }

    if $once {
        scan_and_spawn $state_file $issue $dry_run
    } else {
        print $"(ansi blue)ℹ Running in continuous mode (interval: ($interval)s)(ansi reset)"
        loop {
            scan_and_spawn $state_file $issue $dry_run
            print ""
            print $"(ansi blue)ℹ Sleeping for ($interval)s...(ansi reset)"
            sleep ($interval * 1sec)
            print ""
        }
    }
}

# Initialize state file
def init_state [state_file: string] {
    if not ($state_file | path exists) {
        {spawned_agents: {}} | save $state_file
    }
}

# Load state from file
def load_state [state_file: string]: nothing -> record {
    open $state_file
}

# Save state to file
def save_state [state_file: string, state: record] {
    $state | save -f $state_file
}

# Check if agent already spawned
def is_spawned [state: record, issue: int]: bool {
    $issue | into string | in ($state.spawned_agents)
}

# Mark agent as spawned
def mark_spawned [state_file: string, issue: int] {
    let state = load_state $state_file
    let timestamp = date now | format date '%Y-%m-%dT%H:%M:%SZ'
    let updated = $state | update spawned_agents { |s|
        $s.spawned_agents | insert ($issue | into string) $timestamp
    }
    save_state $state_file $updated
}

# Get spawn comment from issue
def get_spawn_comment [issue: int]: string {
    let comments = gh issue view $issue --json comments | from json | get comments

    $comments
        | where body =~ "ORCHESTRATOR-SPAWN-AGENT"
        | first
        | get body
}

# Parse spawn comment for agent details
def parse_spawn_comment [comment: string]: record {
    let issue = $comment | parse -r '\*\*Issue\*\*: #(?P<num>\d+)' | get num.0 | into int
    let issue_id = $comment | parse -r '\*\*Issue ID\*\*: (?P<id>[\w-]+)' | get id.0
    let agent_type = $comment | parse -r '\*\*Type\*\*: (?P<type>[\w-]+)' | get type.0
    let status = $comment | parse -r '\*\*Status\*\*: (?P<status>\w+)' | get status.0

    {
        issue: $issue,
        issue_id: $issue_id,
        agent_type: $agent_type,
        status: $status
    }
}

# Get issue details
def get_issue_details [issue: int]: record {
    gh issue view $issue --json title,body,labels | from json
}

# Create agent spawn instructions
def create_spawn_instructions [issue: int, title: string, agent_type: string]: string {
    $"You are working on Issue #($issue): ($title)

**Your Mission:**
Complete this issue following the instructions provided.

**Read the Full Issue:**
Visit https://github.com/raibid-labs/raibid-ci/issues/($issue) and read all details, tasks, and acceptance criteria.

**Workflow:**
1. Create appropriate feature branch
2. Follow TDD where applicable \(write tests first\)
3. Implement required changes
4. Ensure all acceptance criteria met
5. Create PR referencing #($issue)

**Important:**
- Read the issue carefully before starting
- Follow project conventions in CLAUDE.md
- Write clear commit messages
- Test your changes thoroughly
- Update documentation as needed

Begin by reading issue #($issue) in detail, then proceed with implementation."
}

# Generate Task tool command for spawning
def generate_task_command [issue: int, title: string, agent_type: string]: string {
    let instructions = create_spawn_instructions $issue $title $agent_type
    let description = $"Issue #($issue)"

    $"Task\(\"($description)\", \"($instructions)\", \"($agent_type)\"\)"
}

# Process a single issue
def process_issue [state_file: string, issue: int, dry_run: bool] {
    let state = load_state $state_file

    # Check if already spawned
    if (is_spawned $state $issue) {
        return
    }

    # Get spawn comment
    let spawn_comment = try { get_spawn_comment $issue } catch { return }

    if ($spawn_comment | is-empty) {
        return
    }

    # Parse spawn comment
    let parsed = parse_spawn_comment $spawn_comment

    # Verify status
    if $parsed.status != "ready" {
        print $"(ansi yellow)⚠ Issue #($issue) status is '($parsed.status)', not 'ready'(ansi reset)"
        return
    }

    # Get issue details
    let details = get_issue_details $issue

    # Generate spawn command
    let task_command = generate_task_command $issue $details.title $parsed.agent_type

    print $"(ansi purple)🤖 Ready to spawn agent for Issue #($issue)(ansi reset)"
    print $"   Workstream: ($parsed.issue_id)"
    print $"   Agent Type: ($parsed.agent_type)"
    print $"   Title: ($details.title)"
    print ""
    print $"(ansi blue)Task Command:(ansi reset)"
    print $task_command
    print ""

    if $dry_run {
        print $"(ansi yellow)⚠ DRY RUN - Not spawning(ansi reset)"
    } else {
        # In a real implementation, this would call Claude Code's Task tool
        # For now, we save the command to a file that can be picked up
        let spawn_file = $"/tmp/spawn_agent_($issue).txt"
        $task_command | save -f $spawn_file
        print $"(ansi green)✓ Spawn command saved to: ($spawn_file)(ansi reset)"
        mark_spawned $state_file $issue
    }
}

# Scan issues and spawn agents
def scan_and_spawn [state_file: string, specific_issue: int, dry_run: bool] {
    print $"(ansi blue)ℹ Scanning for spawn-ready issues...(ansi reset)"

    let issues = if ($specific_issue | is-not-empty) {
        [$specific_issue]
    } else {
        gh issue list --state open --json number
            | from json
            | get number
    }

    let spawn_count = $issues | each { |issue|
        try {
            process_issue $state_file $issue $dry_run
            1
        } catch {
            0
        }
    } | math sum

    if $spawn_count == 0 {
        print $"(ansi blue)ℹ No new agents spawned(ansi reset)"
    } else {
        print $"(ansi green)✓ Generated ($spawn_count) spawn command\(s\)(ansi reset)"
    }

    # Show current state
    let state = load_state $state_file
    let spawned_count = $state.spawned_agents | length
    print $"(ansi blue)ℹ Currently tracking ($spawned_count) spawned agent\(s\)(ansi reset)"
}

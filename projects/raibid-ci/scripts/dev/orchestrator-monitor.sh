#!/bin/bash
# Orchestrator Monitoring Script for raibid-ci WS-01 Issues

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track last check timestamp
LAST_CHECK_FILE="/tmp/raibid_orchestrator_last_check"
STATE_FILE="/tmp/raibid_orchestrator_state.json"

# Initialize state file if not exists
if [ ! -f "$STATE_FILE" ]; then
  echo '{"issues": {}}' > "$STATE_FILE"
fi

echo -e "${BLUE}🤖 Orchestrator Monitor - raibid-ci${NC}"
echo -e "${BLUE}===============================>${NC}"
echo "Timestamp: $(date)"
echo ""

# Function to check if questions are answered
check_questions_answered() {
  local issue_num=$1
  local comments_json=$(gh issue view $issue_num --json comments)
  local comment_count=$(echo "$comments_json" | jq -r '.comments | length')

  # If more than 2 comments (1 questions, 1 orchestrator ack), might have answers
  if [ $comment_count -gt 2 ]; then
    # Check if any comment contains answers (looks for "A1:", "Answer:", etc.)
    local has_answers=$(echo "$comments_json" | jq -r '.comments[].body' | grep -E "(^A[0-9]+:|^Answer:|^Response:|^Decision:)" || true)
    if [ ! -z "$has_answers" ]; then
      return 0  # Has answers
    fi
  fi
  return 1  # No answers yet
}

# Function to get issue status
get_issue_status() {
  local issue_num=$1
  local title=$(gh issue view $issue_num --json title | jq -r '.title')
  local comment_count=$(gh issue view $issue_num --json comments | jq -r '.comments | length')
  local labels=$(gh issue view $issue_num --json labels | jq -r '.labels | map(.name) | join(", ")')

  echo -n "Issue #$issue_num: ${title:0:40}... "
  echo -n "[$comment_count comments]"

  if check_questions_answered $issue_num; then
    echo -e " ${GREEN}✓ ANSWERED${NC}"
    return 0
  else
    echo -e " ${YELLOW}⏸ WAITING${NC}"
    return 1
  fi
}

# Function to spawn agent for answered issue
spawn_agent_for_issue() {
  local issue_num=$1
  local issue_title=$(gh issue view $issue_num --json title | jq -r '.title')
  local issue_id=$(echo "$issue_title" | cut -d: -f1)

  echo -e "${GREEN}🚀 Spawning agent for $issue_id (#$issue_num)${NC}"

  # Post resumption comment
  cat > /tmp/resume_comment.md << EOF
🚀 **Resumption Signal**

Questions have been answered! Spawning development agent.

**Status Change:** ⏸️ PAUSED → ▶️ IN PROGRESS
**Agent Type:** rust-pro
**Workflow:** TDD (Test-Driven Development)
**Branch:** ${issue_id,,}-implementation

The development agent will:
1. Check out the implementation branch
2. Write tests first (TDD approach)
3. Implement features to pass tests
4. Submit PR when complete

---
*Orchestrator Agent v1.0 | Agent Spawned*
EOF

  gh issue comment $issue_num -F /tmp/resume_comment.md

  # Mark in state file that we've spawned an agent
  jq --arg issue "$issue_num" '.issues[$issue] = "agent_spawned"' "$STATE_FILE" > /tmp/state_tmp.json
  mv /tmp/state_tmp.json "$STATE_FILE"
}

# Main monitoring loop
echo -e "${BLUE}Checking WS-01 Issues Status:${NC}"
echo "----------------------------------------"

for issue_num in 1 2 3 4 5 6 7 8; do
  # Check if we've already spawned an agent for this issue
  already_spawned=$(jq -r --arg issue "$issue_num" '.issues[$issue]' "$STATE_FILE")

  if [ "$already_spawned" = "agent_spawned" ]; then
    echo -e "Issue #$issue_num: ${GREEN}✓ Agent already spawned${NC}"
  else
    if get_issue_status $issue_num; then
      # Questions answered, spawn agent
      spawn_agent_for_issue $issue_num
    fi
  fi
done

echo ""
echo "----------------------------------------"
echo -e "${BLUE}Summary:${NC}"
echo "- Total Issues: 8"
echo "- Agents Spawned: $(jq -r '.issues | to_entries | map(select(.value == "agent_spawned")) | length' "$STATE_FILE")"
echo "- Waiting for Answers: $(( 8 - $(jq -r '.issues | to_entries | map(select(.value == "agent_spawned")) | length' "$STATE_FILE") ))"
echo ""
echo "Next check in 5 minutes..."

# Update last check file
date > "$LAST_CHECK_FILE"
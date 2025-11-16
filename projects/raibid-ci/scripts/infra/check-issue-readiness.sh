#!/bin/bash
# Check if issue is ready for agent to start work
# Analyzes clarifying questions and determines if all are answered

set -e

ISSUE_NUM="${ISSUE_NUMBER}"

echo "Checking readiness for issue #${ISSUE_NUM}..."

# Fetch issue details with comments
ISSUE_JSON=$(gh issue view "$ISSUE_NUM" --json title,body,comments,labels)

# Extract issue body
ISSUE_BODY=$(echo "$ISSUE_JSON" | jq -r '.body')

# Check if issue has a "Clarifying Questions" section
if ! echo "$ISSUE_BODY" | grep -q "## Clarifying Questions"; then
  echo "✅ No clarifying questions section found - issue is ready"
  echo "ready=true" >> $GITHUB_OUTPUT
  echo "unanswered_count=0" >> $GITHUB_OUTPUT
  exit 0
fi

# Extract clarifying questions section
QUESTIONS=$(echo "$ISSUE_BODY" | sed -n '/## Clarifying Questions/,/^## /p' | grep -E '^\*?\*?[0-9]+\.' || true)

if [ -z "$QUESTIONS" ]; then
  echo "✅ No questions found in Clarifying Questions section - issue is ready"
  echo "ready=true" >> $GITHUB_OUTPUT
  echo "unanswered_count=0" >> $GITHUB_OUTPUT
  exit 0
fi

# Count total questions
TOTAL_QUESTIONS=$(echo "$QUESTIONS" | wc -l)
echo "Found $TOTAL_QUESTIONS clarifying questions"

# Extract all comments
COMMENTS=$(echo "$ISSUE_JSON" | jq -r '.comments[].body')

# Check each question for answers
UNANSWERED=0
UNANSWERED_LIST=""

while IFS= read -r question; do
  # Extract question number (handle both "1." and "**1.**" formats)
  QUESTION_NUM=$(echo "$question" | sed -E 's/^\*?\*?([0-9]+)\..*/\1/')

  # Check comments for answers to this question
  # Look for patterns: A1:, Answer 1:, Q1: ... A:, 1., **Answer:**
  ANSWER=$(echo "$COMMENTS" | grep -iE "(^A$QUESTION_NUM:|^Answer $QUESTION_NUM:|^Q$QUESTION_NUM:.*A:|^$QUESTION_NUM\.|^\*\*Answer\*\*.*$QUESTION_NUM)" || true)

  if [ -z "$ANSWER" ]; then
    echo "❌ Question $QUESTION_NUM unanswered"
    UNANSWERED=$((UNANSWERED + 1))
    UNANSWERED_LIST="$UNANSWERED_LIST\n- Question $QUESTION_NUM"
  else
    echo "✅ Question $QUESTION_NUM answered"
  fi
done <<< "$QUESTIONS"

# Determine readiness
if [ $UNANSWERED -eq 0 ]; then
  echo "✅ All $TOTAL_QUESTIONS questions answered - issue is ready!"
  echo "ready=true" >> $GITHUB_OUTPUT
  echo "unanswered_count=0" >> $GITHUB_OUTPUT
else
  echo "⏸️ $UNANSWERED of $TOTAL_QUESTIONS questions still unanswered"
  echo "ready=false" >> $GITHUB_OUTPUT
  echo "unanswered_count=$UNANSWERED" >> $GITHUB_OUTPUT
  echo "total_questions=$TOTAL_QUESTIONS" >> $GITHUB_OUTPUT
fi

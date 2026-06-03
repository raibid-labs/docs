#!/usr/bin/env bash
#
# apply-branch-protection.sh
#
# Applies the branch-protection ruleset defined in .github/branch-protection.json
# to the `main` branch of raibid-labs/docs.
#
# REQUIRES: a repository ADMIN token. Branch protection cannot be configured by
# a normal contributor or by the default GITHUB_TOKEN in a workflow — a human
# admin (or an admin PAT) must run this. See docs/RELEASE.md "Branch Protection".
#
# Usage:
#   ./scripts/apply-branch-protection.sh [owner/repo] [branch]
#
# Defaults: owner/repo = raibid-labs/docs, branch = main
#
set -euo pipefail

REPO="${1:-raibid-labs/docs}"
BRANCH="${2:-main}"
POLICY_FILE="$(dirname "$0")/../.github/branch-protection.json"

if [ ! -f "$POLICY_FILE" ]; then
  echo "ERROR: policy file not found: $POLICY_FILE" >&2
  exit 1
fi

echo "Applying branch protection to ${REPO}@${BRANCH} from ${POLICY_FILE}"

# Strip the documentation-only fields and the top-level "branch" key, leaving the
# exact payload the GitHub REST API expects, then PUT it.
jq 'del(._comment, .branch)' "$POLICY_FILE" \
  | gh api \
      --method PUT \
      -H "Accept: application/vnd.github+json" \
      "/repos/${REPO}/branches/${BRANCH}/protection" \
      --input -

echo "Branch protection applied to ${REPO}@${BRANCH}."
echo "Verify in the GitHub UI: Settings -> Branches -> Branch protection rules."

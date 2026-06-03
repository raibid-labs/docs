# Branch Protection Policy

This document defines the branch-protection ruleset for the `main` branch of
`raibid-labs/docs` and explains how to apply it. The machine-readable source of
truth is [`.github/branch-protection.json`](../.github/branch-protection.json).

> **Note:** Branch protection rules live in GitHub repository settings and
> **cannot** be committed into the repo to take effect. A repository **admin**
> must apply this policy once (and re-apply if it changes). The files in this
> repo document the intended state and automate applying it.

## Policy summary

| Setting | Value | Rationale |
| --- | --- | --- |
| Require a pull request before merging | Yes | No direct pushes to `main`. |
| Required approving reviews | 1 | At least one reviewer. |
| Dismiss stale approvals on new commits | Yes | Re-review after changes. |
| Require review from Code Owners | Yes | Enforces `.github/CODEOWNERS`. |
| Require status checks to pass | Yes | CI gates merges. |
| Require branches to be up to date (`strict`) | Yes | No merging stale branches. |
| Require linear history | Yes | Clean, bisectable history. |
| Require conversation resolution | Yes | No unresolved review threads. |
| Allow force pushes | No | Protects history. |
| Allow deletions | No | Protects the branch. |
| Require signed commits | No (optional) | Enable if the org mandates GPG/SSH signing. |
| Include administrators (`enforce_admins`) | No | Admins may bypass for emergencies; flip to `true` to enforce on admins too. |

## Required status checks

These are the **job names** from
[`.github/workflows/docs-lint.yml`](../.github/workflows/docs-lint.yml). The
strings must match exactly, or GitHub will wait forever for a check that never
reports:

- `Markdown Linting`
- `Link Validation`
- `Front Matter Validation`
- `Structure Validation`
- `Phage Policy Enforcement`

If you rename a job in the workflow, update both `branch-protection.json` and
this table, then re-apply the policy.

## Applying the policy

### Option A — script (recommended)

Requires an admin token authenticated with `gh` (or `GH_TOKEN`/`GITHUB_TOKEN`
with admin scope on the repo):

```bash
./scripts/apply-branch-protection.sh raibid-labs/docs main
```

### Option B — GitHub UI (human-only)

1. Go to **Settings → Branches → Add branch protection rule**.
2. Branch name pattern: `main`.
3. Enable the settings from the table above.
4. Under **Require status checks to pass**, add each required check listed above.
5. Save.

> The default `GITHUB_TOKEN` available inside Actions **cannot** set branch
> protection. This step is intentionally manual / admin-only.

## Verifying

```bash
gh api /repos/raibid-labs/docs/branches/main/protection | jq '{
  required_status_checks: .required_status_checks.contexts,
  reviews: .required_pull_request_reviews.required_approving_review_count,
  linear: .required_linear_history.enabled,
  force_pushes: .allow_force_pushes.enabled
}'
```

## Related

- [Release Process](./RELEASE.md#branch-protection)
- [Contributing Guidelines](./CONTRIBUTING.md)
- [CODEOWNERS](../.github/CODEOWNERS)

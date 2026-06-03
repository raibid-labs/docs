# Release Process

This document describes the release process for the raibid-labs/docs documentation hub.

## Overview

The docs repository uses semantic versioning and automated release workflows to create tagged releases with versioned documentation snapshots.

## Release Workflow

### Automated Process

The release workflow (`.github/workflows/release.yml`) handles:

1. **Validation** - Verify version format and prerequisites
2. **Build** - Compile site and run tests
3. **Version Creation** - Create versioned documentation snapshot
4. **Release** - Generate changelog and create GitHub release
5. **Artifacts** - Package and publish build artifacts

### Triggering a Release

**Option 1: Git Tag (Recommended)**

```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

**Option 2: Manual Workflow Dispatch**

Via GitHub UI:
1. Go to **Actions** → **Release** workflow
2. Click **Run workflow**
3. Enter version (e.g., `v1.0.0`)
4. Click **Run workflow**

Via GitHub CLI:

```bash
gh workflow run release.yml -f version=v1.0.0
```

## Versioning Scheme

This project follows [Semantic Versioning](https://semver.org/):

```
vMAJOR.MINOR.PATCH
```

- **MAJOR** - Breaking changes or major restructure
- **MINOR** - New features, content additions
- **PATCH** - Bug fixes, typos, small improvements

### Examples

- `v1.0.0` - First stable release
- `v1.1.0` - Added new documentation sections
- `v1.1.1` - Fixed broken links and typos
- `v2.0.0` - Major restructure of documentation organization

## Pre-Release Checklist

Before creating a release, ensure:

### 1. Documentation is Ready

- [ ] All intended changes merged to `main`
- [ ] Documentation in `docs/versions/vNEXT/` is complete
- [ ] No broken links (run `npm run build` locally)
- [ ] Front matter is valid
- [ ] Code examples are tested

### 2. Version Planning

- [ ] Determine version number (major/minor/patch)
- [ ] Review changes since last release
- [ ] Update any version-specific references

### 3. Quality Checks

- [ ] All CI checks passing on `main`
- [ ] Documentation linting passes
- [ ] Build succeeds locally
- [ ] Site renders correctly (`npm run dev`)

### 4. Branch Protection

- [ ] Ensure `main` branch is up to date
- [ ] No uncommitted changes
- [ ] All PRs merged

## Release Steps

### Step 1: Prepare the Release

```bash
# Ensure you're on main and up to date
git checkout main
git pull origin main

# Verify build works
npm install
npm run check
npm run build

# Test the site locally
npm run dev
# Visit http://localhost:8080 and verify
```

### Step 2: Create the Tag

```bash
# Create annotated tag with release notes
git tag -a v1.0.0 -m "Release v1.0.0

- Added comprehensive documentation structure
- Implemented versioning system
- Enhanced governance policies"

# Push the tag
git push origin v1.0.0
```

### Step 3: Monitor the Workflow

1. Go to **Actions** tab on GitHub
2. Watch the **Release** workflow execute
3. Verify all jobs complete successfully:
   - ✅ Validate Release
   - ✅ Build and Test
   - ✅ Create Versioned Documentation
   - ✅ Create GitHub Release

### Step 4: Verify the Release

After workflow completes:

1. **Check GitHub Release**
   - Go to **Releases** tab
   - Verify release appears with correct version
   - Check release notes and changelog
   - Verify artifacts are attached

2. **Check Versioned Documentation**
   - Pull latest changes: `git pull origin main`
   - Verify `docs/versions/v1.0.0/` exists
   - Check `docs/versions.json` is updated
   - Verify `docs/versions/latest` symlink points to new version

3. **Check Live Site**
   - Visit https://raibid-labs.github.io/docs
   - Verify site is updated (may take a few minutes)

### Step 5: Post-Release Tasks

- [ ] Announce release (if applicable)
- [ ] Update dependent repositories
- [ ] Create new `vNEXT` if needed
- [ ] Close related issues/milestones

## Versioned Documentation

### How It Works

When a release is created, the workflow:

1. **Copies `vNEXT` to versioned directory**
   ```
   docs/versions/vNEXT/ → docs/versions/v1.0.0/
   ```

2. **Updates `versions.json` manifest**
   ```json
   {
     "latest": "v1.0.0",
     "versions": [
       {
         "name": "v1.0.0",
         "released": "2025-12-05",
         "status": "stable"
       }
     ]
   }
   ```

3. **Creates/updates `latest` symlink**
   ```
   docs/versions/latest → v1.0.0
   ```

### Post-Release vNEXT

After a release, `vNEXT` becomes the working directory for the next release:

```bash
# vNEXT remains for development
docs/versions/
├── v1.0.0/        # Frozen release
├── vNEXT/         # Next release development
└── latest → v1.0.0
```

New changes should go to `vNEXT` until the next release.

## Release Artifacts

Each release includes:

### 1. Git Tag

- Format: `vX.Y.Z`
- Points to commit at release time
- Permanent reference

### 2. GitHub Release

- Release notes with changelog
- Markdown formatted
- Links to documentation

### 3. Build Artifacts

- `docs-vX.Y.Z.tar.gz` - Compiled site
- `docs-vX.Y.Z.tar.gz.sha256` - Checksum file

### 4. Versioned Documentation

- Snapshot in `docs/versions/vX.Y.Z/`
- Entry in `versions.json`
- Updated `latest` symlink

## Hotfix Releases

For urgent fixes to released versions:

### 1. Create Hotfix Branch

```bash
git checkout -b hotfix/v1.0.1 v1.0.0
```

### 2. Make Fixes

```bash
# Fix the issue
vim docs/versions/v1.0.0/some-file.md

# Commit
git add docs/versions/v1.0.0/
git commit -m "fix: correct broken link in installation guide"
```

### 3. Merge to Main

```bash
# Create PR and merge to main
gh pr create --base main --head hotfix/v1.0.1
```

### 4. Tag Hotfix Release

```bash
git checkout main
git pull
git tag v1.0.1
git push origin v1.0.1
```

## Rollback Process

If a release has critical issues:

### Option 1: Create Patch Release

Preferred - fix forward:

```bash
# Fix the issue
# Create v1.0.1 with fixes
git tag v1.0.1
git push origin v1.0.1
```

### Option 2: Delete Release (Extreme Cases)

**Warning**: Avoid if possible. Only for critical issues.

```bash
# Delete remote tag
git push --delete origin v1.0.0

# Delete local tag
git tag -d v1.0.0

# Delete GitHub release via UI or CLI
gh release delete v1.0.0
```

## Changelog Generation

The release workflow automatically generates a changelog from commit messages.

### Commit Message Format

Use conventional commits for better changelogs:

```
type(scope): description

body

footer
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Formatting, styling
- `refactor` - Code restructuring
- `test` - Testing
- `chore` - Maintenance

**Examples:**

```
feat(guides): add installation guide for Windows

Added step-by-step Windows installation instructions with
screenshots and troubleshooting section.

Closes #123
```

```
fix(links): correct broken API reference links

Updated links in getting-started.md to point to correct
API documentation sections.
```

## Branch Protection

Release process requires branch protection on `main`. The full policy, the
machine-readable ruleset, and the apply/verify steps are documented in
[BRANCH_PROTECTION.md](./BRANCH_PROTECTION.md).

Summary:

- ✅ Branch protection on `main` — no direct pushes
- ✅ Pull request with at least one approving review (Code Owner review enforced)
- ✅ Required status checks (job names from `docs-lint.yml`):
  - `Markdown Linting`
  - `Link Validation`
  - `Front Matter Validation`
  - `Structure Validation`
  - `Phage Policy Enforcement`
- ✅ Linear history; force pushes and deletions disabled

> Applying branch protection is an **admin-only** action. Run
> `./scripts/apply-branch-protection.sh` with an admin token, or configure it in
> **Settings → Branches**. See [BRANCH_PROTECTION.md](./BRANCH_PROTECTION.md).

## Permissions

Release workflow requires:

- `contents: write` - Create releases and tags
- `pull-requests: write` - Update documentation

Configured in `.github/workflows/release.yml`

## Troubleshooting

### Release Workflow Failed

**Check:**
1. CI status on `main` branch
2. Version format (must be `vX.Y.Z`)
3. Tag doesn't already exist
4. GitHub Actions permissions

**Fix:**
```bash
# Delete failed tag if needed
git tag -d v1.0.0
git push --delete origin v1.0.0

# Fix issues and retry
git tag v1.0.0
git push origin v1.0.0
```

### Version Already Exists

If `docs/versions/v1.0.0/` already exists:

```bash
# Manually remove and recommit
git rm -r docs/versions/v1.0.0
git commit -m "chore: remove duplicate version"
git push origin main

# Then retry release
git tag v1.0.0
git push origin v1.0.0
```

### Build Failures

```bash
# Test locally first
npm install
npm run check
npm run build

# Fix any errors before tagging
```

### Symlink Issues

```bash
# Recreate latest symlink
cd docs/versions
rm -f latest
ln -s v1.0.0 latest
git add latest
git commit -m "chore: fix latest symlink"
git push origin main
```

## Best Practices

### 1. Regular Releases

- Release frequently (monthly or quarterly)
- Don't accumulate too many changes
- Keep releases focused

### 2. Testing

- Always test build locally before releasing
- Verify links and navigation
- Check site rendering

### 3. Documentation

- Keep `vNEXT` up to date
- Document breaking changes
- Provide migration guides for major versions

### 4. Communication

- Announce releases to team
- Update dependent projects
- Note breaking changes prominently

### 5. Version Planning

- Plan version numbers in advance
- Group related changes
- Consider semantic versioning implications

## Security

### Signed Commits

For enhanced security, use signed commits:

```bash
# Configure GPG signing
git config --global commit.gpgsign true
git config --global user.signingkey YOUR_KEY_ID

# Sign tags
git tag -s v1.0.0 -m "Release v1.0.0"
```

### Release Verification

Users can verify releases:

```bash
# Verify checksum
sha256sum -c docs-v1.0.0.tar.gz.sha256

# Verify tag signature (if signed)
git verify-tag v1.0.0
```

## FAQ

**Q: Can I release from a branch other than main?**
A: No, releases must be from `main` to ensure stability.

**Q: What if I need to skip a version?**
A: Just tag the next version. Semantic versioning allows gaps.

**Q: How do I deprecate a version?**
A: Update `versions.json` with `"status": "deprecated"` and add a notice in the version's README.

**Q: Can I edit a release after creation?**
A: Yes, via GitHub UI or CLI. But prefer creating a new patch release for significant changes.

**Q: What's the minimum version number?**
A: Start with `v1.0.0` for first stable release. Use `v0.x.y` for pre-stable.

## References

- [Semantic Versioning](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [STRUCTURE.md](./STRUCTURE.md) - Documentation organization
- [CONTRIBUTING.md](./CONTRIBUTING.md) - Contribution guidelines

---

**Questions?** Open an issue or see the [Contributing Guide](./CONTRIBUTING.md).

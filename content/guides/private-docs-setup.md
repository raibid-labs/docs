---
title: Setting Up Private Documentation
description: Guide for creating a companion internal documentation repository
tags: [guide, private-docs, setup, internal]
---

# Setting Up Private Documentation

This guide explains how to set up a companion `docs-internal` repository for aggregating both public and private documentation from the raibid-labs organization.

## Architecture Overview

The two-repo approach provides:

- **docs** (this repo) - Public documentation from public repositories only
- **docs-internal** (companion repo) - All documentation (public + private repositories)

Both repositories share the same automation scripts and tooling, ensuring consistency.

## Quick Start

### 1. Create the Internal Repository

```bash
# Create new private repository
gh repo create raibid-labs/docs-internal \
  --private \
  --description "Internal documentation hub aggregating all raibid-labs repos" \
  --clone

cd docs-internal
```

### 2. Copy Configuration Files

```bash
# Copy the entire setup from the public docs repo
git clone https://github.com/raibid-labs/docs.git ../docs-public

# Copy essential files
cp -r ../docs-public/scripts ./
cp -r ../docs-public/config ./
cp ../docs-public/package.json ./
cp ../docs-public/quartz.config.ts ./
cp ../docs-public/.gitignore ./
cp ../docs-public/CLAUDE.md ./

# Copy Quartz installation
cp -r ../docs-public/quartz ./
```

### 3. Update Configuration

Edit `config/ignorelist.json` to allow private repositories:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "version": "1.0.0",
  "repositories": ["docs", "docs-internal"],
  "patterns": ["fork-*", "archive-*", "deprecated-*"],
  "exclude_forks": true,
  "exclude_archived": true,
  "exclude_private": false,
  "require_docs_directory": true
}
```

Edit `quartz.config.ts` to update branding:

```typescript
const config: QuartzConfig = {
  configuration: {
    pageTitle: "Raibid Labs Internal Documentation",
    // ... rest of config
  }
}
```

### 4. Initialize Documentation Structure

```bash
# Create directory structure
mkdir -p docs/content/{projects,guides}

# Create initial index
cat > docs/index.md << 'EOF'
---
title: Raibid Labs Internal Documentation
description: Internal documentation hub for raibid-labs (public + private repos)
tags: [home, documentation, internal]
---

# Welcome to Raibid Labs Internal Documentation

This is the **internal** documentation hub for the raibid-labs organization, aggregating documentation from both public and private repositories.

⚠️ **This site contains confidential information. Access is restricted to organization members.**

## Quick Links

- [[content/projects/index|All Projects]]
- [[content/guides/getting-started|Getting Started]]
- [Public Documentation](https://raibid-labs.github.io/docs) (external link)

## What's Different?

This internal hub includes:
- All public repository documentation
- Private repository documentation
- Internal guides and processes
- Confidential architecture documentation
EOF
```

### 5. Setup GitHub Actions

Create `.github/workflows/sync-and-deploy.yml`:

```yaml
name: Sync Internal Documentation

on:
  schedule:
    - cron: '0 2 * * *'
  workflow_dispatch:
  push:
    branches: [main]

permissions:
  contents: write

jobs:
  sync-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
          token: ${{ secrets.GITHUB_TOKEN }}

      - uses: actions/setup-node@v4
        with:
          node-version: '22'

      - uses: hustcer/setup-nu@v3
        with:
          version: '*'

      - name: Authenticate GitHub CLI
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          echo "${{ secrets.GITHUB_TOKEN }}" | gh auth login --with-token

      - name: Install dependencies
        run: npm ci || npm install

      - name: Make scripts executable
        run: chmod +x scripts/*.nu

      - name: Discover repositories (including private)
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: nu scripts/discover-repos.nu --org raibid-labs --verbose

      - name: Sync submodules
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          git config --global user.name "github-actions[bot]"
          git config --global user.email "github-actions[bot]@users.noreply.github.com"
          nu scripts/sync-submodules.nu --verbose

      - name: Update documentation
        run: nu scripts/update-docs.nu --generate-index --verbose

      - name: Commit changes
        run: |
          git add .gitmodules docs/content/projects
          if ! git diff --cached --quiet; then
            git commit -m "chore: update internal documentation [skip ci]"
            git push
          fi

      - name: Build site
        run: npx quartz build

      # Private deployment options - choose one:

      # Option A: No deployment (build only, view locally)
      - name: Archive build
        uses: actions/upload-artifact@v3
        with:
          name: internal-docs-build
          path: public/

      # Option B: Deploy to private GitHub Pages (requires GitHub Enterprise)
      # - name: Deploy to GitHub Pages
      #   uses: peaceiris/actions-gh-pages@v3
      #   with:
      #     github_token: ${{ secrets.GITHUB_TOKEN }}
      #     publish_dir: ./public

      # Option C: Deploy to Cloudflare Pages
      # (Requires CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID secrets)
      # - name: Deploy to Cloudflare Pages
      #   uses: cloudflare/pages-action@v1
      #   with:
      #     apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
      #     accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
      #     projectName: raibid-labs-internal-docs
      #     directory: public
```

### 6. Run Initial Build

```bash
# Install dependencies
npm install

# Run discovery and sync
npm run discover
npm run sync

# Update docs and build
nu scripts/update-docs.nu --generate-index --verbose
npx quartz build

# View locally
npx quartz build --serve
```

## Deployment Options

### Option 1: Local-Only (Most Secure)

Keep the built site (`public/` directory) available only through GitHub Actions artifacts or local builds.

**Pros:**
- Maximum security
- No hosting costs
- Simple setup

**Cons:**
- Requires git clone and local build to view
- No searchable web interface

**Setup:** Already configured in the workflow above (see "Archive build" step)

### Option 2: Cloudflare Pages + Access (Recommended)

Deploy to Cloudflare Pages with Access for authentication.

**Pros:**
- Free for up to 50 users
- Professional authentication
- Fast global CDN
- No infrastructure management

**Cons:**
- Requires Cloudflare account
- Initial setup complexity

**Setup Guide:**

1. **Create Cloudflare Account** (if needed)
   - Sign up at https://dash.cloudflare.com/sign-up

2. **Create Pages Project**
   ```bash
   # Install Wrangler CLI
   npm install -g wrangler

   # Login to Cloudflare
   wrangler login

   # Create Pages project
   wrangler pages project create raibid-labs-internal-docs
   ```

3. **Configure Access**
   - Go to Cloudflare Dashboard → Zero Trust → Access
   - Create an application for your Pages URL
   - Add access policies (GitHub OAuth, email domains, etc.)

4. **Add GitHub Secrets**
   ```bash
   # Get API token from Cloudflare Dashboard
   gh secret set CLOUDFLARE_API_TOKEN
   gh secret set CLOUDFLARE_ACCOUNT_ID
   ```

5. **Enable Deployment**
   - Uncomment the Cloudflare Pages deployment step in workflow

### Option 3: GitHub Enterprise (If Available)

If your organization has GitHub Enterprise Cloud, you can use private GitHub Pages.

**Setup:**
- Uncomment the GitHub Pages deployment step in workflow
- Enable Pages in repository settings
- Configure access restrictions

## Maintenance

### Daily Automation

Both workflows run automatically:
- Public docs: Updates from public repos daily at 2 AM UTC
- Internal docs: Updates from all repos daily at 2 AM UTC

### Manual Sync

```bash
# Discover new repositories
npm run discover

# Sync submodules
npm run sync

# Update docs and rebuild
nu scripts/update-docs.nu --generate-index --verbose
npm run build
```

### Adding to Ignorelist

Edit `config/ignorelist.json` in both repositories to exclude specific repositories.

## Accessing Documentation

### Public Docs
https://raibid-labs.github.io/docs

### Internal Docs
- **Local:** Run `npx quartz build --serve` after cloning
- **Cloudflare Pages:** Your configured domain (e.g., `internal-docs.pages.dev`)
- **Artifacts:** Download from GitHub Actions runs

## Troubleshooting

### Submodule Authentication Issues

If you encounter authentication errors accessing private repositories:

```bash
# Ensure git is configured to use HTTPS with token
git config --global url."https://x-access-token:${GITHUB_TOKEN}@github.com/".insteadOf "https://github.com/"
```

### Missing Private Repositories

Check `config/ignorelist.json` - ensure `exclude_private` is set to `false`.

### Build Failures

Check that all prerequisites are installed:
- Node.js >= 22.0.0
- Nushell >= 0.105.0
- GitHub CLI (authenticated)

## Security Considerations

1. **Never commit secrets** - Use GitHub Secrets for tokens
2. **Review access policies** - Regularly audit who has access
3. **Monitor usage** - Check Cloudflare Analytics for unusual access patterns
4. **Separate concerns** - Keep public and private repos separate
5. **Audit submodules** - Periodically review what's being aggregated

## Migration Path

If you later want to consolidate into a single repo with dual-deployment:

1. Make the public `docs` repo private
2. Implement dual-build system (public + full versions)
3. Deploy public build to GitHub Pages
4. Deploy full build to Cloudflare Pages + Access
5. Archive the `docs-internal` repo

See [Advanced: Dual Deployment](./dual-deployment.md) for implementation details.

---

*Last updated: 2025-10-29*

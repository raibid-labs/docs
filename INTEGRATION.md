# 🔗 Documentation Hub Integration

## Quick Reference for raibid-labs Repositories

This is a quick reference for integrating your repository with the raibid-labs documentation hub. For the complete guide, see [docs/integration-guide.md](docs/integration-guide.md).

## ⚡ Quick Setup (5 minutes)

### 1. Create `/docs` directory

```bash
mkdir -p docs
echo "# $(basename $(pwd))" > docs/index.md
```

### 2. Add the sync workflow

```bash
mkdir -p .github/workflows
curl -o .github/workflows/notify-docs-hub.yml \
  https://raw.githubusercontent.com/raibid-labs/docs/main/docs/templates/notify-docs-hub.yml
```

### 3. Commit and push

```bash
git add docs/ .github/workflows/
git commit -m "feat: integrate with documentation hub"
git push
```

**That's it!** Your documentation will now:
- ✅ Automatically sync when you push changes to `/docs`
- ✅ Update when you publish releases
- ✅ Display version info and last updated dates
- ✅ Be searchable and browseable at [raibid-labs.github.io/docs](https://raibid-labs.github.io/docs)

## 📊 Features You Get

- **Automatic Syncing**: Push to `/docs` → docs site updates in ~2 minutes
- **Version Tracking**: Releases and tags automatically displayed
- **Graph View**: Visual connections between all documentation
- **Full-Text Search**: Instantly searchable documentation
- **Dark Mode**: User-preferred theme support
- **Backlinks**: See what pages link to yours
- **Zero Configuration**: Works out of the box

## 📁 Documentation Structure

```
your-repo/
├── docs/
│   ├── index.md              # Main page (auto-generated if missing)
│   ├── getting-started.md    # Your docs here
│   ├── api-reference.md
│   └── guides/
│       ├── installation.md
│       └── configuration.md
└── .github/
    └── workflows/
        └── notify-docs-hub.yml  # Copy from template
```

## 🏷️ Versioning

Create releases to display version info on your docs:

```bash
# Tag and release
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
gh release create v1.0.0 --title "v1.0.0" --notes "Release notes"
```

Your docs will automatically show:
- 📦 Latest release version with link
- 🕐 Last updated date
- 🔗 Direct link to GitHub release

## 🔍 Sync Status

### Check if your docs are synced:
1. Visit: https://raibid-labs.github.io/docs
2. Search for your project name
3. Or browse: https://raibid-labs.github.io/docs/projects/

### Trigger manual sync:
Ask a maintainer to run the workflow manually, or wait for the daily scheduled sync at 02:00 UTC.

### Verify workflow:
- Check your repo's Actions tab for `Notify Documentation Hub` workflow
- Check docs repo for `Sync Documentation and Deploy` workflow runs

## 📝 Markdown Tips

### Use wikilinks for internal references:
```markdown
See [[getting-started]] for installation.
See [[api-reference#authentication]] for auth details.
```

### Add frontmatter to all pages:
```yaml
---
title: Page Title
description: Brief description for search
tags: [api, reference, authentication]
---
```

### Include code with syntax highlighting:
```typescript
// TypeScript example
const config: Config = {
  apiKey: process.env.API_KEY,
};
```

## 🆘 Troubleshooting

### Docs not showing up?
- ✅ Repository is public
- ✅ `/docs` directory exists with `.md` files
- ✅ Wait 24 hours or trigger manual sync
- ✅ Check if repo is in ignorelist

### Version info missing?
- ✅ Create a GitHub release or tag
- ✅ Wait for next sync (or trigger manually)
- ✅ Check that project index regenerated

### Workflow not running?
- ✅ Workflow file is in `.github/workflows/`
- ✅ File is named `*.yml` or `*.yaml`
- ✅ Check Actions tab for errors
- ✅ Verify `GITHUB_TOKEN` permissions

## 📚 Additional Resources

- **Complete Guide**: [docs/integration-guide.md](docs/integration-guide.md)
- **Workflow Template**: [docs/templates/notify-docs-hub.yml](docs/templates/notify-docs-hub.yml)
- **Quartz Documentation**: https://quartz.jzhao.xyz
- **Issues**: https://github.com/raibid-labs/docs/issues

## 🎯 What Gets Synced

The hub automatically pulls from your repository:
- All markdown files in `/docs` directory
- Subdirectories and nested docs
- Images and assets (kept in place via submodules)
- Frontmatter metadata
- Git history for dates

What's NOT synced:
- Files outside `/docs`
- Private repositories
- Binary files (large images may slow builds)
- Files starting with `.` or `_`

---

**Questions?** Open an issue at [raibid-labs/docs](https://github.com/raibid-labs/docs/issues)

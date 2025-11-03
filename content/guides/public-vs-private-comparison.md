---
title: Public vs Private Documentation
description: Comparison of the two-repo approach for documentation management
tags: [guide, architecture, comparison, private-docs]
---

# Public vs Private Documentation

Quick reference comparing the public and private documentation repositories.

## Side-by-Side Comparison

| Feature | Public Docs (`docs`) | Internal Docs (`docs-internal`) |
|---------|---------------------|-------------------------------|
| **Repository** | [raibid-labs/docs](https://github.com/raibid-labs/docs) | raibid-labs/docs-internal |
| **Visibility** | Public | Private |
| **Aggregates** | Public repos only | All repos (public + private) |
| **Deployment** | GitHub Pages | Local/Cloudflare Pages/Artifacts |
| **Access** | Anyone | Organization members only |
| **URL** | https://raibid-labs.github.io/docs | Custom or local only |
| **Auto-sync** | Daily at 2 AM UTC | Daily at 2 AM UTC |
| **Content** | Open source docs | Confidential + public docs |

## Configuration Differences

### Public Docs (`config/ignorelist.json`)

```json
{
  "exclude_private": true,
  "require_docs_directory": true,
  "repositories": ["docs"]
}
```

### Internal Docs (`config/ignorelist.json`)

```json
{
  "exclude_private": false,
  "require_docs_directory": true,
  "repositories": ["docs", "docs-internal"]
}
```

**Key Difference:** `exclude_private` setting determines which repos are included.

## When to Use Each

### Use Public Docs (`docs`) For:

- ✅ Open source project documentation
- ✅ Public API documentation
- ✅ Community-facing guides
- ✅ Contributing guidelines
- ✅ Public architecture overviews
- ✅ Tutorial content
- ✅ External developer resources

### Use Internal Docs (`docs-internal`) For:

- ✅ Private repository documentation
- ✅ Confidential architecture details
- ✅ Internal processes and workflows
- ✅ Security implementation details
- ✅ Infrastructure documentation
- ✅ Business logic and algorithms
- ✅ Team-specific guides
- ✅ Deployment procedures with secrets

## Workflow Comparison

### Public Docs Workflow

```mermaid
graph LR
    A[GitHub API] -->|Discover public repos| B[Filter by ignorelist]
    B -->|Add as submodules| C[Git Submodules]
    C -->|Pull docs| D[Build with Quartz]
    D -->|Deploy| E[GitHub Pages]
    E -->|Public access| F[Anyone can view]
```

### Internal Docs Workflow

```mermaid
graph LR
    A[GitHub API] -->|Discover all repos| B[Filter by ignorelist]
    B -->|Add as submodules| C[Git Submodules]
    C -->|Pull docs| D[Build with Quartz]
    D -->|Deploy| E[Cloudflare Pages + Access]
    E -->|Authenticated| F[Org members only]
```

## Access Control

### Public Docs
- **Read access:** Everyone on the internet
- **Write access:** Organization members with repo permissions
- **Edit workflow:** Fork, edit, pull request

### Internal Docs
- **Read access:** Organization members only
- **Write access:** Organization members with repo permissions
- **Edit workflow:** Direct commit or pull request

## Deployment Options

### Public Docs

| Option | Status | Cost | Setup |
|--------|--------|------|-------|
| GitHub Pages | ✅ Enabled | Free | Automatic |

### Internal Docs

| Option | Recommended | Cost | Setup Complexity |
|--------|-------------|------|------------------|
| Local builds | Good for small teams | Free | Easy |
| GitHub Actions artifacts | Good for occasional viewing | Free | Easy |
| Cloudflare Pages + Access | Best for frequent access | Free (<50 users) | Moderate |
| GitHub Enterprise Pages | Enterprise only | $21/user/month | Easy |

## Maintenance

Both repositories share the same automation scripts, making maintenance straightforward:

### Shared Scripts
- `discover-repos.nu` - Repository discovery
- `sync-submodules.nu` - Submodule management
- `update-docs.nu` - Documentation indexing
- `build-site.nu` - Build pipeline

### Repository-Specific
- `config/ignorelist.json` - Different settings per repo
- `.github/workflows/sync-and-deploy.yml` - Different deployment targets
- `quartz.config.ts` - Different site titles/branding

## Cost Analysis

### Public Docs
- **Hosting:** $0 (GitHub Pages)
- **CI/CD:** $0 (GitHub Actions free tier)
- **Total:** $0/month

### Internal Docs (Recommended Setup)
- **Hosting:** $0 (Cloudflare Pages)
- **Authentication:** $0 (Cloudflare Access, <50 users)
- **CI/CD:** $0 (GitHub Actions free tier)
- **Total:** $0/month

**Note:** Both solutions are completely free for most organizations.

## Migration Paths

### From Public-Only to Two-Repo

✅ **Current status** - Just add internal repo following the [setup guide](./private-docs-setup.md)

### From Two-Repo to Single-Repo Dual-Build

If you later want to consolidate (advanced):

1. Make public `docs` repo private
2. Implement dual-build system:
   - Public build (filtered content)
   - Full build (all content)
3. Deploy public build to GitHub Pages
4. Deploy full build to Cloudflare Pages + Access
5. Archive `docs-internal` repo

**Complexity:** High | **Benefit:** Single source of truth

**Recommendation:** Stick with two-repo approach unless you have specific needs for consolidation.

## Security Considerations

### Public Docs
- ⚠️ Never commit secrets or sensitive information
- ⚠️ Review all docs before making repo public
- ⚠️ Use `.gitignore` for sensitive files
- ✅ Great for open source transparency

### Internal Docs
- ⚠️ Still avoid committing secrets (use env vars/secrets)
- ⚠️ Review access permissions regularly
- ⚠️ Monitor for unauthorized access attempts
- ✅ Safe for confidential architecture and processes

## Best Practices

### Documentation Organization

**Public Docs:**
```
docs/content/
├── projects/          # Public project documentation
├── guides/            # Public getting started guides
└── api/               # Public API documentation
```

**Internal Docs:**
```
docs/content/
├── projects/          # All project documentation
├── guides/            # Internal processes and guides
├── architecture/      # Confidential system architecture
├── infrastructure/    # Infrastructure and deployment
└── security/          # Security implementations
```

### Content Tagging

Use frontmatter tags to categorize content:

**Public Docs:**
```yaml
---
tags: [public, tutorial, api, getting-started]
---
```

**Internal Docs:**
```yaml
---
tags: [internal, confidential, architecture, infrastructure]
---
```

## Quick Decision Tree

```
Do you need to share docs publicly?
├─ Yes → Use public docs repo
│   └─ Do you also have private repos with docs?
│       ├─ Yes → Also setup internal docs repo
│       └─ No → Only public docs repo needed
└─ No → Only internal docs repo needed
```

## FAQ

### Q: Can I link between public and internal docs?

**A:** Yes, but links from public → internal will be broken for external users. Use conditional content if needed.

### Q: How do I move a doc from internal to public?

**A:**
1. Move the doc in the source repository from private → public
2. Update the repository visibility if needed
3. Wait for next sync or trigger manually

### Q: Can I have different themes for each?

**A:** Yes, edit `quartz.config.ts` in each repository independently to customize branding and themes.

### Q: What if I accidentally commit secrets to public docs?

**A:**
1. Immediately rotate the compromised secrets
2. Remove from git history: `git filter-branch` or `BFG Repo-Cleaner`
3. Force push cleaned history
4. Notify security team

### Q: How do I control who can access internal docs?

**A:** If using Cloudflare Access:
1. Configure access policies (GitHub OAuth, email domains, etc.)
2. Regularly review access logs
3. Use short session timeouts
4. Enable 2FA requirements

## Resources

- [[private-docs-setup|Private Docs Setup Guide]] - Complete setup instructions
- [[getting-started|Getting Started]] - General documentation guide
- [[obsidian-usage|Obsidian Usage]] - Using Obsidian with the vaults
- [Cloudflare Pages Documentation](https://developers.cloudflare.com/pages/)
- [Cloudflare Access Documentation](https://developers.cloudflare.com/cloudflare-one/policies/access/)

---

*Last updated: 2025-10-29*

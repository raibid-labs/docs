---
title: Quartz Setup Guide
description: How to install and configure Quartz for the documentation hub
tags: [guide, quartz, setup, installation]
---

# Quartz Setup Guide

This guide walks you through setting up Quartz for the raibid-labs documentation hub.

## 📋 Prerequisites

Before installing Quartz, ensure you have:

- **Node.js v22+** and **npm v10.9.2+**
- **Git** installed and configured
- **GitHub CLI** (`gh`) for authentication
- **Nushell** for running automation scripts

### Verify Prerequisites

```bash
# Check Node.js version
node --version  # Should be v22 or higher

# Check npm version
npm --version   # Should be v10.9.2 or higher

# Check Git
git --version

# Check GitHub CLI
gh --version

# Check Nushell
nu --version
```

## 🚀 Installation

### 1. Clone the Repository

```bash
git clone https://github.com/raibid-labs/docs.git
cd docs
```

### 2. Initialize Quartz

Quartz can be set up in two ways:

#### Option A: Fresh Quartz Installation

```bash
# Run Quartz create command
npx quartz create

# Choose "Empty Quartz" option when prompted
# This preserves the docs/ directory structure
```

#### Option B: Install Dependencies Only

```bash
# If package.json already has Quartz dependencies
npm install
```

### 3. Install Project Dependencies

```bash
npm install
```

### 4. Configure GitHub Authentication

```bash
# Login to GitHub CLI
gh auth login

# Follow the prompts to authenticate
```

## ⚙️ Configuration

### Quartz Configuration

Create or edit `quartz.config.ts` in the root directory:

```typescript
import { QuartzConfig } from "./quartz/cfg"
import * as Plugin from "./quartz/plugins"

const config: QuartzConfig = {
  configuration: {
    pageTitle: "Raibid Labs Documentation",
    enableSPA: true,
    enablePopovers: true,
    analytics: {
      provider: "plausible",
    },
    locale: "en-US",
    baseUrl: "raibid-labs.github.io/docs",
    ignorePatterns: ["private", "templates", ".obsidian"],
    defaultDateType: "created",
    theme: {
      cdnCaching: true,
      typography: {
        header: "Schibsted Grotesk",
        body: "Source Sans Pro",
        code: "IBM Plex Mono",
      },
      colors: {
        lightMode: {
          light: "#faf8f8",
          lightgray: "#e5e5e5",
          gray: "#b8b8b8",
          darkgray: "#4e4e4e",
          dark: "#2b2b2b",
          secondary: "#284b63",
          tertiary: "#84a59d",
          highlight: "rgba(143, 159, 169, 0.15)",
        },
        darkMode: {
          light: "#161618",
          lightgray: "#393639",
          gray: "#646464",
          darkgray: "#d4d4d4",
          dark: "#ebebec",
          secondary: "#7b97aa",
          tertiary: "#84a59d",
          highlight: "rgba(143, 159, 169, 0.15)",
        },
      },
    },
  },
  plugins: {
    transformers: [
      Plugin.FrontMatter(),
      Plugin.CreatedModifiedDate({
        priority: ["frontmatter", "filesystem"],
      }),
      Plugin.Latex({ renderEngine: "katex" }),
      Plugin.SyntaxHighlighting({
        theme: {
          light: "github-light",
          dark: "github-dark",
        },
        keepBackground: false,
      }),
      Plugin.ObsidianFlavoredMarkdown({ enableInHtmlEmbed: false }),
      Plugin.GitHubFlavoredMarkdown(),
      Plugin.TableOfContents(),
      Plugin.CrawlLinks({ markdownLinkResolution: "shortest" }),
      Plugin.Description(),
    ],
    filters: [Plugin.RemoveDrafts()],
    emitters: [
      Plugin.AliasRedirects(),
      Plugin.ComponentResources({ fontOrigin: "googleFonts" }),
      Plugin.ContentPage(),
      Plugin.FolderPage(),
      Plugin.TagPage(),
      Plugin.ContentIndex({
        enableSiteMap: true,
        enableRSS: true,
      }),
      Plugin.Assets(),
      Plugin.Static(),
      Plugin.NotFoundPage(),
    ],
  },
}

export default config
```

### Content Directory Structure

Quartz expects content in the `docs/` directory:

```
docs/
├── index.md           # Homepage
└── content/
    ├── projects/      # Project documentation (submodules)
    └── guides/        # Local guides
```

## 🔨 Building the Site

### Development Server

Start a local development server with hot reload:

```bash
npx quartz build --serve
```

Visit `http://localhost:8080` to preview the site.

### Production Build

Build the static site for deployment:

```bash
npx quartz build
```

Output will be in the `public/` directory.

### Clean Build

Remove cache and rebuild:

```bash
rm -rf .quartz-cache public
npx quartz build
```

## 🔄 Sync Documentation

### Manual Sync

```bash
# Discover repositories
nu scripts/discover-repos.nu --org raibid-labs --verbose

# Sync submodules
nu scripts/sync-submodules.nu --verbose

# Update documentation
nu scripts/update-docs.nu --generate-index --verbose
```

### Full Build Pipeline

```bash
# Run the complete build pipeline
nu scripts/build-site.nu --verbose

# With live preview
nu scripts/build-site.nu --serve
```

## 🎨 Customization

### Custom Components

Add custom components to `quartz/components/`:

```typescript
// quartz/components/Custom.tsx
export default function CustomComponent() {
  return <div>Custom content</div>
}
```

Register in `quartz.layout.ts`:

```typescript
import Custom from "./components/Custom"

export const layout = {
  // Add to desired location
  beforeBody: [Custom()],
}
```

### Custom Styles

Add CSS to `quartz/styles/custom.scss`:

```scss
.custom-class {
  color: var(--secondary);
  padding: 1rem;
}
```

### Plugins

Quartz supports custom plugins. See [Quartz Plugin API](https://quartz.jzhao.xyz/plugins) for details.

## 🐛 Troubleshooting

### Port Already in Use

```bash
# Kill process on port 8080
lsof -ti:8080 | xargs kill -9

# Or specify different port
npx quartz build --serve --port 8081
```

### Build Errors

```bash
# Clear cache
rm -rf .quartz-cache

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Rebuild
npx quartz build
```

### Submodule Issues

```bash
# Reset submodules
git submodule deinit -f .
git submodule update --init --recursive
```

### Node Version Issues

```bash
# Install Node v22 using nvm
nvm install 22
nvm use 22

# Or using n
n 22
```

## 📚 Resources

- [Quartz Documentation](https://quartz.jzhao.xyz/)
- [Quartz GitHub](https://github.com/jackyzha0/quartz)
- [Obsidian Flavored Markdown](https://quartz.jzhao.xyz/features/Obsidian%20compatibility)
- [Quartz Plugins](https://quartz.jzhao.xyz/plugins)

## 🎯 Next Steps

- [[getting-started|Getting Started Guide]]
- [[../projects/index|Browse Projects]]
- [Contribute to Documentation](https://github.com/raibid-labs/docs)

---

[[index|← Back to Guides]]

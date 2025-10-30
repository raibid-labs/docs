---
title: Using Obsidian with This Vault
description: How to use Obsidian for local editing of the documentation hub
tags: [guide, obsidian, local-editing]
---

# Using Obsidian with This Vault

This documentation hub is designed to work seamlessly with Obsidian, allowing you to edit and navigate documentation locally using all of Obsidian's powerful features.

## 🎯 Why Use Obsidian?

- **Graph View**: Visualize connections between documents
- **Backlinks**: See all pages that reference the current page
- **Quick Switcher**: Jump to any file instantly
- **Live Preview**: See rendered markdown as you type
- **Plugins**: Extend functionality with community plugins
- **Tags**: Organize and discover content
- **Daily Notes**: Track changes and ideas

## 📥 Setup

### 1. Install Obsidian

Download from [obsidian.md](https://obsidian.md) and install for your platform.

### 2. Open This Vault

```bash
# Clone the repository if you haven't already
git clone https://github.com/raibid-labs/docs.git
cd docs
```

In Obsidian:
1. Click "Open folder as vault"
2. Navigate to the cloned `docs/` directory
3. Click "Open"

### 3. Recommended Settings

**Settings → Files & Links:**
- New link format: **Shortest path**
- Use [[Wikilinks]]: **Enabled**
- Automatically update internal links: **Enabled**

**Settings → Editor:**
- Default view for new tabs: **Editing view**
- Show line numbers: **Enabled**
- Strict line breaks: **Disabled**

**Settings → Appearance:**
- Theme: Choose your preference
- Base color scheme: Light or Dark

## 🔧 Recommended Plugins

### Core Plugins (Built-in)

Enable these in Settings → Core plugins:

- **Graph view**: Visualize connections
- **Backlinks**: Show incoming links
- **Tag pane**: Browse by tags
- **Page preview**: Hover to preview links
- **Outline**: Document structure
- **Search**: Full-text search
- **Quick switcher**: Fast navigation

### Community Plugins

Install these from Settings → Community plugins:

#### Essential
- **Dataview**: Query and display data from notes
- **Templater**: Advanced templates
- **Calendar**: Daily notes calendar
- **Obsidian Git**: Auto-commit changes

#### Nice to Have
- **Advanced Tables**: Better table editing
- **Kanban**: Project management boards
- **Excalidraw**: Embedded drawings
- **Mind Map**: Visual brainstorming

## 📂 Vault Structure

```
docs/
├── index.md                    # Homepage
├── content/
│   ├── projects/              # Project docs (submodules)
│   │   ├── project-1/
│   │   │   ├── index.md
│   │   │   └── ...
│   │   └── project-2/
│   │       ├── index.md
│   │       └── ...
│   └── guides/                # Local guides
│       ├── index.md
│       ├── getting-started.md
│       └── ...
├── .obsidian/                 # Obsidian settings (gitignored)
└── templates/                 # Note templates (optional)
```

## ✍️ Editing Guidelines

### Front Matter

All documentation pages should include front matter:

```yaml
---
title: Page Title
description: Brief description for search and previews
tags: [tag1, tag2, tag3]
---
```

### Wikilinks

Use wikilinks for internal references:

```markdown
- Link to page: [[page-name]]
- Link with custom text: [[page-name|Custom Text]]
- Link to heading: [[page-name#heading]]
- Link to block: [[page-name#^block-id]]
```

### Tags

Add tags for organization:

```markdown
#tag #nested/tag #multi-word-tag
```

Or in front matter:

```yaml
tags: [guide, obsidian, documentation]
```

### Callouts

Use Obsidian callouts for special content:

```markdown
> [!note]
> This is a note callout

> [!warning]
> Important warning message

> [!tip]
> Helpful tip

> [!example]
> Example code or explanation
```

## 🔍 Navigation Tips

### Quick Switcher

- **Cmd/Ctrl + O**: Open any file by name
- **Cmd/Ctrl + Shift + O**: Quick switcher for commands

### Search

- **Cmd/Ctrl + Shift + F**: Search in all files
- Use search operators:
  - `path:projects/` - Search in specific folder
  - `tag:#guide` - Search by tag
  - `line:(text)` - Search in same line

### Graph View

- **Cmd/Ctrl + G**: Open graph view
- Filters:
  - By tags
  - By folders
  - By links
- Use to discover connections between topics

## 🔄 Git Integration

### Obsidian Git Plugin

Install and configure the Obsidian Git plugin for automatic commits:

**Settings → Obsidian Git:**
- Vault backup interval: 10 minutes (or your preference)
- Commit message: `vault backup: {{date}}`
- Auto pull on startup: Enabled
- Auto push: Enabled

### Manual Git Operations

Or manage Git manually:

```bash
# Pull latest changes
git pull --recurse-submodules

# Update submodules
git submodule update --remote --merge

# Commit changes
git add .
git commit -m "docs: update documentation"
git push
```

## 📝 Templates

Create reusable templates in `templates/`:

### Guide Template

```markdown
---
title: {{title}}
description:
tags: [guide]
created: {{date}}
---

# {{title}}

## Overview

Brief introduction.

## Prerequisites

- Prerequisite 1
- Prerequisite 2

## Steps

### Step 1

Details...

### Step 2

Details...

## Conclusion

Summary and next steps.

---

[[guides/index|← Back to Guides]]
```

### Project Index Template

```markdown
---
title: {{title}}
description: Documentation for {{title}}
tags: [project, {{title}}]
---

# {{title}}

## Overview

Project description.

## Documentation

- [[page1|Page Title 1]]
- [[page2|Page Title 2]]

## Resources

- Repository: URL
- Issues: URL
- Discussions: URL
```

## 🎨 Customization

### CSS Snippets

Create custom styles in `.obsidian/snippets/`:

```css
/* custom.css */
.markdown-preview-view {
  font-family: 'Your Preferred Font';
}

/* Custom callout colors */
.callout[data-callout="custom"] {
  --callout-color: 100, 100, 255;
}
```

Enable in Settings → Appearance → CSS snippets

### Hotkeys

Customize keyboard shortcuts in Settings → Hotkeys:

- Frequently used commands
- Plugin commands
- Custom shortcuts

## 🔗 Integration with Quartz

Obsidian features that work with Quartz:

✅ **Supported:**
- Wikilinks
- Backlinks
- Front matter
- Tags
- Headings
- Lists
- Code blocks
- Tables
- Callouts (as blockquotes)

⚠️ **Partial Support:**
- Embedded notes (converted to links)
- DataView queries (rendered statically)

❌ **Not Supported:**
- Canvas files
- Obsidian-specific plugins
- Dynamic queries

## 🐛 Troubleshooting

### Submodule Content Not Showing

```bash
# Ensure submodules are initialized
git submodule update --init --recursive
```

### Links Not Working

- Check link format (use shortest path)
- Ensure wikilinks are enabled
- Verify file exists in vault

### Graph View Performance

- Exclude folders: Settings → Graph view → Filters
- Reduce node count with filters
- Close graph when not needed

## 📚 Resources

- [Obsidian Help](https://help.obsidian.md/)
- [Obsidian Forum](https://forum.obsidian.md/)
- [Obsidian Discord](https://discord.gg/obsidianmd)
- [Community Plugins](https://obsidian.md/plugins)

## 🎯 Next Steps

- [[quartz-setup|Quartz Setup]] - Configure Quartz for publishing
- [[getting-started|Getting Started]] - Begin contributing
- [[../projects/index|Browse Projects]] - Explore documentation

---

[[index|← Back to Guides]]

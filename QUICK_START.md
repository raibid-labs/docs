# Rust-Doc Bridge - Quick Start Guide

## What Is It?

A Quartz plugin that makes `[[MyStruct]]` links in your markdown automatically point to Rust API documentation.

## Install & Use

### Nothing to install! It's already integrated.

Just start writing wikilinks to API items:

```markdown
# My Guide

The [[Request]] struct is the main entry point.
Use [[RequestBuilder]] to construct one.

Errors return [[RequestError]].
```

When built, these automatically link to your API docs at `/api/crate_name/`.

## How to Build

```bash
# Local development
npm run dev

# Full build
npm run build

# Or with the pipeline
npm run pipeline
```

## How It Works (30-second version)

```
1. You write: [[MyStruct]]
2. Quartz converts: <a href="#MyStruct">MyStruct</a>
3. RustDocBridge checks rustdoc.json
4. Resolves to: <a href="/api/crate/struct.MyStruct.html">
5. Done!
```

## Smart Name Matching

All of these work:

```markdown
[[MyStruct]]        # Exact match
[[my_function]]     # PascalCase function
[[my_function]]     # Also matches MyFunction
[[Error]]           # Works for enums too
[[module::MyStruct]]  # Fully qualified
```

## Configuration

In `quartz.config.ts`:

```typescript
Plugin.RustDocBridge({
  rustdocJsonPath: "public/rustdoc.json",  // Where JSON is
  apiDocsPath: "/api",                      // Base URL
})
```

## Common Tasks

### Add a new project with docs

1. Create `/docs` folder in your repo
2. Add markdown files
3. Push to GitHub
4. Docs sync automatically

### Link to API items

```markdown
See [[MyStruct]] for details.
Use [[my_function]]() for examples.
[[Error]] is thrown on failure.
```

### Debug links not working

```bash
# Check if rustdoc.json exists
ls -lah public/rustdoc.json

# Verify it's not empty
jq . public/rustdoc.json | head

# Count items
jq '.index | length' public/rustdoc.json

# Rebuild if needed
nu scripts/build-rustdoc.nu --verbose
```

## Examples

### Basic Link

```markdown
The `[[Request]]` struct handles API requests.
```

→ Links to `/api/my_crate/struct.Request.html`

### Builder Pattern

```markdown
Use `[[RequestBuilder]]` to construct requests:

\`\`\`rust
let req = RequestBuilder::new()
    .method("POST")
    .build()?;
\`\`\`
```

### Error Handling

```markdown
Returns `[[Result]]<[[Response]], [[ApiError]]>`.

See [[ApiError]] for possible error types.
```

### Cross-Module References

```markdown
The `[[auth::Token]]` struct from the auth module
works with `[[Request]]`.
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Links don't work | Check rustdoc.json exists: `ls public/rustdoc.json` |
| Wrong item name | Rust names are case-sensitive: `[[MyStruct]]` not `[[mystruct]]` |
| Build fails | Ensure Rust is installed: `rustc --version` |
| Too slow | It's normal (~1-5s for index build). Cached across docs. |

## Files You Care About

- **Plugin:** `quartz/plugins/transformers/rustdoc-bridge.ts`
- **Config:** `quartz.config.ts`
- **Build script:** `scripts/build-rustdoc.nu`
- **Documentation:** `docs/rustdoc-bridge-guide.md`

## Next Steps

1. **Try it:** Write some `[[MyStruct]]` links in your docs
2. **Test:** Run `npm run dev` and click a link
3. **Verify:** It should go to `/api/crate_name/struct.MyStruct.html`
4. **Celebrate:** It works! 🎉

## Need Help?

- **Guide:** `docs/rustdoc-bridge-guide.md` (complete guide)
- **Technical:** `docs/rustdoc-bridge-implementation.md` (deep dive)
- **Issues:** GitHub Issues tracker
- **Summary:** `RUSTDOC_BRIDGE_README.md` (overview)

## Key Points

✅ **Automatic** - No manual configuration needed
✅ **Smart** - Handles naming convention mismatches  
✅ **Safe** - Graceful error handling
✅ **Fast** - O(1) link resolution
✅ **Documented** - Extensive guides and examples

---

**Ready to start?** Just write `[[MyStruct]]` and build! 🚀

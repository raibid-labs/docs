import { QuartzTransformerPlugin } from "../types"
import { visit } from "unist-util-visit"
import { Root, Element } from "hast"
import path from "path"
import fs from "fs"
import { BuildCtx } from "../../util/ctx"

/**
 * RustDocBridge Plugin
 *
 * Bridges the gap between Obsidian markdown docs and Rust API documentation.
 * Resolves [[MyStruct]], [[my_function]], etc. to rustdoc URLs.
 *
 * How it works:
 * 1. Loads rustdoc JSON output (generated via `cargo doc --output-format json`)
 * 2. Builds an index of all items (structs, functions, traits, etc.) → their URLs
 * 3. When it finds a wikilink [[name]], checks if it matches a rustdoc item
 * 4. Converts matching wikilinks to proper URLs pointing to the API docs
 *
 * Configuration:
 * - rustdocJsonPath: Path to rustdoc JSON output (default: "public/rustdoc.json")
 * - apiDocsPath: Base URL for API docs (default: "/api")
 * - stubPages: Generate markdown stub pages for API items (default: false)
 */

interface RustDocItem {
  name: string
  id: string
  kind: string
  crate: string
  module: string
  inner?: {
    doc?: string
  }
}

interface RustDocIndex {
  [key: string]: {
    url: string
    kind: string
    crate: string
    doc?: string
  }
}

interface Options {
  rustdocJsonPath?: string
  apiDocsPath?: string
  generateStubPages?: boolean
  crateNames?: string[]
}

const defaultOptions: Options = {
  rustdocJsonPath: "public/rustdoc.json",
  apiDocsPath: "/api",
  generateStubPages: false,
  crateNames: [],
}

/**
 * Parse rustdoc JSON and build an index of all items
 * Maps item names to their URLs
 */
function buildRustDocIndex(jsonPath: string, apiBasePath: string): RustDocIndex {
  const index: RustDocIndex = {}

  try {
    if (!fs.existsSync(jsonPath)) {
      console.warn(`[RustDocBridge] rustdoc.json not found at ${jsonPath}`)
      return index
    }

    const data = JSON.parse(fs.readFileSync(jsonPath, "utf-8"))

    // Traverse the crates section
    if (data.root && data.index && data.paths) {
      // Build a map of ID → path info
      const paths: Record<string, any> = data.paths || {}

      // Index all items
      for (const [id, item] of Object.entries(data.index)) {
        if (!item) continue

        const itemData = item as any
        const name = itemData.name || ""
        const kind = itemData.kind || ""
        const crate = itemData.crate || ""
        const path = paths[id] || {}

        // Build URL to the item
        const crateUrl = itemData.docs?.split("//")[0] || ""
        let itemUrl = `${apiBasePath}/${crate}/`

        // Handle different kinds
        if (path[0]) {
          const modulePath = path.slice(0, -1).join("/")
          itemUrl += `${modulePath}/${name.toLowerCase()}.html`
        } else {
          itemUrl += `${name.toLowerCase()}.html`
        }

        // Create multiple entries for lookups:
        // 1. By name (for simple lookups like [[MyStruct]])
        // 2. By full path (for fully qualified lookups like [[module::MyStruct]])

        if (!index[name]) {
          index[name] = {
            url: itemUrl,
            kind,
            crate,
            doc: itemData.inner?.doc,
          }
        }

        // Also index by full path if available
        if (path.length > 0) {
          const fullPath = path.join("::")
          if (!index[fullPath]) {
            index[fullPath] = {
              url: itemUrl,
              kind,
              crate,
              doc: itemData.inner?.doc,
            }
          }
        }
      }
    }
  } catch (error) {
    console.warn(`[RustDocBridge] Error parsing rustdoc.json: ${error}`)
  }

  return index
}

/**
 * Resolve a wikilink to a rustdoc item
 * Returns the URL if found, null otherwise
 */
function resolveRustDocLink(
  linkText: string,
  index: RustDocIndex,
  ctx: BuildCtx,
): string | null {
  // Try exact match first
  if (index[linkText]) {
    return index[linkText].url
  }

  // Try case-insensitive match for snake_case to CamelCase
  const pascalCase = linkText.replace(/(_[a-z])/g, (match) => match[1].toUpperCase())
  if (index[pascalCase]) {
    return index[pascalCase].url
  }

  // Try snake_case for CamelCase lookups
  const snakeCase = linkText.replace(/([A-Z])/g, "_$1").toLowerCase()
  if (index[snakeCase]) {
    return index[snakeCase].url
  }

  return null
}

export const RustDocBridge: QuartzTransformerPlugin<Partial<Options>> = (userOpts) => {
  const opts = { ...defaultOptions, ...userOpts }

  return {
    name: "RustDocBridge",
    htmlPlugins(ctx: BuildCtx) {
      // Build the rustdoc index once per build
      const rustdocIndex = buildRustDocIndex(opts.rustdocJsonPath!, opts.apiDocsPath!)

      return [
        () => {
          return (tree: Root, file) => {
            // Only process if we have rustdoc items indexed
            if (Object.keys(rustdocIndex).length === 0) {
              return
            }

            // Find all links that look like wikilinks to rust items
            visit(tree, "element", (node: Element) => {
              // Look for links with "internal" class (created by CrawlLinks)
              if (
                node.tagName === "a" &&
                node.properties &&
                typeof node.properties.href === "string"
              ) {
                const href = node.properties.href as string
                const classes = (node.properties.className ?? []) as string[]

                // Check if this is an internal link to a missing page
                // These are created by CrawlLinks for [[wikilinks]] that don't exist
                if (classes.includes("internal") && !href.startsWith("http")) {
                  // Extract the link text (the display name)
                  const linkText = (node.children[0] as any)?.value || ""

                  // Try to resolve as a rustdoc item
                  const rustdocUrl = resolveRustDocLink(linkText, rustdocIndex, ctx)

                  if (rustdocUrl) {
                    node.properties.href = rustdocUrl
                    classes.push("rustdoc")
                    node.properties.className = classes

                    // Add a title attribute with kind info
                    const item = rustdocIndex[linkText]
                    if (item && item.kind) {
                      node.properties.title = `${item.kind} - ${item.crate}`
                    }
                  }
                }
              }
            })
          }
        },
      ]
    },
  }
}

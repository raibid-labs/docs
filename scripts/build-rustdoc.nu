#!/usr/bin/env nu
# Rust Documentation Generator for Documentation Hub
#
# Purpose: Generate unified rustdoc JSON from all Rust projects in the org
# Usage: nu scripts/build-rustdoc.nu [--output path] [--verbose]

# Configuration
const PROJECTS_DIR = "content/projects"
const DEFAULT_OUTPUT = "public/rustdoc.json"

# Main function with proper argument handling
def main [
  --output (-o): string  # Output file path (default: public/rustdoc.json)
  --verbose (-v)         # Show detailed progress
] {
  let output_file = if ($output | is-empty) { $DEFAULT_OUTPUT } else { $output }

  # Create output directory
  let output_dir = ($output_file | path dirname)
  if not ($output_dir | path exists) {
    mkdir $output_dir
  }

  if $verbose {
    print $"📚 Starting Rust documentation build..."
    print $"📁 Projects directory: ($PROJECTS_DIR)"
    print $"📝 Output file: ($output_file)"
  }

  # Find all Rust projects (with Cargo.toml)
  let cargo_files = try { glob $"($PROJECTS_DIR)/*/Cargo.toml" } catch { [] }
  let projects = ($cargo_files | each { |cargo_file|
    {
      path: ($cargo_file | path dirname),
      name: ($cargo_file | path dirname | path basename),
    }
  })

  if ($projects | is-empty) {
    if $verbose {
      print "⚠️  No Rust projects found with Cargo.toml"
    }
    # Create empty rustdoc structure
    let empty_doc = {
      crate_version: null,
      includes: [],
      index: {},
      paths: {},
      root: null,
    }
    $empty_doc | to json | save --force $output_file
    return
  }

  if $verbose {
    print $"🔍 Found ($projects | length) Rust project\(s\)"
  }

  # Generate rustdoc JSON for each project
  mut merged_index = {}
  mut merged_paths = {}
  mut all_crates = []
  let start_dir = (pwd)

  for project in $projects {
    if $verbose {
      print $"  📦 Building docs for: ($project.name)"
    }

    try {
      # Generate rustdoc JSON with --output-format json
      cd $project.path

      # Build rustdoc JSON output
      let result = (do { cargo doc --output-format json --no-deps } | complete)

      if ($result.exit_code != 0) {
        if $verbose {
          print $"    ❌ Failed to generate docs for ($project.name): ($result.stderr | str trim)"
        }
        cd $start_dir
        continue
      }

      # Look for target/doc/*.json files
      let json_files = try { glob "target/doc/*.json" } catch { [] }

      if ($json_files | is-empty) {
        if $verbose {
          print $"    ℹ️  No JSON output found \(may not be in this Rust version\)"
        }
        cd $start_dir
        continue
      }

      # Read and merge documentation
      for json_file in $json_files {
        try {
          let doc_data = (open $json_file)

          # Merge crate information
          $all_crates = ($all_crates | append $project.name)

          # Merge index and paths
          let idx = ($doc_data | get -o index)
          if ($idx != null and ($idx | describe | str starts-with "record")) {
            $merged_index = ($merged_index | merge $idx)
          }
          let pth = ($doc_data | get -o paths)
          if ($pth != null and ($pth | describe | str starts-with "record")) {
            $merged_paths = ($merged_paths | merge $pth)
          }

          if $verbose {
            print $"    ✅ Merged documentation from ($json_file)"
          }
        } catch { |err|
          if $verbose {
            print $"    ⚠️  Error reading ($json_file): ($err.msg)"
          }
        }
      }

      cd $start_dir
    } catch { |err|
      if $verbose {
        print $"    ❌ Error building docs for ($project.name): ($err.msg)"
      }
      cd $start_dir
    }
  }

  # Build final rustdoc JSON structure
  let rustdoc_output = {
    crate_version: "0.1.0",
    includes: [],
    index: $merged_index,
    paths: $merged_paths,
    root: null,
    format_version: 28,
  }

  # Save the merged documentation
  $rustdoc_output | to json | save --force $output_file

  if $verbose {
    let output_size = if ($output_file | path exists) {
      let file_info = (ls $output_file | first)
      $"($file_info.size)"
    } else {
      "0 B"
    }
    let total_items = ($merged_index | columns | length)
    let crate_count = ($all_crates | length)
    print ""
    print $"✅ Documentation generation completed!"
    print $"📊 Statistics:"
    print $"  - Total items indexed: ($total_items)"
    print $"  - Crates processed: ($crate_count)"
    print $"  - Output file: ($output_file)"
    print $"  - File size: ($output_size)"
  }
}
